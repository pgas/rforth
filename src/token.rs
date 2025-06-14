use logos::Logos;
use std::fmt;

// Define the error type for lexing
#[derive(Debug, Clone, PartialEq, Default)] // Added Default
pub struct LexingError;

impl fmt::Display for LexingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Lexing Error")
    }
}

// Define the tokens using Logos
#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(error = LexingError)] // Use our defined error type
pub enum Token {
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Whitespace,

    // Parentheses comments (skip)
    #[regex(r"\([^)]*\)", logos::skip, priority = 4)]
    Comment,
    // Line comments starting with backslash (skip)
    #[regex(r"\\[^\n]*", logos::skip, priority = 4)]
    LineComment,

    #[token(":")]
    Colon,
    #[token(";")]
    Semicolon,

    // Integer: optional '-' then digits
    #[regex(r"-?[0-9]+", |lex| lex.slice().parse::<i64>().ok(), priority = 3)]
    Integer(i64),

    // Word: alphanumeric and permitted symbols
    #[regex(r"[A-Za-z0-9+*/.?=<>-]+", |lex| Some(lex.slice().to_string()), priority = 2)]
    Word(String),
    // Logos will emit errors for unrecognized chars which are filtered out
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Integer(i) => write!(f, "{}", i),
            Token::Word(s) => write!(f, "{}", s),
            Token::Colon => write!(f, ":"),
            Token::Semicolon => write!(f, ";"),
            Token::Whitespace => write!(f, " "), // Should ideally not be displayed directly
            Token::Comment => write!(f, "(comment)"), // Should ideally not be displayed directly
            Token::LineComment => write!(f, "\\\\ comment"), // Should ideally not be displayed directly
                                                             // No Error variant in Token enum
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;

    // Helper to lex a string and collect tokens, filtering out errors for simplicity
    fn lex_string(input: &str) -> Vec<Token> {
        Token::lexer(input).filter_map(|res| res.ok()).collect()
    }

    // Helper to lex a string and collect results including errors
    fn lex_string_results(input: &str) -> Vec<Result<Token, LexingError>> {
        Token::lexer(input).collect()
    }

    #[test]
    fn test_lex_simple_word() {
        assert_eq!(lex_string("word"), vec![Token::Word("word".to_string())]);
    }

    #[test]
    fn test_lex_word_with_number() {
        // These should be single Word tokens
        assert_eq!(lex_string("sq3"), vec![Token::Word("sq3".to_string())]);
        assert_eq!(
            lex_string("test-1"),
            vec![Token::Word("test-1".to_string())]
        );
        assert_eq!(
            lex_string("word123"),
            vec![Token::Word("word123".to_string())]
        );
    }

    #[test]
    fn test_lex_number_word_mix() {
        // Should be Word("1abc") not Integer(1), Word("abc")
        assert_eq!(lex_string("1abc"), vec![Token::Word("1abc".to_string())]);
        // Should be Word("a1b2")
        assert_eq!(lex_string("a1b2"), vec![Token::Word("a1b2".to_string())]);
    }

    #[test]
    fn test_lex_pure_number() {
        assert_eq!(lex_string("123"), vec![Token::Integer(123)]);
        assert_eq!(lex_string("-45"), vec![Token::Integer(-45)]);
        // Ensure leading zero doesn't make it a word if it's just a number
        assert_eq!(lex_string("0"), vec![Token::Integer(0)]);
        assert_eq!(lex_string("-0"), vec![Token::Integer(0)]);
    }

    #[test]
    fn test_lex_number_followed_by_letter_no_space() {
        // "123word" should be a single word token according to space delimiting rule
        assert_eq!(
            lex_string("123word"),
            vec![Token::Word("123word".to_string())]
        );
    }

    #[test]
    fn test_lex_colon_semicolon() {
        assert_eq!(lex_string(":"), vec![Token::Colon]);
        assert_eq!(lex_string(";"), vec![Token::Semicolon]);
    }

    #[test]
    fn test_lex_mixed_sequence() {
        assert_eq!(
            lex_string("10 sq3 + : foo ;"),
            vec![
                Token::Integer(10),
                Token::Word("sq3".to_string()), // Correctly lexed as one word
                Token::Word("+".to_string()),
                Token::Colon,
                Token::Word("foo".to_string()),
                Token::Semicolon,
            ]
        );
    }

