// Copyright 2025 Divy Srivastava <dj.srivastava23@gmail.com>
//
// https://github.com/littledivy/resym/blob/59db8fce68e8cf786b22dd37979afec70ddc363d/src/win64.rs

use crate::vlq::vlq_encode;

type WORD = u16;
type DWORD = u32;
type DWORDLONG = u64;
type HMODULE = *mut u8;
type BOOL = i32;

#[repr(C)]
struct M128A {
  low: u64,
  high: i64,
}

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
      let ip = context.Rip;
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
      if context.Rip == 0 {
        break;
      }
    }
  }

  // Safety: `encoded` is guaranteed to be valid UTF-8
  unsafe { String::from_utf8_unchecked(encoded) }
}
