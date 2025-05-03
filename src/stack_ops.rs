use anyhow::{anyhow, Result};

// Stack operations
pub fn dup(stack: &mut Vec<i64>) -> Result<()> {
    if stack.is_empty() {
        return Err(anyhow!("Stack underflow"));
    }
    let value = *stack.last().unwrap();
    stack.push(value);
    Ok(())
}

pub fn drop_(stack: &mut Vec<i64>) -> Result<()> {
    if stack.is_empty() {
        return Err(anyhow!("Stack underflow"));
    }
    stack.pop();
    Ok(())
}

pub fn swap(stack: &mut Vec<i64>) -> Result<()> {
    if stack.len() < 2 {
        return Err(anyhow!("Stack underflow"));
    }
    let len = stack.len();
    stack.swap(len - 1, len - 2);
    Ok(())
}

pub fn over(stack: &mut Vec<i64>) -> Result<()> {
    if stack.len() < 2 {
        return Err(anyhow!("Stack underflow"));
    }
    let value = stack[stack.len() - 2];
    stack.push(value);
    Ok(())
}

pub fn rot(stack: &mut Vec<i64>) -> Result<()> {
    if stack.len() < 3 {
        return Err(anyhow!("Stack underflow"));
    }
    let len = stack.len();
    let value = stack.remove(len - 3);
    stack.push(value);
    Ok(())
}

pub fn minus_rot(stack: &mut Vec<i64>) -> Result<()> {
    if stack.len() < 3 {
        return Err(anyhow!("Stack underflow"));
    }
    let len = stack.len();
    let value = stack.pop().unwrap();
    stack.insert(len - 3, value);
    Ok(())
}

pub fn q_dup(stack: &mut Vec<i64>) -> Result<()> {
    if stack.is_empty() {
        return Err(anyhow!("Stack underflow"));
    }
    let value = *stack.last().unwrap();
    if value != 0 {
        stack.push(value);
    }
    Ok(())
}

pub fn two_dup(stack: &mut Vec<i64>) -> Result<()> {
    if stack.len() < 2 {
        return Err(anyhow!("Stack underflow"));
    }
    let len = stack.len();
    let value1 = stack[len - 2];
    let value2 = stack[len - 1];
    stack.push(value1);
    stack.push(value2);
    Ok(())
}

pub fn two_drop(stack: &mut Vec<i64>) -> Result<()> {
    if stack.len() < 2 {
        return Err(anyhow!("Stack underflow"));
    }
    stack.pop();
    stack.pop();
    Ok(())
}

pub fn two_swap(stack: &mut Vec<i64>) -> Result<()> {
    if stack.len() < 4 {
        return Err(anyhow!("Stack underflow"));
    }
    let len = stack.len();
    stack.swap(len - 1, len - 3);
    stack.swap(len - 2, len - 4);
    Ok(())
}

pub fn two_over(stack: &mut Vec<i64>) -> Result<()> {
    if stack.len() < 4 {
        return Err(anyhow!("Stack underflow"));
    }
    let len = stack.len();
    let value1 = stack[len - 4];
    let value2 = stack[len - 3];
    stack.push(value1);
    stack.push(value2);
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
        assert!(dup(&mut vec![]).is_err());
    }

    #[test]
    fn test_drop() {
        let mut stack = vec![10, 20];
        assert!(drop_(&mut stack).is_ok());
        assert_eq!(stack, vec![10]);
        assert!(drop_(&mut stack).is_ok());
        assert_eq!(stack, vec![]);
        assert!(drop_(&mut stack).is_err());
    }

    #[test]
    fn test_swap() {
        let mut stack = vec![10, 20];
        assert!(swap(&mut stack).is_ok());
        assert_eq!(stack, vec![20, 10]);
        assert!(swap(&mut vec![1]).is_err());
    }

    #[test]
    fn test_over() {
        let mut stack = vec![10, 20];
        assert!(over(&mut stack).is_ok());
        assert_eq!(stack, vec![10, 20, 10]);
        assert!(over(&mut vec![1]).is_err());
    }

    #[test]
    fn test_rot() {
        let mut stack = vec![10, 20, 30];
        assert!(rot(&mut stack).is_ok());
        assert_eq!(stack, vec![20, 30, 10]);
        assert!(rot(&mut vec![1, 2]).is_err());
    }

    #[test]
    fn test_q_dup() {
        let mut stack = vec![10];
        assert!(q_dup(&mut stack).is_ok());
        assert_eq!(stack, vec![10, 10]);
        let mut stack = vec![0];
        assert!(q_dup(&mut stack).is_ok());
        assert_eq!(stack, vec![0]);
        assert!(q_dup(&mut vec![]).is_err());
    }

    #[test]
    fn test_two_dup() {
        let mut stack = vec![10, 20];
        assert!(two_dup(&mut stack).is_ok());
        assert_eq!(stack, vec![10, 20, 10, 20]);
        assert!(two_dup(&mut vec![1]).is_err());
    }

    #[test]
    fn test_two_drop() {
        let mut stack = vec![10, 20, 30];
        assert!(two_drop(&mut stack).is_ok());
        assert_eq!(stack, vec![10]);
        assert!(two_drop(&mut vec![1]).is_err());
    }

    #[test]
    fn test_two_swap() {
        let mut stack = vec![10, 20, 30, 40];
        assert!(two_swap(&mut stack).is_ok());
        assert_eq!(stack, vec![30, 40, 10, 20]);
        assert!(two_swap(&mut vec![1, 2, 3]).is_err());
    }

    #[test]
    fn test_two_over() {
        let mut stack = vec![10, 20, 30, 40];
        assert!(two_over(&mut stack).is_ok());
        assert_eq!(stack, vec![10, 20, 30, 40, 10, 20]);
        assert!(two_over(&mut vec![1, 2, 3]).is_err());
    }

    #[test]
    fn test_minus_rot() {
        let mut stack = vec![10, 20, 30];
        assert!(minus_rot(&mut stack).is_ok());
        assert_eq!(stack, vec![30, 10, 20]);
        assert!(minus_rot(&mut vec![1, 2]).is_err());
    }
}
