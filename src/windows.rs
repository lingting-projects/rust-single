use crate::{AnyResult, SingleError, SingleHandler};

use std::ptr;
use widestring::WideCString;
use winapi::shared::winerror::{ERROR_ALREADY_EXISTS, ERROR_INVALID_HANDLE};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::CloseHandle;
use winapi::um::synchapi::CreateMutexW;
use winapi::um::winnt::HANDLE;

struct WindowsHandler {
    inner: HANDLE,
}

impl SingleHandler for WindowsHandler {}

impl Drop for WindowsHandler {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.inner);
        }
    }
}

pub fn new(key: &str) -> AnyResult<Option<Box<dyn SingleHandler>>> {
    let name = WideCString::from_str(key)?;
    unsafe {
        let handle = CreateMutexW(ptr::null_mut(), 0, name.as_ptr());
        let last_error = GetLastError();

        // https://docs.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-createmutexexw
        if handle.is_null() || handle == ERROR_INVALID_HANDLE as _ {
            return Err(Box::new(SingleError::InvalidCheck));
        }

        if last_error == ERROR_ALREADY_EXISTS {
            CloseHandle(handle);
            Ok(None)
        } else {
            Ok(Some(Box::new(WindowsHandler { inner: handle })))
        }
    }
}
