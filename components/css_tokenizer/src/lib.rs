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
            let token = self.consume_token();

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

    fn next_input_code_point(&self) -> Option<char> {
        self.peek(1)
    }

    fn current_input_code_point(&self) -> Option<char> {
        self.peek(0)
    }

    fn reconsume_current_input_code_point(&mut self) {
        self.position -= 1;
    }

    fn consume_next_code_point(&mut self) -> Option<char> {
        self.position += 1;
        self.current_input_code_point()
    }

    // https://www.w3.org/TR/css-syntax-3/#consume-token
    fn consume_token(&mut self) -> Result<Token, CssParsingError> {
        self.consume_comments()?;

        let code_point = self.consume_next_code_point();

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
                '\\' => todo!(),
                ']' => Ok(Token::RightSquareBracket),
                '{' => Ok(Token::LeftCurlyBracket),
                '}' => Ok(Token::RightCurlyBracket),
                definition!(digit) => todo!(),
                definition!(ident_start_code_point) => {
                    self.reconsume_current_input_code_point();
                    Ok(self.consume_ident_like_token())
                }
                _ => Ok(Token::Delim { value: code_point }),
            },
            definition!(eof_code_point) => Ok(Token::EndOfFile),
        }
    }

    fn consume_comments(&mut self) -> Result<(), CssParsingError> {
        loop {
            let next_two = self.next_two_input_code_points();

            if !(next_two.first == Some('/') && next_two.second == Some('*')) {
                break;
            }

            self.consume_next_code_point();
            self.consume_next_code_point();

            loop {
                let inner_next_two = self.next_two_input_code_points();
                if inner_next_two.first.is_none() || inner_next_two.second.is_none() {
                    return Err(CssParsingError::InvalidEndOfFile);
                }

                if inner_next_two.first == Some('*') && inner_next_two.second == Some('/') {
                    self.consume_next_code_point();
                    self.consume_next_code_point();
                    break;
                }

                self.consume_next_code_point();
            }
        }

        Ok(())
    }

    fn consume_ident_like_token(&mut self) -> Token {
        let string = self.consume_an_ident_sequence();

        if string.eq_ignore_ascii_case("url") {
            todo!()
        }

        if self
            .next_input_code_point()
            .is_some_and(|code_point| code_point == '(')
        {
            self.consume_next_code_point();
            return Token::Function { value: string };
        }

        Token::Ident { value: string }
    }

    fn consume_an_ident_sequence(&mut self) -> String {
        let mut result = "".to_string();
        loop {
            if let Some(input) = self.consume_next_code_point() {
                match input {
                    definition!(ident_code_point) => {
                        result.push(input);
                    }
                    '\\' => todo!(), // FIXME: This should use a seperate function to check if it really is an escape function.
                    _ => {
                        self.reconsume_current_input_code_point();
                        break;
                    }
                }
            } else {
                break;
            }
        }
        result
    }

    fn consume_as_much_whitespace_as_possible(&mut self) {
        while self
            .next_input_code_point()
            .is_some_and(|code_point| code_point.is_whitespace())
        {
            self.consume_next_code_point();
        }
    }

    fn next_two_input_code_points(&self) -> NextTwoInputCodePoints {
        NextTwoInputCodePoints {
            first: self.peek(0),
            second: self.peek(1),
        }
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.input.chars().nth(self.position + offset)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NextTwoInputCodePoints {
    first: Option<char>,
    second: Option<char>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CssParsingError {
    InvalidEndOfFile,
}
