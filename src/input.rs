use std::io::{self, Write};

/// Read a device index from stdin (command-line input)
pub fn read_index(max: usize) -> Result<usize, Box<dyn std::error::Error>> {
    loop {
        print!("Enter index: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim().parse::<usize>() {
            Ok(idx) if idx < max => return Ok(idx),
            Ok(_) => println!("Index out of range. Please enter a number between 0 and {}", max - 1),
            Err(_) => println!("Invalid input. Please enter a number."),
        }
    }
}

/// Read an optional device index from stdin (-1 to skip, command-line input)
pub fn read_index_optional(max: usize) -> Result<Option<usize>, Box<dyn std::error::Error>> {
    loop {
        print!("Enter index: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let trimmed = input.trim();
        if trimmed == "-1" {
            return Ok(None);
        }
        
        match trimmed.parse::<usize>() {
            Ok(idx) if idx < max => return Ok(Some(idx)),
            Ok(_) => println!("Index out of range. Please enter a number between 0 and {} (or -1 to skip)", max - 1),
            Err(_) => println!("Invalid input. Please enter a number or -1 to skip."),
        }
    }
}

