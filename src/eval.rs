use crate::number_ops; // Import arithmetic and comparison ops
use crate::parser::ForthOp;
use crate::parser::ParseError;  // Add this import for ParseError
use crate::stack_ops; // Import the stack_ops module
use std::collections::HashMap; // Import HashMap
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum EvalError {
    StackUnderflow,
    DivisionByZero,
    UnknownWord(String),
    CompileOnlyWord(String),  // e.g. IF, THEN, DO, LOOP used at runtime
    LoopStackUnderflow,       // Added: Trying to use LOOP/I without DO
    ControlStructureMismatch, // Added: DO without matching LOOP at runtime (should be caught by parser ideally)
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::StackUnderflow => write!(f, "Stack underflow"),
            EvalError::DivisionByZero => write!(f, "Division by zero"),
            EvalError::UnknownWord(s) => write!(f, "Unknown word: {}", s),
            EvalError::CompileOnlyWord(s) => write!(f, "Interpreting a compile-only word: {}", s),
            EvalError::LoopStackUnderflow => write!(f, "Loop control stack underflow"),
            EvalError::ControlStructureMismatch => {
                write!(f, "Control structure mismatch during execution")
            }
        }
    }
}

// Helper function to find the matching LOOP/THEN for DO/IF
// Returns the index *after* the matching LOOP/THEN
fn find_matching_end(
    ops: &[ForthOp],
    start_idx: usize,
    open_op: ForthOp, // Pass by value
    close_op: ForthOp,
) -> Result<usize, EvalError> {
    let mut depth = 1;
    let mut current_idx = start_idx + 1;
    while current_idx < ops.len() {
        // Compare enum variants directly (since they derive PartialEq)
        if ops[current_idx] == open_op {
            depth += 1;
        } else if ops[current_idx] == close_op {
            depth -= 1;
            if depth == 0 {
                return Ok(current_idx + 1); // Return index *after* the closing op
            }
        }
        current_idx += 1;
    }
    Err(EvalError::ControlStructureMismatch) // Should be caught by parser, but safeguard
}

