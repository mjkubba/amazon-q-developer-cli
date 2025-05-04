#![cfg(feature = "minimal")]

use eyre::Result;
use std::io::{self, Write};

pub struct MinimalSelector {
    items: Vec<String>,
    prompt: String,
}

impl MinimalSelector {
    pub fn new(items: Vec<String>, prompt: &str) -> Result<Self> {
        Ok(Self {
            items,
            prompt: prompt.to_string(),
        })
    }
    
    pub fn run(&mut self) -> Result<Option<String>> {
        println!("{}", self.prompt);
        println!();
        
        // Print items with numbers
        for (i, item) in self.items.iter().enumerate() {
            println!("{}. {}", i + 1, item);
        }
        
        println!();
        print!("Enter selection (1-{}) or 0 to cancel: ", self.items.len());
        io::stdout().flush()?;
        
        // Read input
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        // Parse input
        match input.trim().parse::<usize>() {
            Ok(0) => Ok(None),
            Ok(n) if n <= self.items.len() => Ok(Some(self.items[n - 1].clone())),
            _ => {
                println!("Invalid selection. Cancelling.");
                Ok(None)
            }
        }
    }
}
