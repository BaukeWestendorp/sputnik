use token::{NumberType, Token};

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
    fn consume_a_token(&mut self) -> Result<Token, CssParsingError> {
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
                '+' => {
                    // If the input stream starts with a number, reconsume the current input code point, consume a numeric token, and return it.
                    if self.stream_starts_with_a_number()? {
                        self.reconsume_current_input_code_point();
                        return self.consume_a_numeric_token();
                    }

                    return Ok(Token::Delim { value: code_point });
                }
                ',' => Ok(Token::Comma),
                '-' => {
                    // If the input stream starts with a number, reconsume the current input code point, consume a numeric token, and return it.
                    if self.stream_starts_with_a_number()? {
                        self.reconsume_current_input_code_point();
                        return self.consume_a_numeric_token();
                    }

                    // Otherwise, if the next 2 input code points are U+002D HYPHEN-MINUS U+003E GREATER-THAN SIGN (->), consume them and return a <CDC-token>.
                    if let ('-', '>') = self.next_two_input_code_points()? {
                        self.consume_next_code_point();
                        self.consume_next_code_point();
                        return Ok(Token::Cdc);
                    }

                    return Ok(Token::Delim { value: code_point });
                }
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
                definition!(digit) => {
                    self.reconsume_current_input_code_point();
                    self.consume_a_numeric_token()
                }
                definition!(ident_start_code_point) => {
                    self.reconsume_current_input_code_point();
                    Ok(self.consume_a_ident_like_token())
                }
                _ => Ok(Token::Delim { value: code_point }),
            },
            definition!(eof_code_point) => Ok(Token::EndOfFile),
        }
    }

    fn consume_comments(&mut self) -> Result<(), CssParsingError> {
        loop {
            let (first, second) = match self.next_two_input_code_points() {
                Ok(pair) => pair,
                Err(_) => break,
            };

            if !(first == '/' && second == '*') {
                break;
            }

            self.consume_next_code_point();
            self.consume_next_code_point();

            loop {
                let (inner_first, inner_second) = self.next_two_input_code_points()?;

                if inner_first == '*' && inner_second == '/' {
                    self.consume_next_code_point();
                    self.consume_next_code_point();
                    break;
                }

                self.consume_next_code_point();
            }
        }

        Ok(())
    }

    // https://www.w3.org/TR/css-syntax-3/#consume-numeric-token
    fn consume_a_numeric_token(&mut self) -> Result<Token, CssParsingError> {
        // Consume a number and let number be the result.
        let number = self.consume_a_number()?;

        // If the next 3 input code points would start an ident sequence, then:
        let (first, second, third) = self.next_three_input_code_points()?;
        if self.would_start_an_ident_sequence(first, second, third) {
            // 1. Create a <dimension-token> with the same value and type flag as number, and a unit set initially to the empty string.
            let dimension_token = Token::Dimension {
                value: number.value,
                number_type: number.number_type,
                // 2. Consume an ident sequence. Set the <dimension-token>’s unit to the returned value.
                unit: self.consume_an_ident_sequence(),
            };

            // 3. Return the <dimension-token>.
            return Ok(dimension_token);
        }

        if self
            .next_input_code_point()
            .is_some_and(|code_point| code_point == '%')
        {
            self.consume_next_code_point();
            return Ok(Token::Percentage {
                value: number.value,
            });
        }

        Ok(Token::Number {
            value: number.value,
            number_type: number.number_type,
        })
    }

    fn consume_a_ident_like_token(&mut self) -> Token {
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

    fn stream_starts_with_a_number(&self) -> Result<bool, CssParsingError> {
        let (first, second, third) = self.next_three_input_code_points()?;

        match first {
            '+' | '-' => match second {
                definition!(digit) => Ok(true),
                '.' => Ok(definition!(digit).contains(&third)),
                _ => Ok(false),
            },
            '.' => Ok(definition!(digit).contains(&second)),
            definition!(digit) => Ok(true),
            _ => Ok(false),
        }
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

    fn consume_a_number(&mut self) -> Result<CssNumber, CssParsingError> {
        // 1. Initially set type to "integer". Let repr be the empty string.
        let mut number_type = NumberType::Integer;
        let mut repr = "".to_string();

        macro_rules! consume_and_append_to_repr {
            () => {
                if let Some(code_point) = self.consume_next_code_point() {
                    repr.push(code_point);
                }
            };
        }

        macro_rules! next_codepoint_is_digit {
            () => {
                self.next_input_code_point()
                    .is_some_and(|code_point| definition!(digit).contains(&code_point))
            };
        }

        // 2. If the next input code point is U+002B PLUS SIGN (+) or U+002D HYPHEN-MINUS (-), consume it and append it to repr.
        if self
            .next_input_code_point()
            .is_some_and(|code_point| code_point == '+' || code_point == '-')
        {
            consume_and_append_to_repr!();
        }

        // 3. While the next input code point is a digit, consume it and append it to repr.
        while next_codepoint_is_digit!() {
            consume_and_append_to_repr!();
        }

        // 4. If the next 2 input code points are U+002E FULL STOP (.) followed by a digit, then:
        let (first, second) = self.next_two_input_code_points()?;
        if first == '.' && definition!(digit).contains(&second) {
            // 1. Consume them.
            // 2. Append them to repr.
            consume_and_append_to_repr!();
            consume_and_append_to_repr!();

            // 3. Set type to "number".
            number_type = NumberType::Number;

            // 4. While the next input code point is a digit, consume it and append it to repr.
            while next_codepoint_is_digit!() {
                consume_and_append_to_repr!();
            }
        }

        // 5. If the next 2 or 3 input code points are U+0045 LATIN CAPITAL LETTER E (E) or U+0065 LATIN SMALL LETTER E (e), optionally followed by U+002D HYPHEN-MINUS (-) or U+002B PLUS SIGN (+), followed by a digit, then:
        let (first, second, third) = self.next_three_input_code_points()?;
        if first == 'E' || first == 'e' {
            macro_rules! handle_digit {
                ($amount:literal) => {
                    // 1. Consume them.
                    // 2. Append them to repr.
                    for _ in 0..$amount {
                        consume_and_append_to_repr!();
                    }

                    // 3. Set type to "number".
                    number_type = NumberType::Number;

                    // 4. While the next input code point is a digit, consume it and append it to repr.
                    while next_codepoint_is_digit!() {
                        consume_and_append_to_repr!();
                    }
                };
            }

            if second == '-' || second == '+' {
                if definition!(digit).contains(&third) {
                    handle_digit!(3);
                }
            } else if definition!(digit).contains(&second) {
                handle_digit!(2);
            }
        }

        // 6. Convert repr to a number, and set the value to the returned value.
        let value = repr.parse::<f32>().unwrap_or_else(|_| {
            panic!(
                "Failed to parse {} as a float. We should have already checked that it was a valid number.",
                repr
            )
        });

        Ok(CssNumber { value, number_type })
    }

    fn consume_as_much_whitespace_as_possible(&mut self) {
        while self
            .next_input_code_point()
            .is_some_and(|code_point| code_point.is_whitespace())
        {
            self.consume_next_code_point();
        }
    }

    // https://www.w3.org/TR/css-syntax-3/#check-if-three-code-points-would-start-an-ident-sequence
    fn would_start_an_ident_sequence(&self, first: char, second: char, _third: char) -> bool {
        match first {
            '-' => {
                match second {
                    definition!(ident_start_code_point) | '-' => true,
                    '\\' => {
                        // FIXME: this should check for the second and third code point and check if it is a valid escape sequence
                        true
                    }
                    _ => false,
                }
            }
            definition!(ident_start_code_point) => true,
            '\\' => {
                // FIXME: this should check for the second and third code point and check if it is a valid escape sequence
                true
            }
            _ => false,
        }
    }

    fn next_two_input_code_points(&self) -> Result<(char, char), CssParsingError> {
        match (self.peek(1), self.peek(2)) {
            (Some(first), Some(second)) => Ok((first, second)),
            _ => Err(CssParsingError::InvalidEndOfFile),
        }
    }

    fn next_three_input_code_points(&self) -> Result<(char, char, char), CssParsingError> {
        match (self.peek(1), self.peek(2), self.peek(3)) {
            (Some(first), Some(second), Some(third)) => Ok((first, second, third)),
            _ => Err(CssParsingError::InvalidEndOfFile),
        }
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.input.chars().nth(self.position + offset)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CssNumber {
    value: f32,
    number_type: NumberType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CssParsingError {
    InvalidEndOfFile,
}
