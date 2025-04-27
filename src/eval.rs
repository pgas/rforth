use crate::parser::ForthOp;
use crate::stack_ops; // Import the stack_ops module
use std::collections::HashMap; // Import HashMap
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum EvalError {
    StackUnderflow,
    DivisionByZero,
    UnknownWord(String),
    // Add other potential runtime errors here
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::StackUnderflow => write!(f, "Stack underflow"),
            EvalError::DivisionByZero => write!(f, "Division by zero"),
            EvalError::UnknownWord(s) => write!(f, "Unknown word: {}", s),
        }
    }
}

// Modify eval to accept the dictionary
pub fn eval(
    ops: &[ForthOp],
    stack: &mut Vec<i64>,
    dictionary: &mut HashMap<String, Vec<ForthOp>>, // Added dictionary
) -> Result<(), EvalError> {
    for op in ops {
        match op {
            ForthOp::Push(i) => stack.push(*i),
            // Arithmetic
            ForthOp::Add => {
                let b = stack.pop().ok_or(EvalError::StackUnderflow)?;
                let a = stack.pop().ok_or(EvalError::StackUnderflow)?;
                stack.push(a + b);
            }
            ForthOp::Subtract => {
                let b = stack.pop().ok_or(EvalError::StackUnderflow)?;
                let a = stack.pop().ok_or(EvalError::StackUnderflow)?;
                stack.push(a - b);
            }
            ForthOp::Multiply => {
                let b = stack.pop().ok_or(EvalError::StackUnderflow)?;
                let a = stack.pop().ok_or(EvalError::StackUnderflow)?;
                stack.push(a * b);
            }
            ForthOp::Divide => {
                let b = stack.pop().ok_or(EvalError::StackUnderflow)?;
                let a = stack.pop().ok_or(EvalError::StackUnderflow)?;
                if b == 0 {
                    return Err(EvalError::DivisionByZero);
                }
                stack.push(a / b); // Integer division
            }
            // Stack Operations
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
            // Output
            ForthOp::Print => {
                let top = stack.pop().ok_or(EvalError::StackUnderflow)?;
                println!("{} ", top); // Print with a space, like traditional Forth
            }
            ForthOp::PrintStack => {
                print!("Stack: <{}> ", stack.len());
                for item in stack.iter() {
                    print!("{} ", item);
                }
                println!(); // Newline after printing stack
            }
            // Handle Definition
            ForthOp::Define(name, body) => {
                // Insert or update the word definition in the dictionary
                dictionary.insert(name.clone(), body.clone());
            }
            // Handle Word Lookup and Execution
            ForthOp::Word(s) => {
                let upper_s = s.to_uppercase(); // Match definition convention
                if let Some(defined_ops) = dictionary.get(&upper_s) {
                    // Execute the sequence of operations defined for this word
                    // We need to clone the ops because eval takes a slice
                    // and we might modify the dictionary while executing
                    let ops_to_run = defined_ops.clone();
                    eval(&ops_to_run, stack, dictionary)?; // Recursive call
                } else {
                    // If not in the dictionary, it's an unknown word
                    return Err(EvalError::UnknownWord(s.clone()));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ForthOp;
    use std::collections::HashMap; // Import HashMap for tests

    // Helper to create a default dictionary for tests
    fn default_dict() -> HashMap<String, Vec<ForthOp>> {
        HashMap::new()
    }

    // Update existing tests to pass the dictionary
    #[test]
    fn test_eval_push_add() {
        let ops = vec![ForthOp::Push(10), ForthOp::Push(20), ForthOp::Add];
        let mut stack = Vec::new();
        let mut dict = default_dict();
        let result = eval(&ops, &mut stack, &mut dict); // Pass dict
        assert!(result.is_ok());
        assert_eq!(stack, vec![30]);
    }

    #[test]
    fn test_eval_arithmetic() {
        // 10 5 * 2 / 3 -
        let ops = vec![
            ForthOp::Push(10),
            ForthOp::Push(5),
            ForthOp::Multiply,
            ForthOp::Push(2),
            ForthOp::Divide,
            ForthOp::Push(3),
            ForthOp::Subtract,
        ];
        let mut stack = Vec::new();
        let mut dict = default_dict();
        let result = eval(&ops, &mut stack, &mut dict); // Pass dict
        assert!(result.is_ok());
        assert_eq!(stack, vec![22]); // (10 * 5) / 2 - 3 = 50 / 2 - 3 = 25 - 3 = 22
    }

    #[test]
    fn test_eval_print() {
        // We can't easily test stdout here, but we can check stack consumption
        let ops = vec![ForthOp::Push(42), ForthOp::Print];
        let mut stack = Vec::new();
        let mut dict = default_dict();
        let result = eval(&ops, &mut stack, &mut dict); // Pass dict
        assert!(result.is_ok());
        assert!(stack.is_empty());
    }

    #[test]
    fn test_eval_print_stack() {
        // Similar to print, check stack is unchanged
        let ops = vec![ForthOp::Push(1), ForthOp::Push(2), ForthOp::PrintStack];
        let mut stack = Vec::new();
        let mut dict = default_dict();
        let result = eval(&ops, &mut stack, &mut dict); // Pass dict
        assert!(result.is_ok());
        assert_eq!(stack, vec![1, 2]);
    }

    #[test]
    fn test_eval_stack_underflow() {
        let ops = vec![ForthOp::Add];
        let mut stack = Vec::new();
        let mut dict = default_dict();
        let result = eval(&ops, &mut stack, &mut dict); // Pass dict
        assert_eq!(result, Err(EvalError::StackUnderflow));

        let ops = vec![ForthOp::Push(5), ForthOp::Subtract];
        let mut stack = Vec::new();
        let mut dict = default_dict(); // Need a new dict for the second part
        let result = eval(&ops, &mut stack, &mut dict); // Pass dict
        assert_eq!(result, Err(EvalError::StackUnderflow));
    }

    #[test]
    fn test_eval_division_by_zero() {
        let ops = vec![ForthOp::Push(10), ForthOp::Push(0), ForthOp::Divide];
        let mut stack = Vec::new();
        let mut dict = default_dict();
        let result = eval(&ops, &mut stack, &mut dict); // Pass dict
        assert_eq!(result, Err(EvalError::DivisionByZero));
    }

    #[test]
    fn test_eval_unknown_word() {
        let ops = vec![ForthOp::Word("foo".to_string())];
        let mut stack = Vec::new();
        let mut dict = default_dict();
        let result = eval(&ops, &mut stack, &mut dict); // Pass dict
        assert_eq!(result, Err(EvalError::UnknownWord("foo".to_string())));
    }

    #[test]
    fn test_eval_stack_ops_sequence() {
        // 1 2 3 rot -> 2 3 1
        // dup -> 2 3 1 1
        // over -> 2 3 1 1 1 (Corrected trace)
        // swap -> 2 3 1 1 1 (Corrected trace)
        // drop -> 2 3 1 1   (Corrected trace)
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
        let mut stack = Vec::new();
        let mut dict = default_dict();
        let result = eval(&ops, &mut stack, &mut dict); // Pass dict
        assert!(result.is_ok());
        // Correct the expected stack state
        assert_eq!(stack, vec![2, 3, 1, 1]);
    }

    #[test]
    fn test_eval_2stack_ops() {
        // 1 2 3 4 2swap -> 3 4 1 2
        // 2dup -> 3 4 1 2 1 2
        // 2over -> 3 4 1 2 1 2 3 4
        // 2drop -> 3 4 1 2 1 2
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
        let mut stack = Vec::new();
        let mut dict = default_dict();
        let result = eval(&ops, &mut stack, &mut dict); // Pass dict
        assert!(result.is_ok());
        assert_eq!(stack, vec![3, 4, 1, 2, 1, 2]);
    }

    // New tests for definitions
    #[test]
    fn test_eval_define_word() {
        let ops = vec![
            ForthOp::Define("DOUBLE".to_string(), vec![ForthOp::Push(2), ForthOp::Multiply])
        ];
        let mut stack = Vec::new();
        let mut dict = default_dict();
        let result = eval(&ops, &mut stack, &mut dict);
        assert!(result.is_ok());
        assert!(stack.is_empty()); // Definition shouldn't change stack
        assert!(dict.contains_key("DOUBLE"));
        assert_eq!(dict["DOUBLE"], vec![ForthOp::Push(2), ForthOp::Multiply]);
    }

    #[test]
    fn test_eval_execute_defined_word() {
        let ops = vec![
            // : DOUBLE 2 * ;
            ForthOp::Define("DOUBLE".to_string(), vec![ForthOp::Push(2), ForthOp::Multiply]),
            // 10 DOUBLE .
            ForthOp::Push(10),
            ForthOp::Word("DOUBLE".to_string()), // Execute the defined word
            // ForthOp::Print, // We can't test print easily, check stack instead
        ];
        let mut stack = Vec::new();
        let mut dict = default_dict();
        let result = eval(&ops, &mut stack, &mut dict);
        assert!(result.is_ok());
        assert_eq!(stack, vec![20]); // 10 * 2 = 20
    }

    #[test]
    fn test_eval_redefine_word() {
        let ops = vec![
            // : TEST 1 ;
            ForthOp::Define("TEST".to_string(), vec![ForthOp::Push(1)]),
             // : TEST 2 ;
            ForthOp::Define("TEST".to_string(), vec![ForthOp::Push(2)]),
            // TEST
            ForthOp::Word("TEST".to_string()),
        ];
        let mut stack = Vec::new();
        let mut dict = default_dict();
        let result = eval(&ops, &mut stack, &mut dict);
        assert!(result.is_ok());
        assert_eq!(stack, vec![2]); // Should execute the latest definition
        assert_eq!(dict["TEST"], vec![ForthOp::Push(2)]);
    }

    #[test]
    fn test_eval_defined_word_uses_primitives() {
         let ops = vec![
            // : SQUARE DUP * ;
            ForthOp::Define("SQUARE".to_string(), vec![ForthOp::Dup, ForthOp::Multiply]),
            // 5 SQUARE
            ForthOp::Push(5),
            ForthOp::Word("SQUARE".to_string()),
        ];
        let mut stack = Vec::new();
        let mut dict = default_dict();
        let result = eval(&ops, &mut stack, &mut dict);
        assert!(result.is_ok());
        assert_eq!(stack, vec![25]); // 5 * 5 = 25
    }

     #[test]
    fn test_eval_defined_word_calls_defined_word() {
         let ops = vec![
            // : DOUBLE 2 * ;
            ForthOp::Define("DOUBLE".to_string(), vec![ForthOp::Push(2), ForthOp::Multiply]),
            // : QUADRUPLE DOUBLE DOUBLE ;
            ForthOp::Define("QUADRUPLE".to_string(), vec![ForthOp::Word("DOUBLE".to_string()), ForthOp::Word("DOUBLE".to_string())]),
            // 3 QUADRUPLE
            ForthOp::Push(3),
            ForthOp::Word("QUADRUPLE".to_string()),
        ];
        let mut stack = Vec::new();
        let mut dict = default_dict();
        let result = eval(&ops, &mut stack, &mut dict);
        assert!(result.is_ok());
        assert_eq!(stack, vec![12]); // 3 * 2 * 2 = 12
    }

     #[test]
    fn test_eval_unknown_defined_word() {
         let ops = vec![
            // : TEST UNKNOWN ; (Define TEST using a word that doesn't exist yet)
            ForthOp::Define("TEST".to_string(), vec![ForthOp::Word("UNKNOWN".to_string())]),
            // TEST (Execute TEST)
            ForthOp::Word("TEST".to_string()),
        ];
        let mut stack = Vec::new();
        let mut dict = default_dict();
        let result = eval(&ops, &mut stack, &mut dict);
        // The error occurs when TEST tries to execute UNKNOWN
        assert_eq!(result, Err(EvalError::UnknownWord("UNKNOWN".to_string())));
    }
}
