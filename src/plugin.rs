use crate::{
  batch::{self, Cost},
  native::{self, Method, Object},
  profile,
  session::{Dispatch, Execution, Session},
};
use core::{
  cell::{Cell, RefCell},
  ptr,
};

std::thread_local! {
  static SESSION: RefCell<Session> = RefCell::new(Session::default());
  static DISPLAYING: Cell<bool> = const { Cell::new(false) };
  static EXECUTION: RefCell<Execution> = RefCell::new(Execution::default());
  static IN_TRANSACTION: Cell<bool> = const { Cell::new(false) };
}

pub fn install() {
  if !supported_game() {
    return;
  }
  skyline::install_hooks!(
    on_select,
    on_decide,
    on_yes,
    on_cancel,
    destroy_menu,
    execute_merge,
    unload,
    meld_one,
    yes_no_dialog
  );
}

fn supported_game() -> bool {
  // Cobalt does not export Skyline's get_program_id. The mapped executable
  // identity and instruction checks below identify the supported game instead.
  let mut version = skyline::nn::oe::DisplayVersion { name: [0; 16] };
  unsafe {
    skyline::nn::oe::GetDisplayVersion(&mut version);
  }
  if version.name[..6] != [b'2', b'.', b'0', b'.', b'0', 0] {
    return false;
  }
  let base = native::base();
  let rodata = unsafe { skyline::hooks::getRegionAddress(skyline::hooks::Region::Rodata) as usize };
  let data = unsafe { skyline::hooks::getRegionAddress(skyline::hooks::Region::Data) as usize };
  if base == 0
    || rodata.checked_sub(base) != Some(profile::RODATA_OFFSET)
    || data <= rodata
    || data - base < profile::BUILD_ID_OFFSET + 16
  {
    return false;
  }
  // This executable carries a 16-byte GNU Build ID note. Its NSO header pads
  // the same identifier to 32 bytes; the loaded image has no NSO header.
  let id =
    unsafe { core::slice::from_raw_parts((base + profile::BUILD_ID_OFFSET) as *const u8, 16) };
  if id != profile::BUILD_ID {
    return false;
  }
  for &(offset, words) in profile::TEXT_SIGNATURES {
    if offset + 16 > profile::RODATA_OFFSET {
      return false;
    }
    for (i, expected) in words.iter().enumerate() {
      if unsafe { ((base + offset + i * 4) as *const u32).read_volatile() } != *expected {
        return false;
      }
    }
  }
  true
}

#[skyline::hook(offset = profile::ON_SELECT)]
fn on_select(menu: Object, ring: Object, method: Method) {
  SESSION.with(|s| s.borrow_mut().clear());
  call_original!(menu, ring, method);
}

#[skyline::hook(offset = profile::ON_DECIDE)]
fn on_decide(
  menu: Object,
  ring: Object,
  rings: i32,
  fragments: i32,
  god_index: i32,
  menu_select: Object,
  method: Method,
) {
  let selected = unsafe { native::selection(menu, ring, Cost { rings, fragments }) };
  SESSION.with(|s| s.borrow_mut().prepare(selected));
  let previous = DISPLAYING.with(|v| v.replace(true));
  // Original per-meld costs are never overwritten, even while constructing
  // the confirmation dialog. Result creation therefore remains native.
  call_original!(menu, ring, rings, fragments, god_index, menu_select, method);
  DISPLAYING.with(|v| v.set(previous));
}

#[skyline::hook(offset = profile::YES_NO_DIALOG)]
fn yes_no_dialog(
  parent: Object,
  message: Object,
  yes: Object,
  no: Object,
  method: Method,
) -> Object {
  let replacement = if DISPLAYING.with(Cell::get) {
    let prompt = SESSION.with(|s| s.borrow().prompt().map(str::to_owned));
    prompt.map(|p| native::new_text(&p)).unwrap_or(ptr::null_mut())
  } else {
    ptr::null_mut()
  };
  let message = if replacement.is_null() { message } else { replacement };
  let dialog = call_original!(parent, message, yes, no, method);
  if !replacement.is_null() && !dialog.is_null() {
    SESSION.with(|s| s.borrow_mut().mark_displayed());
  }
  dialog
}

#[skyline::hook(offset = profile::ON_YES)]
fn on_yes(menu: Object, method: Method) {
  SESSION.with(|s| s.borrow_mut().confirm(menu as usize));
  call_original!(menu, method);
}

#[skyline::hook(offset = profile::ON_CANCEL)]
fn on_cancel(menu: Object, method: Method) {
  SESSION.with(|s| s.borrow_mut().clear());
  call_original!(menu, method);
}

#[skyline::hook(offset = profile::DESTROY_MENU)]
fn destroy_menu(menu: Object, method: Method) {
  SESSION.with(|s| s.borrow_mut().destroy_menu(menu as usize));
  call_original!(menu, method);
}

#[skyline::hook(offset = profile::EXECUTE_MERGE)]
fn execute_merge(sequence: Object, method: Method) {
  let plan = SESSION.with(|s| s.borrow_mut().take_confirmed());
  // Nested execution is not another opportunity to consume inventory.
  if !EXECUTION.with(|e| e.borrow_mut().begin(plan)) {
    return;
  }
  // Keep footer refresh, acquisition state, and the original outer flow.
  call_original!(sequence, method);
  EXECUTION.with(|e| e.borrow_mut().end());
}

#[skyline::hook(offset = profile::MELD_ONE)]
fn meld_one(rnid: Object, rings: i32, fragments: i32, method: Method) -> bool {
  if IN_TRANSACTION.with(Cell::get) {
    return false;
  }
  let selected = match EXECUTION.with(|e| e.borrow_mut().dispatch()) {
    Dispatch::Vanilla => return call_original!(rnid, rings, fragments, method),
    Dispatch::Suppress => return false,
    Dispatch::Batch(selected) => selected,
  };
  let cost = Cost { rings, fragments };
  if cost != selected.plan.cost || unsafe { native::text(rnid) }.as_deref() != Some(&selected.rnid)
  {
    return false;
  }
  let Some(mut inventory) = (unsafe { native::GameInventory::new(rnid, method, original!()) })
  else {
    return false;
  };
  IN_TRANSACTION.with(|v| v.set(true));
  let outcome = batch::execute(&mut inventory, selected.plan);
  IN_TRANSACTION.with(|v| v.set(false));
  // Never fall back to another original call after any attempted batch.
  outcome.any_new
}

#[skyline::hook(offset = profile::UNLOAD)]
fn unload(sequence: Object, method: Method) {
  SESSION.with(|s| s.borrow_mut().clear());
  EXECUTION.with(|e| e.borrow_mut().end());
  call_original!(sequence, method);
}
