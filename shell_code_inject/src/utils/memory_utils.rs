use std::ffi::c_void;
use windows::Win32::Foundation::{GetLastError, HANDLE};
use windows::Win32::System::Memory::{
    VirtualQueryEx, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READWRITE,
};
pub fn check_memory_protection(address: *const c_void, handle: HANDLE) {
    unsafe {
        let mut mbi: MEMORY_BASIC_INFORMATION = std::mem::zeroed();
        let result = VirtualQueryEx(
            handle,
            Some(address as *mut c_void),
            &mut mbi,
            size_of::<MEMORY_BASIC_INFORMATION>() as usize,
        );
        if result == 0 {
            eprintln!("VirtualQueryEx failed. Error: {:?}", GetLastError());
            return;
        }
        if mbi.Protect == PAGE_EXECUTE_READWRITE {
            println!(
                "Memory at address {:?} is executable and writable.",
                address
            );
        } else {
            println!(
                "Memory at address {:?} has protection flags: {:?}",
                address, mbi.Protect
            );
        }
    }
}
