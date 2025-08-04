use crate::{AnyResult, SingleHandler};

use libc::{__error, flock, EWOULDBLOCK, LOCK_EX, LOCK_NB};
use std::fs::File;
use std::os::fd::RawFd;
use std::os::unix::io::AsRawFd;
use std::path::Path;

/// A struct representing one running instance.
pub struct MacHandler {
    inner: RawFd,
}

impl SingleHandler for MacHandler {}

impl Drop for MacHandler {
    fn drop(&mut self) {
        //
    }
}

pub fn new(key: &str) -> AnyResult<Option<Box<dyn SingleHandler>>> {
    let path = Path::new(key);
    let file = if path.exists() {
        File::open(path)?
    } else {
        File::create(path)?
    };
    unsafe {
        let fd = file.as_raw_fd();
        let rc = flock(fd, LOCK_EX | LOCK_NB);
        let is_single = rc == 0 || EWOULDBLOCK != *__error();
        if is_single {
            Ok(Some(Box::new(MacHandler { inner: fd })))
        } else {
            Ok(None)
        }
    }
}
