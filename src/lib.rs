use std::env;
use crate::gui::application::run_application;
use crate::error::AppRunError;

pub mod device;
pub mod gui;
pub mod sim;
pub mod error;
pub mod resources;
pub mod config;
pub mod os;

pub fn init_logging() {
    let mut dispatch = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339(std::time::SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stderr());

    if let Ok(log_file) = env::var("LOG_FILE") {
        dispatch = dispatch.chain(
            fern::log_file(log_file).expect("Failed to open LOG_FILE")
        );
    }
    
    dispatch.apply().expect("Failed to initialize logger");

}

pub fn run(mut _args: env::Args) -> Result<(), AppRunError> {
    run_application()?;
    Ok(())
}
