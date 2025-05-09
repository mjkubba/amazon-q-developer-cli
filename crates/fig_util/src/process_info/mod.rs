use std::path::PathBuf;
use std::{
    fmt,
    str,
};

use cfg_if::cfg_if;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(target_os = "freebsd")]
mod freebsd;
#[cfg(target_os = "freebsd")]
pub use freebsd::*;

cfg_if! {
    if #[cfg(windows)] {
        pub type PidType = u32;
    } else if #[cfg(unix)] {
        pub type PidType = i32;
    } else {
        pub type PidType = u32;
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Pid(pub(crate) PidType);

impl From<PidType> for Pid {
    fn from(v: PidType) -> Self {
        Self(v)
    }
}

impl From<Pid> for PidType {
    fn from(v: Pid) -> Self {
        v.0
    }
}

impl str::FromStr for Pid {
    type Err = <PidType as str::FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(<PidType>::from_str(s)?))
    }
}

impl fmt::Display for Pid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Pid {
    pub fn as_u32(&self) -> u32 {
        self.0 as u32
    }
}

pub trait PidExt {
    fn current() -> Self;
    fn parent(&self) -> Option<Box<Self>>;
    fn exe(&self) -> Option<PathBuf>;
    fn cmdline(&self) -> Option<String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_pid() {
        let mut pid = Pid::current();
        assert!(pid.0 > 0);
    }

    #[test]
    fn test_parent_pid() {
        let pid = Pid::current();
        let parent = pid.parent();
        assert!(parent.is_some());
    }

    #[test]
    fn test_exe() {
        let pid = Pid::current();
        let exe = pid.exe();
        assert!(exe.is_some());
    }

    #[test]
    fn test_cmdline() {
        let pid = Pid::current();
        let cmdline = pid.cmdline();
        // cmdline might be None on some platforms
    }
}
