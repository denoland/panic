use std::ffi::c_int;
use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::ops::Range;

use crate::vlq::vlq_encode;

const MAX_FRAMES: usize = 256;

#[inline(always)]
fn frame_pointer() -> usize {
  let fp: usize;

  #[cfg(target_arch = "x86_64")]
  unsafe {
    // SAFETY: Reading the frame-pointer register has no side effects.
    core::arch::asm!(
      "mov {}, rbp",
      out(reg) fp,
      options(nomem, nostack, preserves_flags)
    );
  }

  #[cfg(target_arch = "aarch64")]
  unsafe {
    // SAFETY: Reading the frame-pointer register has no side effects.
    core::arch::asm!(
      "mov {}, x29",
      out(reg) fp,
      options(nomem, nostack, preserves_flags)
    );
  }

  fp
}

struct MemoryAccessor {
  // -1 means unopened and -2 means unavailable.
  mem_fd: c_int,
}

impl MemoryAccessor {
  fn new() -> Self {
    Self { mem_fd: -1 }
  }

  fn read_usize(&mut self, address: usize) -> Option<usize> {
    let mut bytes = [0; size_of::<usize>()];
    if self.read(address, &mut bytes) {
      Some(usize::from_ne_bytes(bytes))
    } else {
      None
    }
  }

  fn read(&mut self, address: usize, output: &mut [u8]) -> bool {
    if self.mem_fd == -1 {
      let local = libc::iovec {
        iov_base: output.as_mut_ptr().cast(),
        iov_len: output.len(),
      };
      let remote = libc::iovec {
        iov_base: address as *mut c_void,
        iov_len: output.len(),
      };
      let bytes_read = unsafe {
        // SAFETY: Both iovecs describe valid buffers for their stated lengths.
        libc::process_vm_readv(
          libc::getpid(),
          &raw const local,
          1,
          &raw const remote,
          1,
          0,
        )
      };
      if bytes_read >= 0 {
        return bytes_read as usize == output.len();
      }

      self.mem_fd = unsafe {
        // SAFETY: The path is a static NUL-terminated string.
        libc::open(c"/proc/self/mem".as_ptr(), libc::O_RDONLY | libc::O_CLOEXEC)
      };
      if self.mem_fd < 0 {
        self.mem_fd = -2;
      }
    }

    if self.mem_fd < 0 {
      return false;
    }

    let bytes_read = unsafe {
      // SAFETY: mem_fd is open and output is writable for output.len() bytes.
      libc::pread(
        self.mem_fd,
        output.as_mut_ptr().cast(),
        output.len(),
        address as libc::off_t,
      )
    };
    bytes_read >= 0 && bytes_read as usize == output.len()
  }
}

impl Drop for MemoryAccessor {
  fn drop(&mut self) {
    if self.mem_fd >= 0 {
      unsafe {
        // SAFETY: mem_fd is a file descriptor opened by MemoryAccessor.
        libc::close(self.mem_fd);
      }
    }
  }
}

struct FramePointerIter {
  fp: usize,
  stack: Option<Range<usize>>,
  memory: MemoryAccessor,
}

impl FramePointerIter {
  fn new(fp: usize) -> Self {
    Self {
      fp,
      stack: current_stack_bounds(),
      memory: MemoryAccessor::new(),
    }
  }
}

impl Iterator for FramePointerIter {
  type Item = usize;

  fn next(&mut self) -> Option<Self::Item> {
    let stack = self.stack.as_ref()?;
    let frame_end = self.fp.checked_add(2 * size_of::<usize>())?;
    if self.fp == 0
      || self.fp % align_of::<usize>() != 0
      || self.fp < stack.start
      || frame_end > stack.end
    {
      return None;
    }

    let parent_fp = self.memory.read_usize(self.fp)?;
    let return_address = self
      .memory
      .read_usize(self.fp.checked_add(size_of::<usize>())?)?;

    // The stack grows downward, so every caller's frame must be above its
    // callee. This also rejects self-linked and corrupt frame chains.
    if parent_fp != 0
      && (parent_fp <= self.fp
        || parent_fp % align_of::<usize>() != 0
        || parent_fp >= stack.end)
    {
      return None;
    }
    self.fp = parent_fp;

    Some(return_address)
  }
}

