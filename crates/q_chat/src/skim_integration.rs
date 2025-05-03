#![cfg_attr(feature = "no-tuikit", allow(unused_imports))]
#[cfg(feature = "unix-terminal")]
use std::io::{
    BufReader,
    Cursor,
    Write,
    stdout,
};

#[cfg(feature = "unix-terminal")]
use crossterm::execute;
#[cfg(feature = "unix-terminal")]
use crossterm::terminal::{
    EnterAlternateScreen,
    LeaveAlternateScreen,
};
use eyre::{
    Result,
    eyre,
};
use rustyline::{
    Cmd,
    ConditionalEventHandler,
    EventContext,
    RepeatCount,
};
#[cfg(feature = "unix-terminal")]
use skim::prelude::*;
use tempfile::NamedTempFile;
use std::sync::Arc;

use super::context::ContextManager;


// Add this section to handle the case when no-tuikit is enabled
#[cfg(feature = "no-tuikit")]
pub fn select_profile_with_skim(context_manager: &ContextManager) -> Result<Option<String>> {
    // For no-tuikit builds, use the minimal selector
    super::minimal_selector::select_profile_with_minimal_selector(context_manager)
}

#[cfg(feature = "no-tuikit")]
pub fn select_files_with_skim() -> Result<Option<Vec<String>>> {
    // For no-tuikit builds, use the minimal selector
    super::minimal_selector::select_files_with_minimal_selector()
}

#[cfg(feature = "no-tuikit")]
pub fn select_context_paths_with_skim(context_manager: &ContextManager) -> Result<Option<(Vec<String>, bool)>> {
    // For no-tuikit builds, use the minimal selector
    super::minimal_selector::select_context_paths_with_minimal_selector(context_manager)
}

#[cfg(feature = "no-tuikit")]
pub fn select_command(context_manager: &ContextManager, tools: &[String]) -> Result<Option<String>> {
    // For no-tuikit builds, use the minimal selector
    super::minimal_selector::select_command_with_minimal_selector(context_manager, tools)
}

#[cfg(feature = "unix-terminal")]
pub fn select_profile_with_skim(context_manager: &ContextManager) -> Result<Option<String>> {
    let profiles = context_manager.list_profiles_blocking()?;

    launch_skim_selector(&profiles, "Select profile: ", false)
        .map(|selected| selected.and_then(|s| s.into_iter().next()))
}

#[cfg(feature = "windows-terminal")]
pub fn select_profile_with_skim(context_manager: &ContextManager) -> Result<Option<String>> {
    // For Windows, use the Windows selector
    super::windows_selector::select_profile_with_windows_selector(context_manager)
}


#[cfg(not(any(feature = "unix-terminal", feature = "windows-terminal", feature = "no-tuikit")))]
pub fn select_profile_with_skim(_context_manager: &ContextManager) -> Result<Option<String>> {
    // Fallback for when no terminal feature is enabled
    Err(eyre!("Profile selection is not supported on this platform without terminal features"))
}

pub struct SkimCommandSelector {
    context_manager: Arc<ContextManager>,
    tool_names: Vec<String>,
}

impl SkimCommandSelector {
    /// This allows the ConditionalEventHandler handle function to be bound to a KeyEvent.
    pub fn new(context_manager: Arc<ContextManager>, tool_names: Vec<String>) -> Self {
        Self {
            context_manager,
            tool_names,
        }
    }
}

#[cfg(feature = "unix-terminal")]
impl ConditionalEventHandler for SkimCommandSelector {
    fn handle(
        &self,
        _evt: &rustyline::Event,
        _n: RepeatCount,
        _positive: bool,
        _ctx: &EventContext<'_>,
    ) -> Option<Cmd> {
        // Launch skim command selector with the context manager if available
        match select_command(self.context_manager.as_ref(), &self.tool_names) {
            Ok(Some(command)) => Some(Cmd::Insert(1, command)),
            _ => {
                // If cancelled or error, do nothing
                Some(Cmd::Noop)
            },
        }
    }
}

#[cfg(feature = "windows-terminal")]
impl ConditionalEventHandler for SkimCommandSelector {
    fn handle(
        &self,
        _evt: &rustyline::Event,
        _n: RepeatCount,
        _positive: bool,
        _ctx: &EventContext<'_>,
    ) -> Option<Cmd> {
        // Launch Windows command selector with the context manager if available
        match super::windows_selector::select_command_with_windows_selector(self.context_manager.as_ref(), &self.tool_names) {
            Ok(Some(command)) => Some(Cmd::Insert(1, command)),
            _ => {
                // If cancelled or error, do nothing
                Some(Cmd::Noop)
            },
        }
    }
}

