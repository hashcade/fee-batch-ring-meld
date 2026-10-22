//! Narrow, version-pinned native ABI. Never retain inventory entry pointers.

use crate::{
  batch::{Cost, Inventory, Plan, Stock},
  profile,
  session::Selection,
};
use core::{ffi::c_void, ptr};

pub type Object = *mut c_void;
pub type Method = *const c_void;
pub type Meld = unsafe extern "C" fn(Object, i32, i32, Method) -> bool;
const NONE: Method = ptr::null();

pub fn base() -> usize {
  unsafe { skyline::hooks::getRegionAddress(skyline::hooks::Region::Text) as usize }
}

/// The caller supplies live game objects and a field offset verified for 2.0.0.
pub unsafe fn field<T: Copy>(object: Object, offset: usize) -> T {
  unsafe { object.cast::<u8>().add(offset).cast::<T>().read_unaligned() }
}

pub unsafe fn text(string: Object) -> Option<String> {
  if string.is_null() {
    return None;
  }
  let len: i32 = unsafe { field(string, 0x10) };
  if !(0..=4096).contains(&len) {
    return None;
  }
  let units = unsafe {
    core::slice::from_raw_parts(string.cast::<u8>().add(0x14).cast::<u16>(), len as usize)
  };
  String::from_utf16(units).ok()
}

pub fn new_text(value: &str) -> Object {
  let units: Vec<u16> = value.encode_utf16().collect();
  if units.len() > 4096 {
    return ptr::null_mut();
  }
  let allocate: unsafe extern "C" fn(i32, Method) -> Object =
    unsafe { core::mem::transmute(base() + profile::STRING_NEW_SIZE) };
  let string = unsafe { allocate(units.len() as i32, NONE) };
  if !string.is_null() {
    // The native allocator initializes the IL2CPP header, length, terminator,
    // and GC tracking. Only the newly allocated UTF-16 payload is filled here.
    unsafe {
      ptr::copy_nonoverlapping(
        units.as_ptr(),
        string.cast::<u8>().add(0x14).cast::<u16>(),
        units.len(),
      );
    }
  }
  string
}

unsafe fn next_data(rnid: Object) -> Object {
  let f: unsafe extern "C" fn(Object, Method) -> Object =
    unsafe { core::mem::transmute(base() + profile::NEXT_RANK) };
  unsafe { f(rnid, NONE) }
}

unsafe fn source_data(rnid: Object) -> Object {
  let f: unsafe extern "C" fn(Object, Object, Method) -> Object =
    unsafe { core::mem::transmute(base() + profile::TRY_GET_RING) };
  let entry = unsafe { f(rnid, ptr::null_mut(), NONE) };
  if entry.is_null() {
    return ptr::null_mut();
  }
  unsafe { field(entry, 0x20) }
}

unsafe fn user_data() -> Object {
  let slot: Object = unsafe { ((base() + profile::USER_DATA_METHOD) as *const Object).read() };
  if slot.is_null() {
    return ptr::null_mut();
  }
  let method: Method = unsafe { slot.cast::<Method>().read() };
  if method.is_null() {
    return ptr::null_mut();
  }
  let f: unsafe extern "C" fn(Method) -> Object =
    unsafe { core::mem::transmute(base() + profile::SINGLETON_GET) };
  unsafe { f(method) }
}

unsafe fn stock(rnid: Object, source: Object) -> Option<Stock> {
  let output = unsafe { next_data(rnid) };
  let user = unsafe { user_data() };
  if source.is_null() || output.is_null() || user.is_null() {
    return None;
  }
  let count: unsafe extern "C" fn(Object, bool, Method) -> i32 =
    unsafe { core::mem::transmute(base() + profile::STOCK_COUNT) };
  let total: unsafe extern "C" fn(Object, Method) -> i32 =
    unsafe { core::mem::transmute(base() + profile::ALL_COUNT) };
  let can_add: unsafe extern "C" fn(Object, i32, Method) -> bool =
    unsafe { core::mem::transmute(base() + profile::CAN_ADD) };
  let output_id: Object = unsafe { field(output, 0x20) };
  Some(Stock {
    source_free: unsafe { count(source, false, NONE) },
    source_equipped: unsafe { count(source, true, NONE) },
    output_total: unsafe { total(output, NONE) },
    output_equipped: unsafe { count(output, true, NONE) },
    fragments: unsafe { field(user, 0x6C) },
    can_add: !output_id.is_null() && unsafe { can_add(output_id, 1, NONE) },
  })
}

unsafe fn label(data: Object) -> Option<String> {
  let mid: Object = unsafe { field(data, 0x28) };
  if mid.is_null() {
    return None;
  }
  let get: unsafe extern "C" fn(Object, Method) -> Object =
    unsafe { core::mem::transmute(base() + profile::MESS_GET) };
  let name = unsafe { text(get(mid, NONE)) }?;
  let rank: i32 = unsafe { field(data, 0x4C) };
  let letter = *['C', 'B', 'A', 'S'].get(usize::try_from(rank).ok()?)?;
  Some(format!("{name} {letter}"))
}

pub unsafe fn selection(menu: Object, source: Object, cost: Cost) -> Option<Selection> {
  if menu.is_null() || source.is_null() || !cost.is_valid() {
    return None;
  }
  let kind: i32 = unsafe { field(source, 0x48) };
  let rank: i32 = unsafe { field(source, 0x4C) };
  if kind != 0 || !matches!((rank, cost.rings), (0, 2) | (1, 3) | (2, 4)) {
    return None;
  }
  let rnid: Object = unsafe { field(source, 0x20) };
  let id = unsafe { text(rnid) }?;
  let output = unsafe { next_data(rnid) };
  if output.is_null() {
    return None;
  }
  let output_kind: i32 = unsafe { field(output, 0x48) };
  let output_rank: i32 = unsafe { field(output, 0x4C) };
  if output_kind != 0 || output_rank != rank + 1 {
    return None;
  }
  let plan = Plan::new(cost, unsafe { stock(rnid, source) }?)?;
  let source_name = unsafe { label(source) }?;
  let output_name = unsafe { label(output) }?;
  let total = plan.total_cost();
  let prompt = format!(
    "{source_name} ×{} → {output_name} ×{}\n\nCost: {} Bond Fragments\nMeld this batch?",
    total.rings, plan.count, total.fragments
  );
  Some(Selection { menu: menu as usize, rnid: id, prompt, plan })
}

pub struct GameInventory {
  rnid: Object,
  method: Method,
  original: Meld,
  source: Object,
}

impl GameInventory {
  pub unsafe fn new(rnid: Object, method: Method, original: Meld) -> Option<Self> {
    let source = unsafe { source_data(rnid) };
    if source.is_null() {
      return None;
    }
    // RingData belongs to the immutable game database, not the mutable pool.
    // Keep it only during this synchronous batch so the final zero-source
    // stock can still be checked after Sub removes the last pool entry.
    Some(Self { rnid, method, original, source })
  }
}

impl Inventory for GameInventory {
  fn stock(&mut self) -> Option<Stock> {
    unsafe { stock(self.rnid, self.source) }
  }

  fn meld_one(&mut self, cost: Cost) -> bool {
    unsafe { (self.original)(self.rnid, cost.rings, cost.fragments, self.method) }
  }
}
