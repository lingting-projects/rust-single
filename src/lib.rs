mod core;
#[cfg(feature = "ipc")]
mod ipc;

use crate::core::{try_unique, write};
use fs2::FileExt;
use std::error::Error;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::Read;
use std::path::Path;
use std::process;
use std::sync::Arc;

#[derive(thiserror::Error, Debug)]
pub enum SingleError {
    #[error("pid and info write err!")]
    SingleWrite,
}

type AnyResult<T> = Result<T, Box<dyn Error>>;

pub struct Single {
    pub is_single: bool,
    pub pid: Option<u32>,
    pub info: String,
    pub path: String,
    pub path_info: String,
    #[cfg(feature = "ipc")]
    pub path_ipc: String,
    #[cfg(feature = "ipc")]
    on_wake: Option<Arc<Box<dyn Fn(AnyResult<Vec<u8>>) + Send + Sync>>>,
    file: Option<File>,
}

impl Single {
    fn create(build: SingleBuild) -> AnyResult<Single> {
        let path_lock = Path::new(&build.path);
        if let Some(parent) = path_lock.parent() {
            create_dir_all(parent)?;
        }
        let path_info = format!("{}.info", build.path);
        let path_ipc = format!("{}.ipc", build.path);

        match try_unique(path_lock) {
            Ok(lock) => {
                let pid = process::id();
                let context = format!("{}\\n{}", pid, &build.info);

                match write(&path_info, context) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("write pid and info to {} err! {}", path_info, e);
                        return Err(Box::new(SingleError::SingleWrite));
                    }
                }

                let mut single = Single {
                    is_single: true,
                    pid: Some(pid),
                    info: build.info,
                    path: build.path,
                    path_info,
                    #[cfg(feature = "ipc")]
                    path_ipc,
                    #[cfg(feature = "ipc")]
                    on_wake: build.on_wake,
                    file: Some(lock),
                };

                #[cfg(feature = "ipc")]
                {
                    single = core::ipc_create(single)?;
                }

                return Ok(single);
            }
            Err(e) => {
                log::error!("获取独占锁异常! {}", e)
            }
        }

        let mut options = OpenOptions::new();
        options.read(true);

        let mut file = options.open(&path_info)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let (pid_str, info) = content.split_once('\n').unwrap_or(("0", &content));
        let pid = pid_str.parse::<u32>().ok();

        let single = Single {
            is_single: false,
            pid,
            info: info.to_string(),
            path: build.path,
            path_info,
            #[cfg(feature = "ipc")]
            path_ipc,
            #[cfg(feature = "ipc")]
            on_wake: None,
            file: None,
        };

        Ok(single)
    }

    #[cfg(feature = "ipc")]
    pub fn wake(&self, content: &str) -> AnyResult<()> {
        let bytes = content.as_bytes();
        self.wake_bytes(bytes)
    }

    #[cfg(feature = "ipc")]
    pub fn wake_bytes(&self, bytes: &[u8]) -> AnyResult<()> {
        use crate::ipc::IpcStream;
        let mut stream = IpcStream::new(&self.path_ipc)?;
        stream.write_bytes(bytes)
    }
}

impl Drop for Single {
    fn drop(&mut self) {
        let option = self.file.take();
        if let Some(file) = option {
            let _ = FileExt::unlock(&file);
            drop(file);
        }
    }
}

#[derive(Default)]
pub struct SingleBuild {
    pub info: String,
    pub path: String,
    #[cfg(feature = "ipc")]
    pub on_wake: Option<Arc<Box<dyn Fn(AnyResult<Vec<u8>>) + Send + Sync>>>,
}

impl SingleBuild {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let mut s = SingleBuild::default();
        s.path = path
            .as_ref()
            .to_str()
            .expect("failed get single path")
            .to_string();
        s
    }

    pub fn with_info(mut self, info: String) -> Self {
        self.info = info;
        self
    }

    #[cfg(feature = "ipc")]
    pub fn with_ipc<F: Fn(AnyResult<Vec<u8>>) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_wake = Some(Arc::new(Box::new(f)));
        self
    }

    pub fn build(self) -> AnyResult<Single> {
        Single::create(self)
    }
}
