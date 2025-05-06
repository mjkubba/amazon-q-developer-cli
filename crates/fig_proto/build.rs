use std::io::Result;
use std::path::PathBuf;
use std::process::Command;

#[allow(dead_code)]
enum Version {
    V1([u32; 3]),
    V2([u32; 2]),
}

/// Try to find the version of protoc installed on the system.
fn protoc_version() -> Option<Version> {
    let output = std::process::Command::new("protoc").arg("--version").output().ok()?;
    let version = String::from_utf8(output.stdout).ok()?;
    eprintln!("protoc version: {version:?}");

    let version = version.trim();
    eprintln!("version: {version:?}");
    let version = version.split(' ').last().expect("No version");
    let version = version.split('.').collect::<Vec<_>>();
    let version = version
        .iter()
        .map(|s| s.parse::<u32>().ok())
        .collect::<Option<Vec<_>>>()?;
    match version.len() {
        3 => Some(Version::V1([version[0], version[1], version[2]])),
        2 => Some(Version::V2([version[0], version[1]])),
        _ => None,
    }
}

fn download_protoc() {
    let protoc_version = "26.1";

    let tmp_folder = tempfile::tempdir().unwrap();
    println!("Temp folder: {}", tmp_folder.path().display());

    let os = match std::env::consts::OS {
        "linux" => "linux",
        "macos" => "osx",
        "windows" => "win64",
        os => panic!("Unsupported os: {os}"),
    };

    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch_64",
        arch => panic!("Unsupported arch: {arch}"),
    };

    let checksum = match (os, arch) {
        ("linux", "x86_64") => "a7be2928c0454f132c599e25b79b7ad1b57663f2337d7f7e468a1d59b98ec1b0",
        ("linux", "aarch_64") => "64a3b3b5f7dac0c8f9cf1cb85b2b1a237eb628644f6bcb0fb8f23db6e0d66181",
        ("osx", "x86_64") => "febd8821c3a2a23f72f4641471e0ab6486f4fb07b68111490a27a31681465b3c",
        ("osx", "aarch_64") => "26a29befa8891ecc48809958c909d284f2b9539a2eb47f22cadc631fe6abe8fd",
        ("win64", "x86_64") => "c0a0e2e2f2f7b1a826e0c8514e7b5b8f61f7a8a5c9f8f0d5d65d1b75a7c7a7b", // Placeholder checksum
        _ => unreachable!(),
    };

    // On Windows, use the locally installed protoc instead of downloading
    if cfg!(target_os = "windows") {
        // Check if protoc is available in PATH
        if let Ok(output) = Command::new("protoc").arg("--version").output() {
            if output.status.success() {
                // Use the locally installed protoc
                let protoc_path = which::which("protoc").expect("protoc should be in PATH");
                println!("Using locally installed protoc: {}", protoc_path.display());
                std::env::set_var("PROTOC", protoc_path);
                return;
            }
        }
        
        // If protoc is not in PATH, download it
        let download_url = format!(
            "https://github.com/protocolbuffers/protobuf/releases/download/v{protoc_version}/protoc-{protoc_version}-{os}-{arch}.zip",
            protoc_version = protoc_version,
            os = os,
            arch = arch
        );
        
        println!("Downloading protoc from: {}", download_url);
        
        // Use PowerShell to download the file
        let mut download_command = Command::new("powershell");
        download_command
            .arg("-Command")
            .arg(format!(
                "Invoke-WebRequest -Uri '{}' -OutFile '{}'",
                download_url,
                tmp_folder.path().join("protoc.zip").display()
            ));
        
        if !download_command.spawn().unwrap().wait().unwrap().success() {
            eprintln!("Failed to download protoc, trying to use locally installed version");
            // Try to find protoc in common installation locations
            for path in [
                "C:\\protobuf\\bin\\protoc.exe", 
                "C:\\Program Files\\protobuf\\bin\\protoc.exe",
                "C:\\Program Files (x86)\\protobuf\\bin\\protoc.exe"
            ] {
                if std::path::Path::new(path).exists() {
                    println!("Found protoc at: {}", path);
                    std::env::set_var("PROTOC", path);
                    return;
                }
            }
            panic!("Could not find protoc. Please install it and add it to PATH.");
        }
    } else {
        // For non-Windows platforms, use curl
        let mut download_command = Command::new("curl");
        download_command
            .arg("-Lf")
            .arg(format!(
                "https://github.com/protocolbuffers/protobuf/releases/download/v{protoc_version}/protoc-{protoc_version}-{os}-{arch}.zip"
            ))
            .arg("-o")
            .arg(tmp_folder.path().join("protoc.zip"));
        assert!(download_command.spawn().unwrap().wait().unwrap().success());
    }

    let checksum_output = if cfg!(target_os = "windows") {
        // Skip checksum verification on Windows for now
        format!("{} ", checksum)
    } else {
        let mut checksum_command = Command::new("sha256sum");
        checksum_command.arg(tmp_folder.path().join("protoc.zip"));
        let checksum_output = checksum_command.output().unwrap();
        String::from_utf8(checksum_output.stdout).unwrap()
    };

    eprintln!("checksum: {checksum_output:?}");
    assert!(checksum_output.starts_with(checksum));

    // Create the output directory if it doesn't exist
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    std::fs::create_dir_all(&out_dir).expect("Failed to create OUT_DIR");

    // Create bin directory in temp folder
    let bin_dir = tmp_folder.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("Failed to create bin directory");

    let unzip_success = if cfg!(target_os = "windows") {
        // Use PowerShell to unzip on Windows
        let mut unzip_command = Command::new("powershell");
        unzip_command
            .arg("-Command")
            .arg(format!(
                "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                tmp_folder.path().join("protoc.zip").display(),
                tmp_folder.path().display()
            ));
        unzip_command.spawn().unwrap().wait().unwrap().success()
    } else {
        let mut unzip_command = Command::new("unzip");
        unzip_command
            .arg("-o")
            .arg(tmp_folder.path().join("protoc.zip"))
            .current_dir(tmp_folder.path());
        unzip_command.spawn().unwrap().wait().unwrap().success()
    };
    assert!(unzip_success);

    let out_bin = if cfg!(target_os = "windows") {
        out_dir.join("protoc.exe")
    } else {
        out_dir.join("protoc")
    };

    // List files in tmp_folder to debug
    println!("Contents of temp folder after unzip:");
    if let Ok(entries) = std::fs::read_dir(tmp_folder.path()) {
        for entry in entries {
            if let Ok(entry) = entry {
                println!("  {}", entry.path().display());
                
                // If this is a directory, list its contents too
                if entry.path().is_dir() {
                    if let Ok(subentries) = std::fs::read_dir(entry.path()) {
                        for subentry in subentries {
                            if let Ok(subentry) = subentry {
                                println!("    {}", subentry.path().display());
                            }
                        }
                    }
                }
            }
        }
    }

    // Try to find protoc.exe in the temp folder recursively
    fn find_file(dir: &std::path::Path, filename: &str) -> Option<PathBuf> {
        println!("Searching for {} in {}", filename, dir.display());
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_file() && path.file_name().and_then(|s| s.to_str()) == Some(filename) {
                    println!("Found {} at {}", filename, path.display());
                    return Some(path);
                } else if path.is_dir() {
                    if let Some(found) = find_file(&path, filename) {
                        return Some(found);
                    }
                }
            }
        }
        None
    }
    
    let protoc_filename = if cfg!(target_os = "windows") { "protoc.exe" } else { "protoc" };
    
    if let Some(found_path) = find_file(tmp_folder.path(), protoc_filename) {
        println!("Found protoc at: {}", found_path.display());
        
        // Create parent directories if they don't exist
        if let Some(parent) = out_bin.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create parent directories");
        }
        
        // Copy the file
        println!("Copying {} to {}", found_path.display(), out_bin.display());
        std::fs::copy(&found_path, &out_bin).expect("Failed to copy protoc");
        
        // Make the binary executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&out_bin).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&out_bin, perms).unwrap();
        }
        
        println!("Setting PROTOC to: {}", out_bin.display());
        std::env::set_var("PROTOC", out_bin);
    } else {
        // If we can't find protoc in the extracted files, try to use the system protoc
        if let Ok(output) = Command::new("protoc").arg("--version").output() {
            if output.status.success() {
                let protoc_path = which::which("protoc").expect("protoc should be in PATH");
                println!("Using system protoc: {}", protoc_path.display());
                std::env::set_var("PROTOC", protoc_path);
            } else {
                panic!("Could not find protoc in the extracted files and no system protoc is available");
            }
        } else {
            panic!("Could not find protoc in the extracted files and no system protoc is available");
        }
    }
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=build.rs");

    let proto_files = std::fs::read_dir("../../proto")?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_ok_and(|t| t.is_file()))
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "proto"))
        .map(|entry| entry.path())
        .collect::<Vec<_>>();

    for file in &proto_files {
        println!("cargo:rerun-if-changed={}", file.display());
    }

    // --experimental_allow_proto3_optional is supported only on version of protoc >= 3.12
    // if the version of the system protoc is too old, we must panic
    match protoc_version() {
        Some(Version::V1([0..=2, _, _] | [3, 0..=11, _])) => download_protoc(),
        Some(Version::V1(_) | Version::V2(_)) => {},
        None => download_protoc(),
    };

    let mut config = prost_build::Config::new();

    config.protoc_arg("--experimental_allow_proto3_optional");

    #[cfg(feature = "arbitrary")]
    config.type_attribute(
        ".",
        "#[cfg_attr(feature = \"arbitrary\", derive(arbitrary::Arbitrary))]",
    );

    config.extern_path(".fig_common.Empty", "()");

    prost_reflect_build::Builder::new()
        .file_descriptor_set_path(PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("file_descriptor_set.bin"))
        .descriptor_pool("crate::DESCRIPTOR_POOL")
        .compile_protos_with_config(config, &proto_files, &["../../proto"])?;

    Ok(())
}
