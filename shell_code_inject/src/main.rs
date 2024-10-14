mod utils;
use std::ffi::c_void;
use std::mem::transmute;
use std::process;
use windows::Win32::Foundation::{CloseHandle, GetLastError, WAIT_TIMEOUT};
use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows::Win32::System::Memory::{
    VirtualAllocEx, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE,
};
use windows::Win32::System::Threading::{
    CreateRemoteThread, OpenProcess, WaitForSingleObject, PROCESS_VM_OPERATION, PROCESS_VM_READ,
    PROCESS_VM_WRITE,
};

fn main() {
    let target_process_id = process::id();
    println!("My pid is {}", target_process_id);

    let handler_result = unsafe {
        OpenProcess(
            PROCESS_VM_OPERATION | PROCESS_VM_WRITE | PROCESS_VM_READ,
            false,
            target_process_id,
        )
    };

    match handler_result {
        Ok(h_process_1) => {
            eprintln!("Process opened");

            let alloc_mem_address = unsafe {
                VirtualAllocEx(
                    h_process_1,
                    None,
                    1024,
                    MEM_RESERVE | MEM_COMMIT,
                    PAGE_EXECUTE_READWRITE,
                )
            };
            if alloc_mem_address.is_null() {
                eprintln!("Cannot allocate memory. Error: {:?}", unsafe {
                    GetLastError()
                });
                return;
            }
            eprintln!("Memory allocated at address: {:?}", alloc_mem_address);
            utils::memory_utils::check_memory_protection(alloc_mem_address, h_process_1);

            let shellcode: [u8; 11] = [
                0xB8, 0x01, 0x00, 0x00, 0x00, // MOV EAX, 0x00000001
                0x89, 0xC1, // MOV ECX, EAX
                0x89, 0xC2, // MOV EDX, EAX
                0x90, // NOP
                0xC3, // RET
            ];

            let mut bytes_written: usize = 0;
            let write_result = unsafe {
                WriteProcessMemory(
                    h_process_1,
                    alloc_mem_address,
                    shellcode.as_ptr() as *const c_void,
                    shellcode.len(),
                    Some(&mut bytes_written),
                )
            };

            if write_result.is_err() {
                eprintln!("Failed to write to memory. Error: {:?}", unsafe {
                    GetLastError()
                });
                return;
            }
            eprintln!("Bytes written: {:?}", bytes_written);

            //let func: extern "system" fn() = unsafe { std::mem::transmute(alloc_mem_address) };
            //func();
            let thread_entry_point = unsafe {
                transmute::<*mut c_void, extern "system" fn(*mut c_void) -> u32>(alloc_mem_address)
            };

            let mut thread_id: u32 = 0;
            let h_thread_result = unsafe {
                CreateRemoteThread(
                    h_process_1,
                    None,
                    0,
                    Some(thread_entry_point),
                    None,
                    0,
                    Some(&mut thread_id),
                )
            };

            match h_thread_result {
                Ok(h_thread) => {
                    if h_thread.is_invalid() {
                        eprintln!("Cannot create thread. Error: {:?}", unsafe {
                            GetLastError()
                        });
                        return;
                    }

                    let wait_result = unsafe { WaitForSingleObject(h_thread, 5000) };

                    println!("Shellcode injected and remote thread created successfully.");

                    match wait_result {
                        WAIT_TIMEOUT => eprintln!("Thread timed out."),
                        _ => println!("Thread finished."),
                    }

                    let close_h1_result = unsafe { CloseHandle(h_process_1) };

                    match close_h1_result {
                        Ok(_) => {
                            eprintln!("Handler closed.")
                        }
                        Err(err) => {
                            eprintln!("Failed to close handler. Error: {:?}", err);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Failed to create remote thread. Error: {:?}", err);
                }
            }
        }
        Err(err) => eprintln!("Cannot open process. Error: {:?}", err),
    }
}
