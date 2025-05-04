use anyhow::Result;
use q_cli::{Color, Input, PlatformTerminal, Terminal};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the terminal
    let mut terminal = PlatformTerminal::init()?;
    
    // Clear the screen
    terminal.clear_screen()?;
    
    // Hide cursor
    terminal.hide_cursor()?;
    
    // Get terminal size
    let size = terminal.get_size()?;
    
    // Move to center of screen
    let center_x = size.width / 2;
    let center_y = size.height / 2;
    
    // Display welcome message with colors
    terminal.move_cursor(center_x - 15, center_y - 2)?;
    terminal.set_color(Color::BrightCyan, Color::Reset)?;
    terminal.write_output("Terminal Abstraction Demo")?;
    
    terminal.move_cursor(center_x - 20, center_y)?;
    terminal.set_color(Color::Yellow, Color::Reset)?;
    terminal.write_output("Press any key to see its representation")?;
    
    terminal.move_cursor(center_x - 10, center_y + 2)?;
    terminal.set_color(Color::BrightRed, Color::Reset)?;
    terminal.write_output("Press ESC to exit")?;
    
    // Show cursor
    terminal.show_cursor()?;
    
    // Input loop
    loop {
        let input = terminal.read_input()?;
        
        // Exit on ESC
        if input == Input::Esc {
            break;
        }
        
        // Display the input
        terminal.move_cursor(center_x - 10, center_y + 4)?;
        terminal.set_color(Color::BrightGreen, Color::Reset)?;
        terminal.write_output(&format!("You pressed: {:?}    ", input))?;
        
        // Small delay to prevent too rapid updates
        sleep(Duration::from_millis(10)).await;
    }
    
    // Clean up
    terminal.reset_color()?;
    terminal.clear_screen()?;
    terminal.move_cursor(0, 0)?;
    terminal.write_output("Goodbye!\n")?;
    terminal.cleanup()?;
    
    Ok(())
}
