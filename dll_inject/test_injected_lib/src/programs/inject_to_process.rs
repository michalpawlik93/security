use std::ffi::{c_void, CString};
use std::io;
use windows::core::{PCSTR, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, GetLastError, HANDLE, WAIT_TIMEOUT};
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::System::Memory::{
    VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE,
};
use windows::Win32::System::Threading::{
    CreateRemoteThread, GetExitCodeThread, OpenProcess, WaitForSingleObject, PROCESS_CREATE_THREAD,
    PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
};

use crate::dll;

const KERNEL_32_DLL: &str = "kernel32";
const LOAD_LIBRARY_A_FUNCTION_NAME: &str = "LoadLibraryA";

pub fn inject_dll() {
    println!("Please enter process PID:");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let pid = match input.trim().parse::<u32>() {
        Ok(p) => p,
        Err(err) => {
            eprintln!("PID parsing error: {:?}", err);
            return;
        }
    };
    let target_process_handler = match unsafe {
        OpenProcess(
            PROCESS_VM_OPERATION
                | PROCESS_VM_WRITE
                | PROCESS_VM_READ
                | PROCESS_CREATE_THREAD
                | PROCESS_QUERY_INFORMATION,
            false,
            pid,
        )
    } {
        Ok(h) => h,
        Err(er) => {
            eprintln!("Cannot open proccess. Error: {:?}", er);
            return;
        }
    };
    let dll_path = match dll::dll_utils::get_dll_path(dll::dll_utils::DLL_PATH) {
        Ok(p) => p,
        Err(_) => {
            unsafe {
                let _ = CloseHandle(target_process_handler);
            };
            return;
        }
    };
    println!(
        "Resolved DLL Path from cstrng: {}",
        dll_path.to_str().unwrap()
    );
    let dll_path_address =
        if let Some(address) = save_dll_path_to_memory(target_process_handler, dll_path) {
            address
        } else {
            unsafe {
                let _ = CloseHandle(target_process_handler);
            };
            return;
        };

    // Read dll path from memory address to confirm valid path name
    match read_string_from_memory_address(target_process_handler, dll_path_address) {
        Ok(contents) => println!("Contents as string: {}", contents),
        Err(error) => {
            eprintln!("{}", error);
            unsafe {
                let _ = CloseHandle(target_process_handler);
            };
            unsafe {
                let _ = VirtualFreeEx(target_process_handler, dll_path_address, 0, MEM_RELEASE);
            };
            return;
        }
    };

    let module_name_wide: Vec<u16> = KERNEL_32_DLL
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let h_kernel32 = match unsafe { GetModuleHandleW(PCWSTR::from_raw(module_name_wide.as_ptr())) }
    {
        Ok(h) => h,
        Err(_) => {
            eprintln!("Failed to write to load kernel32. Error: {:?}", unsafe {
                GetLastError()
            });
            unsafe {
                let _ = VirtualFreeEx(target_process_handler, dll_path_address, 0, MEM_RELEASE);
            };
            unsafe {
                let _ = CloseHandle(target_process_handler);
            };
            return;
        }
    };

    let load_library_function_name = CString::new(LOAD_LIBRARY_A_FUNCTION_NAME).unwrap();
    let load_library_a_address = match unsafe {
        GetProcAddress(
            h_kernel32,
            PCSTR::from_raw(load_library_function_name.as_ptr() as *const u8),
        )
    } {
        Some(h) => h,
        None => {
            eprintln!(
                "Can not get proc address: {}. Error:{:?}",
                LOAD_LIBRARY_A_FUNCTION_NAME,
                unsafe { GetLastError() }
            );
            unsafe {
                let _ = VirtualFreeEx(target_process_handler, dll_path_address, 0, MEM_RELEASE);
            };
            unsafe {
                let _ = CloseHandle(target_process_handler);
            };
            return;
        }
    };

    let load_library_func: Option<unsafe extern "system" fn(*mut c_void) -> u32> =
        unsafe { Some(std::mem::transmute(load_library_a_address)) };

    let thread_handler = match unsafe {
        CreateRemoteThread(
            target_process_handler,
            None,
            0,
            load_library_func,
            Some(dll_path_address),
            0,
            None,
        )
    } {
        Ok(h) => h,
        Err(_) => {
            eprintln!("Failed to create thread. Error: {:?}", unsafe {
                GetLastError()
            });
            unsafe {
                let _ = VirtualFreeEx(target_process_handler, dll_path_address, 0, MEM_RELEASE);
            };
            unsafe {
                let _ = CloseHandle(target_process_handler);
            };
            return;
        }
    };
    let wait_result = unsafe { WaitForSingleObject(thread_handler, 10000) };

    match wait_result {
        WAIT_TIMEOUT => eprintln!("Thread timed out."),
        _ => {
            let mut exit_code = 0;
            unsafe {
                let _ = GetExitCodeThread(thread_handler, &mut exit_code);
            }
            println!("Thread finished with exit code: {}", exit_code);
        }
    }

    unsafe {
        let _ = CloseHandle(thread_handler);
    };
    unsafe {
        let _ = VirtualFreeEx(target_process_handler, dll_path_address, 0, MEM_RELEASE);
    };
    unsafe {
        let _ = CloseHandle(target_process_handler);
    };
}

fn read_string_from_memory_address(
    target_process_handler: HANDLE,
    dll_path_address: *mut c_void,
) -> Result<String, String> {
    let mut buffer = vec![0u8; 1024];
    let mut bytes_read = 0;

    let success = unsafe {
        ReadProcessMemory(
            target_process_handler,
            dll_path_address,
            buffer.as_mut_ptr() as *mut c_void,
            buffer.len(),
            Some(&mut bytes_read),
        )
    };

    match success {
        Ok(_) => {
            let output_string =
                String::from_utf8_lossy(&buffer[..bytes_read as usize]).into_owned();
            Ok(output_string)
        }
        Err(err) => Err(format!("Failed to read memory. Error: {:?}", err)),
    }
}

fn save_dll_path_to_memory(
    target_process_handler: HANDLE,
    dll_path_cstring: CString,
) -> Option<*mut c_void> {
    let dll_path = dll_path_cstring.as_bytes_with_nul();

    let dll_path_address = unsafe {
        VirtualAllocEx(
            target_process_handler,
            None,
            dll_path.len(),
            MEM_RESERVE | MEM_COMMIT,
            PAGE_EXECUTE_READWRITE,
        )
    };

    if dll_path_address.is_null() {
        eprintln!("Cannot allocate memory. Error: {:?}", unsafe {
            GetLastError()
        });
        return None;
    }

    let mut bytes_written: usize = 0;
    let write_result = unsafe {
        WriteProcessMemory(
            target_process_handler,
            dll_path_address,
            dll_path.as_ptr() as *const c_void,
            dll_path.len(),
            Some(&mut bytes_written),
        )
    };

    match write_result {
        Ok(_) => {
            if bytes_written != dll_path.len() {
                eprintln!(
                    "Incomplete write: only {} out of {} bytes written",
                    bytes_written,
                    dll_path.len()
                );
                unsafe {
                    let _ = VirtualFreeEx(target_process_handler, dll_path_address, 0, MEM_RELEASE);
                }
                return None;
            }

            println!("Bytes written: {}", bytes_written);
            Some(dll_path_address)
        }
        Err(_) => {
            eprintln!("Failed to write to process memory. Error: {:?}", unsafe {
                GetLastError()
            });

            unsafe {
                let _ = VirtualFreeEx(target_process_handler, dll_path_address, 0, MEM_RELEASE);
            }
            None
        }
    }
}
