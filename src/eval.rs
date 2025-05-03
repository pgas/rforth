use crate::parser::ForthOp;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

#[cfg(feature = "jit")]
use crate::jit::jit_impl::ForthJit;
#[cfg(feature = "jit")]
use inkwell::context::Context;

// Store the JIT context globally in a thread-local variable to extend its lifetime
// This avoids the Sync requirement that causes issues with thread safety
#[cfg(feature = "jit")]
thread_local! {
    static JIT_CONTEXT: Context = Context::create();
}

// Dictionary entry definition
#[derive(Clone)]
pub struct DictEntry {
    pub body: Vec<ForthOp>,
    pub immediate: bool,
    #[cfg(feature = "jit")]
    pub compiled_code: Option<unsafe extern "C" fn(*mut i64, i64) -> i64>,
}

// The Forth evaluator
pub struct Evaluator {
    stack: Vec<i64>,
    dictionary: HashMap<String, DictEntry>,
    source_mode: bool,
    compile_mode: bool,
    defining_word: Option<String>,
    defining_body: Vec<ForthOp>,
    immediate_flag: bool,
    loop_stack: Vec<(usize, i64, i64)>,
    #[cfg(feature = "jit")]
    jit: Option<ForthJit<'static>>,
}

impl Evaluator {
    pub fn new(source_mode: bool) -> Self {
        // Initialize the JIT compiler if the JIT feature is enabled
        #[cfg(feature = "jit")]
        let jit = JIT_CONTEXT.with(|ctx| {
            // We use a static lifetime for the JIT compiler to avoid lifetime issues
            let ctx_ptr = ctx as *const Context;
            // SAFETY: This is safe because the JIT_CONTEXT is stored in thread_local storage
            // and will live for the duration of the thread
            let ctx_ref = unsafe { &*ctx_ptr };
            ForthJit::new(ctx_ref).ok()
        });

        Self {
            stack: Vec::new(),
            dictionary: HashMap::new(),
            source_mode,
            compile_mode: false,
            defining_word: None,
            defining_body: Vec::new(),
            immediate_flag: false,
            loop_stack: Vec::new(),
            #[cfg(feature = "jit")]
            jit,
        }
    }

    // Evaluate a sequence of Forth operations
    pub fn eval(&mut self, ops: &[ForthOp]) -> Result<()> {
        for op in ops {
            self.eval_op(op)?;
        }
        Ok(())
    }

