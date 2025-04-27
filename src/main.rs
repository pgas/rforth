use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::collections::HashMap; // Import HashMap
use std::fs;
use std::io::{self, BufRead};
use std::path::PathBuf;

mod eval;
mod number_ops; // Declare the number_ops module for arithmetic and comparisons
mod parser;
mod stack_ops; // Declare the stack_ops module
mod token;

use crate::eval::eval; // Import eval function
use crate::parser::ForthOp; // Import ForthOp
use crate::parser::parse;
use logos::Logos;
use token::Token; // Import parse function

fn get_history_path() -> Option<PathBuf> {
    home::home_dir().map(|mut path| {
        path.push(".rforth");
        path.push("history");
        path
    })
}

// Use std::result::Result to avoid conflict with rustyline::Result
fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("welcome to rforth");

    let history_path = get_history_path();
    let mut stack: Vec<i64> = Vec::new(); // The Forth stack
    let mut dictionary: HashMap<String, Vec<ForthOp>> = HashMap::new(); // Create the dictionary

    if atty::is(atty::Stream::Stdin) {
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

        let mut pending_tokens = Vec::new(); // Buffer for multi-line definitions
        loop {
            let readline = rl.readline(">> ");
            match readline {
                Ok(line) => {
                    // Lex this line
                    let line_tokens: Vec<Token> =
                        Token::lexer(&line).filter_map(|r| r.ok()).collect();
                    // Append into pending buffer
                    pending_tokens.extend(line_tokens);
                    if pending_tokens.is_empty() {
                        continue; // nothing to do
                    }
                    // Try parsing buffered tokens
                    match parse(pending_tokens.clone()) {
                        Ok(ops) => {
                            // Successfully parsed a complete definition or sequence
                            pending_tokens.clear();
                            if let Err(e) = eval(&ops, &mut stack, &mut dictionary) {
                                eprintln!("Error: {}", e);
                            }
                        }
                        Err(e) => {
                            // If still inside definition or conditional, wait for more lines
                            if matches!(
                                e,
                                crate::parser::ParseError::UnterminatedDefinition
                                    | crate::parser::ParseError::UnterminatedConditional
                            ) {
                                continue;
                            }
                            // Otherwise report and clear buffer
                            eprintln!("Parse Error: {:?}", e);
                            pending_tokens.clear();
                        }
                    }
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
        let mut pending_tokens = Vec::new();
        for line in stdin.lock().lines() {
            match line {
                Ok(l) => {
                    // Lex this line
                    let line_tokens: Vec<Token> = Token::lexer(&l).filter_map(|r| r.ok()).collect();
                    pending_tokens.extend(line_tokens);
                    if pending_tokens.is_empty() {
                        continue;
                    }
                    // Try parsing buffered tokens
                    match parse(pending_tokens.clone()) {
                        Ok(ops) => {
                            pending_tokens.clear();
                            if let Err(e) = eval(&ops, &mut stack, &mut dictionary) {
                                eprintln!("Error: {}", e);
                            }
                        }
                        Err(e) => {
                            if matches!(
                                e,
                                crate::parser::ParseError::UnterminatedDefinition
                                    | crate::parser::ParseError::UnterminatedConditional
                            ) {
                                continue;
                            }
                            eprintln!("Parse Error: {:?}", e);
                            pending_tokens.clear();
                        }
                    }
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
