use std::env::{current_exe};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use directories_next::{ProjectDirs};
use tokio::fs::{File};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use serde_json;
use fd_lock::{RwLock, RwLockWriteGuard};
use std::fs::OpenOptions;
use std::str;

use crate::config::types::Config;
use crate::error::ConfigError;

// creates a path to CONFIG_FILE_NAME in the same directory as the executable
// this could be useful for usb sticks
fn get_portable_config_path() -> Option<PathBuf> {
    match current_exe() {
        Ok(mut path) => {
            // F:\foo.exe => F:\foo.josn
            if !path.set_extension("json") {
                eprintln!("current exe has no filename: {}", path.to_string_lossy());
                return None
            }

            Some(path)
        },
        Err(err) => {
            eprintln!("failed to get current exe path: {:?}", err);
            None
        },
    }
}

// creates a path to groovtube-hotkey.config.json in an os dependent standard directory, such as %AppData% on
// windows.
fn get_local_config_path() -> Option<PathBuf> {
    ProjectDirs::from("nl", "groovtube", "groovtube-hotkey").map(|dirs| {
        dirs.config_dir().join("groovtube-hotkey.json")
    })
}

fn get_config_path() -> Result<PathBuf, ConfigError> {
    let portable = get_portable_config_path();
    if let Some(path) = portable {
        let attr = std::fs::metadata(&path);
        match attr {
            Ok(attr) => {
                if attr.is_file() {
                    return Ok(path);
                }
            }
            Err(err) => {
                eprintln!("Could not read metadata of: {}; Using local path instead. ({:?})", path.to_string_lossy(), err);
            },
        }

    }

    match get_local_config_path() {
        None => Err(ConfigError::NoConfigPath),
        Some(path) => Ok(path),
    }
}

pub struct ConfigIOLocker {
    rw_lock: RwLock<std::fs::File>,
}

impl ConfigIOLocker {
    pub fn lock(&mut self) -> Result<RwLockWriteGuard<std::fs::File>, ConfigError> {
        match self.rw_lock.try_write() {
            Ok(guard) => Ok(guard),
            Err(source) =>{
                return Err(ConfigError::CanNotLock { source });
            },
        }
    }
}

struct ConfigIOInner {
    file: std::fs::File,
}

#[derive(Clone)]
pub struct ConfigIO {
    inner: Arc<Mutex<ConfigIOInner>>,
}

impl ConfigIO {
    pub fn new_sync() -> Result<Self, ConfigError> {
        let path = get_config_path()?;
        println!("Using config file {}", path.to_string_lossy());

        let directory = path.parent().expect("Failed to determine parent path of config path");
        std::fs::create_dir_all(directory)?;

        // obtain an exclusive file lock so that this config file is used by only one instance of
        // this application.
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
            .append(false)
            .create(true)
            .open(path)?;

        let inner = ConfigIOInner {
            file,
        };
        Ok(ConfigIO { inner: Arc::new(Mutex::new(inner)) })
    }

    pub fn locker(&mut self) -> Result<ConfigIOLocker, ConfigError> {
        let inner = self.inner.lock().expect("Failed to lock ConfigIO inner");

        Ok(ConfigIOLocker {
            rw_lock: RwLock::new(inner.file.try_clone()?),
        })
    }

    // The File returned from here should never be closed!
    fn get_file(&self) -> Result<File, ConfigError> {
        let inner = self.inner.lock().expect("Failed to lock ConfigIO inner");
        let file = inner.file.try_clone()?; // std File
        Ok(File::from_std(file)) // tokio File
    }

    pub async fn read(&self) -> Result<Config, ConfigError> {
        let mut file = self.get_file()?;
        println!("Reading config file");

        let mut content = vec![];
        file.read_to_end(&mut content).await?;

        if content.is_empty() {
            return Ok(Config::default());
        }

        let content = str::from_utf8(&content)?;

        let mut config: Config = serde_json::from_str(content)?;
        config.sort_hotkeys();
        Ok(config)
    }

    pub async fn save(&self, config: Config) -> Result<(), ConfigError> {
        let mut file = self.get_file()?;
        println!("Saving config");

        let content = serde_json::to_string_pretty(&config)?;
        file.rewind().await?;
        file.set_len(0).await?;
        file.write_all(content.as_bytes()).await?;
        file.flush().await?;
        Ok(())
    }
}
