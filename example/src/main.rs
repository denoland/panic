fn main() {
  // Enabling frame-pointer keeps the original API available.
  println!("{}", deno_panic::trace());

  #[cfg(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
  ))]
  println!("{}", deno_panic::trace_frame_pointer());
}
