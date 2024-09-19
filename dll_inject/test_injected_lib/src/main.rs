use std::ffi::c_void;
use std::io;
use std::str::FromStr;
use windows::core::{PCSTR, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, GetLastError, WAIT_TIMEOUT};
use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::System::Memory::{
    VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE,
};
use windows::Win32::System::Threading::{
    CreateRemoteThread, OpenProcess, WaitForSingleObject, PROCESS_VM_OPERATION, PROCESS_VM_READ,
    PROCESS_VM_WRITE,
};
mod dll;

const KERNEL_32_DLL: &str = "Kernel32";
const LOAD_LIBRARY_A_FUNCTION_NAME: &str = "LoadLibraryA";

#[derive(Debug)]
enum ProgramMode {
    LoadDll,
    InjectToProcess,
}

fn main() {
    println!("Please enter program mode (LoadDll or InjectToProcess): ");

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    match input.trim().parse::<ProgramMode>() {
        Ok(mode) => {
            println!("You selected: {:?}", mode);
            match mode {
                ProgramMode::LoadDll => load_dll(),
                ProgramMode::InjectToProcess => inject_to_process(),
            }
        }
        Err(_) => println!("Invalid input, please enter 'LoadDll' or 'InjectToProcess'."),
    }
}

fn inject_to_process() {
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
    let dll_path = dll::dll_utils::get_dll_path(dll::dll_utils::DLL_PATH)
        .as_bytes_with_nul()
        .as_ptr() as *const u8;
    let target_process_handler = match unsafe {
        OpenProcess(
            PROCESS_VM_OPERATION | PROCESS_VM_WRITE | PROCESS_VM_READ,
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
    //save dll path to memory
    let dll_path_address = unsafe {
        VirtualAllocEx(
            target_process_handler,
            None,
            8, // size of dll_path
            MEM_RESERVE | MEM_COMMIT,
            PAGE_EXECUTE_READWRITE,
        )
    };
    if dll_path_address.is_null() {
        eprintln!("Cannot allocate memory. Error: {:?}", unsafe {
            GetLastError()
        });
        unsafe {
            let _ = CloseHandle(target_process_handler);
        };
        return;
    }
    let mut bytes_written: usize = 0;
    let _ = match unsafe {
        WriteProcessMemory(
            target_process_handler,
            dll_path_address,
            dll_path as *const c_void,
            8, // size of dll_path,
            Some(&mut bytes_written),
        )
    } {
        Ok(_) => {
            eprintln!("Bytes written: {:?}", bytes_written);
        }
        Err(_) => {
            eprintln!("Failed to write to process memory. Error: {:?}", unsafe {
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
    let kerenl32_ptr = PCWSTR::from_raw(KERNEL_32_DLL.as_ptr() as *const u16);
    let h_kernel32 = match unsafe { GetModuleHandleW(kerenl32_ptr) } {
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
    let load_library_a_ptr = PCSTR::from_raw(LOAD_LIBRARY_A_FUNCTION_NAME.as_ptr());

    let load_library_a_address = match unsafe { GetProcAddress(h_kernel32, load_library_a_ptr) } {
        Some(h) => h,
        None => {
            eprintln!("Can not get proc address: {}", LOAD_LIBRARY_A_FUNCTION_NAME);
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
    let wait_result = unsafe { WaitForSingleObject(thread_handler, 5000) };

    match wait_result {
        WAIT_TIMEOUT => eprintln!("Thread timed out."),
        _ => println!("Thread finished."),
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

fn load_dll() {
    let dll_address = dll::dll_utils::allocate_and_write_dll_address(dll::dll_utils::DLL_PATH);
    dll::dll_utils::execute_dll(dll_address, dll::dll_utils::DLL_ENTRY_POINT);
}

impl FromStr for ProgramMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "LoadDll" => Ok(ProgramMode::LoadDll),
            "InjectToProcess" => Ok(ProgramMode::InjectToProcess),
            _ => Err(()),
        }
    }
}
