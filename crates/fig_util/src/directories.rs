use std::convert::TryInto;
use std::fmt::Display;
use std::path::PathBuf;

use camino::Utf8PathBuf;
use fig_os_shim::{
    Context,
    EnvProvider,
    FsProvider,
    Os,
    PlatformProvider,
    Shim,
};
use thiserror::Error;
use time::OffsetDateTime;

use crate::env_var::{
    Q_BUNDLE_METADATA_PATH,
    Q_PARENT,
};
use crate::linux::PACKAGE_NAME;
use crate::system_info::{
    in_cloudshell,
    is_remote,
};
use crate::{
    RUNTIME_DIR_NAME,
    TAURI_PRODUCT_NAME,
};

macro_rules! utf8_dir {
    ($name:ident, $($arg:ident: $type:ty),*) => {
        paste::paste! {
            pub fn [<$name _utf8>]($($arg: $type),*) -> Result<Utf8PathBuf> {
                Ok($name($($arg),*)?.try_into()?)
            }
        }
    };
    ($name:ident) => {
        utf8_dir!($name,);
    };
}

#[derive(Debug, Error)]
pub enum DirectoryError {
    #[error("home directory not found")]
    NoHomeDirectory,
    #[error("runtime directory not found: neither XDG_RUNTIME_DIR nor TMPDIR were found")]
    NoRuntimeDirectory,
    #[error("non absolute path: {0:?}")]
    NonAbsolutePath(PathBuf),
    #[error("invalid UTF-8 path: {0:?}")]
    InvalidUtf8Path(PathBuf),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("invalid path: {0}")]
    InvalidPath(#[from] camino::FromPathBufError),
}

type Result<T, E = DirectoryError> = std::result::Result<T, E>;

/// The directory of the users home
///
/// - Linux: /home/Alice
/// - MacOS: /Users/Alice
/// - Windows: C:\Users\Alice
pub fn home_dir() -> Result<PathBuf> {
    dirs::home_dir().ok_or(DirectoryError::NoHomeDirectory)
}

pub fn home_dir_ctx<Ctx: FsProvider + EnvProvider>(ctx: &Ctx) -> Result<PathBuf> {
    if ctx.env().is_real() {
        home_dir()
    } else {
        ctx.env()
            .get("HOME")
            .map_err(|_err| DirectoryError::NoHomeDirectory)
            .and_then(|h| {
                if h.is_empty() {
                    Err(DirectoryError::NoHomeDirectory)
                } else {
                    Ok(h)
                }
            })
            .map(PathBuf::from)
            .map(|p| ctx.fs().chroot_path(p))
    }
}

utf8_dir!(home_dir);
utf8_dir!(home_dir_ctx, ctx: &Context);

/// The base directory for all Fig files
///
/// - Linux: `$HOME/.fig`
/// - MacOS: `$HOME/.fig`
/// - Windows: `%APPDATA%\Amazon Q`
pub fn fig_dir() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            Ok(home_dir()?.join(".fig"))
        } else if #[cfg(windows)] {
            Ok(dirs::data_local_dir()
                .ok_or(DirectoryError::NoHomeDirectory)?
                .join("Amazon Q"))
        }
    }
}

pub fn fig_dir_ctx(ctx: &Context) -> Result<PathBuf> {
    // For Windows compatibility, don't use is_real()
    #[cfg(unix)]
    {
        if ctx.env().is_real() {
            return fig_dir();
        }
    }
    
    Ok(home_dir_ctx(ctx)?.join(".fig"))
}

utf8_dir!(fig_dir);
utf8_dir!(fig_dir_ctx, ctx: &Context);

/// This should be removed at some point in the future, once all our users have migrated
/// - MacOS: `$HOME/Library/Application Support/codewhisperer`
pub fn old_fig_data_dir() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            Ok(dirs::data_local_dir()
                .ok_or(DirectoryError::NoHomeDirectory)?
                .join("codewhisperer"))
        } else if #[cfg(windows)] {
            Ok(fig_dir()?.join("userdata"))
        }
    }
}

