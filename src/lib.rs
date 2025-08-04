#[cfg(target_os = "macos")]
mod mac;
#[cfg(any(target_os = "linux", target_os = "android"))]
mod unix;
#[cfg(target_os = "windows")]
mod windows;

use std::error::Error;

type AnyResult<T> = Result<T, Box<dyn Error>>;

#[derive(thiserror::Error, Debug)]
pub enum SingleError {
    #[error("invalid check single!")]
    InvalidCheck,
}

trait SingleHandler
where
    Self: Drop,
{}

pub struct Single {
    pub key: String,
    pub is_single: bool,
    _handler: Option<Box<dyn SingleHandler>>,
}

impl Single {
    /// 根据key新建实例.
    pub fn new(key: &str) -> AnyResult<Self> {
        #[cfg(target_os = "windows")]
        let option: Option<Box<dyn SingleHandler>> = windows::new(key)?;
        #[cfg(target_os = "macos")]
        let option = mac::new(key)?;
        #[cfg(any(target_os = "linux", target_os = "android"))]
        let option = unix::new(key)?;

        Ok(Single {
            key: key.to_string(),
            is_single: option.is_some(),
            _handler: option,
        })
    }
}