#[cfg(feature = "no-tuikit")]
impl ConditionalEventHandler for SkimCommandSelector {
    fn handle(
        &self,
        _evt: &rustyline::Event,
        _n: RepeatCount,
        _positive: bool,
        _ctx: &EventContext<'_>,
    ) -> Option<Cmd> {
        // Launch minimal command selector with the context manager if available
        match super::minimal_selector::select_command_with_minimal_selector(self.context_manager.as_ref(), &self.tool_names) {
            Ok(Some(command)) => Some(Cmd::Insert(1, command)),
            _ => {
                // If cancelled or error, do nothing
                Some(Cmd::Noop)
            },
        }
    }
}

#[cfg(not(any(feature = "unix-terminal", feature = "windows-terminal", feature = "no-tuikit")))]
impl ConditionalEventHandler for SkimCommandSelector {
    fn handle(
        &self,
        _evt: &rustyline::Event,
        _n: RepeatCount,
        _positive: bool,
        _ctx: &EventContext<'_>,
    ) -> Option<Cmd> {
        // Simple fallback when no terminal feature is enabled
        Some(Cmd::Noop)
    }
}

pub fn get_available_commands() -> Vec<String> {
    // Import the COMMANDS array directly from prompt.rs
    // This is the single source of truth for available commands
    let commands_array = super::prompt::COMMANDS;

    let mut commands = Vec::new();
    for &cmd in commands_array {
        commands.push(cmd.to_string());
    }

    commands
}

#[cfg(feature = "unix-terminal")]
/// Format commands for skim display
/// Create a standard set of skim options with consistent styling
fn create_skim_options(prompt: &str, multi: bool) -> Result<SkimOptions> {
    SkimOptionsBuilder::default()
        .height("100%".to_string())
        .prompt(prompt.to_string())
        .reverse(true)
        .multi(multi)
        .build()
        .map_err(|e| eyre!("Failed to build skim options: {}", e))
}

#[cfg(feature = "unix-terminal")]
/// Run skim with the given options and items in an alternate screen
/// This helper function handles entering/exiting the alternate screen and running skim
fn run_skim_with_options(options: &SkimOptions, items: SkimItemReceiver) -> Result<Option<Vec<Arc<dyn SkimItem>>>> {
    // Enter alternate screen to prevent skim output from persisting in terminal history
    execute!(stdout(), EnterAlternateScreen).map_err(|e| eyre!("Failed to enter alternate screen: {}", e))?;

    let selected_items =
        Skim::run_with(options, Some(items)).and_then(|out| if out.is_abort { None } else { Some(out.selected_items) });

    execute!(stdout(), LeaveAlternateScreen).map_err(|e| eyre!("Failed to leave alternate screen: {}", e))?;

    Ok(selected_items)
}

#[cfg(feature = "unix-terminal")]
/// Extract string selections from skim items
fn extract_selections(items: Vec<Arc<dyn SkimItem>>) -> Vec<String> {
    items.iter().map(|item| item.output().to_string()).collect()
}

#[cfg(feature = "unix-terminal")]
/// Launch skim with the given items and return the selected item
pub fn launch_skim_selector(items: &[String], prompt: &str, multi: bool) -> Result<Option<Vec<String>>> {
    let mut temp_file_for_skim_input = NamedTempFile::new()?;
    temp_file_for_skim_input.write_all(items.join("\n").as_bytes())?;

    let options = create_skim_options(prompt, multi)?;
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(BufReader::new(std::fs::File::open(temp_file_for_skim_input.path())?));

    // Run skim and get selected items
    match run_skim_with_options(&options, items)? {
        Some(items) if !items.is_empty() => {
            let selections = extract_selections(items);
            Ok(Some(selections))
        },
        _ => Ok(None), // User cancelled or no selection
    }
}

#[cfg(feature = "unix-terminal")]
/// Select files using skim
pub fn select_files_with_skim() -> Result<Option<Vec<String>>> {
    // Create skim options with appropriate settings
    let options = create_skim_options("Select files: ", true)?;

    // Create a command that will be executed by skim
    // This avoids loading all files into memory at once
    let find_cmd = "find . -type f -not -path '*/\\.*'";

    // Create a command collector that will execute the find command
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(BufReader::new(
        std::process::Command::new("sh")
            .args(["-c", find_cmd])
            .stdout(std::process::Stdio::piped())
            .spawn()?
            .stdout
            .ok_or_else(|| eyre!("Failed to get stdout from command"))?,
    ));

    // Run skim with the command output as a stream
    match run_skim_with_options(&options, items)? {
        Some(items) if !items.is_empty() => {
            let selections = extract_selections(items);
            Ok(Some(selections))
        },
        _ => Ok(None), // User cancelled or no selection
    }
}

#[cfg(feature = "windows-terminal")]
/// Select files using Windows selector
pub fn select_files_with_skim() -> Result<Option<Vec<String>>> {
    // For Windows, use the Windows selector
    super::windows_selector::select_files_with_windows_selector()
}

