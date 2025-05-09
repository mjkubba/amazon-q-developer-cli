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
    BOOL,
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
use windows::core::{PSTR, Error};

use super::{
    Pid,
    PidExt,
    RawPid,
};
use std::sync::Weak;
use fig_os_shim::Context;

pub fn current(ctx: Weak<Context>) -> Pid {
    let pid = unsafe { GetCurrentProcessId() };
    Pid::Real(ctx, RawPid(pid))
}

pub fn parent(ctx: Weak<Context>, pid: &RawPid) -> Option<Box<Pid>> {
    // This is a simplified implementation for Windows
    // In a real implementation, we would need to use the Windows API to get the parent process ID
    // For now, we'll just return None
    None
}

pub fn exe(ctx: Weak<Context>, pid: &RawPid) -> Option<PathBuf> {
    let handle = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION,
            false,
            pid.0,
        )
    };

    if handle.is_invalid() {
        return None;
    }

    let mut process_name = [0u8; MAX_PATH as usize];
    let mut len = MAX_PATH;

    let result = unsafe {
        let success = QueryFullProcessImageNameA(
            handle,
            PROCESS_NAME_FORMAT(0),
            PSTR(process_name.as_mut_ptr()),
            &mut len,
        );
        
        CloseHandle(handle);
        
        if success.as_bool() {
            let path = CStr::from_ptr(process_name.as_ptr() as *const _)
                .to_string_lossy()
                .into_owned();
            Some(PathBuf::from(path))
        } else {
            None
        }
    };

    result
}

pub fn cmdline(_ctx: Weak<Context>, _pid: &RawPid) -> Option<String> {
    // This is a simplified implementation for Windows
    // In a real implementation, we would need to use the Windows API to get the command line
    // For now, we'll just return None
    None
}

// Helper extension trait for BOOL
trait BoolExt {
    fn as_bool(self) -> bool;
}

impl BoolExt for BOOL {
    fn as_bool(self) -> bool {
        self.0 != 0
    }
}
