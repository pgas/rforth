#[cfg(feature = "jit")]
use inkwell::context::Context;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::collections::HashMap; // Import HashMap
use std::fs;
use std::io::{self, BufRead};
use std::path::PathBuf;

mod eval;
mod jit;
mod number_ops; // Declare the number_ops module for arithmetic and comparisons
mod parser;
mod stack_ops; // Declare the stack_ops module
mod token; // Add the JIT module

use crate::eval::{DictEntry, Evaluator}; // Updated import to use Evaluator instead of eval function
#[cfg(feature = "jit")]
use crate::jit::jit_impl::ForthJit;
use crate::parser::{parse, ForthOp};
use logos::Logos;
use token::Token; // Import parse function

// Store JIT context in a thread-local variable instead of a static mut
#[cfg(feature = "jit")]
thread_local! {
    static JIT_CONTEXT: Context = Context::create();
}

// Create a new function to evaluate operations to replace direct eval calls
fn evaluate_ops(
    ops: &[ForthOp],
    stack: &mut Vec<i64>,
    dictionary: &mut HashMap<String, DictEntry>,
    _loop_control_stack: &mut Vec<(usize, i64, i64)>, // Prefixed with _ to indicate it's intentionally unused
    _latest_word: &mut Option<String>, // Prefixed with _ to indicate it's intentionally unused
) -> Result<(), anyhow::Error> {
    // Create a temporary evaluator that processes the operations
    let mut evaluator = Evaluator::new(false);

    // Initialize the evaluator with current state
    *evaluator.get_stack_mut() = stack.clone();
    evaluator.import_dictionary(dictionary);

    // Process operations
    evaluator.eval(ops)?;

    // Update the caller's state
    *stack = evaluator.get_stack().clone();
    *dictionary = evaluator.get_dictionary().clone();

    Ok(())
}

// Function to process a line of input
fn process_line(
    line: &str,
    pending_tokens: &mut Vec<Token>,
    stack: &mut Vec<i64>,
    dictionary: &mut HashMap<String, DictEntry>,
    loop_control_stack: &mut Vec<(usize, i64, i64)>, // Added loop stack
    latest_word: &mut Option<String>,                // Added latest word tracking
) {
    // Lex this line
    let line_tokens: Vec<Token> = Token::lexer(line).filter_map(|r| r.ok()).collect();
    // Append into pending buffer
    pending_tokens.extend(line_tokens);
    if pending_tokens.is_empty() {
        return; // nothing to do
    }

    // Try parsing buffered tokens
    match parse(pending_tokens.clone()) {
        Ok(ops) => {
            // Successfully parsed a complete definition or sequence
            pending_tokens.clear();

            // Check for Define operation to update latest_word
            for op in &ops {
                if let ForthOp::Define(name, body, immediate) = op {
                    *latest_word = Some(name.clone());

                    // JIT compile newly defined words when appropriate
                    #[cfg(feature = "jit")]
                    if !immediate {
                        if let Err(e) = jit_compile_word(name, body, dictionary) {
                            eprintln!("JIT compilation error: {}", e);
                        }
                    }
                }
            }

            // Pass loop_control_stack and latest_word to eval
            if let Err(e) = evaluate_ops(&ops, stack, dictionary, loop_control_stack, latest_word) {
                eprintln!("Error: {}", e);
                // Consider clearing loop_control_stack on error? Maybe not, depends on desired behavior.
            }
        }
        Err(e) => {
            // If still inside definition or conditional, wait for more lines
            if matches!(
                e,
                crate::parser::ParseError::UnterminatedDefinition
                    | crate::parser::ParseError::UnterminatedConditional
            ) {
                // Do nothing, wait for more input
            } else {
                // Otherwise report and clear buffer
                eprintln!("Parse Error: {:?}", e);
                pending_tokens.clear();
            }
        }
    }
}