/// The q data directory
///
/// - Linux: `$XDG_DATA_HOME/amazon-q` or `$HOME/.local/share/amazon-q`
/// - MacOS: `$HOME/Library/Application Support/amazon-q`
/// - Windows: `%APPDATA%\Amazon Q\userdata`
pub fn fig_data_dir() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            Ok(dirs::data_local_dir()
                .ok_or(DirectoryError::NoHomeDirectory)?
                .join("amazon-q"))
        } else if #[cfg(windows)] {
            Ok(fig_dir()?.join("userdata"))
        }
    }
}

pub fn fig_data_dir_ctx(fs: &impl FsProvider) -> Result<PathBuf> {
    // For Windows compatibility, don't use is_real()
    #[cfg(unix)]
    {
        if fs.is_real() {
            return fig_data_dir();
        }
    }
    
    #[cfg(unix)]
    {
        Ok(fs.chroot_path(fig_data_dir()?))
    }
    
    #[cfg(not(unix))]
    {
        fig_data_dir()
    }
}

utf8_dir!(fig_data_dir);
utf8_dir!(fig_data_dir_ctx, fs: &impl FsProvider);

/// The q cache directory
///
/// - Linux: `$XDG_CACHE_HOME/amazon-q` or `$HOME/.cache/amazon-q`
/// - MacOS: `$HOME/Library/Caches/amazon-q`
/// - Windows: `%APPDATA%\Amazon Q\cache`
pub fn fig_cache_dir() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            Ok(dirs::cache_dir()
                .ok_or(DirectoryError::NoHomeDirectory)?
                .join("amazon-q"))
        } else if #[cfg(windows)] {
            Ok(fig_dir()?.join("cache"))
        }
    }
}

pub fn fig_cache_dir_ctx(fs: &impl FsProvider) -> Result<PathBuf> {
    // For Windows compatibility, don't use is_real()
    #[cfg(unix)]
    {
        if fs.is_real() {
            return fig_cache_dir();
        }
    }
    
    #[cfg(unix)]
    {
        Ok(fs.chroot_path(fig_cache_dir()?))
    }
    
    #[cfg(not(unix))]
    {
        fig_cache_dir()
    }
}

utf8_dir!(fig_cache_dir);
utf8_dir!(fig_cache_dir_ctx, fs: &impl FsProvider);

/// The q runtime directory
///
/// - Linux: `$XDG_RUNTIME_DIR/amazon-q` or `/tmp/amazon-q-$UID`
/// - MacOS: `/tmp/amazon-q-$UID`
/// - Windows: `%APPDATA%\Amazon Q\runtime`
pub fn fig_runtime_dir() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            let uid = unsafe { libc::getuid() };
            let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
                .map(PathBuf::from)
                .or_else(|_| std::env::var("TMPDIR").map(PathBuf::from))
                .or_else(|_| Ok::<_, std::env::VarError>(PathBuf::from("/tmp")))
                .unwrap();
            Ok(runtime_dir.join(format!("amazon-q-{}", uid)))
        } else if #[cfg(windows)] {
            Ok(fig_dir()?.join("runtime"))
        }
    }
}

pub fn fig_runtime_dir_ctx(ctx: &Context) -> Result<PathBuf> {
    // For Windows compatibility, don't use is_real()
    #[cfg(unix)]
    {
        if ctx.env().is_real() {
            return fig_runtime_dir();
        }
    }
    
    #[cfg(unix)]
    {
        Ok(ctx.fs().chroot_path(fig_runtime_dir()?))
    }
    
    #[cfg(not(unix))]
    {
        fig_runtime_dir()
    }
}

utf8_dir!(fig_runtime_dir);
utf8_dir!(fig_runtime_dir_ctx, ctx: &Context);

