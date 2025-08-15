use crate::{AnyResult, Single};
use fs2::FileExt;
use std::any::Any;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::{panic, thread};

pub fn try_unique<P: AsRef<Path>>(p: P) -> AnyResult<File> {
    let mut options = OpenOptions::new();
    options.write(true).create(true);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options
            .share_mode(0x0)
            .attributes(0)
            .security_qos_flags(0x0)
            .custom_flags(0x0)
            .access_mode(0xC0000000);
    }
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o444);
    }

    let file = options.open(p)?;
    FileExt::lock_exclusive(&file)?;
    Ok(file)
}

pub fn write(path: &str, content: String) -> AnyResult<()> {
    let mut file = File::create(path)?;
    file.set_len(0)?;
    file.write_all(content.as_bytes())?;
    file.flush()?;
    file.sync_all()?;
    Ok(())
}

pub fn panic_msg(p: Box<dyn Any + Send>) -> String {
    match p.downcast_ref::<String>() {
        Some(s) => s.to_string(),
        None => match p.downcast_ref::<&str>() {
            Some(s) => (*s).to_string(),
            None => "panic 未提供错误信息".to_string(),
        },
    }
}

#[cfg(feature = "ipc")]
pub fn ipc_create(single: Single) -> AnyResult<Single> {
    let option = &single.on_wake;
    if option.is_none() {
        return Ok(single);
    }
    let _f = option.clone().unwrap();

    use crate::ipc::*;
    let server = IpcServer::new(&single.path_ipc)?;

    thread::spawn(move || {
        loop {
            match panic::catch_unwind(|| server.next()) {
                Ok(Ok(mut stream)) => {
                    let r = stream.read();
                    _f(r);
                }
                Ok(Err(e)) => {
                    log::error!("ipc server read err! {}", e)
                }
                Err(p) => {
                    log::error!("ipc server read err! {}", panic_msg(p))
                }
            }
        }
    });

    Ok(single)
}