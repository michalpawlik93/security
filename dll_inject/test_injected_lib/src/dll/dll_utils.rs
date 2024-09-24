use std::env;
use std::ffi::CString;
use std::path::PathBuf;
use windows::core::PCSTR;
use windows::Win32::Foundation::{GetLastError, HMODULE};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};

pub const DLL_PATH: &str = "src\\injected_lib.dll";
pub const KEY_LOGGER_DLL_PATH: &str = "src\\key_logger_lib.dll";
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

pub fn get_dll_path(path: &str) -> Result<CString, String> {
    let current_dir =
        env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    let dll_path: PathBuf = current_dir.join(path);

    let dll_path_str = dll_path
        .to_str()
        .ok_or_else(|| String::from("Failed to convert path to string"))?
        .replace("\\", "/");

    CString::new(dll_path_str).map_err(|_| String::from("Failed to create CString from path"))
}

pub fn allocate_and_write_dll_address(path: &str) -> Result<HMODULE, String> {
    let dll_path =
        get_dll_path(path).map_err(|_| String::from("Failed to create CString from path"))?;

    let address_result = unsafe {
        LoadLibraryA(PCSTR::from_raw(
            dll_path.as_bytes_with_nul().as_ptr() as *const u8
        ))
    };

    let address = address_result.map_err(|e| {
        format!(
            "LoadLibraryA failed {}, path: {}",
            e,
            dll_path.to_string_lossy()
        )
    })?;
    Ok(address)
}

#[cfg(test)]
mod dll_utils_tests {
    use super::*;

    #[test]
    fn test_allocate_and_write_dll_address_cstring_error() {
        let result = allocate_and_write_dll_address("invalid\0path");

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error, "Failed to create CString from path");
    }

    #[test]
    fn test_allocate_and_write_dll_address_success() {
        let result = allocate_and_write_dll_address(DLL_PATH);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_dll_success() {
        execute_dll(
            allocate_and_write_dll_address(DLL_PATH).unwrap(),
            DLL_ENTRY_POINT,
        );
    }
    #[test]
    #[should_panic(expected = "Dll not found")]
    fn test_execute_dll_panic() {
        execute_dll(
            allocate_and_write_dll_address(DLL_PATH).unwrap(),
            "not existing entry point",
        );
    }
}
