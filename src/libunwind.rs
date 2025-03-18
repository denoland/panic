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
unsafe fn dyn_slide(addr: u64) -> usize {
    let cmd = getsegbyname(c"__TEXT".as_ptr() as _);
    addr as usize - (_dyld_get_image_vmaddr_slide(0) + (&*cmd).vmaddr as usize)
}

#[cfg(target_os = "linux")]
unsafe fn dyn_slide(addr: u64) -> u64 {
    use std::ffi::{c_int, c_void};

    struct Data {
        addr: u64,
        out: Option<u64>,
    }

    let mut data = Data { addr, out: None };

    unsafe extern "C" fn callback(
        info: *mut libc::dl_phdr_info,
        _size: usize,
        data: *mut c_void,
    ) -> c_int {
        let dlpi_addr = unsafe { *info }.dlpi_addr;
        let data = data.cast::<Data>();
        let addr = unsafe { (*data).addr };
        if addr < dlpi_addr {
            return 0;
        }
        let mut current = unsafe { (*info).dlpi_phdr };
        let end = current.add(unsafe { (*info).dlpi_phnum } as usize);
        while current < end {
            if unsafe { (*current).p_type != libc::PT_LOAD } {
                current = current.add(1);
                continue;
            }

            let segment_start = dlpi_addr.wrapping_add((*current).p_vaddr);
            let segment_end = segment_start + (*current).p_memsz;
            if addr >= segment_start && addr < segment_end {
                (*data).out = Some(addr.saturating_sub(dlpi_addr));
                return 1;
            }
            current = current.add(1);
        }
        0
    }

    unsafe { libc::dl_iterate_phdr(Some(callback), (&mut data) as *mut _ as _) };

    data.out.unwrap_or(0)
}

pub fn trace() -> String {
    let mut encoded = Vec::new();

    unsafe extern "C" fn trace_fn(ctx: *mut c_void, arg: *mut c_void) -> i32 {
        let ip = _Unwind_GetIP(ctx);

        #[cfg(not(target_os = "macos"))]
        let ip = _Unwind_FindEnclosingFunction(ip as *mut c_void) as usize;

        let stack_addr = dyn_slide(ip as u64);

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
