use logos::Logos;
use std::fmt;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")] // Ignore whitespace
#[logos(skip r"\\.*\n")] // Skip line comments starting with '\'
#[logos(skip r"\([^)]*\)")] // Skip non-nested block comments like ( this )
// Removed: #[logos(error = logos::Skip)] - Errors become Result::Err(()) and are filtered in main
pub enum Token {
    #[regex(r"-?[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Integer(i64),

    #[regex(r"[a-zA-Z_+\-*/<>=.?]+", |lex| lex.slice().to_string())]
    Word(String),
    // Removed the explicit Error variant
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Integer(i) => write!(f, "{}", i),
            Token::Word(s) => write!(f, "{}", s),
            // Token::Error => write!(f, "<Error>"), // Removed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic() {
        let input = "10 20 + .s \\ comment\n( another comment ) -5 * .";
        // The lexer now produces Result<Token, ()>, collect ignores errors
        let tokens: Vec<Token> = Token::lexer(input).filter_map(Result::ok).collect();

        assert_eq!(
            tokens,
            vec![
                Token::Integer(10),
                Token::Integer(20),
                Token::Word("+".to_string()),
                Token::Word(".s".to_string()),
                Token::Integer(-5),
                Token::Word("*".to_string()),
                Token::Word(".".to_string()),
            ]
        );
    }

    #[test]
    fn test_lexer_words() {
        let input = "hello world dup swap rot";
        let tokens: Vec<Token> = Token::lexer(input).filter_map(Result::ok).collect();
        assert_eq!(
            tokens,
            vec![
                Token::Word("hello".to_string()),
                Token::Word("world".to_string()),
                Token::Word("dup".to_string()),
                Token::Word("swap".to_string()),
                Token::Word("rot".to_string()),
            ]
        );
    }

    #[test]
    fn test_lexer_mixed() {
        let input = "1 2 swap ( comment ) 3 4 drop";
        let tokens: Vec<Token> = Token::lexer(input).filter_map(Result::ok).collect();
        assert_eq!(
            tokens,
            vec![
                Token::Integer(1),
                Token::Integer(2),
                Token::Word("swap".to_string()),
                Token::Integer(3),
                Token::Integer(4),
                Token::Word("drop".to_string()),
            ]
        );
    }

    #[test]
    fn test_lexer_skips_unknown() {
        let input = "1 #$% 2"; // #$% should be skipped
        let tokens: Vec<Token> = Token::lexer(input).filter_map(Result::ok).collect();
        assert_eq!(tokens, vec![Token::Integer(1), Token::Integer(2)]);
    }
}
