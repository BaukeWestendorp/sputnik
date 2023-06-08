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

    // https://www.w3.org/TR/css-syntax-3/#next-input-code-point
    fn next_input_code_point(&self) -> Option<char> {
        self.peek(1)
    }

    fn consume_next_input_code_point(&mut self) -> Option<char> {
        self.position += 1;
        self.current_input_code_point()
    }

    // https://www.w3.org/TR/css-syntax-3/#current-input-code-point
    fn current_input_code_point(&self) -> Option<char> {
        self.peek(0)
    }

    // https://www.w3.org/TR/css-syntax-3/#reconsume-the-current-input-code-point
    fn reconsume_current_input_code_point(&mut self) {
        self.position -= 1;
    }

    // https://www.w3.org/TR/css-syntax-3/#consume-token
    fn consume_a_token(&mut self) -> Result<Token, CssParsingError> {
        // Consume comments.
        self.consume_comments()?;

        // Consume the next input code point.
        let code_point = self.consume_next_input_code_point();

        match code_point {
            Some(code_point) => match code_point {
                definition!(whitespace) => {
                    // Consume as much whitespace as possible.
                    self.consume_as_much_whitespace_as_possible();

                    // Return a <whitespace-token>.
                    Ok(Token::Whitespace)
                }
                '"' => todo!(),
                '#' => todo!(),
                '\'' => todo!(),
                '(' => Ok(Token::LeftParenthesis),
                ')' => Ok(Token::RightParenthesis),
                '+' => {
                    // If the input stream starts with a number,
                    if self.stream_starts_with_a_number() {
                        // reconsume the current input code point,
                        self.reconsume_current_input_code_point();
                        // consume a numeric token, and return it.
                        return self.consume_a_numeric_token();
                    }

                    // Otherwise, return a <delim-token> with its value set to the current input code point.
                    return Ok(Token::Delim { value: code_point });
                }
                ',' => Ok(Token::Comma),
                '-' => {
                    // If the input stream starts with a number,
                    if self.stream_starts_with_a_number() {
                        // reconsume the current input code point,
                        self.reconsume_current_input_code_point();
                        // consume a numeric token, and return it.
                        return self.consume_a_numeric_token();
                    }

                    // Otherwise, if the next 2 input code points are U+002D HYPHEN-MINUS U+003E GREATER-THAN SIGN (->),
                    if let ('-', '>') = self.next_two_input_code_points()? {
                        // consume them and return a <CDC-token>.
                        self.consume_next_input_code_point();
                        self.consume_next_input_code_point();
                        return Ok(Token::Cdc);
                    }

                    // Otherwise, if the input stream starts with an ident sequence,
                    if self.stream_starts_with_an_ident_sequence() {
                        // reconsume the current input code point,
                        self.reconsume_current_input_code_point();
                        // consume an ident-like token, and return it.
                        return Ok(self.consume_an_ident_like_token());
                    }

                    // Otherwise, return a <delim-token> with its value set to the current input code point.
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
                    // Reconsume the current input code point,
                    self.reconsume_current_input_code_point();
                    // consume a numeric token, and return it.
                    self.consume_a_numeric_token()
                }
                definition!(ident_start_code_point) => {
                    // Reconsume the current input code point,
                    self.reconsume_current_input_code_point();
                    // consume a numeric token, and return it.
                    Ok(self.consume_an_ident_like_token())
                }
                _ => Ok(Token::Delim { value: code_point }),
            },
            definition!(eof_code_point) => Ok(Token::EndOfFile),
        }
    }

    // https://www.w3.org/TR/css-syntax-3/#consume-comment
    fn consume_comments(&mut self) -> Result<(), CssParsingError> {
        loop {
            let (first, second) = match self.next_two_input_code_points() {
                Ok(pair) => pair,
                Err(_) => break,
            };

            if !(first == '/' && second == '*') {
                break;
            }
            // If the next two input code point are U+002F SOLIDUS (/) followed by a U+002A ASTERISK (*),

            // consume them and all following code points
            self.consume_next_input_code_point();
            self.consume_next_input_code_point();
            loop {
                // up to and including the first U+002A ASTERISK (*) followed by a U+002F SOLIDUS (/),
                // or up to an EOF code point.
                let (inner_first, inner_second) = self.next_two_input_code_points()?;
                if inner_first == '*' && inner_second == '/' {
                    self.consume_next_input_code_point();
                    self.consume_next_input_code_point();
                    break;
                }
                self.consume_next_input_code_point();
                // Return to the start of this step.
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
        if self.check_if_three_code_points_would_start_an_ident_sequence(first, second, third) {
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

        // Otherwise, if the next input code point is U+0025 PERCENTAGE SIGN (%),
        if self
            .next_input_code_point()
            .is_some_and(|code_point| code_point == '%')
        {
            // consume it.
            self.consume_next_input_code_point();
            // Create a <percentage-token> with the same value as number, and return it.
            return Ok(Token::Percentage {
                value: number.value,
            });
        }

        // Otherwise, create a <number-token> with the same value and type flag as number, and return it.
        Ok(Token::Number {
            value: number.value,
            number_type: number.number_type,
        })
    }

    fn consume_an_ident_like_token(&mut self) -> Token {
        // Consume an ident sequence, and let string be the result.
        let string = self.consume_an_ident_sequence();

        // FIXME: Implement
        // If string’s value is an ASCII case-insensitive match for "url",
        // and the next input code point is U+0028 LEFT PARENTHESIS ((),
        // consume it.
        // While the next two input code points are whitespace,
        // consume the next input code point.
        // If the next one or two input code points are U+0022 QUOTATION MARK ("), U+0027 APOSTROPHE ('),
        // or whitespace followed by U+0022 QUOTATION MARK (") or U+0027 APOSTROPHE ('),
        // then create a <function-token> with its value set to string and return it.
        // Otherwise, consume a url token, and return it.
        if string.eq_ignore_ascii_case("url") {
            todo!()
        }

        // Otherwise, if the next input code point is U+0028 LEFT PARENTHESIS ((),
        if self
            .next_input_code_point()
            .is_some_and(|code_point| code_point == '(')
        {
            // consume it.
            self.consume_next_input_code_point();
            // Create a <function-token> with its value set to string and return it.
            return Token::Function { value: string };
        }

        // Otherwise, create an <ident-token> with its value set to string and return it.
        Token::Ident { value: string }
    }

    // https://www.w3.org/TR/css-syntax-3/#check-if-three-code-points-would-start-an-ident-sequence
    fn check_if_three_code_points_would_start_an_ident_sequence(
        &self,
        first: char,
        second: char,
        _third: char,
    ) -> bool {
        // Look at the first code point:
        match first {
            '-' => {
                // If the second code point is an ident-start code point or a U+002D HYPHEN-MINUS,
                match second {
                    definition!(ident_start_code_point) | '-' => true,
                    // FIXME: or the second and third code points are a valid escape, return true. Otherwise, return false.
                    '\\' => true,
                    _ => false,
                }
            }
            definition!(ident_start_code_point) => true,
            '\\' => {
                // FIXME: If the first and second code points are a valid escape, return true. Otherwise, return false.
                true
            }
            _ => false,
        }
    }

    fn stream_starts_with_an_ident_sequence(&self) -> bool {
        // the three code points in question are
        // the current input code point and
        // the next two input code points,
        // in that order.
        let first = match self.current_input_code_point() {
            Some(first) => first,
            None => return false,
        };
        let (second, third) = match self.next_two_input_code_points() {
            Ok(pair) => pair,
            Err(_) => return false,
        };

        self.check_if_three_code_points_would_start_an_ident_sequence(first, second, third)
    }

    // https://www.w3.org/TR/css-syntax-3/#starts-with-a-number
    fn check_if_three_code_points_would_start_a_number(
        &self,
        first: char,
        second: char,
        third: char,
    ) -> bool {
        // Look at the first code point:
        match first {
            '+' | '-' => match second {
                // If the second code point is a digit, return true.
                definition!(digit) => true,
                // Otherwise, if the second code point is a U+002E FULL STOP (.)
                // and the third code point is a digit, return true.
                '.' => definition!(digit).contains(&third),
                _ => false,
            },
            '.' => {
                // If the second code point is a digit, return true.
                // Otherwise, return false.
                definition!(digit).contains(&second)
            }
            definition!(digit) => true,
            _ => false,
        }
    }

    fn stream_starts_with_a_number(&self) -> bool {
        // the three code points in question are
        // the current input code point and
        // the next two input code points,
        // in that order.
        let first = match self.current_input_code_point() {
            Some(first) => first,
            None => return false,
        };
        let (second, third) = match self.next_two_input_code_points() {
            Ok(pair) => pair,
            Err(_) => return false,
        };

        self.check_if_three_code_points_would_start_a_number(first, second, third)
    }

    // https://www.w3.org/TR/css-syntax-3/#consume-name
    fn consume_an_ident_sequence(&mut self) -> String {
        // Let result initially be an empty string.
        let mut result = "".to_string();

        // Repeatedly consume the next input code point from the stream:
        loop {
            if let Some(input) = self.consume_next_input_code_point() {
                match input {
                    definition!(ident_code_point) => {
                        // Append the code point to result.
                        result.push(input);
                    }
                    '\\' => todo!(), // FIXME: This should use a seperate function to check if it really is an escape function.
                    _ => {
                        // Reconsume the current input code point.
                        self.reconsume_current_input_code_point();
                        // Return result.
                        break;
                    }
                }
            } else {
                break;
            }
        }
        result
    }

    // https://www.w3.org/TR/css-syntax-3/#consume-number
    fn consume_a_number(&mut self) -> Result<CssNumber, CssParsingError> {
        // 1. Initially set type to "integer". Let repr be the empty string.
        let mut number_type = NumberType::Integer;
        let mut repr = "".to_string();

        macro_rules! consume_and_append_to_repr {
            () => {
                if let Some(code_point) = self.consume_next_input_code_point() {
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

        // 5. If the next 2 or 3 input code points are
        let (first, second, third) = self.next_three_input_code_points()?;
        // U+0045 LATIN CAPITAL LETTER E (E) or U+0065 LATIN SMALL LETTER E (e),
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

            // optionally followed by U+002D HYPHEN-MINUS (-) or U+002B PLUS SIGN (+),
            // followed by a digit, then:
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

        // Return value and type.
        Ok(CssNumber { value, number_type })
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

    fn consume_as_much_whitespace_as_possible(&mut self) {
        while self
            .next_input_code_point()
            .is_some_and(|code_point| code_point.is_whitespace())
        {
            self.consume_next_input_code_point();
        }
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