/// The q sockets directory
///
/// - Linux: `$XDG_RUNTIME_DIR/amazon-q/sockets` or `/tmp/amazon-q-$UID/sockets`
/// - MacOS: `/tmp/amazon-q-$UID/sockets`
/// - Windows: `%APPDATA%\Amazon Q\sockets`
pub fn sockets_dir() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            Ok(fig_runtime_dir()?.join("sockets"))
        } else if #[cfg(windows)] {
            Ok(fig_dir()?.join("sockets"))
        }
    }
}

pub fn sockets_dir_ctx(ctx: &Context) -> Result<PathBuf> {
    // For Windows compatibility, don't use is_real()
    #[cfg(unix)]
    {
        if ctx.env().is_real() {
            return sockets_dir();
        }
    }
    
    #[cfg(unix)]
    {
        Ok(ctx.fs().chroot_path(sockets_dir()?))
    }
    
    #[cfg(not(unix))]
    {
        sockets_dir()
    }
}

utf8_dir!(sockets_dir);
utf8_dir!(sockets_dir_ctx, ctx: &Context);

/// The path to the figterm socket
///
/// - Linux: `$XDG_RUNTIME_DIR/cwrun/t/$SESSION_ID.sock`
/// - Windows: `%APPDATA%\Fig\$SESSION_ID.sock`
pub fn figterm_socket_path(session_id: impl Display) -> Result<PathBuf> {
    Ok(sockets_dir()?.join("t").join(format!("{session_id}.sock")))
}

/// The path to the resources directory
///
/// - MacOS: "/Applications/Amazon Q.app/Contents/Resources"
/// - Linux: "/usr/share/fig"
/// - Windows: "%PROGRAMFILES%\Amazon Q\resources"
pub fn resources_path() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(all(unix, not(target_os = "macos")))] {
            Ok(std::path::Path::new("/usr/share/fig").into())
        } else if #[cfg(target_os = "macos")] {
            Ok(crate::app_bundle_path().join(crate::macos::BUNDLE_CONTENTS_RESOURCE_PATH))
        } else if #[cfg(windows)] {
            // On Windows, resources are typically in the Program Files directory
            let program_files = std::env::var("PROGRAMFILES")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("C:\\Program Files"));
            Ok(program_files.join("Amazon Q").join("resources"))
        }
    }
}

pub fn resources_path_ctx<Ctx: EnvProvider + PlatformProvider>(ctx: &Ctx) -> Result<PathBuf> {
    // For Windows compatibility, don't use is_real()
    #[cfg(unix)]
    {
        if ctx.env().is_real() {
            return resources_path();
        }
    }
    
    #[cfg(unix)]
    {
        // This requires FsProvider, which we don't have on Windows
        if let Some(fs) = ctx.fs() {
            return Ok(fs.chroot_path(resources_path()?));
        }
    }
    
    resources_path()
}

utf8_dir!(resources_path);
utf8_dir!(resources_path_ctx, ctx: &impl PlatformProvider);

/// The path to the managed binaries directory
///
/// - MacOS: "/Applications/Amazon Q.app/Contents/Resources/bin"
/// - Linux: "/usr/share/fig/bin"
/// - Windows: "%PROGRAMFILES%\Amazon Q\bin"
pub fn managed_binaries_dir() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(all(unix, not(target_os = "macos")))] {
            Ok(std::path::Path::new("/usr/share/fig/bin").into())
        } else if #[cfg(target_os = "macos")] {
            Ok(resources_path()?.join("bin"))
        } else if #[cfg(windows)] {
            // On Windows, binaries are typically in the Program Files directory
            let program_files = std::env::var("PROGRAMFILES")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("C:\\Program Files"));
            Ok(program_files.join("Amazon Q").join("bin"))
        }
    }
}

