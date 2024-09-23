use std::{env, fs, path::PathBuf, thread};
const DLL_PROCESS_ATTACH: u32 = 1;
const LOG_DESTINATION_PATH: &str = "E:\\Nauka\\security";

#[no_mangle]
pub extern "system" fn TestExport() {
    println!("Hello from thread");
    let current_dir: PathBuf = env::current_dir().unwrap();
    let handle = thread::spawn(move || save_file(current_dir));

    handle.join().expect("Thread panicked");
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn DllMain(_: *mut u8, fdw_reason: u32, _: *mut u8) -> i32 {
    println!("DllMain");
    if fdw_reason == DLL_PROCESS_ATTACH {
        println!("Hello from attach");
        let fixed_dir = PathBuf::from(LOG_DESTINATION_PATH);
        save_file(fixed_dir);
    }
    1
}
fn save_file(dir: PathBuf) {
    let mut log_path = dir.clone();
    println!("Path {}", log_path.to_str().unwrap());
    log_path.push("dll_log.txt");
    match fs::write(&log_path, "Log from Injected Lib\n") {
        Ok(_) => println!(
            "File written successfully to: {}",
            log_path.to_str().unwrap()
        ),
        Err(e) => println!("Failed to write file: {}", e),
    }
}
