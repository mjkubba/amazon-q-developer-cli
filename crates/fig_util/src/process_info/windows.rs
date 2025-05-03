use std::ffi::CStr;
use std::mem::{
    MaybeUninit,
    size_of,
};
use std::ops::Deref;
use std::path::PathBuf;

use windows::Win32::Foundation::{
    CloseHandle,
    HANDLE,
    MAX_PATH,
};
use windows::Win32::System::Threading::{
    GetCurrentProcessId,
    OpenProcess,
    PROCESS_NAME_FORMAT,
    PROCESS_QUERY_INFORMATION,
    PROCESS_QUERY_LIMITED_INFORMATION,
    PROCESS_VM_READ,
    QueryFullProcessImageNameA,
};
// These imports are not available in the current windows crate version
// We'll need to implement a workaround
use windows::core::PSTR;

use super::{
    Pid,
    PidExt,
};

struct SafeHandle(HANDLE);

impl SafeHandle {
    fn new(handle: HANDLE) -> Option<Self> {
        if !handle.is_invalid() { Some(Self(handle)) } else { None }
    }
}

impl Drop for SafeHandle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

impl Deref for SafeHandle {
    type Target = HANDLE;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn get_process_handle(pid: &Pid) -> Option<SafeHandle> {
    if pid.0 == 0 {
        return None;
    }

    let handle = unsafe {
        match OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid.0) {
            Ok(handle) => handle,
            Err(_) => match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid.0) {
                Ok(handle) => handle,
                Err(_) => return None,
            },
        }
    };

    SafeHandle::new(handle)
}

impl PidExt for Pid {
    fn current() -> Self {
        unsafe { Pid(GetCurrentProcessId()) }
    }

    fn parent(&self) -> Option<Pid> {
        // Temporarily return None as we need to implement a different approach
        // for getting the parent process ID on Windows without NtQueryInformationProcess
        None
    }

    fn exe(&self) -> Option<PathBuf> {
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, self.0).ok()?;

            // Get the terminal name
            let mut len = MAX_PATH;
            let mut process_name = [0; MAX_PATH as usize + 1];
            process_name[MAX_PATH as usize] = u8::try_from('\0').unwrap();

            let result = QueryFullProcessImageNameA(
                handle,
                PROCESS_NAME_FORMAT(0),
                PSTR(process_name.as_mut_ptr()),
                &mut len,
            );
            
            if result.is_err() {
                return None;
            }

            let title = CStr::from_bytes_with_nul(&process_name[0..=len as usize])
                .ok()?
                .to_str()
                .ok()?;

            Some(PathBuf::from(title))
        }
    }
}
