use crate::{AnyResult, SingleHandler};
use nix::sys::socket::{self, UnixAddr};
use nix::unistd;
use std::os::unix::prelude::RawFd;

pub struct UnixHandler {
    inner: RawFd,
}

impl SingleHandler for UnixHandler {}

impl Drop for UnixHandler {
    fn drop(&mut self) {
        let _ = unistd::close(self.inner);
    }
}

pub fn new(key: &str) -> AnyResult<Option<Box<dyn SingleHandler>>> {
    let addr = UnixAddr::new_abstract(key.as_bytes())?;
    let sock = socket::socket(
        socket::AddressFamily::Unix,
        socket::SockType::Stream,
        socket::SockFlag::SOCK_CLOEXEC,
        None,
    )?;

    socket::bind(sock, &socket::SockAddr::Unix(addr))?;
    Ok(Some(Box::new(UnixHandler { inner: sock })))
}
