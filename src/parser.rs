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
    Word(String),                 // For words not yet defined or handled
    Define(String, Vec<ForthOp>), // Added: Name and body of the definition
    // Conditional: IF-ELSE-THEN branches
    IfElse(Vec<ForthOp>, Vec<ForthOp>),
    // Comparisons
    Eq,
    Lt,
    Gt,
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
            ForthOp::Define(name, ops) => write!(f, "Define({}, {:?})", name, ops), // Added
            ForthOp::IfElse(then_ops, else_ops) => {
                write!(f, "IfElse({:?}, {:?})", then_ops, else_ops)
            }
            ForthOp::Eq => write!(f, "Eq"),
            ForthOp::Lt => write!(f, "Lt"),
            ForthOp::Gt => write!(f, "Gt"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    // Removed UnknownWord as it's handled by ForthOp::Word
    UnexpectedToken(Token),       // E.g., Semicolon without Colon
    ExpectedWordName,             // E.g., Colon not followed by a Word
    UnterminatedDefinition,       // E.g., Reached end of input inside definition
    NestedDefinitionNotSupported, // E.g., Colon inside a definition
    UnterminatedConditional,
}

// Helper function to parse a single token into a ForthOp (used in interpret and compile modes)
fn parse_token_to_op(token: Token) -> Option<ForthOp> {
    match token {
        Token::Integer(i) => Some(ForthOp::Push(i)),
        Token::Word(s) => {
            match s.to_lowercase().as_str() {
                // Comparison operators
                "=" => Some(ForthOp::Eq),
                "<" => Some(ForthOp::Lt),
                ">" => Some(ForthOp::Gt),
                "+" => Some(ForthOp::Add),
                "-" => Some(ForthOp::Subtract),
                "*" => Some(ForthOp::Multiply),
                "/" => Some(ForthOp::Divide),
                "." => Some(ForthOp::Print),
                ".s" => Some(ForthOp::PrintStack),
                "dup" => Some(ForthOp::Dup),
                "drop" => Some(ForthOp::Drop),
                "swap" => Some(ForthOp::Swap),
                "over" => Some(ForthOp::Over),
                "rot" => Some(ForthOp::Rot),
                "?dup" => Some(ForthOp::QDup),
                "2dup" => Some(ForthOp::TwoDup),
                "2drop" => Some(ForthOp::TwoDrop),
                "2swap" => Some(ForthOp::TwoSwap),
                "2over" => Some(ForthOp::TwoOver),
                "-rot" => Some(ForthOp::MinusRot),
                _ => Some(ForthOp::Word(s)),
            }
        }
        // Colon and Semicolon handled in parse(), skip other tokens
        _ => None,
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<ForthOp>, ParseError> {
    let mut ops = Vec::new();
    let mut token_iter = tokens.into_iter().peekable();
    let mut compiling = false; // Are we inside a : ... ; definition?
    let mut current_def_name: Option<String> = None;
    let mut current_def_body: Vec<ForthOp> = Vec::new();

    while let Some(token) = token_iter.next() {
        // Skip whitespace and comments
        if matches!(
            token,
            Token::Whitespace | Token::Comment | Token::LineComment
        ) {
            continue;
        }

        if compiling {
            // Compile-only IF ... ELSE ... THEN
            if let Token::Word(s) = &token {
                if s.to_lowercase() == "if" {
                    // Collect then- and else- tokens
                    let mut then_toks = Vec::new();
                    let mut else_toks = Vec::new();
                    let mut depth = 1;
                    let mut in_else = false;
                    while let Some(next_tok) = token_iter.next() {
                        if let Token::Word(w) = &next_tok {
                            let wl = w.to_lowercase();
                            if wl == "if" {
                                depth += 1;
                            } else if wl == "else" && depth == 1 {
                                in_else = true;
                                continue;
                            } else if wl == "then" {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                        }
                        if in_else {
                            else_toks.push(next_tok.clone());
                        } else {
                            then_toks.push(next_tok.clone());
                        }
                    }
                    if depth != 0 {
                        return Err(ParseError::UnterminatedConditional);
                    }
                    // Parse branches and append to definition body
                    let then_ops = parse(then_toks)?;
                    let else_ops = if in_else {
                        parse(else_toks)?
                    } else {
                        Vec::new()
                    };
                    current_def_body.push(ForthOp::IfElse(then_ops, else_ops));
                    continue;
                }
            }
            match token {
                Token::Semicolon => {
                    // End definition
                    let name = current_def_name.take().unwrap();
                    ops.push(ForthOp::Define(name, current_def_body.clone()));
                    current_def_body.clear();
                    compiling = false;
                }
                Token::Colon => return Err(ParseError::NestedDefinitionNotSupported),
                _ => {
                    if let Some(op) = parse_token_to_op(token.clone()) {
                        current_def_body.push(op);
                    } else {
                        return Err(ParseError::UnexpectedToken(token));
                    }
                }
            }
        } else {
            // Interpret mode: compile-only words treated as data
            match token {
                Token::Colon => {
                    // Start new definition
                    match token_iter.next() {
                        Some(Token::Word(name)) => {
                            compiling = true;
                            current_def_name = Some(name.to_uppercase());
                        }
                        _ => return Err(ParseError::ExpectedWordName),
                    }
                }
                Token::Semicolon => return Err(ParseError::UnexpectedToken(Token::Semicolon)),
                other => {
                    // If this is 'if', consume until matching 'then', pushing all as Word
                    if let Token::Word(ref s) = other {
                        if s.eq_ignore_ascii_case("if") {
                            ops.push(ForthOp::Word(s.clone()));
                            let mut depth = 1;
                            while let Some(next_tok) = token_iter.next() {
                                if let Token::Word(ref w) = next_tok {
                                    let wl = w.to_lowercase();
                                    ops.push(ForthOp::Word(w.clone()));
                                    if wl == "if" {
                                        depth += 1;
                                    } else if wl == "then" {
                                        depth -= 1;
                                        if depth == 0 {
                                            break;
                                        }
                                    }
                                    continue;
                                }
                                // Non-word tokens: parse normally
                                if let Some(op) = parse_token_to_op(next_tok.clone()) {
                                    ops.push(op);
                                } else {
                                    return Err(ParseError::UnexpectedToken(next_tok));
                                }
                            }
                            continue; // done handling 'if'
                        }
                    }
                    // Regular token: parse normally
                    if let Some(op) = parse_token_to_op(other.clone()) {
                        ops.push(op);
                    } else {
                        return Err(ParseError::UnexpectedToken(other));
                    }
                }
            }
        }
    }

    // Check if we ended mid-definition
    if compiling {
        return Err(ParseError::UnterminatedDefinition);
    }

    Ok(ops)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Token;

    // ... existing test_parse_basic_ops ...
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

    // ... existing test_parse_stack_ops ...
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

    // ... existing test_parse_all_known_words ...
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

    // ... existing test_parse_empty ...
    #[test]
    fn test_parse_empty() {
        let tokens = vec![];
        let expected_ops = Ok(vec![]);
        assert_eq!(parse(tokens), expected_ops);
    }

    #[test]
    fn test_parse_definition() {
        let tokens = vec![
            Token::Colon,
            Token::Word("DOUBLE".to_string()),
            Token::Integer(2),
            Token::Word("*".to_string()),
            Token::Semicolon,
        ];
        let expected_ops = Ok(vec![ForthOp::Define(
            "DOUBLE".to_string(),
            vec![ForthOp::Push(2), ForthOp::Multiply],
        )]);
        assert_eq!(parse(tokens), expected_ops);
    }

    #[test]
    fn test_parse_mixed_definition_and_execution() {
        let tokens = vec![
            Token::Integer(10),
            Token::Colon,
            Token::Word("SQUARE".to_string()),
            Token::Word("DUP".to_string()),
            Token::Word("*".to_string()),
            Token::Semicolon,
            Token::Word("SQUARE".to_string()), // This will be ForthOp::Word("SQUARE")
            Token::Word(".".to_string()),
        ];
        let expected_ops = Ok(vec![
            ForthOp::Push(10),
            ForthOp::Define("SQUARE".to_string(), vec![ForthOp::Dup, ForthOp::Multiply]),
            ForthOp::Word("SQUARE".to_string()),
            ForthOp::Print,
        ]);
        assert_eq!(parse(tokens), expected_ops);
    }

    #[test]
    fn test_parse_error_unterminated_definition() {
        let tokens = vec![
            Token::Colon,
            Token::Word("TEST".to_string()),
            Token::Integer(1),
        ];
        assert_eq!(parse(tokens), Err(ParseError::UnterminatedDefinition));
    }

    #[test]
    fn test_parse_error_unexpected_semicolon() {
        let tokens = vec![Token::Integer(1), Token::Semicolon];
        assert_eq!(
            parse(tokens),
            Err(ParseError::UnexpectedToken(Token::Semicolon))
        );
    }

    #[test]
    fn test_parse_error_colon_no_name() {
        let tokens = vec![
            Token::Colon,
            Token::Integer(5), // Not a word name
        ];
        assert_eq!(parse(tokens), Err(ParseError::ExpectedWordName));
    }

    #[test]
    fn test_parse_error_colon_eof() {
        let tokens = vec![Token::Colon];
        assert_eq!(parse(tokens), Err(ParseError::ExpectedWordName));
    }

    #[test]
    fn test_parse_error_nested_definition() {
        let tokens = vec![
            Token::Colon,
            Token::Word("OUTER".to_string()),
            Token::Colon, // Nested colon
            Token::Word("INNER".to_string()),
            Token::Semicolon,
            Token::Semicolon,
        ];
        assert_eq!(parse(tokens), Err(ParseError::NestedDefinitionNotSupported));
    }

    #[test]
    fn test_parse_if_then() {
        let tokens = vec![
            Token::Integer(1),
            Token::Word("if".to_string()),
            Token::Word("dup".to_string()),
            Token::Word("then".to_string()),
        ];
        let expected = Ok(vec![
            ForthOp::Push(1),
            ForthOp::Word("if".to_string()),
            ForthOp::Word("dup".to_string()),
            ForthOp::Word("then".to_string()),
        ]);
        assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn test_parse_if_else_then() {
        let tokens = vec![
            Token::Integer(0),
            Token::Word("if".to_string()),
            Token::Word("dup".to_string()),
            Token::Word("else".to_string()),
            Token::Word("swap".to_string()),
            Token::Word("then".to_string()),
        ];
        let expected = Ok(vec![
            ForthOp::Push(0),
            ForthOp::Word("if".to_string()),
            ForthOp::Word("dup".to_string()),
            ForthOp::Word("else".to_string()),
            ForthOp::Word("swap".to_string()),
            ForthOp::Word("then".to_string()),
        ]);
        assert_eq!(parse(tokens), expected);
    }

    #[test]
    fn test_parse_nested_if() {
        let tokens = vec![
            Token::Word("if".to_string()),
            Token::Word("if".to_string()),
            Token::Word("dup".to_string()),
            Token::Word("then".to_string()),
            Token::Word("then".to_string()),
        ];
        let expected = Ok(vec![
            ForthOp::Word("if".to_string()),
            ForthOp::Word("if".to_string()),
            ForthOp::Word("dup".to_string()),
            ForthOp::Word("then".to_string()),
            ForthOp::Word("then".to_string()),
        ]);
        assert_eq!(parse(tokens), expected);
    }
}
