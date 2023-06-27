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

pub fn run(mut _args: env::Args) -> Result<(), AppRunError> {
    run_application()?;
    Ok(())
}
