// Copyright 2025 Divy Srivastava <dj.srivastava23@gmail.com>
//
// Windows x86_64 and aarch64 stack unwinding support using RtlVirtualUnwind.

use crate::vlq::vlq_encode;

type WORD = u16;
type DWORD = u32;
type DWORDLONG = u64;
type HMODULE = *mut u8;
type BOOL = i32;

#[cfg(target_arch = "x86_64")]
#[repr(C)]
struct M128A {
  low: u64,
  high: i64,
}

#[cfg(target_arch = "x86_64")]
#[repr(C, align(16))]
struct CONTEXT {
  P1Home: DWORDLONG,
  P2Home: DWORDLONG,
  P3Home: DWORDLONG,
  P4Home: DWORDLONG,
  P5Home: DWORDLONG,
  P6Home: DWORDLONG,

  ContextFlags: DWORD,
  MxCsr: DWORD,

  SegCs: WORD,
  SegDs: WORD,
  SegEs: WORD,
  SegFs: WORD,
  SegGs: WORD,
  SegSs: WORD,
  EFlags: DWORD,

  Dr0: DWORDLONG,
  Dr1: DWORDLONG,
  Dr2: DWORDLONG,
  Dr3: DWORDLONG,
  Dr6: DWORDLONG,
  Dr7: DWORDLONG,

  Rax: DWORDLONG,
  Rcx: DWORDLONG,
  Rdx: DWORDLONG,
  Rbx: DWORDLONG,
  Rsp: DWORDLONG,
  Rbp: DWORDLONG,
  Rsi: DWORDLONG,
  Rdi: DWORDLONG,
  R8: DWORDLONG,
  R9: DWORDLONG,
  R10: DWORDLONG,
  R11: DWORDLONG,
  R12: DWORDLONG,
  R13: DWORDLONG,
  R14: DWORDLONG,
  R15: DWORDLONG,

  Rip: DWORDLONG,

  FltSave: [u8; 512],

  VectorRegister: [M128A; 26],
  VectorControl: DWORDLONG,

  DebugControl: DWORDLONG,
  LastBranchToRip: DWORDLONG,
  LastBranchFromRip: DWORDLONG,
  LastExceptionToRip: DWORDLONG,
  LastExceptionFromRip: DWORDLONG,
}

#[cfg(target_arch = "x86_64")]
impl CONTEXT {
  #[inline(always)]
  fn ip(&self) -> DWORDLONG {
    self.Rip
  }

  #[inline(always)]
  fn sp(&self) -> DWORDLONG {
    self.Rsp
  }
}

#[cfg(target_arch = "aarch64")]
#[repr(C)]
struct ARM64_NT_NEON128 {
  low: u64,
  high: i64,
}

#[cfg(target_arch = "aarch64")]
#[repr(C, align(16))]
struct CONTEXT {
  ContextFlags: DWORD,
  Cpsr: DWORD,
  X0: DWORDLONG,
  X1: DWORDLONG,
  X2: DWORDLONG,
  X3: DWORDLONG,
  X4: DWORDLONG,
  X5: DWORDLONG,
  X6: DWORDLONG,
  X7: DWORDLONG,
  X8: DWORDLONG,
  X9: DWORDLONG,
  X10: DWORDLONG,
  X11: DWORDLONG,
  X12: DWORDLONG,
  X13: DWORDLONG,
  X14: DWORDLONG,
  X15: DWORDLONG,
  X16: DWORDLONG,
  X17: DWORDLONG,
  X18: DWORDLONG,
  X19: DWORDLONG,
  X20: DWORDLONG,
  X21: DWORDLONG,
  X22: DWORDLONG,
  X23: DWORDLONG,
  X24: DWORDLONG,
  X25: DWORDLONG,
  X26: DWORDLONG,
  X27: DWORDLONG,
  X28: DWORDLONG,
  Fp: DWORDLONG,
  Lr: DWORDLONG,
  Sp: DWORDLONG,
  Pc: DWORDLONG,
  V: [ARM64_NT_NEON128; 32],
  Fpcr: DWORD,
  Fpsr: DWORD,
  Bcr: [DWORD; 8],
  Bvr: [DWORDLONG; 8],
  Wcr: [DWORD; 2],
  Wvr: [DWORDLONG; 2],
}

#[cfg(target_arch = "aarch64")]
impl CONTEXT {
  #[inline(always)]
  fn ip(&self) -> DWORDLONG {
    self.Pc
  }

  #[inline(always)]
  fn sp(&self) -> DWORDLONG {
    self.Sp
  }
}

extern "system" {
  fn GetModuleHandleExW(
    dwFlags: DWORD,
    name: *const u8,
    handle: *mut HMODULE,
  ) -> BOOL;
  fn RtlCaptureContext(r: *mut CONTEXT);
  fn RtlLookupFunctionEntry(
    ip: DWORDLONG,
    base: *mut DWORDLONG,
    hstable: *mut (),
  ) -> *mut ();
  fn RtlVirtualUnwind(
    ty: u32,
    base: DWORDLONG,
    ip: DWORDLONG,
    entry: *mut (),
    r: *mut CONTEXT,
    hnd_data: *mut *mut (),
    est_frame: *mut DWORDLONG,
    ctx_ptrs: *mut (),
  ) -> *mut ();
}

pub fn trace() -> String {
  let mut encoded = Vec::new();

  unsafe {
    let mut context = core::mem::zeroed::<CONTEXT>();
    RtlCaptureContext(&mut context);

    loop {
      let ip = context.ip();
      let sp = context.sp();
      let mut base = 0;
      let fn_entry =
        RtlLookupFunctionEntry(ip, &mut base, std::ptr::null_mut());
      if fn_entry.is_null() {
        break;
      }

      let addr = ip as usize;
      let mut handle = std::ptr::null_mut();
      const GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS: u32 = 0x4;
      GetModuleHandleExW(
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
        addr as _,
        &mut handle,
      );

      let addr = addr - handle as usize;
      vlq_encode(addr as i32, &mut encoded);

      let mut hnd_data = 0usize;
      let mut est_frame = 0;
      RtlVirtualUnwind(
        0,
        base,
        ip,
        fn_entry,
        &mut context,
        std::ptr::addr_of_mut!(hnd_data) as _,
        &mut est_frame,
        std::ptr::null_mut(),
      );

      // RtlVirtualUnwind indicates end of stack differently per arch:
      // - x86_64: sets instruction pointer to 0
      // - aarch64: leaves context unchanged (check if ip and sp are same)
      let new_ip = context.ip();
      if new_ip == 0 || (new_ip == ip && context.sp() == sp) {
        break;
      }
    }
  }

  // Safety: `encoded` is guaranteed to be valid UTF-8
  unsafe { String::from_utf8_unchecked(encoded) }
}