pub fn managed_binaries_dir_ctx<Ctx: EnvProvider + PlatformProvider>(ctx: &Ctx) -> Result<PathBuf> {
    // For Windows compatibility, don't use is_real()
    #[cfg(unix)]
    {
        if ctx.env().is_real() {
            return managed_binaries_dir();
        }
    }
    
    #[cfg(unix)]
    {
        // This requires FsProvider, which we don't have on Windows
        if let Some(fs) = ctx.fs() {
            return Ok(fs.chroot_path(managed_binaries_dir()?));
        }
    }
    
    managed_binaries_dir()
}

utf8_dir!(managed_binaries_dir);
utf8_dir!(managed_binaries_dir_ctx, ctx: &impl PlatformProvider);

/// The path to the managed binaries manifest
///
/// - MacOS: "/Applications/Amazon Q.app/Contents/Resources/bin/manifest.json"
/// - Linux: "/usr/share/fig/bin/manifest.json"
/// - Windows: "%PROGRAMFILES%\Amazon Q\bin\manifest.json"
pub fn managed_binaries_manifest() -> Result<PathBuf> {
    Ok(managed_binaries_dir()?.join("manifest.json"))
}

pub fn managed_binaries_manifest_ctx<Ctx: EnvProvider + PlatformProvider>(ctx: &Ctx) -> Result<PathBuf> {
    // For Windows compatibility, don't use is_real()
    #[cfg(unix)]
    {
        if ctx.env().is_real() {
            return managed_binaries_manifest();
        }
    }
    
    #[cfg(unix)]
    {
        // This requires FsProvider, which we don't have on Windows
        if let Some(fs) = ctx.fs() {
            return Ok(fs.chroot_path(managed_binaries_manifest()?));
        }
    }
    
    managed_binaries_manifest()
}

utf8_dir!(managed_binaries_manifest);
utf8_dir!(managed_binaries_manifest_ctx, ctx: &impl PlatformProvider);

/// The path to the logs directory
///
/// - Linux: `$XDG_STATE_HOME/amazon-q/logs` or `$HOME/.local/state/amazon-q/logs`
/// - MacOS: `$HOME/Library/Logs/amazon-q`
/// - Windows: `%APPDATA%\Amazon Q\logs`
pub fn logs_dir() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            Ok(home_dir()?.join("Library").join("Logs").join("amazon-q"))
        } else if #[cfg(all(unix, not(target_os = "macos")))] {
            // XDG_STATE_HOME is a relatively new standard, so we need to fall back to the default
            let state_home = std::env::var("XDG_STATE_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| home_dir().unwrap().join(".local").join("state"));
            Ok(state_home.join("amazon-q").join("logs"))
        } else if #[cfg(windows)] {
            Ok(fig_dir()?.join("logs"))
        }
    }
}

pub fn logs_dir_ctx(ctx: &Context) -> Result<PathBuf> {
    // For Windows compatibility, don't use is_real()
    #[cfg(unix)]
    {
        if ctx.env().is_real() {
            return logs_dir();
        }
    }
    
    #[cfg(unix)]
    {
        Ok(ctx.fs().chroot_path(logs_dir()?))
    }
    
    #[cfg(not(unix))]
    {
        logs_dir()
    }
}

utf8_dir!(logs_dir);
utf8_dir!(logs_dir_ctx, ctx: &Context);

/// The path to the log file
///
/// - Linux: `$XDG_STATE_HOME/amazon-q/logs/amazon-q.log` or `$HOME/.local/state/amazon-q/logs/amazon-q.log`
/// - MacOS: `$HOME/Library/Logs/amazon-q/amazon-q.log`
/// - Windows: `%APPDATA%\Amazon Q\logs\amazon-q.log`
pub fn log_file() -> Result<PathBuf> {
    Ok(logs_dir()?.join("amazon-q.log"))
}

pub fn log_file_ctx(ctx: &Context) -> Result<PathBuf> {
    if ctx.is_real() {
        log_file()
    } else {
        Ok(ctx.fs().chroot_path(log_file()?))
    }
}

utf8_dir!(log_file);
utf8_dir!(log_file_ctx, ctx: &Context);

