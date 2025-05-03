#[cfg(feature = "windows-terminal")]
use std::io::{self, Write};
use std::time::Duration;

#[cfg(feature = "windows-terminal")]
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{self, Color, Stylize},
    terminal::{self, ClearType},
};

use eyre::{Result, eyre};

use super::context::ContextManager;

#[cfg(feature = "windows-terminal")]
/// A simple selector for Windows using crossterm
pub struct WindowsSelector {
    items: Vec<String>,
    prompt: String,
    selected_index: usize,
    multi_select: bool,
    selected_items: Vec<bool>,
    width: u16,
    height: u16,
    scroll_offset: usize,
}

#[cfg(feature = "windows-terminal")]
impl WindowsSelector {
    /// Create a new selector with the given items and prompt
    pub fn new(items: &[String], prompt: &str, multi_select: bool) -> Self {
        let selected_items = vec![false; items.len()];
        let (width, height) = terminal::size().unwrap_or((80, 24));
        
        Self {
            items: items.to_vec(),
            prompt: prompt.to_string(),
            selected_index: 0,
            multi_select,
            selected_items,
            width,
            height,
            scroll_offset: 0,
        }
    }

    /// Run the selector and return the selected items
    pub fn run(&mut self) -> Result<Option<Vec<String>>> {
        // Enter raw mode and alternate screen
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), terminal::EnterAlternateScreen)?;
        execute!(io::stdout(), cursor::Hide)?;

        let result = self.run_loop();

        // Clean up terminal state
        execute!(io::stdout(), terminal::LeaveAlternateScreen)?;
        execute!(io::stdout(), cursor::Show)?;
        terminal::disable_raw_mode()?;

        result
    }

    /// Main event loop for the selector
    fn run_loop(&mut self) -> Result<Option<Vec<String>>> {
        let mut result = None;

        loop {
            self.draw()?;

            // Handle input
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Enter => {
                        if self.multi_select {
                            let selected = self.selected_items
                                .iter()
                                .enumerate()
                                .filter_map(|(i, &selected)| {
                                    if selected {
                                        Some(self.items[i].clone())
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>();
                            
                            if !selected.is_empty() {
                                result = Some(selected);
                            } else if self.selected_index < self.items.len() {
                                result = Some(vec![self.items[self.selected_index].clone()]);
                            }
                        } else if self.selected_index < self.items.len() {
                            result = Some(vec![self.items[self.selected_index].clone()]);
                        }
                        break;
                    },
                    KeyCode::Up => {
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                            self.update_scroll();
                        }
                    },
                    KeyCode::Down => {
                        if self.selected_index < self.items.len() - 1 {
                            self.selected_index += 1;
                            self.update_scroll();
                        }
                    },
                    KeyCode::Char(' ') if self.multi_select => {
                        if self.selected_index < self.items.len() {
                            self.selected_items[self.selected_index] = !self.selected_items[self.selected_index];
                        }
                    },
                    _ => {},
                }
            }
        }

        Ok(result)
    }

    /// Update the scroll offset to keep the selected item visible
    fn update_scroll(&mut self) {
        let visible_items = self.height as usize - 3; // Account for prompt and borders
        
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + visible_items {
            self.scroll_offset = self.selected_index - visible_items + 1;
        }
    }

    /// Draw the selector UI
    fn draw(&self) -> Result<()> {
        let mut stdout = io::stdout();
        
        // Clear screen
        queue!(stdout, terminal::Clear(ClearType::All))?;
        
        // Draw prompt
        queue!(stdout, cursor::MoveTo(0, 0))?;
        queue!(stdout, style::PrintStyledContent(self.prompt.clone().with(Color::Blue)))?;
        
        // Draw items
        let visible_items = self.height as usize - 3; // Account for prompt and borders
        let end_idx = (self.scroll_offset + visible_items).min(self.items.len());
        
        for (i, item) in self.items[self.scroll_offset..end_idx].iter().enumerate() {
            let idx = i + self.scroll_offset;
            let y = (i + 2) as u16; // +2 for prompt and spacing
            
            // Highlight selected item
            let is_current = idx == self.selected_index;
            let is_selected = self.multi_select && self.selected_items[idx];
            
            queue!(stdout, cursor::MoveTo(0, y))?;
            
            // Draw selection indicator
            if is_selected {
                queue!(stdout, style::PrintStyledContent("[*] ".with(Color::Green)))?;
            } else {
                queue!(stdout, style::Print("[ ] "))?;
            }
            
            // Draw item text
            if is_current {
                queue!(stdout, style::PrintStyledContent(item.clone().with(Color::White).on(Color::Blue)))?;
            } else {
                queue!(stdout, style::Print(item))?;
            }
        }
        
        // Draw scroll indicators if needed
        if self.scroll_offset > 0 {
            queue!(stdout, cursor::MoveTo(self.width - 2, 2))?;
            queue!(stdout, style::PrintStyledContent("↑".with(Color::Yellow)))?;
        }
        
        if end_idx < self.items.len() {
            queue!(stdout, cursor::MoveTo(self.width - 2, (visible_items + 1) as u16))?;
            queue!(stdout, style::PrintStyledContent("↓".with(Color::Yellow)))?;
        }
        
        // Draw help text
        let help_text = if self.multi_select {
            "ESC: Cancel | ENTER: Confirm | SPACE: Toggle selection | ↑/↓: Navigate"
        } else {
            "ESC: Cancel | ENTER: Confirm | ↑/↓: Navigate"
        };
        
        queue!(stdout, cursor::MoveTo(0, self.height - 1))?;
        queue!(stdout, style::PrintStyledContent(help_text.with(Color::DarkGrey)))?;
        
        stdout.flush()?;
        Ok(())
    }
}

