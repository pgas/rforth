use crate::eval::EvalError;

// Arithmetic operations
pub fn add(stack: &mut Vec<i64>) -> Result<(), EvalError> {
    let b = stack.pop().ok_or(EvalError::StackUnderflow)?;
    let a = stack.pop().ok_or(EvalError::StackUnderflow)?;
    stack.push(a + b);
    Ok(())
}

pub fn subtract(stack: &mut Vec<i64>) -> Result<(), EvalError> {
    let b = stack.pop().ok_or(EvalError::StackUnderflow)?;
    let a = stack.pop().ok_or(EvalError::StackUnderflow)?;
    stack.push(a - b);
    Ok(())
}

pub fn multiply(stack: &mut Vec<i64>) -> Result<(), EvalError> {
    let b = stack.pop().ok_or(EvalError::StackUnderflow)?;
    let a = stack.pop().ok_or(EvalError::StackUnderflow)?;
    stack.push(a * b);
    Ok(())
}

pub fn divide(stack: &mut Vec<i64>) -> Result<(), EvalError> {
    let b = stack.pop().ok_or(EvalError::StackUnderflow)?;
    let a = stack.pop().ok_or(EvalError::StackUnderflow)?;
    if b == 0 {
        return Err(EvalError::DivisionByZero);
    }
    stack.push(a / b);
    Ok(())
}

// Comparison operations: push 1 for true, 0 for false
pub fn eq(stack: &mut Vec<i64>) -> Result<(), EvalError> {
    let b = stack.pop().ok_or(EvalError::StackUnderflow)?;
    let a = stack.pop().ok_or(EvalError::StackUnderflow)?;
    // Traditional Forth uses -1 (all bits set) for true, 0 for false
    stack.push(if a == b { -1 } else { 0 });
    Ok(())
}

pub fn lt(stack: &mut Vec<i64>) -> Result<(), EvalError> {
    let b = stack.pop().ok_or(EvalError::StackUnderflow)?;
    let a = stack.pop().ok_or(EvalError::StackUnderflow)?;
    // True: -1, False: 0
    stack.push(if a < b { -1 } else { 0 });
    Ok(())
}

pub fn gt(stack: &mut Vec<i64>) -> Result<(), EvalError> {
    let b = stack.pop().ok_or(EvalError::StackUnderflow)?;
    let a = stack.pop().ok_or(EvalError::StackUnderflow)?;
    // True: -1, False: 0
    stack.push(if a > b { -1 } else { 0 });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() {
        let mut s = vec![2, 3];
        assert!(add(&mut s).is_ok());
        assert_eq!(s, vec![5]);
    }

    #[test]
    fn test_subtract() {
        let mut s = vec![5, 2];
        assert!(subtract(&mut s).is_ok());
        assert_eq!(s, vec![3]);
    }

    #[test]
    fn test_multiply() {
        let mut s = vec![4, 3];
        assert!(multiply(&mut s).is_ok());
        assert_eq!(s, vec![12]);
    }

    #[test]
    fn test_divide() {
        let mut s = vec![10, 2];
        assert!(divide(&mut s).is_ok());
        assert_eq!(s, vec![5]);
        let mut z = vec![1, 0];
        assert_eq!(divide(&mut z), Err(EvalError::DivisionByZero));
    }

    #[test]
    fn test_eq() {
        let mut s = vec![2, 2];
        assert!(eq(&mut s).is_ok());
        assert_eq!(s, vec![-1]);
        let mut t = vec![2, 3];
        assert!(eq(&mut t).is_ok());
        assert_eq!(t, vec![0]);
    }

    #[test]
    fn test_lt_gt() {
        let mut a = vec![1, 2];
        assert!(lt(&mut a).is_ok());
        assert_eq!(a, vec![-1]);
        let mut b = vec![2, 1];
        assert!(lt(&mut b).is_ok());
        assert_eq!(b, vec![0]);
        let mut c = vec![3, 1];
        assert!(gt(&mut c).is_ok());
        assert_eq!(c, vec![-1]);
        let mut d = vec![1, 3];
        assert!(gt(&mut d).is_ok());
        assert_eq!(d, vec![0]);
    }
}