#[allow(unused)]
#[derive(Debug, Copy, Clone)]
enum State {
    Data,
    RcData,
    RawText,
    ScriptData,
    PlainText,
    TagOpen,
    EndTagOpen,
    TagName,
    RcDataLessThanSign,
    RcDataEndTagOpen,
    RcDataEndTagName,
    RawTextLessThanSign,
    RawTextEndTagOpen,
    RawTextEndTagName,
    ScriptDataLessThanSign,
    ScriptDataEndTagOpen,
    ScriptDataEndTagName,
    ScriptDataEscapeStart,
    ScriptDataEscapeStartDash,
    ScriptDataEscaped,
    ScriptDataEscapedDash,
    ScriptDataEscapedDashDash,
    ScriptDataEscapedLessThanSign,
    ScriptDataEscapedEndTagOpen,
    ScriptDataEscapedEndTagName,
    ScriptDataDoubleEscapeStart,
    ScriptDataDoubleEscaped,
    ScriptDataDoubleEscapedDash,
    ScriptDataDoubleEscapedDashDash,
    ScriptDataDoubleEscapedLessThanSign,
    ScriptDataDoubleEscapeEnd,
    BeforeAttributeName,
    AttributeName,
    AfterAttributeName,
    BeforeAttributeValue,
    AttributeValueDoubleQuoted,
    AttributeValueSingleQuoted,
    AttributeValueUnquoted,
    AfterAttributeValueQuoted,
    SelfClosingStartTag,
    BogusComment,
    MarkupDeclarationOpen,
    CommentStart,
    CommentStartDash,
    Comment,
    CommentLessThanSign,
    CommentLessThanSignBang,
    CommentLessThanSignBangDash,
    CommentLessThanSignBangDashDash,
    CommentEndDash,
    CommentEnd,
    CommentEndBang,
    Doctype,
    BeforeDoctypeName,
    DoctypeName,
    AfterDoctypeName,
    AfterDoctypePublicKeyword,
    BeforeDoctypePublicIdentifier,
    DoctypePublicIdentifierDoubleQuoted,
    DoctypePublicIdentifierSingleQuoted,
    AfterDoctypePublicIdentifier,
    BetweenDoctypePublicAndSystemIdentifiers,
    AfterDoctypeSystemKeyword,
    BeforeDoctypeSystemIdentifier,
    DoctypeSystemIdentifierDoubleQuoted,
    DoctypeSystemIdentifierSingleQuoted,
    AfterDoctypeSystemIdentifier,
    BogusDoctype,
    CDataSection,
    CDataSectionBracket,
    CDataSectionEnd,
    CharacterReference,
    NamedCharacterReference,
    AmbiguousAmpersand,
    NumericCharacterReference,
    HexadecimalCharacterReferenceStart,
    DecimalCharacterReferenceStart,
    HexadecimalCharacterReference,
    DecimalCharacterReference,
    NumericCharacterReferenceEnd,
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Doctype {
        name: Option<String>,
        public_identifier: Option<String>,
        system_identifier: Option<String>,
        force_quirks: bool,
    },
    StartTag {
        name: String,
        self_closing: bool,
        attributes: Vec<Attribute>,
    },
    EndTag {
        name: String,
        self_closing: bool,
        attributes: Vec<Attribute>,
    },
    Comment {
        data: String,
    },
    Character {
        data: String,
    },
    EndOfFile,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    name: String,
    value: String,
}

#[derive(Debug)]
pub struct Tokenizer {
    state: State,
    tokens: Vec<Token>,
    input: String,
    insertion_point: Option<usize>,
    current_input_character: Option<char>,
    eof_emitted: bool,
    current_builder: String,
}

impl Tokenizer {
    pub fn new(input: &str) -> Self {
        Self {
            state: State::Data,
            tokens: Vec::new(),
            input: String::from(input),
            insertion_point: None,
            current_input_character: None,
            eof_emitted: false,
            current_builder: String::new(),
        }
    }

    fn next_input_character(&mut self) -> Option<char> {
        if self.insertion_point.is_none() {
            self.insertion_point = Some(0);
        }

        if self.insertion_point > Some(self.input.len()) {
            return None;
        }

        if let Some(insertion_point) = self.insertion_point {
            return self.input.chars().nth(insertion_point);
        }
        None
    }

    fn consume_next_input_character(&mut self) {
        self.current_input_character = self.next_input_character();
        if let Some(insertion_point) = self.insertion_point {
            self.insertion_point = Some(insertion_point + 1);
        }
    }

    fn consume_characters(&mut self, characters: &str) {
        if let Some(insertion_point) = self.insertion_point {
            self.insertion_point = Some(insertion_point + characters.len());
        }
    }

