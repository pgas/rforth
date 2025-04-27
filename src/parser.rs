use crate::token::Token;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum ForthOp {
    Push(i64),
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    // Stack
    Dup,      // dup
    Drop,     // drop
    Swap,     // swap
    Over,     // over
    Rot,      // rot
    QDup,     // ?dup
    TwoDup,   // 2dup
    TwoDrop,  // 2drop
    TwoSwap,  // 2swap
    TwoOver,  // 2over
    MinusRot, // -rot
    // Output
    Print,      // .
    PrintStack, // .s
    // Other
    Word(String), // For words not yet defined or handled
}

impl fmt::Display for ForthOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ForthOp::Push(i) => write!(f, "Push({})", i),
            ForthOp::Add => write!(f, "Add"),
            ForthOp::Subtract => write!(f, "Subtract"),
            ForthOp::Multiply => write!(f, "Multiply"),
            ForthOp::Divide => write!(f, "Divide"),
            ForthOp::Dup => write!(f, "Dup"),
            ForthOp::Drop => write!(f, "Drop"),
            ForthOp::Swap => write!(f, "Swap"),
            ForthOp::Over => write!(f, "Over"),
            ForthOp::Rot => write!(f, "Rot"),
            ForthOp::QDup => write!(f, "QDup"),
            ForthOp::TwoDup => write!(f, "TwoDup"),
            ForthOp::TwoDrop => write!(f, "TwoDrop"),
            ForthOp::TwoSwap => write!(f, "TwoSwap"),
            ForthOp::TwoOver => write!(f, "TwoOver"),
            ForthOp::MinusRot => write!(f, "MinusRot"),
            ForthOp::Print => write!(f, "Print"),
            ForthOp::PrintStack => write!(f, "PrintStack"),
            ForthOp::Word(s) => write!(f, "Word({})", s),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnknownWord(String),
    // Add other potential parse errors here if needed
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<ForthOp>, ParseError> {
    let mut ops = Vec::new();
    for token in tokens {
        match token {
            Token::Integer(i) => ops.push(ForthOp::Push(i)),
            Token::Word(s) => {
                match s.to_lowercase().as_str() {
                    // Match case-insensitively
                    // Arithmetic
                    "+" => ops.push(ForthOp::Add),
                    "-" => ops.push(ForthOp::Subtract),
                    "*" => ops.push(ForthOp::Multiply),
                    "/" => ops.push(ForthOp::Divide),
                    // Stack
                    "dup" => ops.push(ForthOp::Dup),
                    "drop" => ops.push(ForthOp::Drop),
                    "swap" => ops.push(ForthOp::Swap),
                    "over" => ops.push(ForthOp::Over),
                    "rot" => ops.push(ForthOp::Rot),
                    "?dup" => ops.push(ForthOp::QDup),
                    "2dup" => ops.push(ForthOp::TwoDup),
                    "2drop" => ops.push(ForthOp::TwoDrop),
                    "2swap" => ops.push(ForthOp::TwoSwap),
                    "2over" => ops.push(ForthOp::TwoOver),
                    "-rot" => ops.push(ForthOp::MinusRot),
                    // Output
                    "." => ops.push(ForthOp::Print),
                    ".s" => ops.push(ForthOp::PrintStack),
                    // Unknown words
                    _ => ops.push(ForthOp::Word(s)),
                }
            }
        }
    }
    Ok(ops)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Token;

    #[test]
    fn test_parse_basic_ops() {
        let tokens = vec![
            Token::Integer(10),
            Token::Integer(5),
            Token::Word("+".to_string()),
            Token::Word(".".to_string()),
        ];
        let expected_ops = Ok(vec![
            ForthOp::Push(10),
            ForthOp::Push(5),
            ForthOp::Add,
            ForthOp::Print,
        ]);
        assert_eq!(parse(tokens), expected_ops);
    }

    #[test]
    fn test_parse_stack_ops() {
        let tokens = vec![
            Token::Word("DUP".to_string()), // Test case-insensitivity
            Token::Word("drop".to_string()),
            Token::Word("swap".to_string()),
            Token::Word("over".to_string()),
            Token::Word("rot".to_string()),
            Token::Word("?dup".to_string()),
            Token::Word("2dup".to_string()),
            Token::Word("2drop".to_string()),
            Token::Word("2swap".to_string()),
            Token::Word("2over".to_string()),
            Token::Word("-rot".to_string()),
        ];
        let expected_ops = Ok(vec![
            ForthOp::Dup,
            ForthOp::Drop,
            ForthOp::Swap,
            ForthOp::Over,
            ForthOp::Rot,
            ForthOp::QDup,
            ForthOp::TwoDup,
            ForthOp::TwoDrop,
            ForthOp::TwoSwap,
            ForthOp::TwoOver,
            ForthOp::MinusRot,
        ]);
        assert_eq!(parse(tokens), expected_ops);
    }

    #[test]
    fn test_parse_all_known_words() {
        let tokens = vec![
            Token::Integer(1),
            Token::Integer(2),
            Token::Word("+".to_string()),
            Token::Word("-".to_string()),
            Token::Word("*".to_string()),
            Token::Word("/".to_string()),
            Token::Word(".".to_string()),
            Token::Word(".s".to_string()),
            Token::Word("dup".to_string()),
            Token::Word("drop".to_string()),
            Token::Word("swap".to_string()),
            Token::Word("over".to_string()),
            Token::Word("rot".to_string()),
            Token::Word("unknown".to_string()),
        ];
        let expected_ops = Ok(vec![
            ForthOp::Push(1),
            ForthOp::Push(2),
            ForthOp::Add,
            ForthOp::Subtract,
            ForthOp::Multiply,
            ForthOp::Divide,
            ForthOp::Print,
            ForthOp::PrintStack,
            ForthOp::Dup,
            ForthOp::Drop,
            ForthOp::Swap,
            ForthOp::Over,
            ForthOp::Rot,
            ForthOp::Word("unknown".to_string()), // Unknown words are passed through
        ]);
        assert_eq!(parse(tokens), expected_ops);
    }

    #[test]
    fn test_parse_empty() {
        let tokens = vec![];
        let expected_ops = Ok(vec![]);
        assert_eq!(parse(tokens), expected_ops);
    }
}
