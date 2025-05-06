use std::path::PathBuf;
use std::sync::Weak;
use std::collections::HashMap;

use windows::Win32::Foundation::{CloseHandle, HANDLE, HINSTANCE, HWND};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
};
use windows::core::{PWSTR, HRESULT};
use wmi::{COMLibrary, WMIConnection, FilterValue, WMIError};
use serde::Deserialize;

use crate::Context;
use super::{Pid, RawPid};

pub fn current(ctx: Weak<Context>) -> Pid {
    let pid = std::process::id();
    Pid::Real(ctx, RawPid(pid))
}

pub fn parent(ctx: Weak<Context>, pid: &RawPid) -> Option<Box<Pid>> {
    // Create a snapshot of all processes
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot.is_err() {
        return None;
    }

    let snapshot = snapshot.unwrap();

    // Initialize process entry structure
    let mut process_entry = PROCESSENTRY32 {
        dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32,
        ..Default::default()
    };

    // Get the first process
    let mut result = unsafe { Process32First(snapshot, &mut process_entry) };
    if result.is_err() {
        unsafe { CloseHandle(snapshot) };
        return None;
    }

    // Find the process with the matching PID
    let target_pid = pid.as_u32();
    let mut parent_pid = 0;

    while result.is_ok() {
        if process_entry.th32ProcessID == target_pid {
            parent_pid = process_entry.th32ParentProcessID;
            break;
        }
        result = unsafe { Process32Next(snapshot, &mut process_entry) };
    }

    // Clean up
    unsafe { CloseHandle(snapshot) };

    // Return the parent PID if found
    if parent_pid > 0 {
        Some(Box::new(Pid::Real(ctx, RawPid(parent_pid))))
    } else {
        None
    }
}

pub fn exe(_ctx: Weak<Context>, pid: &RawPid) -> Option<PathBuf> {
    // Open the process with query information access
    let process_handle = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION,
            false,
            pid.as_u32(),
        )
    };

    if process_handle.is_err() {
        return None;
    }

    let handle = process_handle.unwrap();

    // Buffer to store the path
    let mut buffer = [0u16; 1024];
    let mut size = buffer.len() as u32;

    // Get the process executable path
    let result = unsafe {
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_FORMAT(0),
            PWSTR(buffer.as_mut_ptr()),
            &mut size,
        )
    };

    // Clean up
    unsafe { CloseHandle(handle) };

    if result.is_ok() && size > 0 {
        // Convert to PathBuf
        let path = String::from_utf16_lossy(&buffer[..size as usize]);
        Some(PathBuf::from(path))
    } else {
        None
    }
}

#[derive(Deserialize, Debug)]
struct WmiProcess {
    CommandLine: Option<String>,
}

pub fn cmdline(_ctx: Weak<Context>, pid: &RawPid) -> Option<String> {
    // Initialize COM library with better error handling
    let com_lib = match COMLibrary::new() {
        Ok(lib) => lib,
        Err(err) => {
            tracing::error!("Failed to initialize COM library: {:?}", err);
            return None;
        }
    };
    
    // Connect to WMI with better error handling
    let wmi_con = match WMIConnection::new(com_lib) {
        Ok(con) => con,
        Err(err) => {
            tracing::error!("Failed to connect to WMI: {:?}", err);
            return None;
        }
    };
    
    // Create filter for the process ID
    let mut filters = HashMap::new();
    filters.insert("ProcessId".to_string(), FilterValue::Number(pid.as_u32() as i64));
    
    // Query WMI for the process with the given PID
    let processes: Result<Vec<WmiProcess>, WMIError> = wmi_con.filtered_query(&filters);
    
    match processes {
        Ok(procs) => {
            // Return the command line if found
            procs.first().and_then(|p| p.CommandLine.clone())
        },
        Err(err) => {
            tracing::error!("Failed to query WMI for process {}: {:?}", pid.as_u32(), err);
            None
        }
    }
}

// Helper function to handle Windows errors
fn handle_windows_error(hr: HRESULT, operation: &str) -> Option<()> {
    if hr.is_ok() {
        Some(())
    } else {
        tracing::error!("Windows API error during {}: {:?}", operation, hr);
        None
    }
}