#[cfg(feature = "unix-terminal")]
/// Select context paths using skim
pub fn select_context_paths_with_skim(context_manager: &ContextManager) -> Result<Option<(Vec<String>, bool)>> {
    let mut global_paths = Vec::new();
    let mut profile_paths = Vec::new();

    // Get global paths
    for path in &context_manager.global_config.paths {
        global_paths.push(format!("(global) {}", path));
    }

    // Get profile-specific paths
    for path in &context_manager.profile_config.paths {
        profile_paths.push(format!("(profile: {}) {}", context_manager.current_profile, path));
    }

    // Combine paths, but keep track of which are global
    let mut all_paths = Vec::new();
    all_paths.extend(global_paths);
    all_paths.extend(profile_paths);

    if all_paths.is_empty() {
        return Ok(None); // No paths to select
    }

    // Create skim options
    let options = create_skim_options("Select paths to remove: ", true)?;

    // Create item reader
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(all_paths.join("\n")));

    // Run skim and get selected paths
    match run_skim_with_options(&options, items)? {
        Some(items) if !items.is_empty() => {
            let selected_paths = extract_selections(items);

            // Check if any global paths were selected
            let has_global = selected_paths.iter().any(|p| p.starts_with("(global)"));

            // Extract the actual paths from the formatted strings
            let paths: Vec<String> = selected_paths
                .iter()
                .map(|p| {
                    // Extract the path part after the prefix
                    let parts: Vec<&str> = p.splitn(2, ") ").collect();
                    if parts.len() > 1 {
                        parts[1].to_string()
                    } else {
                        p.clone()
                    }
                })
                .collect();

            Ok(Some((paths, has_global)))
        },
        _ => Ok(None), // User cancelled selection
    }
}

#[cfg(feature = "windows-terminal")]
/// Select context paths using Windows selector
pub fn select_context_paths_with_skim(context_manager: &ContextManager) -> Result<Option<(Vec<String>, bool)>> {
    // For Windows, use the Windows selector
    super::windows_selector::select_context_paths_with_windows_selector(context_manager)
}

#[cfg(feature = "unix-terminal")]
/// Launch the command selector and handle the selected command
pub fn select_command(context_manager: &ContextManager, tools: &[String]) -> Result<Option<String>> {
    let commands = get_available_commands();

    match launch_skim_selector(&commands, "Select command: ", false)? {
        Some(selections) if !selections.is_empty() => {
            let selected_command = &selections[0];

            match CommandType::from_str(selected_command) {
                Some(CommandType::ContextAdd(cmd)) => {
                    // For context add commands, we need to select files
                    match select_files_with_skim()? {
                        Some(files) if !files.is_empty() => {
                            // Construct the full command with selected files
                            let mut cmd = cmd.clone();
                            for file in files {
                                cmd.push_str(&format!(" {}", file));
                            }
                            Ok(Some(cmd))
                        },
                        _ => Ok(Some(selected_command.clone())), // User cancelled file selection
                    }
                },
                Some(CommandType::ContextRemove(cmd)) => {
                    // For context rm commands, we need to select from existing context paths
                    match select_context_paths_with_skim(context_manager)? {
                        Some((paths, has_global)) if !paths.is_empty() => {
                            // Construct the full command with selected paths
                            let mut full_cmd = cmd.clone();
                            if has_global {
                                full_cmd.push_str(" --global");
                            }
                            for path in paths {
                                full_cmd.push_str(&format!(" {}", path));
                            }
                            Ok(Some(full_cmd))
                        },
                        _ => Ok(Some(selected_command.clone())), // User cancelled path selection
                    }
                },
                Some(CommandType::Tool(cmd)) => {
                    // For tool commands, we need to select from available tools
                    match launch_skim_selector(tools, "Select tool: ", false)? {
                        Some(selections) if !selections.is_empty() => {
                            let selected_tool = &selections[0];
                            Ok(Some(format!("{} {}", cmd, selected_tool)))
                        },
                        _ => Ok(Some(selected_command.clone())), // User cancelled tool selection
                    }
                },
                _ => Ok(Some(selected_command.clone())), // For other commands, just return as is
            }
        },
        _ => Ok(None), // User cancelled command selection
    }
}

#[cfg(feature = "windows-terminal")]
/// Launch the command selector and handle the selected command
pub fn select_command(context_manager: &ContextManager, tools: &[String]) -> Result<Option<String>> {
    // For Windows, use the Windows selector
    super::windows_selector::select_command_with_windows_selector(context_manager, tools)
}

#[derive(PartialEq)]
enum CommandType {
    ContextAdd(String),
    ContextRemove(String),
    Tool(&'static str),
}

impl CommandType {
    fn from_str(cmd: &str) -> Option<CommandType> {
        if cmd.starts_with("/context add") {
            Some(CommandType::ContextAdd(cmd.to_string()))
        } else if cmd.starts_with("/context rm") {
            Some(CommandType::ContextRemove(cmd.to_string()))
        } else if cmd.starts_with("/tool") {
            Some(CommandType::Tool("tool"))
        } else {
            None
        }
    }
}
