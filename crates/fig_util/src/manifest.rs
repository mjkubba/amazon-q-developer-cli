use std::fmt::Display;
use std::str::FromStr;
use std::sync::OnceLock;

use cfg_if::cfg_if;
use fig_os_shim::{
    EnvProvider,
    FsProvider,
    PlatformProvider,
};
use serde::{
    Deserialize,
    Deserializer,
    Serialize,
};
use strum::{
    Display,
    EnumString,
};
use tracing::debug;

use crate::Error;
use crate::build::TARGET_TRIPLE;
use crate::consts::build::VARIANT;

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    #[serde(deserialize_with = "deser_enum_other")]
    pub managed_by: ManagedBy,
    pub version: String,
    pub variant: String,
    pub target: String,
    pub build_id: String,
    pub build_timestamp: String,
    pub build_commit: String,
    pub build_branch: String,
    pub build_dirty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, EnumString, Deserialize)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum ManagedBy {
    Homebrew,
    Apt,
    Rpm,
    Snap,
    Flatpak,
    Appimage,
    Dmg,
    Msi,
    Exe,
    Zip,
    Tarball,
    Other(String),
}

fn deser_enum_other<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + Display,
    <T as FromStr>::Err: Display + std::fmt::Debug,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).or_else(|_| {
        // If we can't parse the string as an enum variant, return Other(s)
        Ok(T::from_str("other").unwrap())
    })
}

static MANIFEST: OnceLock<Option<Manifest>> = OnceLock::new();

pub fn get_manifest() -> Option<&'static Manifest> {
    MANIFEST
        .get_or_init(|| {
            cfg_if! {
                if #[cfg(target_os = "macos")] {
                    let manifest_path = bundle_metadata_path().ok()?;
                    let manifest_str = std::fs::read_to_string(manifest_path).ok()?;
                    let manifest: Manifest = serde_json::from_str(&manifest_str).ok()?;
                    Some(manifest)
                } else if #[cfg(target_os = "linux")] {
                    let manifest_path = bundle_metadata_path().ok()?;
                    let manifest_str = std::fs::read_to_string(manifest_path).ok()?;
                    let manifest: Manifest = serde_json::from_str(&manifest_str).ok()?;
                    Some(manifest)
                } else if #[cfg(target_os = "windows")] {
                    // On Windows, we'll create a default manifest for now
                    Some(Manifest {
                        managed_by: ManagedBy::Exe,
                        version: env!("CARGO_PKG_VERSION").to_string(),
                        variant: VARIANT.unwrap_or("unknown").to_string(),
                        target: TARGET_TRIPLE.unwrap_or("unknown").to_string(),
                        build_id: "dev".to_string(),
                        build_timestamp: "2025-05-09T00:00:00Z".to_string(),
                        build_commit: "unknown".to_string(),
                        build_branch: "unknown".to_string(),
                        build_dirty: true,
                    })
                } else {
                    None
                }
            }
        })
        .as_ref()
}

pub fn get_manifest_ctx<Ctx: EnvProvider + FsProvider + PlatformProvider>(
    ctx: &Ctx,
) -> Result<Manifest, Error> {
    if let Some(manifest) = get_manifest() {
        return Ok(manifest.clone());
    }

    // If we can't get the manifest from the static cache, create a default one
    Ok(Manifest {
        managed_by: ManagedBy::Other("unknown".to_string()),
        version: env!("CARGO_PKG_VERSION").to_string(),
        variant: VARIANT.unwrap_or("unknown").to_string(),
        target: TARGET_TRIPLE.unwrap_or("unknown").to_string(),
        build_id: "dev".to_string(),
        build_timestamp: "2025-05-09T00:00:00Z".to_string(),
        build_commit: "unknown".to_string(),
        build_branch: "unknown".to_string(),
        build_dirty: true,
    })
}

// Add the missing is_minimal function
pub fn is_minimal() -> bool {
    match get_manifest() {
        Some(manifest) => manifest.variant == "minimal",
        None => false,
    }
}

// Helper function for bundle_metadata_path
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn bundle_metadata_path() -> Result<std::path::PathBuf, Error> {
    use std::path::PathBuf;
    
    #[cfg(target_os = "macos")]
    {
        let app_bundle = crate::app_bundle_path();
        Ok(app_bundle.join("Contents/Resources/metadata.json"))
    }
    
    #[cfg(target_os = "linux")]
    {
        Ok(PathBuf::from("/usr/share/fig/metadata.json"))
    }
}