/// The path to the log file for a specific date
///
/// - Linux: `$XDG_STATE_HOME/amazon-q/logs/amazon-q-$DATE.log` or `$HOME/.local/state/amazon-q/logs/amazon-q-$DATE.log`
/// - MacOS: `$HOME/Library/Logs/amazon-q/amazon-q-$DATE.log`
/// - Windows: `%APPDATA%\Amazon Q\logs\amazon-q-$DATE.log`
pub fn log_file_for_date(date: OffsetDateTime) -> Result<PathBuf> {
    let date_str = date.format(&time::format_description::parse("[year]-[month]-[day]").unwrap()).unwrap();
    Ok(logs_dir()?.join(format!("amazon-q-{}.log", date_str)))
}

pub fn log_file_for_date_ctx(ctx: &Context, date: OffsetDateTime) -> Result<PathBuf> {
    if ctx.is_real() {
        log_file_for_date(date)
    } else {
        Ok(ctx.fs().chroot_path(log_file_for_date(date)?))
    }
}

utf8_dir!(log_file_for_date, date: OffsetDateTime);
utf8_dir!(log_file_for_date_ctx, ctx: &Context, date: OffsetDateTime);

/// The path to the config directory
///
/// - Linux: `$XDG_CONFIG_HOME/amazon-q` or `$HOME/.config/amazon-q`
/// - MacOS: `$HOME/Library/Application Support/amazon-q`
/// - Windows: `%APPDATA%\Amazon Q\config`
pub fn config_dir() -> Result<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            Ok(dirs::config_dir()
                .ok_or(DirectoryError::NoHomeDirectory)?
                .join("amazon-q"))
        } else if #[cfg(all(unix, not(target_os = "macos")))] {
            Ok(dirs::config_dir()
                .ok_or(DirectoryError::NoHomeDirectory)?
                .join("amazon-q"))
        } else if #[cfg(windows)] {
            Ok(fig_dir()?.join("config"))
        }
    }
}

pub fn config_dir_ctx(ctx: &Context) -> Result<PathBuf> {
    if ctx.is_real() {
        config_dir()
    } else {
        Ok(ctx.fs().chroot_path(config_dir()?))
    }
}

utf8_dir!(config_dir);
utf8_dir!(config_dir_ctx, ctx: &Context);

/// The path to the config file
///
/// - Linux: `$XDG_CONFIG_HOME/amazon-q/config.json` or `$HOME/.config/amazon-q/config.json`
/// - MacOS: `$HOME/Library/Application Support/amazon-q/config.json`
/// - Windows: `%APPDATA%\Amazon Q\config\config.json`
pub fn config_file() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.json"))
}

pub fn config_file_ctx(ctx: &Context) -> Result<PathBuf> {
    if ctx.is_real() {
        config_file()
    } else {
        Ok(ctx.fs().chroot_path(config_file()?))
    }
}

utf8_dir!(config_file);
utf8_dir!(config_file_ctx, ctx: &Context);

/// The path to the credentials file
///
/// - Linux: `$XDG_CONFIG_HOME/amazon-q/credentials.json` or `$HOME/.config/amazon-q/credentials.json`
/// - MacOS: `$HOME/Library/Application Support/amazon-q/credentials.json`
/// - Windows: `%APPDATA%\Amazon Q\config\credentials.json`
pub fn credentials_file() -> Result<PathBuf> {
    Ok(config_dir()?.join("credentials.json"))
}

pub fn credentials_file_ctx(ctx: &Context) -> Result<PathBuf> {
    if ctx.is_real() {
        credentials_file()
    } else {
        Ok(ctx.fs().chroot_path(credentials_file()?))
    }
}

utf8_dir!(credentials_file);
utf8_dir!(credentials_file_ctx, ctx: &Context);

