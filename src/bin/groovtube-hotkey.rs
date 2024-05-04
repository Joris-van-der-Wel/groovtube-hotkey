#![windows_subsystem = "windows"]

use std::env;
use log::info;
use msgbox::IconType;
use groovtube_hotkey::{init_logging, run};
use groovtube_hotkey::error::{error_msgbox, AppRunError, ConfigError};

// This embedded Info.plist is used when launching the binary directly, instead of the app bundle.
// Example: `open ./target/debug/groovtube-hotkey`
#[cfg(all(target_os = "macos"))]
embed_plist::embed_info_plist!(concat!(env!("OUT_DIR"), "/Info.plist"));

fn main() -> Result<(), AppRunError> {
    init_logging();
    info!(concat!("GroovTube Hotkey ", env!("CARGO_PKG_VERSION")));

    let args = env::args();

    // msgbox should not be used before run() otherwise winit will crash on macOS
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
