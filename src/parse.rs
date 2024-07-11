use std::collections::HashMap;
use crate::tokenize::{Token, tokenize, TokenizeError};
use crate::Value;

// suggestion: put this near the top, just below `mod` and `use` statements
pub fn parse(input: String) -> Result<Value, ParseError> {
    let tokens = tokenize(input)?;
    let value = parse_tokens(&tokens, &mut 0)?;
    Ok(value)
}

// suggestion: put this below the definition of `Value`
#[derive(Debug, PartialEq)]
pub enum ParseError {
    TokenizeError(TokenizeError),
    ParseError(TokenParseError),
}

impl From<TokenParseError> for ParseError {
    fn from(err: TokenParseError) -> Self {
        Self::ParseError(err)
    }
}

impl From<TokenizeError> for ParseError {
    fn from(err: TokenizeError) -> Self {
        Self::TokenizeError(err)
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenParseError {
    /// An escape sequence was started without 4 hexadecimal digits afterward
    UnfinishedEscape,
    /// A character in an escape sequence was not valid hexadecimal
    InvalidHexValue,
    /// Invalid unicode value
    InvalidCodePointValue,
    ExpectedComma,
    ExpectedProperty,
    ExpectedColon,
    ExpectedValue,
}

type ParseResult = Result<Value, TokenParseError>;

fn parse_tokens(tokens: &Vec<Token>, index: &mut usize) -> ParseResult {
    let token = &tokens[*index];

    if matches!(
        token,
        Token::Null | Token::False | Token::True | Token::Number(_) | Token::String(_)
    ) {
        *index += 1
    }

    match token {
        Token::Null => Ok(Value::Null),
        Token::False => Ok(Value::Boolean(false)),
        Token::True => Ok(Value::Boolean(true)),
        Token::Number(number) => Ok(Value::Number(*number)),
        Token::String(string) => parse_string(string),
        Token::LeftBrace => parse_object(tokens, index),
        Token::LeftBracket => parse_array(tokens, index),
        _ => Err(TokenParseError::ExpectedValue)
    }
}

fn parse_string(input: &str) -> ParseResult {
    let mut output = String::new();
    let mut is_escaping = false;
    let mut chars = input.chars();

    while let Some(next_char) = chars.next() {
        if is_escaping {
            match next_char {
                '"' => output.push('"'),
                '\\' => output.push('\\'),
                // `\b` (backspace) is a valid escape in JSON, but not Rust
                'b' => output.push('\u{8}'),
                // `\f` (formfeed) is a valid escape in JSON, but not Rust
                'f' => output.push('\u{12}'),
                'n' => output.push('\n'),
                'r' => output.push('\r'),
                't' => output.push('\t'),
                'u' => {
                    let mut sum = 0;
                    for i in 0..4 {
                        let next_char = chars.next().ok_or(TokenParseError::UnfinishedEscape)?;
                        let digit = next_char
                            .to_digit(16)
                            .ok_or(TokenParseError::InvalidHexValue)?;
                        sum += (16u32).pow(3 - i) * digit;
                    }
                    let unescaped_char =
                        char::from_u32(sum).ok_or(TokenParseError::InvalidCodePointValue)?;
                    output.push(unescaped_char);
                }
                // any other character *may* be escaped, ex. `\q` just push that letter `q`
                _ => output.push(next_char),
            }
            is_escaping = false;
        } else if next_char == '\\' {
            is_escaping = true;
        } else {
            output.push(next_char);
        }
    }

    Ok(Value::String(output))
}

fn parse_array(tokens: &Vec<Token>, index: &mut usize) -> ParseResult {
    let mut array = Vec::new();

    loop {
        *index += 1;
        if tokens[*index] == Token::RightBracket {
            break;
        }

        let value = parse_tokens(tokens, index)?;
        array.push(value);

        let token = &tokens[*index];
        match token {
            Token::RightBracket => break,
            Token::Comma => {},
            _ => return Err(TokenParseError::ExpectedComma),
        }
    }

    *index += 1;

    Ok(Value::Array(array))
}

fn parse_object(tokens: &Vec<Token>, index: &mut usize) -> ParseResult {
    let mut map = HashMap::new();
    loop {
        // consume the previous LeftBrace or Comma token
        *index += 1;
        if tokens[*index] == Token::RightBrace {
            break;
        }

        if let Token::String(s) = &tokens[*index] {
            *index += 1;
            if Token::Colon == tokens[*index] {
                *index += 1;
                let key = s.clone();
                let value = parse_tokens(tokens, index)?;
                map.insert(key, value);
            } else {
                return Err(TokenParseError::ExpectedColon);
            }

            match &tokens[*index] {
                Token::Comma => {}
                Token::RightBrace => break,
                _ => return Err(TokenParseError::ExpectedComma),
            }
        } else {
            return Err(TokenParseError::ExpectedProperty);
        }
    }
    // Consume the RightBrace token
    *index += 1;

    Ok(Value::Object(map))
}

#[cfg(test)]
mod tests {
    use crate::tokenize::Token;
    use crate::Value;

    fn check(input: Vec<Token>, expected: Value) {
        let mut index = 0;
        let value = super::parse_tokens(&input, &mut index).unwrap();
        assert_eq!(value, expected);
    }

    #[test]
    fn test_parse_null() {
        check(vec![Token::Null], Value::Null);
    }

    #[test]
    fn parses_string_no_escapes() {
        let input = vec![Token::String("hello world".into())];
        let expected = Value::String("hello world".into());

        check(input, expected);
    }

    #[test]
    fn parses_string_non_ascii() {
        let input = vec![Token::String("olá_こんにちは_नमस्ते_привіт".into())];
        let expected = Value::String(String::from("olá_こんにちは_नमस्ते_привіт"));

        check(input, expected);
    }

    #[test]
    fn parses_string_with_emoji() {
        let input = vec![Token::String("hello 💩 world".into())];
        let expected = Value::String(String::from("hello 💩 world"));

        check(input, expected);
    }

    #[test]
    fn parses_string_unescape_backslash() {
        let input = vec![Token::String(r#"hello\\world"#.into())];
        let expected = Value::String(r#"hello\world"#.into());

        check(input, expected);
    }

    #[test]
    fn parses_array_one_element() {
        // [true]
        let input = vec![Token::LeftBracket, Token::True, Token::RightBracket];
        let expected = Value::Array(vec![Value::Boolean(true)]);

        check(input, expected);
    }

    #[test]
    fn parses_array_two_elements() {
        // [null, 16]
        let input = vec![
            Token::LeftBracket,
            Token::Null,
            Token::Comma,
            Token::Number(16.0),
            Token::RightBracket,
        ];
        let expected = Value::Array(vec![Value::Null, Value::Number(16.0)]);

        check(input, expected);
    }

    #[test]
    fn parses_empty_array() {
        // []
        let input = vec![Token::LeftBracket, Token::RightBracket];
        let expected = Value::Array(vec![]);

        check(input, expected);
    }

    #[test]
    fn parses_nested_array() {
        // [null, [null]]
        let input = vec![
            Token::LeftBracket,
            Token::Null,
            Token::Comma,
            Token::LeftBracket,
            Token::Null,
            Token::RightBracket,
            Token::RightBracket,
        ];
        let expected = Value::Array(vec![Value::Null, Value::Array(vec![Value::Null])]);

        check(input, expected);
    }

    #[test]
    fn test_parse() {
        let input = String::from(r#"{"key": "value"}"#);
        let expected = Value::Object(
            vec![("key".to_string(), Value::String("value".to_string()))]
                .into_iter()
                .collect(),
        );

        assert_eq!(super::parse(input).unwrap(), expected);
    }
}