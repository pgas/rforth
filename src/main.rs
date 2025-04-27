use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use std::fs;
use std::io::{self, BufRead};
use std::path::PathBuf;

fn get_history_path() -> Option<PathBuf> {
    home::home_dir().map(|mut path| {
        path.push(".rforth");
        path.push("history");
        path
    })
}

fn main() -> Result<()> {
    println!("welcome to rforth");

    let history_path = get_history_path();

    if atty::is(atty::Stream::Stdin) {
        // Interactive terminal session
        let mut rl = DefaultEditor::new()?;

        if let Some(ref path) = history_path {
            // Create the directory if it doesn't exist
            if let Some(dir) = path.parent() {
                let _ = fs::create_dir_all(dir); // Ignore error if dir exists or cannot be created
            }
            // Attempt to load history, ignore error if file doesn't exist
            if rl.load_history(path).is_err() {
                // Optionally print a warning, e.g.:
                // eprintln!("No previous history found at {:?}", path);
            }
        }

        loop {
            let readline = rl.readline(">> ");
            match readline {
                Ok(line) => {
                    if !line.trim().is_empty() {
                        // Don't save empty lines
                        // Use _path to silence the unused variable warning
                        if let Some(ref _path) = history_path {
                            rl.add_history_entry(line.as_str())?;
                        }
                    }
                    println!("Echo: {}", line);
                    // TODO: Add eval step here
                }
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
        if let Some(ref path) = history_path {
            if let Err(err) = rl.save_history(path) {
                eprintln!("Failed to save history to {:?}: {}", path, err);
            }
        }
    } else {
        // Piped input
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(l) => {
                    println!("Echo: {}", l);
                    // TODO: Add eval step here
                }
                Err(e) => {
                    eprintln!("Error reading stdin: {}", e);
                    break;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_history_path_constructs_correctly() {
        // This test assumes the home directory can be found.
        // It might fail in unusual environments where HOME isn't set.
        if let Some(home_dir) = home::home_dir() {
            let expected_path = home_dir.join(".rforth").join("history");
            assert_eq!(get_history_path(), Some(expected_path));
        } else {
            // If home dir is not found, the function should return None
            assert_eq!(get_history_path(), None);
            // Or, we might choose to panic or skip if home dir is essential for the test
            // panic!("Could not determine home directory for testing get_history_path");
        }
    }
}
