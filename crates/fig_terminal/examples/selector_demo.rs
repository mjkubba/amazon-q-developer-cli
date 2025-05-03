use fig_terminal::Selector;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a list of items
    let items = vec![
        "Option 1".to_string(),
        "Option 2".to_string(),
        "Option 3".to_string(),
        "Option 4".to_string(),
        "Option 5".to_string(),
    ];
    
    // Create a selector with a prompt
    let mut selector = Selector::new(items, "Select an option (use arrow keys, Enter to select, q to quit):")?;
    
    // Run the selector and get the result
    let result = selector.run()?;
    
    // Print the result
    match result {
        Some(selected) => println!("You selected: {}", selected),
        None => println!("No selection made"),
    }
    
    Ok(())
}
