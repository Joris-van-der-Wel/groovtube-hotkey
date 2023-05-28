use std::env;
use msgbox::IconType;
use groovtube_hotkey::run;
use groovtube_hotkey::error::{error_msgbox, AppRunError, ConfigError};

#[cfg(target_os = "windows")]
fn hide_console_window() {
    // This function hides the console window if the program was started using explorer, but it
    // still lets us print if the program was started by cmd.

    use std::ptr;
    use winapi::um::wincon::GetConsoleWindow;
    use winapi::um::processthreadsapi::GetCurrentProcessId;
    use winapi::um::winuser::GetWindowThreadProcessId;
    use winapi::um::winuser::{ShowWindow, SW_HIDE};

    let console_window = unsafe {GetConsoleWindow()};
    if console_window == ptr::null_mut() {
        return;
    }

    #[allow(unused_assignments)]
        let mut my_pid: u32 = 0;
    unsafe { my_pid = GetCurrentProcessId(); }

    let mut console_window_pid: u32 = 0;
    unsafe { GetWindowThreadProcessId(console_window, &mut console_window_pid); }

    if my_pid == console_window_pid {
        unsafe { ShowWindow(console_window, SW_HIDE); }
    }
}

#[cfg(not(target_os = "windows"))]
fn hide_console_window() {
    // noop
}

fn main() -> Result<(), AppRunError> {
    println!(concat!("GroovTube Hotkey ", env!("CARGO_PKG_VERSION")));

    hide_console_window();

    let args = env::args();

    match run(args) {
        Err(AppRunError::ConfigError { source: ConfigError::CanNotLock { .. } }) => {
            msgbox::create(
                concat!("GroovTube Hotkey ", env!("CARGO_PKG_VERSION")),
                "This application has already been started",
                IconType::Error,
            ).expect("Could not create msgbox");
            Ok(())
        },
        Err(err) => {
            error_msgbox("Unexpected error", &err);
            Err(err)
        }
        Ok(_) => Ok(())
    }
}