    #[test]
    fn test_lex_word_starting_with_symbol() {
        assert_eq!(lex_string("+word"), vec![Token::Word("+word".to_string())]);
        assert_eq!(lex_string(".s"), vec![Token::Word(".s".to_string())]); // Ensure existing words still work
        assert_eq!(lex_string("."), vec![Token::Word(".".to_string())]);
        assert_eq!(lex_string("+"), vec![Token::Word("+".to_string())]);
    }

    #[test]
    fn test_lex_number_followed_by_symbol_no_space() {
        // This depends on how exactly Forth standard treats this.
        // Assuming "1+" is a single word if not separated by space.
        assert_eq!(lex_string("1+"), vec![Token::Word("1+".to_string())]);
        // But "1 +" should be two tokens
        assert_eq!(
            lex_string("1 +"),
            vec![Token::Integer(1), Token::Word("+".to_string())]
        );
    }

    #[test]
    fn test_lex_whitespace_and_delimiters() {
        assert_eq!(
            lex_string("  word1  word2 "),
            vec![
                Token::Word("word1".to_string()),
                Token::Word("word2".to_string())
            ]
        );
        assert_eq!(
            lex_string("word1:word2"),
            vec![
                Token::Word("word1".to_string()),
                Token::Colon,
                Token::Word("word2".to_string())
            ]
        );
        assert_eq!(
            lex_string("word1;word2"),
            vec![
                Token::Word("word1".to_string()),
                Token::Semicolon,
                Token::Word("word2".to_string())
            ]
        );
        assert_eq!(
            lex_string(":word;"),
            vec![
                Token::Colon,
                Token::Word("word".to_string()),
                Token::Semicolon
            ]
        );
    }

    #[test]
    fn test_lex_error_handling() {
        // First, test the basic case with the filtered lexer function
        assert_eq!(lex_string("#$%"), Vec::<Token>::new());

        // Now test with the unfiltered lexer function to capture errors
        let results = lex_string_results("#");
        assert!(!results.is_empty(), "Should have at least one result");
        assert!(
            matches!(results[0], Err(_)),
            "First result should be an error"
        );

        // Test with multiple invalid characters
        let results = lex_string_results("#$%");
        assert_eq!(
            results.len(),
            3,
            "Should have three results for three invalid chars"
        );
        assert!(
            results.iter().all(|r| matches!(r, Err(_))),
            "All results should be errors"
        );

        // Test mixed valid and invalid input
        let results = lex_string_results("123 # abc");
        // Whitespace is skipped by the lexer, so we only get 3 tokens/errors
        assert_eq!(
            results.len(),
            3,
            "Should have 3 tokens/errors: '123', '#', 'abc'"
        );

        // The first token should be Ok(Integer(123))
        assert!(matches!(results[0], Ok(Token::Integer(123))));

        // The '#' should be an error
        assert!(matches!(results[1], Err(_)));

        // The 'abc' should be Ok(Word("abc"))
        assert!(matches!(results[2], Ok(Token::Word(ref s)) if s == "abc"));
    }

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

    #[test]
    fn test_lexer_definition() {
        let input = ": SQUARE DUP * ;";
        let tokens: Vec<Token> = Token::lexer(input).filter_map(Result::ok).collect();
        assert_eq!(
            tokens,
            vec![
                Token::Colon,
                Token::Word("SQUARE".to_string()),
                Token::Word("DUP".to_string()),
                Token::Word("*".to_string()),
                Token::Semicolon,
            ]
        );
    }

    #[test]
    fn test_lexer_mixed_with_definition() {
        let input = "10 : DOUBLE 2 * ; DOUBLE .";
        let tokens: Vec<Token> = Token::lexer(input).filter_map(Result::ok).collect();
        assert_eq!(
            tokens,
            vec![
                Token::Integer(10),
                Token::Colon,
                Token::Word("DOUBLE".to_string()),
                Token::Integer(2),
                Token::Word("*".to_string()),
                Token::Semicolon,
                Token::Word("DOUBLE".to_string()),
                Token::Word(".".to_string()),
            ]
        );
    }
}
