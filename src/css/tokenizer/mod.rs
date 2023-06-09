#![allow(clippy::manual_is_ascii_check)]

pub use token::{HashType, NumberType, Token};

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

macro_rules! log_parse_error {
    ($message:expr) => {
        eprintln!(
            "\x1b[31m[Parser Error ({}:{})]: {}\x1b[0m",
            file!(),
            line!(),
            $message
        );
    };
    () => {
        eprintln!("\x1b[31m[Parser Error ({}:{})]\x1b[0m", file!(), line!());
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tokenizer<'a> {
    input: &'a str,
    position: isize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            position: -1,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = vec![];

        loop {
            let token = self.consume_a_token();
            log_current_token!(token);
            match token {
                Token::EndOfFile => {
                    tokens.push(token);
                    break;
                }
                _ => tokens.push(token),
            }
        }

        tokens
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
    fn consume_a_token(&mut self) -> Token {
        // Consume comments.
        self.consume_comments();

        // Consume the next input code point.
        let code_point = self.consume_next_input_code_point();

        match code_point {
            Some(code_point) => match code_point {
                definition!(whitespace) => {
                    // Consume as much whitespace as possible.
                    self.consume_as_much_whitespace_as_possible();

                    // Return a <whitespace-token>.
                    Token::Whitespace
                }
                '"' => {
                    // Consume a string token and return it.
                    self.consume_a_string_token(None)
                }
                '#' => {
                    // If the next input code point is an ident code point
                    // FIXME: or the next two input code points are a valid escape, then:
                    if self
                        .next_input_code_point()
                        .map_or(false, |c| matches!(c, definition!(ident_code_point)))
                    {
                        let (first, second, third) = self
                            .next_three_input_code_points()
                            .expect("FIXME: Implement invalid EOF case.");

                        // 1. Create a <hash-token>.
                        let hash_token = Token::Hash {
                            // 2. If the next 3 input code points would start an ident sequence, set the <hash-token>’s type flag to "id".
                            hash_type: match self
                                .check_if_three_code_points_would_start_an_ident_sequence(
                                    first, second, third,
                                ) {
                                true => HashType::Id,
                                false => HashType::Unrestricted,
                            },
                            // 3. Consume an ident sequence, and set the <hash-token>’s value to the returned string.
                            value: self.consume_an_ident_sequence(),
                        };

                        // 4. Return the <hash-token>.
                        return hash_token;
                    }

                    // Otherwise, return a <delim-token> with its value set to the current input code point.
                    Token::Delim { value: code_point }
                }
                '\'' => {
                    // Consume a string token and return it.
                    self.consume_a_string_token(None)
                }
                '(' => Token::LeftParenthesis,
                ')' => Token::RightParenthesis,
                '+' => {
                    // If the input stream starts with a number,
                    if self.stream_starts_with_a_number() {
                        // reconsume the current input code point,
                        self.reconsume_current_input_code_point();
                        // consume a numeric token, and return it.
                        return self.consume_a_numeric_token();
                    }

                    // Otherwise, return a <delim-token> with its value set to the current input code point.
                    Token::Delim { value: code_point }
                }
                ',' => Token::Comma,
                '-' => {
                    // If the input stream starts with a number,
                    if self.stream_starts_with_a_number() {
                        // reconsume the current input code point,
                        self.reconsume_current_input_code_point();
                        // consume a numeric token, and return it.
                        return self.consume_a_numeric_token();
                    }

                    // Otherwise, if the next 2 input code points are U+002D HYPHEN-MINUS U+003E GREATER-THAN SIGN (->),
                    if let ('-', '>') = self
                        .next_two_input_code_points()
                        .expect("FIXME: Implement invalid EOF case.")
                    {
                        // consume them and return a <CDC-token>.
                        self.consume_next_input_code_point();
                        self.consume_next_input_code_point();
                        return Token::Cdc;
                    }

                    // Otherwise, if the input stream starts with an ident sequence,
                    if self.stream_starts_with_an_ident_sequence() {
                        // reconsume the current input code point,
                        self.reconsume_current_input_code_point();
                        // consume an ident-like token, and return it.
                        return self.consume_an_ident_like_token();
                    }

                    // Otherwise, return a <delim-token> with its value set to the current input code point.
                    Token::Delim { value: code_point }
                }
                '.' => {
                    // If the input stream starts with a number,
                    if self.stream_starts_with_a_number() {
                        // reconsume the current input code point,
                        self.reconsume_current_input_code_point();
                        // consume a numeric token, and return it.
                        return self.consume_a_numeric_token();
                    }

                    // Otherwise, return a <delim-token> with its value set to the current input code point.
                    Token::Delim { value: code_point }
                }
                ':' => Token::Colon,
                ';' => Token::Semicolon,
                '<' => todo!(),
                '@' => {
                    // If the next 3 input code points would start an ident sequence,
                    let (first, second, third) = self
                        .next_three_input_code_points()
                        .expect("FIXME: Implement invalid EOF case.");
                    if self.check_if_three_code_points_would_start_an_ident_sequence(
                        first, second, third,
                    ) {
                        // consume an ident sequence,
                        // create an <at-keyword-token> with its value set to the returned value, and return it.
                        return Token::AtKeyword {
                            value: self.consume_an_ident_sequence(),
                        };
                    }

                    // Otherwise, return a <delim-token> with its value set to the current input code point.
                    Token::Delim { value: code_point }
                }
                '[' => Token::LeftSquareBracket,
                '\\' => todo!(),
                ']' => Token::RightSquareBracket,
                '{' => Token::LeftCurlyBracket,
                '}' => Token::RightCurlyBracket,
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
                    self.consume_an_ident_like_token()
                }
                _ => Token::Delim { value: code_point },
            },
            definition!(eof_code_point) => Token::EndOfFile,
        }
    }

    // https://www.w3.org/TR/css-syntax-3/#consume-comment
    fn consume_comments(&mut self) {
        while let Some((first, second)) = self.next_two_input_code_points() {
            // If the next two input code point are U+002F SOLIDUS (/) followed by a U+002A ASTERISK (*),
            if !(first == '/' && second == '*') {
                break;
                // It returns nothing.
            }

            // consume them and all following code points
            self.consume_next_input_code_point();
            self.consume_next_input_code_point();
            loop {
                // up to and including the first U+002A ASTERISK (*) followed by a U+002F SOLIDUS (/),
                // or up to an EOF code point.
                let (inner_first, inner_second) = self
                    .next_two_input_code_points()
                    .expect("FIXME: Implement invalid EOF case.");
                if inner_first == '*' && inner_second == '/' {
                    self.consume_next_input_code_point();
                    self.consume_next_input_code_point();
                    break;
                }
                self.consume_next_input_code_point();
                // Return to the start of this step.
            }
        }
    }

    // https://www.w3.org/TR/css-syntax-3/#consume-numeric-token
    fn consume_a_numeric_token(&mut self) -> Token {
        // Consume a number and let number be the result.
        let number = self.consume_a_number();

        // If the next 3 input code points would start an ident sequence, then:
        let (first, second, third) = self
            .next_three_input_code_points()
            .expect("FIXME: Implement invalid EOF case.");
        if self.check_if_three_code_points_would_start_an_ident_sequence(first, second, third) {
            // 1. Create a <dimension-token> with the same value and type flag as number, and a unit set initially to the empty string.
            let dimension_token = Token::Dimension {
                value: number.value,
                number_type: number.number_type,
                // 2. Consume an ident sequence. Set the <dimension-token>’s unit to the returned value.
                unit: self.consume_an_ident_sequence(),
            };

            // 3. Return the <dimension-token>.
            return dimension_token;
        }

        // Otherwise, if the next input code point is U+0025 PERCENTAGE SIGN (%),
        if self
            .next_input_code_point()
            .is_some_and(|code_point| code_point == '%')
        {
            // consume it.
            self.consume_next_input_code_point();
            // Create a <percentage-token> with the same value as number, and return it.
            return Token::Percentage {
                value: number.value,
            };
        }

        // Otherwise, create a <number-token> with the same value and type flag as number, and return it.
        Token::Number {
            value: number.value,
            number_type: number.number_type,
        }
    }

    // https://www.w3.org/TR/css-syntax-3/#consume-ident-like-token
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

    // https://www.w3.org/TR/css-syntax-3/#consume-string-token
    fn consume_a_string_token(&mut self, ending_code_point: Option<char>) -> Token {
        // This algorithm may be called with an ending code point,
        // which denotes the code point that ends the string.
        // If an ending code point is not specified,
        // the current input code point is used.
        let ending_code_point = match ending_code_point {
            Some(ending_code_point) => Some(ending_code_point),
            None => self.current_input_code_point(),
        };

        // Initially create a <string-token> with its value set to the empty string.
        let mut string_token = Token::String {
            value: "".to_string(),
        };

        loop {
            // Consume the next input code point.
            let code_point = self.consume_next_input_code_point();

            match code_point {
                Some(code_point) if Some(code_point) == ending_code_point => {
                    // Return the <string-token>.
                    return string_token;
                }
                None => {
                    // This is a parse error.
                    log_parse_error!("EOF in string token");
                    // Return the <string-token>.
                    return string_token;
                }
                Some('\n') => {
                    // This is a parse error.
                    log_parse_error!("newline in string token");
                    // Reconsume the current input code point,
                    self.reconsume_current_input_code_point();
                    // create a <bad-string-token>, and return it.
                    return Token::BadString;
                }
                Some('\\') => {
                    // FIXME: If the next input code point is EOF, do nothing.
                    // FIXME: Otherwise, if the next input code point is a newline, consume it.
                    // FIXME: Otherwise, (the stream starts with a valid escape) consume an escaped code point and append the returned code point to the <string-token>’s value.
                    todo!()
                }
                _ => {
                    // Append the current input code point to the <string-token>’s value.
                    if let Token::String { value } = &mut string_token {
                        value.push(
                            code_point.expect("We have already checked for None in the EOF case"),
                        );
                    }
                }
            }
        }
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
            Some(pair) => pair,
            None => return false,
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
            Some(pair) => pair,
            None => return false,
        };

        self.check_if_three_code_points_would_start_a_number(first, second, third)
    }

    // https://www.w3.org/TR/css-syntax-3/#consume-name
    fn consume_an_ident_sequence(&mut self) -> String {
        // Let result initially be an empty string.
        let mut result = "".to_string();

        // Repeatedly consume the next input code point from the stream:
        while let Some(input) = self.consume_next_input_code_point() {
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
        }
        result
    }

    // https://www.w3.org/TR/css-syntax-3/#consume-number
    fn consume_a_number(&mut self) -> CssNumber {
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
        let (first, second) = self
            .next_two_input_code_points()
            .expect("FIXME: Implement invalid EOF case.");
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
        let (first, second, third) = self
            .next_three_input_code_points()
            .expect("FIXME: Implement invalid EOF case.");
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
        CssNumber { value, number_type }
    }

    fn next_two_input_code_points(&self) -> Option<(char, char)> {
        match (self.peek(1), self.peek(2)) {
            (Some(first), Some(second)) => Some((first, second)),
            _ => None,
        }
    }

    fn next_three_input_code_points(&self) -> Option<(char, char, char)> {
        match (self.peek(1), self.peek(2), self.peek(3)) {
            (Some(first), Some(second), Some(third)) => Some((first, second, third)),
            _ => None,
        }
    }

    fn peek(&self, offset: isize) -> Option<char> {
        let n = self.position + offset;
        if n < 0 {
            return None;
        }
        self.input.chars().nth(n as usize)
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