    // Main evaluation function
    fn eval_op(&mut self, op: &ForthOp) -> Result<()> {
        if self.compile_mode {
            match op {
                ForthOp::Define(_, _, _) => {
                    // Allow nested defines (the parser already handles this)
                    self.defining_body.push(op.clone());
                }
                ForthOp::Immediate => {
                    self.immediate_flag = true;
                    // Do not add to the defining body
                }
                ForthOp::Word(word) if word.to_lowercase() == ";" => {
                    // End definition
                    let name = self.defining_word.take().unwrap();
                    let body = std::mem::take(&mut self.defining_body);

                    // Define the word in dictionary
                    self.dictionary.insert(
                        name.clone(),
                        DictEntry {
                            body: body.clone(),
                            immediate: self.immediate_flag,
                            #[cfg(feature = "jit")]
                            compiled_code: None,
                        },
                    );

                    // JIT compile the word if enabled
                    #[cfg(feature = "jit")]
                    if let Some(jit) = &mut self.jit {
                        // Create a clone of the name and body for compilation
                        // This avoids borrowing issues with the dictionary
                        let word_name = name.clone();
                        let word_body = body.clone();

                        // Try to compile the word
                        match jit.compile_word(&word_name, &word_body) {
                            Ok(compiled_fn) => {
                                // Store the compiled function in the dictionary
                                if let Some(entry) = self.dictionary.get_mut(&name) {
                                    entry.compiled_code = Some(compiled_fn);
                                    if !self.source_mode {
                                        println!("JIT compiled: {}", name);
                                    }
                                }
                            }
                            Err(e) => {
                                // Just print the error and continue with interpretation
                                if !self.source_mode {
                                    eprintln!(
                                        "Warning: JIT compilation failed for '{}': {:?}",
                                        name, e
                                    );
                                }
                            }
                        }
                    }

                    self.compile_mode = false;
                    self.immediate_flag = false;
                }
                _ => {
                    // Add to the definition body
                    self.defining_body.push(op.clone());
                }
            }
        } else {
            match op {
                ForthOp::Push(val) => {
                    self.stack.push(*val);
                }
                ForthOp::Add => self.binary_op(|a, b| a + b)?,
                ForthOp::Subtract => self.binary_op(|a, b| a - b)?,
                ForthOp::Multiply => self.binary_op(|a, b| a * b)?,
                ForthOp::Divide => self.binary_op(|a, b| a / b)?,
                ForthOp::Mod => self.binary_op(|a, b| a % b)?,
                ForthOp::Eq => self.binary_op(|a, b| if a == b { -1 } else { 0 })?,
                ForthOp::Lt => self.binary_op(|a, b| if a < b { -1 } else { 0 })?,
                ForthOp::Gt => self.binary_op(|a, b| if a > b { -1 } else { 0 })?,
                ForthOp::Dup => {
                    if self.stack.is_empty() {
                        return Err(anyhow!("Stack underflow"));
                    }
                    let top = *self.stack.last().unwrap();
                    self.stack.push(top);
                }
                ForthOp::Drop => {
                    if self.stack.is_empty() {
                        return Err(anyhow!("Stack underflow"));
                    }
                    self.stack.pop();
                }
                ForthOp::Swap => {
                    if self.stack.len() < 2 {
                        return Err(anyhow!("Stack underflow"));
                    }
                    let len = self.stack.len();
                    self.stack.swap(len - 1, len - 2);
                }
                ForthOp::Over => {
                    if self.stack.len() < 2 {
                        return Err(anyhow!("Stack underflow"));
                    }
                    let second = self.stack[self.stack.len() - 2];
                    self.stack.push(second);
                }
                ForthOp::Rot => {
                    if self.stack.len() < 3 {
                        return Err(anyhow!("Stack underflow"));
                    }
                    let len = self.stack.len();
                    let third = self.stack.remove(len - 3);
                    self.stack.push(third);
                }
                ForthOp::QDup => {
                    if self.stack.is_empty() {
                        return Err(anyhow!("Stack underflow"));
                    }
                    let top = *self.stack.last().unwrap();
                    if top != 0 {
                        self.stack.push(top);
                    }
                }
                ForthOp::TwoDup => {
                    if self.stack.len() < 2 {
                        return Err(anyhow!("Stack underflow"));
                    }
                    let len = self.stack.len();
                    let second = self.stack[len - 2];
                    let top = self.stack[len - 1];
                    self.stack.push(second);
                    self.stack.push(top);
                }
                ForthOp::TwoDrop => {
                    if self.stack.len() < 2 {
                        return Err(anyhow!("Stack underflow"));
                    }
                    self.stack.pop();
                    self.stack.pop();
                }
                ForthOp::TwoSwap => {
                    if self.stack.len() < 4 {
                        return Err(anyhow!("Stack underflow"));
                    }
                    let len = self.stack.len();
                    self.stack.swap(len - 1, len - 3);
                    self.stack.swap(len - 2, len - 4);
                }
                ForthOp::TwoOver => {
                    if self.stack.len() < 4 {
                        return Err(anyhow!("Stack underflow"));
                    }
                    let len = self.stack.len();
                    let third = self.stack[len - 3];
                    let fourth = self.stack[len - 4];
                    self.stack.push(fourth);
                    self.stack.push(third);
                }
                ForthOp::MinusRot => {
                    if self.stack.len() < 3 {
                        return Err(anyhow!("Stack underflow"));
                    }
                    let len = self.stack.len();
                    let top = self.stack.remove(len - 1);
                    self.stack.insert(len - 3, top);
                }
                ForthOp::Print => {
                    if self.stack.is_empty() {
                        return Err(anyhow!("Stack underflow"));
                    }
                    let top = self.stack.pop().unwrap();
                    println!("{}", top);
                }
                ForthOp::PrintStack => {
                    print!("<{}> ", self.stack.len());
                    for &item in &self.stack {
                        print!("{} ", item);
                    }
                    println!();
                }
                ForthOp::Word(word) => {
                    // Check if the word is ":" to start a definition
                    if word.to_lowercase() == ":" {
                        self.compile_mode = true;
                        return Ok(());
                    }

                    // Check if the word exists in the dictionary
                    if let Some(entry) = self.dictionary.get(word) {
                        if entry.immediate || !self.compile_mode {
                            // Try to use JIT-compiled code if available
                            #[cfg(feature = "jit")]
                            if !self.source_mode && !self.compile_mode {
                                if let Some(compiled_fn) = entry.compiled_code {
                                    // Prepare stack for JIT execution
                                    let mut raw_stack = self.stack.clone();
                                    raw_stack.insert(0, 0); // Reserve space for stack operations

                                    // Execute the compiled code
                                    unsafe {
                                        let ptr = raw_stack.as_mut_ptr();
                                        let initial_top = (raw_stack.len() - 1) as i64;
                                        let new_top = compiled_fn(ptr, initial_top);

                                        // Check for errors (negative top means error)
                                        if new_top == -1 {
                                            return Err(anyhow!("Runtime error in JIT code"));
                                        }

                                        // Update the stack
                                        let new_top = new_top as usize;
                                        self.stack = raw_stack[1..=new_top].to_vec();
                                    }
                                    return Ok(());
                                }
                            }

                            // If no JIT or using interpreter, evaluate the body
                            // Clone the body to avoid borrow conflict
                            let body = entry.body.clone();
                            self.eval(&body)?;
                        } else {
                            // In compile mode, add the word to the definition
                            self.defining_body.push(op.clone());
                        }
                    } else {
                        return Err(anyhow!("Unknown word: {}", word));
                    }
                }
                ForthOp::Define(name, body, immediate) => {
                    let entry = DictEntry {
                        body: body.clone(),
                        immediate: *immediate,
                        #[cfg(feature = "jit")]
                        compiled_code: None,
                    };
                    self.dictionary.insert(name.clone(), entry);

                    // JIT compile the word if enabled
                    #[cfg(feature = "jit")]
                    if let Some(jit) = &mut self.jit {
                        // Try to compile the word
                        match jit.compile_word(name, body) {
                            Ok(compiled_fn) => {
                                // Store the compiled function in the dictionary
                                if let Some(entry) = self.dictionary.get_mut(name) {
                                    entry.compiled_code = Some(compiled_fn);
                                    if !self.source_mode {
                                        println!("JIT compiled: {}", name);
                                    }
                                }
                            }
                            Err(e) => {
                                // Just print the error and continue with interpretation
                                if !self.source_mode {
                                    eprintln!(
                                        "Warning: JIT compilation failed for '{}': {:?}",
                                        name, e
                                    );
                                }
                            }
                        }
                    }
                }
                ForthOp::Immediate => {
                    // Mark the most recently defined word as immediate
                    // This operation is a no-op outside of word definition
                }
                ForthOp::IfElse(then_ops, else_ops) => {
                    if self.stack.is_empty() {
                        return Err(anyhow!("Stack underflow"));
                    }
                    let condition = self.stack.pop().unwrap();
                    if condition != 0 {
                        // True branch
                        self.eval(then_ops)?;
                    } else {
                        // False branch
                        self.eval(else_ops)?;
                    }
                }
                ForthOp::Do => {
                    if self.stack.len() < 2 {
                        return Err(anyhow!("Stack underflow"));
                    }
                    let start = self.stack.pop().unwrap();
                    let limit = self.stack.pop().unwrap();

                    // Store the location in the loop stack (index 0 is placeholder)
                    self.loop_stack.push((0, start, limit));
                }
                ForthOp::Loop => {
                    if self.loop_stack.is_empty() {
                        return Err(anyhow!("Loop stack underflow"));
                    }

                    let (_, mut index, limit) = self.loop_stack.pop().unwrap();

                    // Increment the index
                    index += 1;

                    // Check if we need to continue looping
                    if index < limit {
                        // Put the loop state back
                        self.loop_stack.push((0, index, limit));

                        // Loop back to the DO (handled by caller)
                        return Ok(());
                    }
                    // Otherwise, we're done with this loop
                }
                ForthOp::I => {
                    if self.loop_stack.is_empty() {
                        return Err(anyhow!("Loop stack underflow"));
                    }

                    // Get current loop index (don't pop)
                    let (_, index, _) = *self.loop_stack.last().unwrap();

                    // Push to data stack
                    self.stack.push(index);
                }
            }
        }
        Ok(())
    }