#[cfg(feature = "windows-terminal")]
/// Select profile using Windows selector
pub fn select_profile_with_windows_selector(context_manager: &ContextManager) -> Result<Option<String>> {
    let profiles = context_manager.list_profiles_blocking()?;
    
    if profiles.is_empty() {
        return Ok(None);
    }
    
    let mut selector = WindowsSelector::new(&profiles, "Select profile:", false);
    let result = selector.run()?;
    
    Ok(result.and_then(|v| v.into_iter().next()))
}

#[cfg(feature = "windows-terminal")]
/// Select files using Windows selector
pub fn select_files_with_windows_selector() -> Result<Option<Vec<String>>> {
    // Get files in current directory using glob
    let files = glob::glob("./**/*")
        .map_err(|e| eyre!("Failed to read directory: {}", e))?
        .filter_map(|entry| {
            entry.ok().and_then(|path| {
                if path.is_file() {
                    path.to_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();
    
    if files.is_empty() {
        return Ok(None);
    }
    
    let mut selector = WindowsSelector::new(&files, "Select files:", true);
    selector.run()
}

#[cfg(feature = "windows-terminal")]
/// Select context paths using Windows selector
pub fn select_context_paths_with_windows_selector(context_manager: &ContextManager) -> Result<Option<(Vec<String>, bool)>> {
    let mut global_paths = Vec::new();
    let mut profile_paths = Vec::new();
    let mut display_paths = Vec::new();
    
    // Get global paths
    for path in &context_manager.global_config.paths {
        let display = format!("(global) {}", path);
        global_paths.push(path.clone());
        display_paths.push(display);
    }
    
    // Get profile-specific paths
    for path in &context_manager.profile_config.paths {
        let display = format!("(profile: {}) {}", context_manager.current_profile, path);
        profile_paths.push(path.clone());
        display_paths.push(display);
    }
    
    if display_paths.is_empty() {
        return Ok(None);
    }
    
    let mut selector = WindowsSelector::new(&display_paths, "Select paths to remove:", true);
    let result = selector.run()?;
    
    if let Some(selected) = result {
        let mut paths = Vec::new();
        let mut has_global = false;
        
        for item in selected {
            if item.starts_with("(global)") {
                has_global = true;
                let path = item.splitn(2, ") ").nth(1).unwrap_or("").to_string();
                paths.push(path);
            } else if item.starts_with("(profile:") {
                let path = item.splitn(2, ") ").nth(1).unwrap_or("").to_string();
                paths.push(path);
            }
        }
        
        Ok(Some((paths, has_global)))
    } else {
        Ok(None)
    }
}

#[cfg(feature = "windows-terminal")]
/// Select command using Windows selector
pub fn select_command_with_windows_selector(context_manager: &ContextManager, tools: &[String]) -> Result<Option<String>> {
    use super::skim_integration::get_available_commands;
    use super::skim_integration::CommandType;
    
    let commands = get_available_commands();
    
    let mut selector = WindowsSelector::new(&commands, "Select command:", false);
    let result = selector.run()?;
    
    if let Some(selections) = result {
        if !selections.is_empty() {
            let selected_command = &selections[0];
            
            match CommandType::from_str(selected_command) {
                Some(CommandType::ContextAdd(cmd)) => {
                    // For context add commands, we need to select files
                    match select_files_with_windows_selector()? {
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
                    match select_context_paths_with_windows_selector(context_manager)? {
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
                    let mut selector = WindowsSelector::new(tools, "Select tool:", false);
                    match selector.run()? {
                        Some(selections) if !selections.is_empty() => {
                            let selected_tool = &selections[0];
                            Ok(Some(format!("{} {}", cmd, selected_tool)))
                        },
                        _ => Ok(Some(selected_command.clone())), // User cancelled tool selection
                    }
                },
                _ => Ok(Some(selected_command.clone())), // For other commands, just return as is
            }
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}