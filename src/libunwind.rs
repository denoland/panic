use std::ffi::c_void;
use std::ptr::addr_of_mut;

use crate::vlq::vlq_encode;

type TraceFn = unsafe extern "C" fn(ctx: *mut c_void, arg: *mut c_void) -> i32;

extern "C" {
    fn _Unwind_Backtrace(trace: TraceFn, trace_argument: *mut c_void) -> i32;

    fn _Unwind_GetIP(ctx: *mut c_void) -> usize;

    fn _Unwind_FindEnclosingFunction(pc: *mut c_void) -> *mut c_void;

    #[cfg(target_os = "macos")]
    fn getsegbyname(name: *const u8) -> *const SegmentCommand64;
    #[cfg(target_os = "macos")]
    fn _dyld_get_image_vmaddr_slide(image_index: usize) -> usize;
}

#[cfg(target_os = "macos")]
#[repr(C)]
struct SegmentCommand64 {
    cmd: u32,
    cmdsize: u32,
    segname: [u8; 16],
    vmaddr: u64,
    vmsize: u64,
    fileoff: u64,
    filesize: u64,
    maxprot: u32,
    initprot: u32,
    nsects: u32,
    flags: u32,
}

#[cfg(target_os = "macos")]
unsafe fn dyn_slide() -> usize {
    let cmd = getsegbyname(c"__TEXT".as_ptr() as _);
    _dyld_get_image_vmaddr_slide(0) + (&*cmd).vmaddr as usize
}

pub fn trace() -> String {
    let mut encoded = Vec::new();

    unsafe extern "C" fn trace_fn(ctx: *mut c_void, arg: *mut c_void) -> i32 {
        let ip = _Unwind_GetIP(ctx);

        #[cfg(not(target_os = "macos"))]
        let ip = _Unwind_FindEnclosingFunction(ip as *mut c_void) as usize;

        let stack_addr = ip - dyn_slide();

        let encoded = &mut *(arg as *mut Vec<u8>);
        vlq_encode(stack_addr as i32, encoded);

        0
    }

    unsafe {
        _Unwind_Backtrace(trace_fn, addr_of_mut!(encoded).cast());
    }

    unsafe { String::from_utf8_unchecked(encoded) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace() {
        let trace = trace();
        println!("{}", trace);
    }
}