/// The path to the telemetry directory
///
/// - Linux: `$XDG_DATA_HOME/amazon-q/telemetry` or `$HOME/.local/share/amazon-q/telemetry`
/// - MacOS: `$HOME/Library/Application Support/amazon-q/telemetry`
/// - Windows: `%APPDATA%\Amazon Q\userdata\telemetry`
pub fn telemetry_dir() -> Result<PathBuf> {
    Ok(fig_data_dir()?.join("telemetry"))
}

pub fn telemetry_dir_ctx(ctx: &Context) -> Result<PathBuf> {
    if ctx.is_real() {
        telemetry_dir()
    } else {
        Ok(ctx.fs().chroot_path(telemetry_dir()?))
    }
}

utf8_dir!(telemetry_dir);
utf8_dir!(telemetry_dir_ctx, ctx: &Context);

/// The path to the telemetry file
///
/// - Linux: `$XDG_DATA_HOME/amazon-q/telemetry/telemetry.json` or `$HOME/.local/share/amazon-q/telemetry/telemetry.json`
/// - MacOS: `$HOME/Library/Application Support/amazon-q/telemetry/telemetry.json`
/// - Windows: `%APPDATA%\Amazon Q\userdata\telemetry\telemetry.json`
pub fn telemetry_file() -> Result<PathBuf> {
    Ok(telemetry_dir()?.join("telemetry.json"))
}

pub fn telemetry_file_ctx(ctx: &Context) -> Result<PathBuf> {
    if ctx.is_real() {
        telemetry_file()
    } else {
        Ok(ctx.fs().chroot_path(telemetry_file()?))
    }
}

utf8_dir!(telemetry_file);
utf8_dir!(telemetry_file_ctx, ctx: &Context);

/// The path to the telemetry file for a specific date
///
/// - Linux: `$XDG_DATA_HOME/amazon-q/telemetry/telemetry-$DATE.json` or `$HOME/.local/share/amazon-q/telemetry/telemetry-$DATE.json`
/// - MacOS: `$HOME/Library/Application Support/amazon-q/telemetry/telemetry-$DATE.json`
/// - Windows: `%APPDATA%\Amazon Q\userdata\telemetry\telemetry-$DATE.json`
pub fn telemetry_file_for_date(date: OffsetDateTime) -> Result<PathBuf> {
    let date_str = date.format(&time::format_description::parse("[year]-[month]-[day]").unwrap()).unwrap();
    Ok(telemetry_dir()?.join(format!("telemetry-{}.json", date_str)))
}

pub fn telemetry_file_for_date_ctx(ctx: &Context, date: OffsetDateTime) -> Result<PathBuf> {
    if ctx.is_real() {
        telemetry_file_for_date(date)
    } else {
        Ok(ctx.fs().chroot_path(telemetry_file_for_date(date)?))
    }
}

utf8_dir!(telemetry_file_for_date, date: OffsetDateTime);
utf8_dir!(telemetry_file_for_date_ctx, ctx: &Context, date: OffsetDateTime);

/// The path to the telemetry file for a specific date and session
///
/// - Linux: `$XDG_DATA_HOME/amazon-q/telemetry/telemetry-$DATE-$SESSION.json` or `$HOME/.local/share/amazon-q/telemetry/telemetry-$DATE-$SESSION.json`
/// - MacOS: `$HOME/Library/Application Support/amazon-q/telemetry/telemetry-$DATE-$SESSION.json`
/// - Windows: `%APPDATA%\Amazon Q\userdata\telemetry\telemetry-$DATE-$SESSION.json`
pub fn telemetry_file_for_date_session(date: OffsetDateTime, session: &str) -> Result<PathBuf> {
    let date_str = date.format(&time::format_description::parse("[year]-[month]-[day]").unwrap()).unwrap();
    Ok(telemetry_dir()?.join(format!("telemetry-{}-{}.json", date_str, session)))
}

pub fn telemetry_file_for_date_session_ctx(ctx: &Context, date: OffsetDateTime, session: &str) -> Result<PathBuf> {
    if ctx.is_real() {
        telemetry_file_for_date_session(date, session)
    } else {
        Ok(ctx.fs().chroot_path(telemetry_file_for_date_session(date, session)?))
    }
}

