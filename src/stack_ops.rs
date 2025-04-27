use crate::eval::EvalError; // Import EvalError

// Helper macro for checking stack depth
macro_rules! check_depth {
    ($stack:expr, $depth:expr) => {
        if $stack.len() < $depth {
            return Err(EvalError::StackUnderflow);
        }
    };
}

// ( n -- n n )
pub fn dup(stack: &mut Vec<i64>) -> Result<(), EvalError> {
    check_depth!(stack, 1);
    stack.push(stack.last().unwrap().clone());
    Ok(())
}

// ( n -- )
pub fn drop_(stack: &mut Vec<i64>) -> Result<(), EvalError> {
    check_depth!(stack, 1);
    stack.pop();
    Ok(())
}

// ( n1 n2 -- n2 n1 )
pub fn swap(stack: &mut Vec<i64>) -> Result<(), EvalError> {
    check_depth!(stack, 2);
    let n2 = stack.pop().unwrap();
    let n1 = stack.pop().unwrap();
    stack.push(n2);
    stack.push(n1);
    Ok(())
}

// ( n1 n2 -- n1 n2 n1 )
pub fn over(stack: &mut Vec<i64>) -> Result<(), EvalError> {
    check_depth!(stack, 2);
    let n1 = stack[stack.len() - 2];
    stack.push(n1);
    Ok(())
}

// ( n1 n2 n3 -- n2 n3 n1 )
pub fn rot(stack: &mut Vec<i64>) -> Result<(), EvalError> {
    check_depth!(stack, 3);
    let n3 = stack.pop().unwrap();
    let n2 = stack.pop().unwrap();
    let n1 = stack.pop().unwrap();
    stack.push(n2);
    stack.push(n3);
    stack.push(n1);
    Ok(())
}

// ( n -- n n ) or ( 0 -- 0 )
pub fn q_dup(stack: &mut Vec<i64>) -> Result<(), EvalError> {
    check_depth!(stack, 1);
    if *stack.last().unwrap() != 0 {
        stack.push(stack.last().unwrap().clone());
    }
    Ok(())
}

// ( n1 n2 -- n1 n2 n1 n2 )
pub fn two_dup(stack: &mut Vec<i64>) -> Result<(), EvalError> {
    check_depth!(stack, 2);
    let n2 = stack.last().unwrap().clone();
    let n1 = stack[stack.len() - 2];
    stack.push(n1);
    stack.push(n2);
    Ok(())
}

// ( n1 n2 -- )
pub fn two_drop(stack: &mut Vec<i64>) -> Result<(), EvalError> {
    check_depth!(stack, 2);
    stack.pop();
    stack.pop();
    Ok(())
}

// ( n1 n2 n3 n4 -- n3 n4 n1 n2 )
pub fn two_swap(stack: &mut Vec<i64>) -> Result<(), EvalError> {
    check_depth!(stack, 4);
    let n4 = stack.pop().unwrap();
    let n3 = stack.pop().unwrap();
    let n2 = stack.pop().unwrap();
    let n1 = stack.pop().unwrap();
    stack.push(n3);
    stack.push(n4);
    stack.push(n1);
    stack.push(n2);
    Ok(())
}

// ( n1 n2 n3 n4 -- n1 n2 n3 n4 n1 n2 )
pub fn two_over(stack: &mut Vec<i64>) -> Result<(), EvalError> {
    check_depth!(stack, 4);
    let n2 = stack[stack.len() - 3];
    let n1 = stack[stack.len() - 4];
    stack.push(n1);
    stack.push(n2);
    Ok(())
}

// ( n1 n2 n3 -- n3 n1 n2 )
pub fn minus_rot(stack: &mut Vec<i64>) -> Result<(), EvalError> {
    check_depth!(stack, 3);
    let n3 = stack.pop().unwrap();
    let n2 = stack.pop().unwrap();
    let n1 = stack.pop().unwrap();
    stack.push(n3);
    stack.push(n1);
    stack.push(n2);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dup() {
        let mut stack = vec![10];
        assert!(dup(&mut stack).is_ok());
        assert_eq!(stack, vec![10, 10]);
        assert_eq!(dup(&mut vec![]), Err(EvalError::StackUnderflow));
    }

    #[test]
    fn test_drop() {
        let mut stack = vec![10, 20];
        assert!(drop_(&mut stack).is_ok());
        assert_eq!(stack, vec![10]);
        assert!(drop_(&mut stack).is_ok());
        assert_eq!(stack, vec![]);
        assert_eq!(drop_(&mut stack), Err(EvalError::StackUnderflow));
    }

    #[test]
    fn test_swap() {
        let mut stack = vec![10, 20];
        assert!(swap(&mut stack).is_ok());
        assert_eq!(stack, vec![20, 10]);
        assert_eq!(swap(&mut vec![1]), Err(EvalError::StackUnderflow));
    }

    #[test]
    fn test_over() {
        let mut stack = vec![10, 20];
        assert!(over(&mut stack).is_ok());
        assert_eq!(stack, vec![10, 20, 10]);
        assert_eq!(over(&mut vec![1]), Err(EvalError::StackUnderflow));
    }

    #[test]
    fn test_rot() {
        let mut stack = vec![10, 20, 30];
        assert!(rot(&mut stack).is_ok());
        assert_eq!(stack, vec![20, 30, 10]);
        assert_eq!(rot(&mut vec![1, 2]), Err(EvalError::StackUnderflow));
    }

    #[test]
    fn test_q_dup() {
        let mut stack = vec![10];
        assert!(q_dup(&mut stack).is_ok());
        assert_eq!(stack, vec![10, 10]);
        let mut stack = vec![0];
        assert!(q_dup(&mut stack).is_ok());
        assert_eq!(stack, vec![0]);
        assert_eq!(q_dup(&mut vec![]), Err(EvalError::StackUnderflow));
    }

    #[test]
    fn test_two_dup() {
        let mut stack = vec![10, 20];
        assert!(two_dup(&mut stack).is_ok());
        assert_eq!(stack, vec![10, 20, 10, 20]);
        assert_eq!(two_dup(&mut vec![1]), Err(EvalError::StackUnderflow));
    }

    #[test]
    fn test_two_drop() {
        let mut stack = vec![10, 20, 30];
        assert!(two_drop(&mut stack).is_ok());
        assert_eq!(stack, vec![10]);
        assert_eq!(two_drop(&mut vec![1]), Err(EvalError::StackUnderflow));
    }

    #[test]
    fn test_two_swap() {
        let mut stack = vec![10, 20, 30, 40];
        assert!(two_swap(&mut stack).is_ok());
        assert_eq!(stack, vec![30, 40, 10, 20]);
        assert_eq!(two_swap(&mut vec![1, 2, 3]), Err(EvalError::StackUnderflow));
    }

    #[test]
    fn test_two_over() {
        let mut stack = vec![10, 20, 30, 40];
        assert!(two_over(&mut stack).is_ok());
        assert_eq!(stack, vec![10, 20, 30, 40, 10, 20]);
        assert_eq!(two_over(&mut vec![1, 2, 3]), Err(EvalError::StackUnderflow));
    }

    #[test]
    fn test_minus_rot() {
        let mut stack = vec![10, 20, 30];
        assert!(minus_rot(&mut stack).is_ok());
        assert_eq!(stack, vec![30, 10, 20]);
        assert_eq!(minus_rot(&mut vec![1, 2]), Err(EvalError::StackUnderflow));
    }
}
