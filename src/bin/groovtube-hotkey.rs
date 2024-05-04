use std::env;
use log::info;
use msgbox::IconType;
use groovtube_hotkey::{init_logging, run};
use groovtube_hotkey::error::{error_msgbox, AppRunError, ConfigError};

// This embedded Info.plist is used when launching the binary directly, instead of the app bundle.
// Example: `open ./target/debug/groovtube-hotkey`
#[cfg(all(target_os = "macos"))]
embed_plist::embed_info_plist!(concat!(env!("OUT_DIR"), "/Info.plist"));

#[cfg(target_os = "windows")]
fn windows_init() {
    groovtube_hotkey::os::windows::hide_console_window();
}

#[cfg(not(target_os = "windows"))]
fn windows_init() {}


#[cfg(target_os = "macos")]
fn macos_init() {
    if !groovtube_hotkey::os::macos::check_accessibility_access(true) {
        msgbox::create(
            concat!("GroovTube Hotkey ", env!("CARGO_PKG_VERSION")),
            "This application translates human breath input to mouse and keyboard hotkeys. \
            To send mouse and keyboard hotkeys on macOS, \"accessibility\" access is required.

            Currently, this application does NOT have access to accessibility.

            This problem can be remedied by opening the \"System Settings\" app, navigating to \
            \"Privacy & Security\", and then \"Accessibility\". Add a checkmark next to \
            \"GroovTubeHotkey\", and then restart the application.

            If this does not work, remove the application from the list using the minus button, \
            restart the application, and repeat all the steps. \
            ",
            IconType::Error,
        ).expect("Could not create msgbox");
    }
}

#[cfg(not(target_os = "macos"))]
fn macos_init() {}



fn main() -> Result<(), AppRunError> {
    init_logging();
    info!(concat!("GroovTube Hotkey ", env!("CARGO_PKG_VERSION")));

    windows_init();
    macos_init();

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
