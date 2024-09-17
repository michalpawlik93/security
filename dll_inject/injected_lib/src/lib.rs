use std::{env, fs, path::PathBuf, thread};
const DLL_PROCESS_ATTACH: u32 = 1;

#[no_mangle]
pub extern "system" fn TestExport() {
    save_file()
}

#[allow(non_snake_case)]
pub extern "system" fn DllMain(_: *mut u8, fdw_reason: u32, _: *mut u8) -> i32 {
    println!("DllMain");
    if fdw_reason == DLL_PROCESS_ATTACH {
        save_file()
    }
    1
}
fn save_file() {
    let handle = thread::spawn(move || {
        let current_dir: PathBuf = env::current_dir().unwrap();
        let mut log_path = current_dir.clone();
        println!("Path {}", log_path.to_str().unwrap());
        log_path.push("dll_log.txt");
        match fs::write(&log_path, "Log from TestExport\n") {
            Ok(_) => println!(
                "File written successfully to: {}",
                log_path.to_str().unwrap()
            ),
            Err(e) => println!("Failed to write file: {}", e),
        }
    });

    handle.join().expect("Thread panicked");
}