fn current_stack_bounds() -> Option<Range<usize>> {
  let mut attributes = MaybeUninit::<libc::pthread_attr_t>::uninit();
  if unsafe {
    // SAFETY: attributes points to writable storage and pthread_self returns
    // the calling thread's valid handle.
    libc::pthread_getattr_np(libc::pthread_self(), attributes.as_mut_ptr())
  } != 0
  {
    return None;
  }

  let mut attributes = unsafe {
    // SAFETY: pthread_getattr_np initialized attributes on its success path.
    attributes.assume_init()
  };
  let mut stack_start = std::ptr::null_mut();
  let mut stack_size = 0;
  let result = unsafe {
    // SAFETY: attributes was initialized above and both output pointers are
    // valid for writes.
    libc::pthread_attr_getstack(&attributes, &mut stack_start, &mut stack_size)
  };
  unsafe {
    // SAFETY: attributes was initialized and has not yet been destroyed.
    libc::pthread_attr_destroy(&mut attributes);
  }
  if result != 0 {
    return None;
  }

  let start = stack_start as usize;
  Some(start..start.checked_add(stack_size)?)
}

unsafe fn image_relative_address(address: usize) -> Option<usize> {
  struct Data {
    address: usize,
    relative: Option<usize>,
  }

  unsafe extern "C" fn callback(
    info: *mut libc::dl_phdr_info,
    _size: usize,
    opaque: *mut c_void,
  ) -> c_int {
    let data = unsafe { &mut *opaque.cast::<Data>() };
    let info = unsafe { &*info };
    let is_main_executable =
      info.dlpi_name.is_null() || unsafe { *info.dlpi_name.cast::<u8>() == 0 };
    if !is_main_executable {
      return 0;
    }
    let mut phdr = info.dlpi_phdr;
    let end = unsafe { phdr.add(info.dlpi_phnum as usize) };

    while phdr < end {
      let header = unsafe { &*phdr };
      if header.p_type == libc::PT_LOAD {
        let start = info.dlpi_addr.wrapping_add(header.p_vaddr) as usize;
        let end = start.saturating_add(header.p_memsz as usize);
        if data.address >= start && data.address < end {
          data.relative =
            Some(data.address.saturating_sub(info.dlpi_addr as usize));
          return 1;
        }
      }
      phdr = unsafe { phdr.add(1) };
    }
    0
  }

  let mut data = Data {
    address,
    relative: None,
  };
  unsafe {
    libc::dl_iterate_phdr(Some(callback), (&mut data as *mut Data).cast());
  }
  data.relative
}

pub fn trace() -> String {
  let mut encoded = Vec::new();
  let fp = frame_pointer();

  for return_address in FramePointerIter::new(fp).take(MAX_FRAMES) {
    // A return address points just after the call. Move it inside the calling
    // instruction so an address on a function boundary is symbolicated as the
    // caller rather than the next function.
    let address = return_address.saturating_sub(1);
    if let Some(relative) = unsafe { image_relative_address(address) } {
      vlq_encode(relative as i32, &mut encoded);
    }
  }

  unsafe { String::from_utf8_unchecked(encoded) }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn invalid_frame_pointer_stops_safely() {
    assert_eq!(FramePointerIter::new(1).next(), None);
    assert_eq!(FramePointerIter::new(usize::MAX & !7).next(), None);
  }

  #[test]
  fn trace_is_not_empty() {
    let trace = trace();
    println!("{trace}");
    assert!(!trace.is_empty());
  }
}
