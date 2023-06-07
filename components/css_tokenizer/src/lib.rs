use token::Token;

pub mod token;

macro_rules! definition {
    (eof_code_point) => {
        None
    };
    (digit) => {
        '0'..='9'
    };
    (uppercase_letter) => {
        'A'..='Z'
    };
    (lowercase_letter) => {
        'a'..='z'
    };
    (letter) => {
        definition!(uppercase_letter) | definition!(lowercase_letter)
    };
    (non_ascii_code_point) => {
        '\u{0080}'..='\u{10FFFF}'
    };
    (ident_start_code_point) => {
        definition!(letter) | definition!(non_ascii_code_point) | '_'
    };
    (ident_code_point) => {
        definition!(ident_start_code_point) | definition!(digit) | '-'
    };
    (whitespace) => {
        '\n' | '\t' | ' '
    };
}

macro_rules! log_current_token {
    ($token:expr) => {
        if std::env::var("CSS_TOKENIZER_LOGGING").is_ok() {
            eprintln!("\x1b[34m[CssTokenizer] {:?}\x1b[0m", $token);
        }
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tokenizer<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, CssParsingError> {
        let mut tokens = vec![];

        loop {
            let token = self.consume_a_token();

            log_current_token!(token);

            match token {
                Ok(token) => match token {
                    Token::EndOfFile => {
                        tokens.push(token);
                        break;
                    }
                    _ => tokens.push(token),
                },
                Err(err) => return Err(err),
            }
        }

        Ok(tokens)
    }

    // https://www.w3.org/TR/css-syntax-3/#consume-token
    fn consume_a_token(&mut self) -> Result<Token, CssParsingError> {
        // FIXME: Consume comments.

        let code_point = self.next_code_point();

        match code_point {
            Some(code_point) => match code_point {
                definition!(whitespace) => {
                    self.consume_as_much_whitespace_as_possible();
                    Ok(Token::Whitespace)
                }
                '"' => todo!(),
                '#' => todo!(),
                '\'' => todo!(),
                '(' => Ok(Token::LeftParenthesis),
                ')' => Ok(Token::RightParenthesis),
                '+' => todo!(),
                ',' => Ok(Token::Comma),
                '-' => todo!(),
                '.' => todo!(),
                ':' => Ok(Token::Colon),
                ';' => Ok(Token::Semicolon),
                '<' => todo!(),
                '@' => todo!(),
                '[' => Ok(Token::LeftSquareBracket),
                '/' => todo!(),
                ']' => Ok(Token::RightSquareBracket),
                '{' => Ok(Token::LeftCurlyBracket),
                '}' => Ok(Token::RightCurlyBracket),
                definition!(digit) => todo!(),
                definition!(ident_start_code_point) => {
                    self.reconsume_next_current_input_codepoint();
                    Ok(self.consume_ident_like_token())
                }
                _ => Ok(Token::Delim { value: code_point }),
            },
            definition!(eof_code_point) => Ok(Token::EndOfFile),
        }
    }

    fn next_code_point(&mut self) -> Option<char> {
        let code_point = self.input.chars().nth(self.position);
        self.position += 1;
        code_point
    }

    fn consume_as_much_whitespace_as_possible(&mut self) {
        while self
            .next_code_point()
            .is_some_and(|code_point| code_point.is_whitespace())
        {
            self.next_code_point();
        }
    }

    fn reconsume_next_current_input_codepoint(&mut self) {
        self.position -= 1;
    }

    fn consume_ident_like_token(&mut self) -> Token {
        let string = self.consume_an_ident_sequence();

        if string.eq_ignore_ascii_case("url") {
            todo!()
        }

        if self
            .next_code_point()
            .is_some_and(|code_point| code_point == '(')
        {
            return Token::Function { value: string };
        }

        Token::Ident { value: string }
    }

    fn consume_an_ident_sequence(&mut self) -> String {
        let mut result = "".to_string();
        while let Some(code_point) = self.next_code_point() {
            match code_point {
                definition!(ident_code_point) => {
                    result.push(code_point);
                }
                '\\' => todo!(), // FIXME: This should use a seperate function to check if it really is an escape function.
                _ => {
                    self.reconsume_next_current_input_codepoint();
                    return result;
                }
            }
        }
        result
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CssParsingError {}
