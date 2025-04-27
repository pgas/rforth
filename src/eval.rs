use crate::number_ops; // Import arithmetic and comparison ops
use crate::parser::ForthOp;
use crate::stack_ops; // Import the stack_ops module
use std::collections::HashMap; // Import HashMap
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum EvalError {
    StackUnderflow,
    DivisionByZero,
    UnknownWord(String),
    CompileOnlyWord(String),
    // Add other potential runtime errors here
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::StackUnderflow => write!(f, "Stack underflow"),
            EvalError::DivisionByZero => write!(f, "Division by zero"),
            EvalError::UnknownWord(s) => write!(f, "Unknown word: {}", s),
            EvalError::CompileOnlyWord(s) => write!(f, "Interpreting a compile-only word: {}", s),
        }
    }
}

// Modify eval to accept the dictionary
pub fn eval(
    ops: &[ForthOp],
    stack: &mut Vec<i64>,
    dictionary: &mut HashMap<String, Vec<ForthOp>>, // Added dictionary
) -> Result<(), EvalError> {
    let mut idx = 0;
    while idx < ops.len() {
        match &ops[idx] {
            ForthOp::Push(i) => stack.push(*i),
            // Arithmetic
            ForthOp::Add => number_ops::add(stack)?,
            ForthOp::Subtract => number_ops::subtract(stack)?,
            ForthOp::Multiply => number_ops::multiply(stack)?,
            ForthOp::Divide => number_ops::divide(stack)?,
            // Comparison ops
            ForthOp::Eq => number_ops::eq(stack)?,
            ForthOp::Lt => number_ops::lt(stack)?,
            ForthOp::Gt => number_ops::gt(stack)?,
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
            // Pre-compiled conditional (inside definitions)
            ForthOp::IfElse(then_ops, else_ops) => {
                let flag = stack.pop().ok_or(EvalError::StackUnderflow)?;
                if flag != 0 {
                    eval(then_ops, stack, dictionary)?;
                } else {
                    eval(else_ops, stack, dictionary)?;
                }
            }
            // Word tokens (including if/else/then) are only executed if defined in dictionary
            ForthOp::Word(s) => {
                let wl = s.to_lowercase();
                // compile-only words not allowed at runtime
                if wl == "if" || wl == "else" || wl == "then" {
                    return Err(EvalError::CompileOnlyWord(s.clone()));
                }
                let upper_s = s.to_uppercase(); // Match definition convention
                if let Some(defined_ops) = dictionary.get(&upper_s) {
                    let ops_to_run = defined_ops.clone();
                    eval(&ops_to_run, stack, dictionary)?;
                } else {
                    return Err(EvalError::UnknownWord(s.clone()));
                }
            }
        }
        idx += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ForthOp;
    use crate::parser::parse;
    use crate::token::Token;
    use logos::Logos; // Bring Logos trait into scope for Token::lexer
    use std::collections::HashMap; // Import HashMap for tests

    // Helper to create a default dictionary for tests
    fn default_dict() -> HashMap<String, Vec<ForthOp>> {
        HashMap::new()
    }

    // Helper to run a code string through lex, parse, and eval and return the final stack
    fn run_forth(code: &str) -> Vec<i64> {
        let tokens: Vec<Token> = Token::lexer(code).filter_map(|r| r.ok()).collect();
        let ops = parse(tokens).expect("parse failed");
        let mut stack = Vec::new();
        let mut dict = default_dict();
        eval(&ops, &mut stack, &mut dict).expect("eval failed");
        stack
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
        let ops = vec![ForthOp::Define(
            "DOUBLE".to_string(),
            vec![ForthOp::Push(2), ForthOp::Multiply],
        )];
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
            ForthOp::Define(
                "DOUBLE".to_string(),
                vec![ForthOp::Push(2), ForthOp::Multiply],
            ),
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
            ForthOp::Define(
                "DOUBLE".to_string(),
                vec![ForthOp::Push(2), ForthOp::Multiply],
            ),
            // : QUADRUPLE DOUBLE DOUBLE ;
            ForthOp::Define(
                "QUADRUPLE".to_string(),
                vec![
                    ForthOp::Word("DOUBLE".to_string()),
                    ForthOp::Word("DOUBLE".to_string()),
                ],
            ),
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
            ForthOp::Define(
                "TEST".to_string(),
                vec![ForthOp::Word("UNKNOWN".to_string())],
            ),
            // TEST (Execute TEST)
            ForthOp::Word("TEST".to_string()),
        ];
        let mut stack = Vec::new();
        let mut dict = default_dict();
        let result = eval(&ops, &mut stack, &mut dict);
        // The error occurs when TEST tries to execute UNKNOWN
        assert_eq!(result, Err(EvalError::UnknownWord("UNKNOWN".to_string())));
    }

    // Functional tests
    #[test]
    fn test_run_arithmetic_sequence() {
        assert_eq!(run_forth("10 5 + 2 *"), vec![30]);
        assert_eq!(run_forth("10 5 - 3 /"), vec![((10 - 5) / 3)]);
    }

    #[test]
    fn test_run_stack_manipulation() {
        // 1 2 3 rot -> 2 3 1
        assert_eq!(run_forth("1 2 3 rot"), vec![2, 3, 1]);
        // dup and drop
        assert_eq!(run_forth("4 dup drop"), vec![4]);
    }

    #[test]
    fn test_run_definitions() {
        assert_eq!(run_forth(": double 2 * ; 6 double"), vec![12]);
        assert_eq!(run_forth(": square dup * ; 3 square"), vec![9]);
        // nested definitions
        let result = std::panic::catch_unwind(|| run_forth(": bad : nested ;"));
        assert!(result.is_err());
    }

    #[test]
    fn test_run_unknown_word() {
        // Unknown word should cause panic in run_forth
        let result = std::panic::catch_unwind(|| run_forth("foo"));
        assert!(result.is_err());
    }

    // Functional tests for conditional execution
    #[should_panic]
    #[test]
    fn test_run_if_then_true() {
        run_forth("1 if 2 then");
    }

    #[should_panic]
    #[test]
    fn test_run_if_then_false() {
        run_forth("0 if 2 then");
    }

    #[should_panic]
    #[test]
    fn test_run_if_else_then_true() {
        run_forth("1 if 2 else 3 then");
    }

    #[should_panic]
    #[test]
    fn test_run_if_else_then_false() {
        run_forth("0 if 2 else 3 then");
    }

    // Functional tests for comparison operators
    #[test]
    fn test_run_eq_true() {
        assert_eq!(run_forth("1 1 ="), vec![-1]);
    }

    #[test]
    fn test_run_eq_false() {
        assert_eq!(run_forth("1 2 ="), vec![0]);
    }

    #[test]
    fn test_run_lt_true() {
        assert_eq!(run_forth("1 2 <"), vec![-1]);
    }

    #[test]
    fn test_run_lt_false() {
        assert_eq!(run_forth("2 1 <"), vec![0]);
    }

    #[test]
    fn test_run_gt_true() {
        assert_eq!(run_forth("2 1 >"), vec![-1]);
    }

    #[test]
    fn test_run_gt_false() {
        assert_eq!(run_forth("1 2 >"), vec![0]);
    }
}