utf8_dir!(telemetry_file_for_date_session, date: OffsetDateTime, session: &str);
utf8_dir!(telemetry_file_for_date_session_ctx, ctx: &Context, date: OffsetDateTime, session: &str);

#[cfg(test)]
mod tests {
    use super::*;
    use fig_os_shim::Context;

    #[test]
    fn test_home_dir() {
        let home_dir = home_dir().unwrap();
        assert!(home_dir.is_absolute());
    }

    #[test]
    fn test_fig_dir() {
        let fig_dir = fig_dir().unwrap();
        assert!(fig_dir.is_absolute());
    }

    #[test]
    fn test_fig_data_dir() {
        let fig_data_dir = fig_data_dir().unwrap();
        assert!(fig_data_dir.is_absolute());
    }

    #[test]
    fn test_fig_cache_dir() {
        let fig_cache_dir = fig_cache_dir().unwrap();
        assert!(fig_cache_dir.is_absolute());
    }

    #[test]
    fn test_fig_runtime_dir() {
        let fig_runtime_dir = fig_runtime_dir().unwrap();
        assert!(fig_runtime_dir.is_absolute());
    }

    #[test]
    fn test_sockets_dir() {
        let sockets_dir = sockets_dir().unwrap();
        assert!(sockets_dir.is_absolute());
    }

    #[test]
    fn test_figterm_socket_path() {
        let figterm_socket_path = figterm_socket_path("test").unwrap();
        assert!(figterm_socket_path.is_absolute());
    }

    #[test]
    fn test_resources_path() {
        let resources_path = resources_path().unwrap();
        assert!(resources_path.is_absolute());
    }

    #[test]
    fn test_managed_binaries_dir() {
        let managed_binaries_dir = managed_binaries_dir().unwrap();
        assert!(managed_binaries_dir.is_absolute());
    }

    #[test]
    fn test_managed_binaries_manifest() {
        let managed_binaries_manifest = managed_binaries_manifest().unwrap();
        assert!(managed_binaries_manifest.is_absolute());
    }

    #[test]
    fn test_logs_dir() {
        let logs_dir = logs_dir().unwrap();
        assert!(logs_dir.is_absolute());
    }

    #[test]
    fn test_log_file() {
        let log_file = log_file().unwrap();
        assert!(log_file.is_absolute());
    }

    #[test]
    fn test_log_file_for_date() {
        let date = OffsetDateTime::now_utc();
        let log_file_for_date = log_file_for_date(date).unwrap();
        assert!(log_file_for_date.is_absolute());
    }

    #[test]
    fn test_config_dir() {
        let config_dir = config_dir().unwrap();
        assert!(config_dir.is_absolute());
    }

    #[test]
    fn test_config_file() {
        let config_file = config_file().unwrap();
        assert!(config_file.is_absolute());
    }

    #[test]
    fn test_credentials_file() {
        let credentials_file = credentials_file().unwrap();
        assert!(credentials_file.is_absolute());
    }

    #[test]
    fn test_telemetry_dir() {
        let telemetry_dir = telemetry_dir().unwrap();
        assert!(telemetry_dir.is_absolute());
    }

    #[test]
    fn test_telemetry_file() {
        let telemetry_file = telemetry_file().unwrap();
        assert!(telemetry_file.is_absolute());
    }

    #[test]
    fn test_telemetry_file_for_date() {
        let date = OffsetDateTime::now_utc();
        let telemetry_file_for_date = telemetry_file_for_date(date).unwrap();
        assert!(telemetry_file_for_date.is_absolute());
    }

    #[test]
    fn test_telemetry_file_for_date_session() {
        let date = OffsetDateTime::now_utc();
        let telemetry_file_for_date_session = telemetry_file_for_date_session(date, "test").unwrap();
        assert!(telemetry_file_for_date_session.is_absolute());
    }
}
