pub mod vlq;

#[cfg(all(
  target_os = "windows",
  any(target_arch = "x86_64", target_arch = "aarch64")
))]
mod win64;

#[cfg(all(
  target_os = "windows",
  any(target_arch = "x86_64", target_arch = "aarch64")
))]
pub use win64::trace;

#[cfg(all(
  feature = "frame-pointer",
  target_os = "linux",
  any(target_arch = "x86_64", target_arch = "aarch64")
))]
mod frame_pointer;

#[cfg(all(
  feature = "frame-pointer",
  target_os = "linux",
  any(target_arch = "x86_64", target_arch = "aarch64")
))]
pub use frame_pointer::trace as trace_frame_pointer;

#[cfg(unix)]
mod libunwind;

#[cfg(unix)]
pub use libunwind::trace;

#[cfg(feature = "symbolicate")]
pub mod symbolicate;
