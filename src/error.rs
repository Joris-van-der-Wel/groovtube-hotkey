use std::any::Any;
use std::io;
use thiserror::Error;
use msgbox::IconType;
use std::fmt::{Debug, Display};
use std::str::Utf8Error;
use btleplug;
use iced;
use serde_json;
use futures::channel::mpsc::SendError;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to determine path to config file")]
    NoConfigPath,

    #[error("Failed to acquire file lock on config file: {source}")]
    CanNotLock { source: io::Error },

    #[error("Failed to encode/decode config as utf-8: {source}")]
    Utf8Error { #[from] source: Utf8Error },

    #[error("Failed to read/write config file: {source}")]
    IOError { #[from] source: io::Error },

    #[error("Failed to parse/build config file: {source}")]
    JsonError { #[from] source: serde_json::Error },

    #[error("Failed to send config over mpsc channel: {source}")]
    SendError { #[from] source: SendError },
}

impl ConfigError {
    pub fn is_file_not_found_error(&self) -> bool {
        match self {
            ConfigError::IOError { source } => source.kind() == io::ErrorKind::NotFound,
            _ => false,
        }
    }
}

#[derive(Error, Debug)]
pub enum AppRunError {
    #[error("Failed to start application (iced): {source}")]
    Iced { #[from] source: iced::Error },

    #[error("Failed to start application (config): {source}")]
    ConfigError { #[from] source: ConfigError },
}

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("Error communicating with device (btleplug): {source}")]
    Btle { #[from] source: btleplug::Error },

    #[error("A required bluetooth characteristic is not available")]
    MissingCharacteristic,
}

pub fn readable_thread_panic_error(error: &Box<dyn Any + Send + 'static>) -> String {
    let mut stringified = String::from("???");

    if let Some(s) = error.downcast_ref::<&str>() {
        stringified = format!("{}", s);
    }
    else if let Some(s) = error.downcast_ref::<String>() {
        stringified = format!("{}", s);
    }
    let type_id = error.type_id();

    format!("panic from thread: [{:?}]: [{}]", type_id, stringified)
}

pub fn error_msgbox<T: Display>(message: &'static str, error: &T) {
    let message = format!("{}: {}", message, error);
    eprintln!("{}", &message);
    if let Err(err) = msgbox::create(concat!("GroovTube Hotkey ", env!("CARGO_PKG_VERSION")), &message, IconType::Error) {
        eprintln!("Failed to create msgbox: {:?}", err);
    }
}