// Modify eval to accept the dictionary AND a loop control stack
pub fn eval(
    ops: &[ForthOp],
    stack: &mut Vec<i64>,
    dictionary: &mut HashMap<String, Vec<ForthOp>>,
    loop_control_stack: &mut Vec<(usize, i64, i64)>, // (loop_start_idx_after_do, current_index, limit)
) -> Result<(), EvalError> {
    let mut idx = 0;
    while idx < ops.len() {
        let op = &ops[idx];
        let mut next_idx = idx + 1; // Default: move to the next instruction

        // println!("DEBUG: Executing {:?} at index {}, Stack: {:?}, LoopStack: {:?}", op, idx, stack, loop_control_stack); // Debugging

        match op {
            // Simple ops that just execute and move to the next instruction
            ForthOp::Push(i) => stack.push(*i),
            ForthOp::Add => number_ops::add(stack)?,
            ForthOp::Subtract => number_ops::subtract(stack)?,
            ForthOp::Multiply => number_ops::multiply(stack)?,
            ForthOp::Divide => number_ops::divide(stack)?,
            ForthOp::Mod => number_ops::mod_op(stack)?,
            ForthOp::Eq => number_ops::eq(stack)?,
            ForthOp::Lt => number_ops::lt(stack)?,
            ForthOp::Gt => number_ops::gt(stack)?,
            ForthOp::Dup => stack_ops::dup(stack)?,
            ForthOp::Drop => stack_ops::drop_(stack)?,
            ForthOp::Swap => stack_ops::swap(stack)?,
            ForthOp::Over => stack_ops::over(stack)?,
            ForthOp::Rot => stack_ops::rot(stack)?,
            ForthOp::QDup => stack_ops::q_dup(stack)?,
            ForthOp::TwoDup => stack_ops::two_dup(stack)?,
            ForthOp::TwoDrop => stack_ops::two_drop(stack)?,
            ForthOp::TwoSwap => stack_ops::two_swap(stack)?,
            ForthOp::TwoOver => stack_ops::two_over(stack)?,
            ForthOp::MinusRot => stack_ops::minus_rot(stack)?,
            ForthOp::Print => {
                let top = stack.pop().ok_or(EvalError::StackUnderflow)?;
                println!("{} ", top);
            }
            ForthOp::PrintStack => {
                print!("Stack: <{}> ", stack.len());
                for item in stack.iter() {
                    print!("{} ", item);
                }
                println!();
            }
            ForthOp::Define(name, body) => {
                dictionary.insert(name.clone(), body.clone());
            }
            ForthOp::I => {
                let (_, current_index, _) = loop_control_stack
                    .last()
                    .ok_or(EvalError::LoopStackUnderflow)?;
                stack.push(*current_index);
            }

            // Ops involving recursive calls or jumps
            ForthOp::Word(s) => {
                let wl = s.to_lowercase();
                if ["if", "else", "then", "do", "loop", "i"].contains(&wl.as_str()) {
                    return Err(EvalError::CompileOnlyWord(s.clone()));
                }
                let upper_s = s.to_uppercase();
                if let Some(defined_ops) = dictionary.get(&upper_s) {
                    let ops_to_run = defined_ops.clone();
                    eval(&ops_to_run, stack, dictionary, loop_control_stack)?;
                } else {
                    return Err(EvalError::UnknownWord(s.clone()));
                }
                // next_idx remains idx + 1
            }
            ForthOp::IfElse(then_ops, else_ops) => {
                let flag = stack.pop().ok_or(EvalError::StackUnderflow)?;
                if flag != 0 {
                    // Forth true is non-zero
                    eval(then_ops, stack, dictionary, loop_control_stack)?;
                } else {
                    eval(else_ops, stack, dictionary, loop_control_stack)?;
                }
                // next_idx remains idx + 1
            }
            ForthOp::Do => {
                let start = stack.pop().ok_or(EvalError::StackUnderflow)?;
                let limit = stack.pop().ok_or(EvalError::StackUnderflow)?;
                if start >= limit {
                    // Loop doesn't execute, jump past matching LOOP
                    // Pass variants by value
                    next_idx = find_matching_end(ops, idx, ForthOp::Do, ForthOp::Loop)?;
                } else {
                    // Enter loop: push control info, next instruction is inside loop
                    loop_control_stack.push((idx + 1, start, limit)); // Store index *after* DO
                    next_idx = idx + 1;
                }
            }
            ForthOp::Loop => {
                // Peek at the top loop control entry
                if let Some((loop_start_idx, current_index, limit)) = loop_control_stack.last_mut()
                {
                    *current_index += 1; // Increment index

                    if *current_index >= *limit {
                        // Loop finished: pop control info, continue after LOOP
                        loop_control_stack.pop();
                        next_idx = idx + 1;
                    } else {
                        // Loop continues: jump back to instruction after DO
                        next_idx = *loop_start_idx;
                    }
                } else {
                    // LOOP without corresponding DO on control stack
                    return Err(EvalError::LoopStackUnderflow);
                }
            }
        } // end match op

        idx = next_idx; // Update instruction pointer for the next iteration
    } // end while loop
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ForthOp;
    use crate::parser::parse;
    use crate::token::Token;
    use logos::Logos;
    use std::collections::HashMap;
    
    // Add the TestError enum
    #[derive(Debug, PartialEq)]
    enum TestError {
        Eval(EvalError),
        Parse(ParseError),
    }
    
    impl From<EvalError> for TestError {
        fn from(error: EvalError) -> Self {
            TestError::Eval(error)
        }
    }
    
    impl From<ParseError> for TestError {
        fn from(error: ParseError) -> Self {
            TestError::Parse(error)
        }
    }

    // Helper to create a default dictionary and loop stack for tests
    fn default_eval_state() -> (
        Vec<i64>,
        HashMap<String, Vec<ForthOp>>,
        Vec<(usize, i64, i64)>,
    ) {
        (Vec::new(), HashMap::new(), Vec::new())
    }

    // Modify run_forth to handle loop stack and return TestError
    fn run_forth(code: &str) -> Result<Vec<i64>, TestError> {
        let tokens: Vec<Token> = Token::lexer(code).filter_map(|r| r.ok()).collect();
        // Parse tokens, converting ParseError to TestError
        let ops = parse(tokens)?; // This will use From<ParseError> for TestError
        let (mut stack, mut dict, mut loop_stack) = default_eval_state();
        // Eval, converting EvalError to TestError
        eval(&ops, &mut stack, &mut dict, &mut loop_stack)?; // This will use From<EvalError> for TestError
        Ok(stack)
    }

    // --- IMPORTANT: Update ALL existing tests below to use the new loop_stack ---

    #[test]
    fn test_eval_push_add() {
        let ops = vec![ForthOp::Push(10), ForthOp::Push(20), ForthOp::Add];
        let (mut stack, mut dict, mut loop_stack) = default_eval_state();
        let result = eval(&ops, &mut stack, &mut dict, &mut loop_stack);
        assert!(result.is_ok());
        assert_eq!(stack, vec![30]);
    }

    #[test]
    fn test_eval_arithmetic() {
        let ops = vec![
            ForthOp::Push(10),
            ForthOp::Push(5),
            ForthOp::Multiply,
            ForthOp::Push(2),
            ForthOp::Divide,
            ForthOp::Push(3),
            ForthOp::Subtract,
        ];
        let (mut stack, mut dict, mut loop_stack) = default_eval_state();
        let result = eval(&ops, &mut stack, &mut dict, &mut loop_stack);
        assert!(result.is_ok());
        assert_eq!(stack, vec![22]);
    }

    #[test]
    fn test_eval_print() {
        let ops = vec![ForthOp::Push(42), ForthOp::Print];
        let (mut stack, mut dict, mut loop_stack) = default_eval_state();
        let result = eval(&ops, &mut stack, &mut dict, &mut loop_stack);
        assert!(result.is_ok());
        assert!(stack.is_empty());
    }

    #[test]
    fn test_eval_print_stack() {
        let ops = vec![ForthOp::Push(1), ForthOp::Push(2), ForthOp::PrintStack];
        let (mut stack, mut dict, mut loop_stack) = default_eval_state();
        let result = eval(&ops, &mut stack, &mut dict, &mut loop_stack);
        assert!(result.is_ok());
        assert_eq!(stack, vec![1, 2]);
    }

    #[test]
    fn test_eval_stack_underflow() {
        let ops = vec![ForthOp::Add];
        let (mut stack, mut dict, mut loop_stack) = default_eval_state();
        let result = eval(&ops, &mut stack, &mut dict, &mut loop_stack);
        assert_eq!(result, Err(EvalError::StackUnderflow));

        let ops_sub = vec![ForthOp::Push(5), ForthOp::Subtract];
        let (mut stack_sub, mut dict_sub, mut loop_stack_sub) = default_eval_state();
        let result_sub = eval(&ops_sub, &mut stack_sub, &mut dict_sub, &mut loop_stack_sub);
        assert_eq!(result_sub, Err(EvalError::StackUnderflow));
    }

    #[test]
    fn test_eval_division_by_zero() {
        let ops = vec![ForthOp::Push(10), ForthOp::Push(0), ForthOp::Divide];
        let (mut stack, mut dict, mut loop_stack) = default_eval_state();
        let result = eval(&ops, &mut stack, &mut dict, &mut loop_stack);
        assert_eq!(result, Err(EvalError::DivisionByZero));

        let ops_mod = vec![ForthOp::Push(10), ForthOp::Push(0), ForthOp::Mod];
        let (mut stack_mod, mut dict_mod, mut loop_stack_mod) = default_eval_state();
        let result_mod = eval(&ops_mod, &mut stack_mod, &mut dict_mod, &mut loop_stack_mod);
        assert_eq!(result_mod, Err(EvalError::DivisionByZero));
    }

    #[test]
    fn test_eval_unknown_word() {
        let ops = vec![ForthOp::Word("foo".to_string())];
        let (mut stack, mut dict, mut loop_stack) = default_eval_state();
        let result = eval(&ops, &mut stack, &mut dict, &mut loop_stack);
        assert_eq!(result, Err(EvalError::UnknownWord("foo".to_string())));
    }

    #[test]
    fn test_eval_stack_ops_sequence() {
        let ops = vec![
            ForthOp::Push(1),
            ForthOp::Push(2),
            ForthOp::Push(3),
            ForthOp::Rot,
            ForthOp::Dup,
            ForthOp::Over,
            ForthOp::Swap,
            ForthOp::Drop,
        ];
        let (mut stack, mut dict, mut loop_stack) = default_eval_state();
        let result = eval(&ops, &mut stack, &mut dict, &mut loop_stack);
        assert!(result.is_ok());
        assert_eq!(stack, vec![2, 3, 1, 1]);
    }

    #[test]
    fn test_eval_2stack_ops() {
        let ops = vec![
            ForthOp::Push(1),
            ForthOp::Push(2),
            ForthOp::Push(3),
            ForthOp::Push(4),
            ForthOp::TwoSwap,
            ForthOp::TwoDup,
            ForthOp::TwoOver,
            ForthOp::TwoDrop,
        ];
        let (mut stack, mut dict, mut loop_stack) = default_eval_state();
        let result = eval(&ops, &mut stack, &mut dict, &mut loop_stack);
        assert!(result.is_ok());
        assert_eq!(stack, vec![3, 4, 1, 2, 1, 2]);
    }

    #[test]
    fn test_eval_define_word() {
        let ops = vec![ForthOp::Define(
            "DOUBLE".to_string(),
            vec![ForthOp::Push(2), ForthOp::Multiply],
        )];
        let (mut stack, mut dict, mut loop_stack) = default_eval_state();
        let result = eval(&ops, &mut stack, &mut dict, &mut loop_stack);
        assert!(result.is_ok());
        assert!(stack.is_empty());
        assert!(dict.contains_key("DOUBLE"));
        assert_eq!(dict["DOUBLE"], vec![ForthOp::Push(2), ForthOp::Multiply]);
    }

    #[test]
    fn test_eval_execute_defined_word() {
        let ops = vec![
            ForthOp::Define(
                "DOUBLE".to_string(),
                vec![ForthOp::Push(2), ForthOp::Multiply],
            ),
            ForthOp::Push(10),
            ForthOp::Word("DOUBLE".to_string()),
        ];
        let (mut stack, mut dict, mut loop_stack) = default_eval_state();
        let result = eval(&ops, &mut stack, &mut dict, &mut loop_stack);
        assert!(result.is_ok());
        assert_eq!(stack, vec![20]);
    }

    #[test]
    fn test_eval_redefine_word() {
        let ops = vec![
            ForthOp::Define("TEST".to_string(), vec![ForthOp::Push(1)]),
            ForthOp::Define("TEST".to_string(), vec![ForthOp::Push(2)]),
            ForthOp::Word("TEST".to_string()),
        ];
        let (mut stack, mut dict, mut loop_stack) = default_eval_state();
        let result = eval(&ops, &mut stack, &mut dict, &mut loop_stack);
        assert!(result.is_ok());
        assert_eq!(stack, vec![2]);
        assert_eq!(dict["TEST"], vec![ForthOp::Push(2)]);
    }

    #[test]
    fn test_eval_defined_word_uses_primitives() {
        let ops = vec![
            ForthOp::Define("SQUARE".to_string(), vec![ForthOp::Dup, ForthOp::Multiply]),
            ForthOp::Push(5),
            ForthOp::Word("SQUARE".to_string()),
        ];
        let (mut stack, mut dict, mut loop_stack) = default_eval_state();
        let result = eval(&ops, &mut stack, &mut dict, &mut loop_stack);
        assert!(result.is_ok());
        assert_eq!(stack, vec![25]);
    }

    #[test]
    fn test_eval_defined_word_calls_defined_word() {
        let ops = vec![
            ForthOp::Define(
                "DOUBLE".to_string(),
                vec![ForthOp::Push(2), ForthOp::Multiply],
            ),
            ForthOp::Define(
                "QUADRUPLE".to_string(),
                vec![
                    ForthOp::Word("DOUBLE".to_string()),
                    ForthOp::Word("DOUBLE".to_string()),
                ],
            ),
            ForthOp::Push(3),
            ForthOp::Word("QUADRUPLE".to_string()),
        ];
        let (mut stack, mut dict, mut loop_stack) = default_eval_state();
        let result = eval(&ops, &mut stack, &mut dict, &mut loop_stack);
        assert!(result.is_ok());
        assert_eq!(stack, vec![12]);
    }

    #[test]
    fn test_eval_unknown_defined_word() {
        let ops = vec![
            ForthOp::Define(
                "TEST".to_string(),
                vec![ForthOp::Word("UNKNOWN".to_string())],
            ),
            ForthOp::Word("TEST".to_string()),
        ];
        let (mut stack, mut dict, mut loop_stack) = default_eval_state();
        let result = eval(&ops, &mut stack, &mut dict, &mut loop_stack);
        assert_eq!(result, Err(EvalError::UnknownWord("UNKNOWN".to_string())));
    }

    // New tests for loops
    #[test]
    fn test_eval_simple_loop() {
        // : TEST 5 0 DO I LOOP ; TEST
        let code = ": TEST 5 0 DO I LOOP ; TEST";
        let result = run_forth(code);
        assert!(result.is_ok(), "Eval failed: {:?}", result.err());
        assert_eq!(result.unwrap(), vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_eval_loop_zero_iterations() {
        // : TEST 0 0 DO I LOOP ; TEST
        let code = ": TEST 0 0 DO I LOOP ; TEST";
        let result = run_forth(code);
        assert!(result.is_ok(), "Eval failed: {:?}", result.err());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_eval_loop_negative_range_no_iterations() {
        // : TEST 0 5 DO I LOOP ; TEST (limit < start)
        let code = ": TEST 0 5 DO I LOOP ; TEST";
        let result = run_forth(code);
        assert!(result.is_ok(), "Eval failed: {:?}", result.err());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_eval_loop_with_body() {
        // : TEST 3 0 DO I 1 + LOOP ; TEST
        let code = ": TEST 3 0 DO I 1 + LOOP ; TEST";
        let result = run_forth(code);
        assert!(result.is_ok(), "Eval failed: {:?}", result.err());
        assert_eq!(result.unwrap(), vec![1, 2, 3]); // 0+1, 1+1, 2+1
    }

    /* // Nested loop test - requires J implementation
    #[test]
    fn test_eval_nested_loop() {
        // : TEST 2 0 DO 3 0 DO I J + LOOP LOOP ; TEST
        let code = ": TEST 2 0 DO 3 0 DO I J + LOOP LOOP ; TEST";
        let result = run_forth(code);
        assert!(result.is_ok(), "Eval failed: {:?}", result.err());
        assert_eq!(result.unwrap(), vec![0, 1, 2, 1, 2, 3]);
    }
    */

    #[test]
    fn test_eval_error_loop_stack_underflow_loop() {
        // LOOP without DO - This should be a ParseError now, but test eval robustness
        let ops = vec![ForthOp::Loop];
        let (mut stack, mut dict, mut loop_stack) = default_eval_state();
        let result = eval(&ops, &mut stack, &mut dict, &mut loop_stack);
        assert_eq!(result, Err(EvalError::LoopStackUnderflow));
    }

    #[test]
    fn test_eval_error_loop_stack_underflow_i() {
        // I without DO
        let ops = vec![ForthOp::I];
        let (mut stack, mut dict, mut loop_stack) = default_eval_state();
        let result = eval(&ops, &mut stack, &mut dict, &mut loop_stack);
        assert_eq!(result, Err(EvalError::LoopStackUnderflow));
    }

    #[test]
    fn test_eval_error_compile_only_word_do() {
        // DO outside definition - Parser creates ForthOp::Word("do")
        let ops = vec![
            ForthOp::Push(5),
            ForthOp::Push(0),
            ForthOp::Word("do".to_string()),
        ];
        let (mut stack, mut dict, mut loop_stack) = default_eval_state();
        let result = eval(&ops, &mut stack, &mut dict, &mut loop_stack);
        assert_eq!(result, Err(EvalError::CompileOnlyWord("do".to_string())));
    }

    // --- Functional tests using run_forth ---

    #[test]
    fn test_run_arithmetic_sequence() {
        assert_eq!(run_forth("10 5 + 2 *").unwrap(), vec![30]);
        assert_eq!(run_forth("10 5 - 3 / 2 mod").unwrap(), vec![1]);
    }

    #[test]
    fn test_run_stack_manipulation() {
        assert_eq!(run_forth("1 2 3 rot").unwrap(), vec![2, 3, 1]);
        assert_eq!(run_forth("4 dup drop").unwrap(), vec![4]);
    }

    #[test]
    fn test_run_definitions() {
        assert_eq!(run_forth(": double 2 * ; 6 double").unwrap(), vec![12]);
        assert_eq!(run_forth(": square dup * ; 3 square").unwrap(), vec![9]);
    }

    #[test]
    fn test_run_loop_functional() {
        // Calculate sum of 0 to 4: 0 1 2 3 4 + + + + -> 10
        let code = ": SUM5 0 5 0 DO I + LOOP ; SUM5";
        let result = run_forth(code);
        assert!(result.is_ok(), "Eval failed: {:?}", result.err());
        assert_eq!(result.unwrap(), vec![10]);
    }

    #[test]
    fn test_run_unknown_word() {
        let result = run_forth("foo");
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            TestError::Eval(EvalError::UnknownWord("foo".to_string()))
        );
    }

    #[test]
    fn test_run_compile_only_word() {
        // Test DO outside definition
        let result_do = run_forth("1 2 do");
        assert!(result_do.is_err());
        match result_do.err().unwrap() {
            TestError::Parse(ParseError::ControlWordOutsideDefinition(s)) => assert_eq!(s, "do"),
            other => panic!(
                "Expected ParseError::ControlWordOutsideDefinition for 'do', got {:?}",
                other
            ),
        }

        // Test LOOP outside definition
        let result_loop = run_forth("loop");
        assert!(result_loop.is_err());
        match result_loop.err().unwrap() {
            TestError::Parse(ParseError::ControlWordOutsideDefinition(s)) => assert_eq!(s, "loop"),
            other => panic!(
                "Expected ParseError::ControlWordOutsideDefinition for 'loop', got {:?}",
                other
            ),
        }

        // Test IF outside definition
        let result_if = run_forth("1 if 2 then");
        assert!(result_if.is_err());
        match result_if.err().unwrap() {
            TestError::Parse(ParseError::ControlWordOutsideDefinition(s)) => assert_eq!(s, "if"),
            other => panic!(
                "Expected ParseError::ControlWordOutsideDefinition for 'if', got {:?}",
                other
            ),
        }

        // Test THEN outside definition
        let result_then = run_forth("1 2 then");
        assert!(result_then.is_err());
        match result_then.err().unwrap() {
            TestError::Parse(ParseError::ControlWordOutsideDefinition(s)) => assert_eq!(s, "then"),
            other => panic!(
                "Expected ParseError::ControlWordOutsideDefinition for 'then', got {:?}",
                other
            ),
        }

        // Test ELSE outside definition
        let result_else = run_forth("1 2 else");
        assert!(result_else.is_err());
        match result_else.err().unwrap() {
            TestError::Parse(ParseError::ControlWordOutsideDefinition(s)) => assert_eq!(s, "else"),
            other => panic!(
                "Expected ParseError::ControlWordOutsideDefinition for 'else', got {:?}",
                other
            ),
        }
    }

    // ... other functional tests (IF/ELSE/THEN, comparisons) should also be updated ...
    // ... if they use run_forth or call eval directly ...
}
