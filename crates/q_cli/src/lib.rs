pub mod cli;
pub mod terminal;
pub mod util;

// Re-export the terminal module for easy access
pub use terminal::{Color, Input, PlatformTerminal, Terminal, TerminalSize};
