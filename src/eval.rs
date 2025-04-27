use crate::parser::ForthOp;
use crate::stack_ops; // Import the stack_ops module
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

pub fn eval(ops: &[ForthOp], stack: &mut Vec<i64>) -> Result<(), EvalError> {
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
            ForthOp::Word(s) => {
                // For now, any unrecognized word is an error.
                // Later, this will involve dictionary lookups.
                return Err(EvalError::UnknownWord(s.clone()));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ForthOp;

    #[test]
    fn test_eval_push_add() {
        let ops = vec![ForthOp::Push(10), ForthOp::Push(20), ForthOp::Add];
        let mut stack = Vec::new();
        let result = eval(&ops, &mut stack);
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
        let result = eval(&ops, &mut stack);
        assert!(result.is_ok());
        assert_eq!(stack, vec![22]); // (10 * 5) / 2 - 3 = 50 / 2 - 3 = 25 - 3 = 22
    }

    #[test]
    fn test_eval_print() {
        // We can't easily test stdout here, but we can check stack consumption
        let ops = vec![ForthOp::Push(42), ForthOp::Print];
        let mut stack = Vec::new();
        let result = eval(&ops, &mut stack);
        assert!(result.is_ok());
        assert!(stack.is_empty());
    }

    #[test]
    fn test_eval_print_stack() {
        // Similar to print, check stack is unchanged
        let ops = vec![ForthOp::Push(1), ForthOp::Push(2), ForthOp::PrintStack];
        let mut stack = Vec::new();
        let result = eval(&ops, &mut stack);
        assert!(result.is_ok());
        assert_eq!(stack, vec![1, 2]);
    }

    #[test]
    fn test_eval_stack_underflow() {
        let ops = vec![ForthOp::Add];
        let mut stack = Vec::new();
        let result = eval(&ops, &mut stack);
        assert_eq!(result, Err(EvalError::StackUnderflow));

        let ops = vec![ForthOp::Push(5), ForthOp::Subtract];
        let mut stack = Vec::new();
        let result = eval(&ops, &mut stack);
        assert_eq!(result, Err(EvalError::StackUnderflow));
    }

    #[test]
    fn test_eval_division_by_zero() {
        let ops = vec![ForthOp::Push(10), ForthOp::Push(0), ForthOp::Divide];
        let mut stack = Vec::new();
        let result = eval(&ops, &mut stack);
        assert_eq!(result, Err(EvalError::DivisionByZero));
    }

    #[test]
    fn test_eval_unknown_word() {
        let ops = vec![ForthOp::Word("foo".to_string())];
        let mut stack = Vec::new();
        let result = eval(&ops, &mut stack);
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
        let result = eval(&ops, &mut stack);
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
        let result = eval(&ops, &mut stack);
        assert!(result.is_ok());
        assert_eq!(stack, vec![3, 4, 1, 2, 1, 2]);
    }
}