#[cfg(feature = "jit")]
fn jit_compile_word(
    name: &str,
    body: &[ForthOp],
    dictionary: &mut HashMap<String, DictEntry>,
) -> Result<(), anyhow::Error> {
    // Use the thread-local JIT context
    JIT_CONTEXT.with(|ctx| {
        // Create a JIT compiler with this context
        let mut jit_compiler = ForthJit::new(ctx)?;

        // Try to compile the word
        match jit_compiler.compile_word(name, body) {
            Ok(compiled_fn) => {
                if let Some(entry) = dictionary.get_mut(name) {
                    entry.compiled_code = Some(compiled_fn);
                    println!("JIT compiled: {}", name);
                }
                Ok(())
            }
            Err(e) => {
                // Just a warning - we'll fall back to interpreter
                eprintln!("JIT compilation warning for {}: {:?}", name, e);
                Ok(())
            }
        }
    })
}

// Use std::result::Result to avoid conflict with rustyline::Result
fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("welcome to rforth");

    #[cfg(feature = "jit")]
    {
        println!("JIT compilation enabled");
        // The JIT context is initialized lazily via the thread_local! macro
    }

    let history_path = get_history_path();
    let mut stack: Vec<i64> = Vec::new(); // The Forth stack
    let mut dictionary: HashMap<String, DictEntry> = HashMap::new(); // Create the dictionary
    let mut loop_control_stack: Vec<(usize, i64, i64)> = Vec::new(); // Initialize loop stack
    let mut latest_word: Option<String> = None; // Initialize latest word tracking

    let mut pending_tokens = Vec::new(); // Buffer for multi-line definitions

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

        loop {
            let readline = rl.readline(">> ");
            match readline {
                Ok(line) => {
                    // Add line to history before processing
                    let _ = rl.add_history_entry(line.as_str());
                    // Pass loop_control_stack and latest_word to process_line
                    process_line(
                        &line,
                        &mut pending_tokens,
                        &mut stack,
                        &mut dictionary,
                        &mut loop_control_stack,
                        &mut latest_word,
                    );
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
                    // Pass loop_control_stack and latest_word to process_line
                    process_line(
                        &l,
                        &mut pending_tokens,
                        &mut stack,
                        &mut dictionary,
                        &mut loop_control_stack,
                        &mut latest_word,
                    );
                }
                Err(e) => {
                    eprintln!("Error reading stdin: {}", e);
                    break;
                }
            }
        }
        // After processing all lines from stdin, check if there's anything left in pending_tokens
        // This might happen if the input ends mid-definition or conditional.
        // We could choose to error, warn, or attempt final processing.
        // For now, let's just clear it if it's an unterminated state, otherwise try one last parse/eval.
        if !pending_tokens.is_empty() {
            match parse(pending_tokens.clone()) {
                Ok(ops) => {
                    // Pass loop_control_stack and latest_word to final eval
                    if let Err(e) = evaluate_ops(
                        &ops,
                        &mut stack,
                        &mut dictionary,
                        &mut loop_control_stack,
                        &mut latest_word,
                    ) {
                        eprintln!("Error processing remaining input: {}", e);
                    }
                }
                Err(e) => {
                    if !matches!(
                        e,
                        crate::parser::ParseError::UnterminatedDefinition
                            | crate::parser::ParseError::UnterminatedConditional
                    ) {
                        eprintln!("Parse Error processing remaining input: {:?}", e);
                    } else {
                        eprintln!(
                            "Warning: Input ended with unterminated definition or conditional."
                        );
                    }
                }
            }
            pending_tokens.clear(); // Clear buffer regardless
        }
        // Check if loop stack is non-empty at the end (indicates unterminated loop in piped input)
        if !loop_control_stack.is_empty() {
            eprintln!("Warning: Input ended with unbalanced DO/LOOP structures.");
            // Optionally clear the loop stack here if desired
            // loop_control_stack.clear();
        }
    }

    Ok(())
}

// Add this function at the end of the file, before the tests module

// Returns the path to the history file
fn get_history_path() -> Option<PathBuf> {
    home::home_dir().map(|dir| dir.join(".rforth").join("history"))
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