    fn next_characters_are_ascii_case_insensitive(&self, chars: &str) -> bool {
        if let Some(insertion_point) = self.insertion_point {
            let next_characters = &self.input[insertion_point..(insertion_point + chars.len())];
            return chars.eq_ignore_ascii_case(next_characters);
        }
        false
    }

    fn emit(&mut self, token: &Token) {
        self.tokens.push(token.clone());

        match token {
            Token::EndOfFile => self.eof_emitted = true,
            _ => {}
        }
    }

    fn consume_current_builder(&mut self) -> String {
        let value = self.current_builder.clone();
        self.current_builder = String::new();
        value
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        macro_rules! whitespace {
            () => {
                Some('\t') | // Tab
                Some('\n') | // Line Feed
                Some('\u{000c}') | // Form Feed
                Some(' ') // Space
            };
        }

        macro_rules! eof {
            () => {
                None
            };
        }

        macro_rules! null {
            () => {
                Some('\u{0000}')
            };
        }

        macro_rules! anything_else {
            () => {
                _
            };
        }

        loop {
            if self.eof_emitted {
                break;
            }

            match self.state {
                // https://html.spec.whatwg.org/multipage/parsing.html#data-state
                State::Data => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        Some('&') => todo!(),
                        Some('<') => self.state = State::TagOpen,
                        null!() => todo!(),
                        eof!() => self.emit(&Token::EndOfFile),
                        anything_else!() => {
                            let data = match self.current_input_character {
                                  Some(character) => String::from(character),
                                  None => panic!("Current input character not found when creating Token::Character"),  
                            };

                            self.emit(&Token::Character { data });
                        }
                    }
                }
                // https://html.spec.whatwg.org/multipage/parsing.html#tag-open-state
                State::TagOpen => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        Some('!') => {
                            self.state = State::MarkupDeclarationOpen;
                            continue;
                        }
                        _ => todo!(),
                    }
                }
                // https://html.spec.whatwg.org/multipage/parsing.html#markup-declaration-open-state
                State::MarkupDeclarationOpen => {
                    if self.next_characters_are_ascii_case_insensitive("DOCTYPE") {
                        self.consume_characters("DOCTYPE");
                        self.state = State::Doctype;
                        continue;
                    }
                    todo!()
                }
                // https://html.spec.whatwg.org/multipage/parsing.html#doctype-state
                State::Doctype => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        whitespace!() => {
                            self.state = State::BeforeDoctypeName;
                            continue;
                        }
                        _ => {
                            todo!()
                        }
                    }
                }
                // https://html.spec.whatwg.org/multipage/parsing.html#doctype-state
                State::BeforeDoctypeName => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        whitespace!() => {
                            // Ignore the character.
                            continue;
                        }
                        // FIXME: Implement ASCII upper alpha
                        // FIXME: Implement NULL
                        // FIXME: Implement >
                        eof!() => {
                            // This is an eof-in-doctype parse error. Create a new DOCTYPE token. Set its force-quirks flag to on. Emit the current token. Emit an end-of-file token.
                            self.emit(&Token::Doctype {
                                name: None,
                                public_identifier: None,
                                system_identifier: None,
                                force_quirks: true,
                            });
                            self.emit(&Token::EndOfFile);
                        }
                        _ => {
                            // Create a new DOCTYPE token. Set the token's name to the current input character. Switch to the DOCTYPE name state.
                            let name = match self.current_input_character {
                                Some(character) => String::from(character),
                                None => String::new(),
                            };
                            self.current_builder = name;
                            self.state = State::DoctypeName;
                        }
                    }
                }
                // https://html.spec.whatwg.org/multipage/parsing.html#doctype-name-state
                State::DoctypeName => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        whitespace!() => {
                            self.state = State::AfterDoctypeName;
                            continue;
                        }
                        Some('>') => {
                            self.state = State::Data;
                            let name = Some(self.consume_current_builder());
                            self.emit(&Token::Doctype {
                                name,
                                public_identifier: None,
                                system_identifier: None,
                                force_quirks: false,
                            });
                            continue;
                        }
                        // FIXME: Implement ASCII upper alpha
                        // FIXME: Implement NULL
                        // FIXME: Implement >
                        eof!() => todo!(),
                        // Anything else
                        Some(current_input_character) => {
                            self.current_builder.push(current_input_character);
                            continue;
                        }
                    }
                }
                // Unimplemented state.
                _ => todo!("state: {:?}", self.state),
            }
        }

        self.tokens.clone()
    }
}

mod tests {
    #[test]
    fn tokenize() {
        use crate::{Token, Tokenizer};

        let html = include_str!("test.html");
        let mut tokenizer = Tokenizer::new(html);
        let tokens = tokenizer.tokenize();
        assert_eq!(
            tokens,
            vec![
                Token::Doctype {
                    name: Some(String::from("html")),
                    public_identifier: None,
                    system_identifier: None,
                    force_quirks: false
                },
                Token::EndOfFile
            ]
        )
    }
}
