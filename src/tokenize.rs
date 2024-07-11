use std::num::ParseFloatError;

#[derive(Debug, PartialEq)]
pub enum Token {
    /// `{`
    LeftBrace,
    /// `}`
    RightBrace,
    /// `[`
    LeftBracket,
    /// `]`
    RightBracket,
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// `null`
    Null,
    /// `false`
    False,
    /// `true`
    True,
    /// Any number literal
    Number(f64),
    /// Key of the key/value pair or a string value
    String(String),
}

#[derive(Debug, PartialEq)]
pub enum TokenizeError {
    UnfinishedLiteralValue,
    UnclosedQuotes,
    UnexpectedEof,
    CharNotRecognized(char),
    ParseNumberError(ParseFloatError),
}

pub fn tokenize(input: String) -> Result<Vec<Token>, TokenizeError> {
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;

    let mut tokens = Vec::new();

    while index < chars.len() {
        let token = make_token(&chars, &mut index)?;
        tokens.push(token);
        index += 1;
    }

    Ok(tokens)
}

fn make_token(chars: &Vec<char>, index: &mut usize) -> Result<Token, TokenizeError> {
    let mut ch = chars[*index];
    while ch.is_ascii_whitespace() {
        *index += 1;
        if *index >= chars.len() {
            return Err(TokenizeError::UnexpectedEof);
        }
        ch = chars[*index];
    }

    let token = match ch {
        '{' => Token::LeftBrace,
        '}' => Token::RightBrace,
        '[' => Token::LeftBracket,
        ']' => Token::RightBracket,
        ':' => Token::Colon,
        ',' => Token::Comma,
        'n' => tokenize_literal(String::from("null"), chars, index)?,
        'f' => tokenize_literal(String::from("false"), chars, index)?,
        't' => tokenize_literal(String::from("true"), chars, index)?,
        '"' => tokenize_string(chars, index)?,
        c if c.is_ascii_digit() || c == '-' => tokenize_float(chars, index)?,
        _ => return Err(TokenizeError::CharNotRecognized(ch)),
    };

    Ok(token)
}

fn tokenize_float(chars: &Vec<char>, index: &mut usize) -> Result<Token, TokenizeError> {
    let mut unparsed = String::new();
    let mut has_decimal = false;

    while *index < chars.len() {
        let ch = chars[*index];

        match ch {
            c if c.is_ascii_digit() => unparsed.push(c),
            c if c == '.' && !has_decimal => {
                unparsed.push(c);
                has_decimal = true;
            }
            _ => break,
        }

        *index += 1;
    }

    match unparsed.parse() {
        Ok(num) => Ok(Token::Number(num)),
        Err(e) => Err(TokenizeError::ParseNumberError(e)),
    }
}

fn tokenize_literal(str: String, chars: &Vec<char>, index: &mut usize) -> Result<Token, TokenizeError> {
    for expected_char in str.chars() {
        let ch = chars[*index];
        if ch != expected_char {
            return Err(TokenizeError::UnfinishedLiteralValue);
        }
        *index += 1;
    }

    match str.as_str() {
        "null" => Ok(Token::Null),
        "false" => Ok(Token::False),
        "true" => Ok(Token::True),
        _ => Err(TokenizeError::UnfinishedLiteralValue),
    }
}

fn tokenize_string(chars: &Vec<char>, index: &mut usize) -> Result<Token, TokenizeError> {
    let mut string = String::new();
    let mut is_escaping = false;

    loop {
        *index += 1;
        if *index > chars.len() {
            return Err(TokenizeError::UnclosedQuotes);
        }

        let ch = chars[*index];
        match ch {
            '"' if !is_escaping => break,
            '\\' => is_escaping = !is_escaping,
            _ => is_escaping = false,
        }

        string.push(ch);
    }
    Ok(Token::String(string))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comma() {
        let input = String::from(",");
        let expected = vec![Token::Comma];
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    fn test_literal(literal: &str, expected: Token) {
        let input = String::from(literal);
        let expected = vec![expected];
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    fn test_null() {
        test_literal("null", Token::Null);
    }

    #[test]
    fn test_false() {
        test_literal("false", Token::False);
    }

    #[test]
    fn test_true() {
        test_literal("true", Token::True);
    }

    #[test]
    fn test_integer() {
        let input = String::from("123");
        let expected = vec![Token::Number(123.0)];
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    fn test_negative_integer() {
        let input = String::from("-123");
        let expected = vec![Token::Number(-123.0)];
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    fn test_string() {
        let input = String::from("\"hello\"");
        let expected = vec![Token::String(String::from("hello"))];
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    fn test_unclosed_quotes() {
        let input = String::from("\"unclosed string");
        assert_eq!(tokenize(input), Err(TokenizeError::UnclosedQuotes));
    }

    #[test]
    fn test_escape_quotes() {
        let input = String::from(r#""the \" us OK""#);
        let expected = vec![Token::String(String::from(r#"the \" us OK"#))];
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    fn test_float() {
        let input = String::from("123.456");
        let expected = vec![Token::Number(123.456)];
        assert_eq!(tokenize(input).unwrap(), expected);

        let input = String::from("-123.456");
        let expected = vec![Token::Number(-123.456)];
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    fn test_all_punctuation() {
        let input = String::from("{}[]:,");
        let expected = vec![
            Token::LeftBrace,
            Token::RightBrace,
            Token::LeftBracket,
            Token::RightBracket,
            Token::Colon,
            Token::Comma,
        ];
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    fn test_tokenize() {
        let input = String::from(r#"{"key": "value"}"#);
        let expected = vec![
            Token::LeftBrace,
            Token::String(String::from("key")),
            Token::Colon,
            Token::String(String::from("value")),
            Token::RightBrace,
        ];
        assert_eq!(tokenize(input).unwrap(), expected);
    }
}
