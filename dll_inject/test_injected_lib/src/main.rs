use std::env;
use std::ffi::CString;
use std::path::PathBuf;
use windows::core::PCSTR;
use windows::Win32::Foundation::{GetLastError, HMODULE};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};

type MyFunction = unsafe extern "C" fn();
const DLL_PATH: &str = "src/injected_lib.dll";
const DLL_ENTRY_POINT: &str = "TestExport";

fn main() {
    let dll_address = get_dll_address(DLL_PATH);
    execute_dll(dll_address, DLL_ENTRY_POINT);
}

fn execute_dll(dll_address: HMODULE, entry_point: &str) {
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

fn get_dll_address(path: &str) -> HMODULE {
    let current_dir: PathBuf = env::current_dir().unwrap();
    let mut dll_path = current_dir.clone();
    dll_path.push(path);

    let dll_path_c = CString::new(dll_path.to_str().unwrap()).unwrap();
    let dll_path_ptr = dll_path_c.as_bytes_with_nul().as_ptr() as *const u8;
    let dll_path_pcstr = PCSTR::from_raw(dll_path_ptr);
    let address_result = unsafe { LoadLibraryA(dll_path_pcstr) };
    let address = match address_result {
        Ok(address) => address,
        Err(e) => {
            panic!("LoadLibraryA failed {}", e);
        }
    };
    if address_result.is_err() {
        let error_code = unsafe { GetLastError() };
        panic!("LoadLibraryA failed with error code: {:?}", error_code);
    }
    address
}

#[cfg(test)]
mod injected_lib_tests {
    use super::*;

    #[test]
    #[should_panic(
        expected = "LoadLibraryA failed The specified module could not be found. (0x8007007E)"
    )]
    fn test_get_dll_address_panic() {
        get_dll_address("not existing path");
    }

    #[test]
    fn test_get_dll_address() {
        get_dll_address(DLL_PATH);
    }

    #[test]
    fn test_execute_dll() {
        execute_dll(get_dll_address(DLL_PATH), DLL_ENTRY_POINT);
    }
    #[test]
    #[should_panic(expected = "Dll not found")]
    fn test_execute_dll_panic() {
        execute_dll(get_dll_address(DLL_PATH), "not existing entry point");
    }
}
