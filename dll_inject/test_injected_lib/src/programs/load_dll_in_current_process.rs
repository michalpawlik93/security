use crate::dll;

pub fn call_dll_exported_function() {
    let dll_address = match dll::dll_utils::allocate_and_write_dll_address(dll::dll_utils::DLL_PATH)
    {
        Ok(p) => p,
        Err(e) => {
            panic!("{}", e)
        }
    };
    dll::dll_utils::execute_dll(dll_address, dll::dll_utils::DLL_ENTRY_POINT);
}

pub fn call_dll_main() {
    match dll::dll_utils::allocate_and_write_dll_address(dll::dll_utils::DLL_PATH) {
        Ok(p) => p,
        Err(e) => {
            panic!("{}", e)
        }
    };
}
