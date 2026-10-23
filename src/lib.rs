#![deny(unsafe_op_in_unsafe_fn)]

pub mod batch;
pub mod confirmation;
pub mod profile;
pub mod session;

#[cfg(target_os = "switch")]
mod native;
#[cfg(target_os = "switch")]
mod plugin;

#[cfg(target_os = "switch")]
#[skyline::main(name = "feemeld")]
pub fn skyline_main() {
  plugin::install();
}
