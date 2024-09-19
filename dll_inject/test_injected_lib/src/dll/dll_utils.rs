use std::env;
use std::ffi::CString;
use windows::core::PCSTR;
use windows::Win32::Foundation::{GetLastError, HMODULE};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};

pub const DLL_PATH: &str = "src\\injected_lib.dll";
pub const DLL_ENTRY_POINT: &str = "TestExport";

type MyFunction = unsafe extern "C" fn();

pub fn execute_dll(dll_address: HMODULE, entry_point: &str) {
    let func_name = CString::new(entry_point).unwrap();
    let func_name_ptr = func_name.as_bytes_with_nul().as_ptr() as *const u8;
    let func_nam_pcstr = PCSTR::from_raw(func_name_ptr);

    let func: MyFunction = unsafe {
        let addr_result = GetProcAddress(dll_address, func_nam_pcstr);
        let addr = match addr_result {
            Some(addr) => addr,
            None => {
                let error_code = GetLastError();
                panic!("Dll not found:{:?}", error_code);
            }
        };
        std::mem::transmute(addr)
    };

    unsafe {
        func();
    }
}

pub fn get_dll_path(path: &str) -> CString {
    let current_dir = env::current_dir().unwrap();
    let dll_path = current_dir.join(path);
    let dll_path_str = dll_path
        .to_str()
        .expect("Failed to convert path to string")
        .replace("\\", "/");

    println!("Resolved DLL Path: {}", dll_path_str);
    CString::new(dll_path_str).expect("Failed to create CString from path")
}

pub fn allocate_and_write_dll_address(path: &str) -> HMODULE {
    let dll_path = get_dll_path(path);
    let dll_path_ptr = dll_path.as_bytes_with_nul().as_ptr() as *const u8;
    let dll_path_pcstr = PCSTR::from_raw(dll_path_ptr);
    let address_result = unsafe { LoadLibraryA(dll_path_pcstr) };
    let address = match address_result {
        Ok(address) => address,
        Err(e) => {
            panic!(
                "LoadLibraryA failed {}, path: {}",
                e,
                dll_path.to_string_lossy()
            );
        }
    };
    address
}

#[cfg(test)]
mod injected_lib_tests {
    use super::*;

    #[test]
    #[should_panic(
        expected = "LoadLibraryA failed The specified module could not be found. (0x8007007E)"
    )]
    fn test_allocate_and_write_dll_address_panic() {
        allocate_and_write_dll_address("not existing path");
    }

    #[test]
    fn test_allocate_and_write_dll_address() {
        allocate_and_write_dll_address(DLL_PATH);
    }

    #[test]
    fn test_execute_dll() {
        execute_dll(allocate_and_write_dll_address(DLL_PATH), DLL_ENTRY_POINT);
    }
    #[test]
    #[should_panic(expected = "Dll not found")]
    fn test_execute_dll_panic() {
        execute_dll(
            allocate_and_write_dll_address(DLL_PATH),
            "not existing entry point",
        );
    }
}
