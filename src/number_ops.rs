use anyhow::{anyhow, Result};

// Arithmetic operations with checks
pub fn add(a: i64, b: i64) -> Result<i64> {
    a.checked_add(b)
        .ok_or_else(|| anyhow!("Integer overflow in addition"))
}

pub fn subtract(a: i64, b: i64) -> Result<i64> {
    a.checked_sub(b)
        .ok_or_else(|| anyhow!("Integer underflow in subtraction"))
}

pub fn multiply(a: i64, b: i64) -> Result<i64> {
    a.checked_mul(b)
        .ok_or_else(|| anyhow!("Integer overflow in multiplication"))
}

pub fn divide(a: i64, b: i64) -> Result<i64> {
    if b == 0 {
        return Err(anyhow!("Division by zero"));
    }
    a.checked_div(b)
        .ok_or_else(|| anyhow!("Arithmetic error in division"))
}

pub fn modulo(a: i64, b: i64) -> Result<i64> {
    if b == 0 {
        return Err(anyhow!("Division by zero in modulo"));
    }
    a.checked_rem(b)
        .ok_or_else(|| anyhow!("Arithmetic error in modulo"))
}

// Comparison operations
pub fn equals(a: i64, b: i64) -> i64 {
    if a == b {
        -1
    } else {
        0
    }
}

pub fn less_than(a: i64, b: i64) -> i64 {
    if a < b {
        -1
    } else {
        0
    }
}

pub fn greater_than(a: i64, b: i64) -> i64 {
    if a > b {
        -1
    } else {
        0
    }
}

// Bitwise operations
pub fn and(a: i64, b: i64) -> i64 {
    a & b
}

pub fn or(a: i64, b: i64) -> i64 {
    a | b
}

pub fn not(a: i64) -> i64 {
    !a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3).unwrap(), 5);
        assert!(add(i64::MAX, 1).is_err());
    }

    #[test]
    fn test_subtract() {
        assert_eq!(subtract(5, 3).unwrap(), 2);
        assert!(subtract(i64::MIN, 1).is_err());
    }

    #[test]
    fn test_multiply() {
        assert_eq!(multiply(2, 3).unwrap(), 6);
        assert!(multiply(i64::MAX, 2).is_err());
    }

    #[test]
    fn test_divide() {
        assert_eq!(divide(6, 3).unwrap(), 2);
        assert!(divide(1, 0).is_err());
    }

    #[test]
    fn test_modulo() {
        assert_eq!(modulo(7, 3).unwrap(), 1);
        assert!(modulo(1, 0).is_err());
    }

    #[test]
    fn test_equals() {
        assert_eq!(equals(5, 5), -1);
        assert_eq!(equals(5, 6), 0);
    }

    #[test]
    fn test_less_than() {
        assert_eq!(less_than(3, 5), -1);
        assert_eq!(less_than(5, 3), 0);
    }

    #[test]
    fn test_greater_than() {
        assert_eq!(greater_than(5, 3), -1);
        assert_eq!(greater_than(3, 5), 0);
    }

    #[test]
    fn test_and() {
        assert_eq!(and(0b1100, 0b1010), 0b1000);
    }

    #[test]
    fn test_or() {
        assert_eq!(or(0b1100, 0b1010), 0b1110);
    }

    #[test]
    fn test_not() {
        assert_eq!(not(0), -1);
        assert_eq!(not(-1), 0);
    }
}