    // Helper function for binary operations
    fn binary_op<F>(&mut self, op: F) -> Result<()>
    where
        F: FnOnce(i64, i64) -> i64,
    {
        if self.stack.len() < 2 {
            return Err(anyhow!("Stack underflow"));
        }
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        self.stack.push(op(a, b));
        Ok(())
    }

    // Get the current stack
    pub fn get_stack(&self) -> &Vec<i64> {
        &self.stack
    }

    // Get mutable reference to stack (for testing)
    pub fn get_stack_mut(&mut self) -> &mut Vec<i64> {
        &mut self.stack
    }

    // Import dictionary entries from another evaluator
    pub fn import_dictionary(&mut self, other: &HashMap<String, DictEntry>) {
        for (name, entry) in other {
            self.dictionary.insert(name.clone(), entry.clone());
        }
    }

    // Get the dictionary
    pub fn get_dictionary(&self) -> &HashMap<String, DictEntry> {
        &self.dictionary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::token::Token;
    use logos::Logos;

    // Simple helper to run Forth code and get the stack
    fn run_forth(code: &str) -> Result<Vec<i64>> {
        // Tokenize the input
        let tokens: Vec<Token> = Token::lexer(code).filter_map(|r| r.ok()).collect();

        // Parse tokens to operations
        let ops = parse(tokens)?;

        // Create a new evaluator and run the code
        let mut evaluator = Evaluator::new(false);
        evaluator.eval(&ops)?;

        // Return the final stack
        Ok(evaluator.get_stack().clone())
    }

    #[test]
    fn test_basic_arithmetic() {
        let result = run_forth("2 3 + 4 *").unwrap();
        assert_eq!(result, vec![20]);
    }

    #[test]
    fn test_stack_manipulation() {
        let result = run_forth("1 2 3 rot").unwrap();
        assert_eq!(result, vec![2, 3, 1]);

        let result = run_forth("5 dup").unwrap();
        assert_eq!(result, vec![5, 5]);

        let result = run_forth("1 2 swap").unwrap();
        assert_eq!(result, vec![2, 1]);
    }

    #[test]
    fn test_word_definition() {
        let result = run_forth(": DOUBLE 2 * ; 5 DOUBLE").unwrap();
        assert_eq!(result, vec![10]);
    }

    #[test]
    fn test_conditionals() {
        let result = run_forth(": TEST IF 42 ELSE 24 THEN ; 1 TEST").unwrap();
        assert_eq!(result, vec![42]);

        let result = run_forth(": TEST IF 42 ELSE 24 THEN ; 0 TEST").unwrap();
        assert_eq!(result, vec![24]);
    }

    #[test]
    fn test_loops() {
        // Sum from 0 to 9
        let result = run_forth(": SUM 0 10 0 DO I + LOOP ; SUM").unwrap();
        assert_eq!(result, vec![45]);
    }

    #[test]
    fn test_error_handling() {
        // Stack underflow
        let result = run_forth("5 + .");
        assert!(result.is_err());

        // Unknown word
        let result = run_forth("UNKNOWN-WORD");
        assert!(result.is_err());
    }
}
