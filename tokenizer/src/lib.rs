use named_character_references::NamedCharacterReference;

use crate::named_character_references::NAMED_CHARACTER_REFERENCES;

mod named_character_references;

macro_rules! on {
    ($c:expr) => {
        Some($c)
    };
}

macro_rules! on_whitespace {
    () => {
        on!('\t') | // Tab
        on!('\n') | // Line Feed
        on!('\u{000c}') | // Form Feed
        on!(' ') // Space
    };
    ($c:ident) => {
        Some($c @ '\t') | // Tab
        Some($c @ '\n') | // Line Feed
        Some($c @ '\u{000c}') | // Form Feed
        Some($c @ ' ') // Space
    };
}

macro_rules! on_null {
    () => {
        on!('\u{0000}')
    };
}

macro_rules! on_anything_else {
    () => {
        Some(_)
    };
    ($c:ident) => {
        Some($c)
    };
}

macro_rules! on_eof {
    () => {
        None
    };
}

macro_rules! c0_control_codepoint {
    () => {
        0x0000..=0x001f
    };
}

macro_rules! control_codepoint {
    () => {
        c0_control_codepoint!() | 0x007f..=0x009f
    };
}

macro_rules! whitespace_codepoint {
    () => {
        0x0009 | // SPEC: TAB,
        0x000A | //       LF
        0x000C | //       FF
        0x000D | //       CR
        0x0020   //       SPACE.
    };
}

macro_rules! leading_surrogate_codepoint {
    () => {
        0xd800..=0xdbff
    };
}
macro_rules! trailing_surrogate_codepoint {
    () => {
        0xdc00..=0xdfff
    };
}
macro_rules! surrogate_codepoint {
    () => {
        leading_surrogate_codepoint!() | trailing_surrogate_codepoint!()
    };
}

macro_rules! on_ascii_digit {
    () => {
        Some('0'..='9')
    };
    ($c:ident) => {
        Some($c @ '0'..='9')
    };
}

macro_rules! on_ascii_upper_hex_digit_alpha {
    () => {
        Some('A'..='F')
    };
    ($c:ident) => {
        Some($c @ 'A'..='F')
    };
}

macro_rules! on_ascii_lower_hex_digit_alpha {
    () => {
        Some('a'..='f')
    };
    ($c:ident) => {
        Some($c @ 'a'..='f')
    };
}

macro_rules! on_ascii_hex_digit {
    () => {
        on_ascii_digit!() | on_ascii_lower_hex_digit_alpha!() | on_ascii_upper_hex_digit_alpha!()
    };
    ($c:ident) => {
        on_ascii_digit!($c)
            | on_ascii_lower_hex_digit_alpha!($c)
            | on_ascii_upper_hex_digit_alpha!($c)
    };
}

macro_rules! on_ascii_upper_alpha {
    () => {
        Some('A'..='Z')
    };
    ($c:ident) => {
        Some($c @ 'A'..='Z')
    };
}

macro_rules! on_ascii_lower_alpha {
    () => {
        Some('a'..='z')
    };
    ($c:ident) => {
        Some($c @ 'a'..='z')
    };
}

macro_rules! on_ascii_alpha {
    () => {
        on_ascii_upper_alpha!() | on_ascii_lower_alpha!()
    };
    ($c:ident) => {
        on_ascii_upper_alpha!($c) | on_ascii_lower_alpha!($c)
    };
}

macro_rules! ascii_alphanumeric {
    () => {
        on_ascii_digit!() | on_ascii_alpha!()
    };
    ($c:ident) => {
        on_ascii_digit!($c) | on_ascii_alpha!($c)
    };
}

#[rustfmt::skip]
macro_rules! noncharacter {
    () => {
        (0xFDD0..=0xFDEF)
        | 0xFFFE  | 0xFFFF  | 0x1FFFE | 0x1FFFF
        | 0x2FFFE | 0x2FFFF | 0x3FFFE | 0x3FFFF
        | 0x4FFFE | 0x4FFFF | 0x5FFFE | 0x5FFFF
        | 0x6FFFE | 0x6FFFF | 0x7FFFE | 0x7FFFF
        | 0x8FFFE | 0x8FFFF | 0x9FFFE | 0x9FFFF
        | 0xAFFFE | 0xAFFFF | 0xBFFFE | 0xBFFFF
        | 0xCFFFE | 0xCFFFF | 0xDFFFE | 0xDFFFF
        | 0xEFFFE | 0xEFFFF | 0xFFFFE | 0xFFFFF
        | 0x10FFFE| 0x10FFFF
    };
}

#[allow(unused)]
#[derive(Debug, Copy, Clone)]
pub enum State {
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
        self_closing_acknowledged: bool,
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
        data: char,
    },
    EndOfFile,
}

impl Token {
    pub fn acknowledge_self_closing_flag(&mut self) {
        if let Token::StartTag {
            self_closing_acknowledged,
            ..
        } = self
        {
            *self_closing_acknowledged = true;
        } else {
            panic!("Tried to acknowledge non-StarTag token!");
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

#[derive(Debug)]
pub struct Tokenizer {
    input: String,
    state: State,
    return_state: Option<State>,
    temporary_buffer: String,
    temporary_named_character_references_buffer: Vec<(String, NamedCharacterReference<'static>)>,
    tokens: Vec<Token>,
    insertion_point: Option<usize>,
    current_input_character: Option<char>,
    token_emitted: bool,
    current_token: Option<Token>,
    current_attribute: Option<Attribute>,
    character_reference_code: u32,
}

impl Tokenizer {
    pub fn new(input: &str) -> Self {
        Self {
            input: String::from(input),
            state: State::Data,
            return_state: None,
            temporary_buffer: String::new(),
            temporary_named_character_references_buffer: Vec::new(),
            tokens: Vec::new(),
            insertion_point: None,
            current_input_character: None,
            token_emitted: false,
            current_token: None,
            current_attribute: None,
            character_reference_code: 0,
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

    fn emit_token(&mut self, token: Token) {
        self.tokens.push(token.clone());
        self.token_emitted = true;
    }

    fn set_current_token(&mut self, token: Token) {
        self.current_token = Some(token);
    }

    fn push_current_attribute_to_current_tag(&mut self) {
        if let Some(current_attribute) = self.current_attribute.clone() {
            if let Some(Token::StartTag { attributes, .. }) = &mut self.current_token {
                attributes.push(current_attribute)
            }
            self.current_attribute = None
        }
    }

    fn emit_current_token(&mut self) {
        // If we have prepared an attribute, add it to the current tag.
        self.push_current_attribute_to_current_tag();

        if let Some(current_token) = self.current_token.clone() {
            self.emit_token(current_token);
        }
        self.current_token = None
    }

    fn set_current_attribute(&mut self, attribute: Attribute) {
        // If an attribute already exists, we should first push it to
        // the attributes of the current tag, so we don't override the previous attribute.
        self.push_current_attribute_to_current_tag();

        self.current_attribute = Some(attribute);
    }

    pub fn switch_to(&mut self, state: State) {
        self.state = state;
    }

    fn switch_to_return_state(&mut self) {
        if let Some(return_state) = self.return_state {
            self.switch_to(return_state);
            self.return_state = None;
        }
    }

    fn reconsume_in(&mut self, state: State) {
        if let Some(insertion_point) = self.insertion_point {
            self.insertion_point = Some(insertion_point - 1);
        }
        self.switch_to(state);
    }

    fn reconsume_in_return_state(&mut self) {
        if let Some(return_state) = self.return_state {
            self.reconsume_in(return_state);
            self.return_state = None;
        }
    }

    fn set_return_state(&mut self, state: State) {
        self.return_state = Some(state);
    }

    fn flush_code_points_consumed_as_a_character_reference(&mut self) {
        // SPEC: When a state says to flush code points consumed as a character reference,
        //       it means that for each code point in the temporary buffer
        //       (in the order they were added to the buffer) user agent must append the code point
        //       from the buffer to the current attribute's value if the character reference was
        //       consumed as part of an attribute,
        if let Some(Attribute { value, .. }) = &mut self.current_attribute {
            for character in self.temporary_buffer.chars() {
                value.push(character);
            }
        } else {
            // SPEC: or emit the code point as a character token otherwise.
            for character in self.temporary_buffer.clone().chars() {
                self.emit_token(Token::Character { data: character });
            }
        }
    }

    pub fn current_token(&self) -> Option<&Token> {
        self.tokens.last()
    }

    pub fn next_token(&mut self) -> Option<&Token> {
        self.token_emitted = false;

        if self.current_token() == Some(&Token::EndOfFile) {
            return None;
        }

        loop {
            if self.token_emitted {
                break;
            }

            match self.state {
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#data-state
                State::Data => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on!('&') => {
                            // SPEC: Set the return state to the data state.
                            self.set_return_state(State::Data);
                            // SPEC: Switch to the character reference state.
                            self.switch_to(State::CharacterReference);
                        }
                        on!('<') => {
                            // SPEC: Switch to the tag open state.
                            self.switch_to(State::TagOpen)
                        }
                        on_null!() => todo!(),
                        on_eof!() => {
                            // SPEC: Emit an end-of-file token.
                            self.emit_token(Token::EndOfFile);
                        }
                        on_anything_else!(character) => {
                            // SPEC: Emit the current input character as a character token.
                            self.emit_token(Token::Character { data: character });
                        }
                    }
                }
                State::RcData => todo!(),
                State::RawText => todo!(),
                State::ScriptData => todo!(),
                State::PlainText => todo!(),
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#tag-open-state
                State::TagOpen => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on!('!') => {
                            // SPEC: Switch to the markup declaration open state.
                            self.switch_to(State::MarkupDeclarationOpen);
                            continue;
                        }
                        on!('/') => {
                            // SPEC: Switch to the end tag open state.
                            self.switch_to(State::EndTagOpen);
                        }
                        on_ascii_alpha!() => {
                            // SPEC: Create a new start tag token.
                            self.set_current_token(Token::StartTag {
                                // SPEC: Set its tag name to the empty string.
                                name: String::new(),
                                self_closing: false,
                                self_closing_acknowledged: false,
                                attributes: Vec::new(),
                            });
                            // SPEC: Reconsume in the tag name state.
                            self.reconsume_in(State::TagName);
                        }
                        on!('?') => todo!(),
                        on_eof!() => todo!(),
                        on_anything_else!() => todo!(),
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#end-tag-open-state
                State::EndTagOpen => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on_ascii_alpha!() => {
                            // SPEC: Create a new end tag token, set its tag name to the empty string.
                            self.set_current_token(Token::EndTag {
                                name: String::new(),
                                self_closing: false,
                                attributes: Vec::new(),
                            });
                            // SPEC: Reconsume in the tag name state.
                            self.reconsume_in(State::TagName);
                        }
                        on!('>') => todo!(),
                        on_eof!() => todo!(),
                        on_anything_else!() => todo!(),
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#tag-name-state
                State::TagName => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on_whitespace!() => {
                            // SPEC: Switch to the before attribute name state.
                            self.switch_to(State::BeforeAttributeName);
                        }
                        on!('/') => {
                            // SPEC: Switch to the self-closing start tag state.
                            self.switch_to(State::SelfClosingStartTag)
                        }
                        on!('>') => {
                            // SPEC: Switch to the data state.
                            self.switch_to(State::Data);
                            // SPEC: Emit the current tag token.
                            self.emit_current_token();
                        }
                        on_null!() => todo!(),
                        on_eof!() => todo!(),
                        on_anything_else!(character) => {
                            // SPEC: ASCII upper alpha
                            //          Append the lowercase version of the current input character
                            //          (add 0x0020 to the character's code point)
                            //          to the current tag token's tag name.
                            let character = character.to_ascii_lowercase();
                            // SPEC: Append the current input character to the current tag token's tag name.
                            match &mut self.current_token {
                                Some(Token::StartTag { name, .. }) => {
                                    name.push(character.to_ascii_lowercase());
                                }
                                Some(Token::EndTag { name, .. }) => {
                                    name.push(character.to_ascii_lowercase());
                                }
                                _ => {}
                            }
                        }
                    }
                }
                State::RcDataLessThanSign => todo!(),
                State::RcDataEndTagOpen => todo!(),
                State::RcDataEndTagName => todo!(),
                State::RawTextLessThanSign => todo!(),
                State::RawTextEndTagOpen => todo!(),
                State::RawTextEndTagName => todo!(),
                State::ScriptDataLessThanSign => todo!(),
                State::ScriptDataEndTagOpen => todo!(),
                State::ScriptDataEndTagName => todo!(),
                State::ScriptDataEscapeStart => todo!(),
                State::ScriptDataEscapeStartDash => todo!(),
                State::ScriptDataEscaped => todo!(),
                State::ScriptDataEscapedDash => todo!(),
                State::ScriptDataEscapedDashDash => todo!(),
                State::ScriptDataEscapedLessThanSign => todo!(),
                State::ScriptDataEscapedEndTagOpen => todo!(),
                State::ScriptDataEscapedEndTagName => todo!(),
                State::ScriptDataDoubleEscapeStart => todo!(),
                State::ScriptDataDoubleEscaped => todo!(),
                State::ScriptDataDoubleEscapedDash => todo!(),
                State::ScriptDataDoubleEscapedDashDash => todo!(),
                State::ScriptDataDoubleEscapedLessThanSign => todo!(),
                State::ScriptDataDoubleEscapeEnd => todo!(),
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-name-state
                State::BeforeAttributeName => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on_whitespace!() => {
                            // SPEC: Ignore the character
                            continue;
                        }
                        on!('/') | on!('>') | on_eof!() => {
                            // SPEC: Reconsume in the after attribute name state.
                            self.reconsume_in(State::AfterAttributeName);
                        }
                        on!('=') => todo!(),
                        on_anything_else!() => {
                            // SPEC: Start a new attribute in the current tag token.
                            self.set_current_attribute(Attribute {
                                // Set that attribute name and value to the empty string.
                                name: String::new(),
                                value: String::new(),
                            });
                            // SPEC: Reconsume in the attribute name state.
                            self.reconsume_in(State::AttributeName);
                        }
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state
                State::AttributeName => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on_whitespace!() | on!('/') | on!('>') | on_eof!() => {
                            // SPEC: Reconsume in the after attribute name state.
                            self.reconsume_in(State::AfterAttributeName);
                        }
                        on!('=') => {
                            // SPEC: Switch to the before attribute value state.
                            self.switch_to(State::BeforeAttributeValue);
                        }
                        on_null!() => todo!(),
                        on_anything_else!(character) => {
                            // SPEC: ASCII upper alpha
                            //          Append the lowercase version of the current input character
                            //          (add 0x0020 to the character's code point)
                            //          to the current attribute's name.
                            let character = character.to_ascii_lowercase();

                            if let '"' | '\'' | '<' = character {
                                // FIXME Implement
                                // SPEC: This is an unexpected-character-in-attribute-name parse error. Treat it as per the "anything else" entry below.
                            }

                            // Append the current input character to the current attribute's name.
                            if let Some(Attribute { name, .. }) = &mut self.current_attribute {
                                name.push(character)
                            }
                        }
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-name-state
                State::AfterAttributeName => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on_whitespace!() => {
                            // SPEC: Ignore the character
                            continue;
                        }
                        on!('/') => {
                            // SPEC: Switch to the self-closing start tag state.
                            self.switch_to(State::SelfClosingStartTag)
                        }
                        on!('=') => {
                            // SPEC: Emit the current tag token.
                            self.switch_to(State::BeforeAttributeValue)
                        }
                        on!('>') => {
                            // SPEC: Switch to the data state.
                            self.switch_to(State::Data);
                            // SPEC: Emit the current tag token.
                            self.emit_current_token();
                        }
                        on_eof!() => todo!(),
                        on_anything_else!() => {
                            // SPEC: Start a new attribute in the current tag token.
                            self.set_current_attribute(Attribute {
                                // SPEC: Set that attribute name and value to the empty string.
                                name: String::new(),
                                value: String::new(),
                            });
                            // SPEC: Reconsume in the attribute name state.
                            self.reconsume_in(State::AttributeName);
                        }
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-value-state
                State::BeforeAttributeValue => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on_whitespace!() => {
                            // SPEC: Ignore the character
                            continue;
                        }
                        on!('"') => {
                            // SPEC: Switch to the attribute value (double-quoted) state.
                            self.switch_to(State::AttributeValueDoubleQuoted);
                        }
                        on!('\'') => {
                            // SPEC: Switch to the attribute value (double-quoted) state.
                            self.switch_to(State::AttributeValueSingleQuoted);
                        }
                        on!('>') => todo!(),
                        on_anything_else!() => {
                            // SPEC: Reconsume in the attribute value (unquoted) state.
                            self.reconsume_in(State::AttributeValueUnquoted);
                        }
                        on_eof!() => {}
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(double-quoted)-state
                State::AttributeValueDoubleQuoted => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on!('"') => {
                            // SPEC: Switch to the after attribute value (quoted) state.
                            self.switch_to(State::AfterAttributeValueQuoted);
                        }
                        on!('&') => {
                            // SPEC: Set the return state to the attribute value (double-quoted) state.
                            self.set_return_state(State::AttributeValueDoubleQuoted);
                            // SPEC: Switch to the character reference state.
                            self.switch_to(State::CharacterReference);
                        }
                        on_null!() => todo!(),
                        on_eof!() => todo!(),
                        on_anything_else!(character) => {
                            // SPEC: Append the current input character to the current attribute's value.
                            if let Some(Attribute { value, .. }) = &mut self.current_attribute {
                                value.push(character)
                            }
                        }
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(single-quoted)-state
                State::AttributeValueSingleQuoted => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on!('\'') => {
                            // SPEC: Switch to the after attribute value (quoted) state.
                            self.switch_to(State::AfterAttributeValueQuoted);
                        }
                        on!('&') => {
                            // SPEC: Set the return state to the attribute value (single-quoted) state.
                            self.set_return_state(State::AttributeValueSingleQuoted);
                            // SPEC: Switch to the character reference state.
                            self.switch_to(State::CharacterReference);
                        }
                        on_null!() => todo!(),
                        on_eof!() => todo!(),
                        on_anything_else!(character) => {
                            // SPEC: Append the current input character to the current attribute's value.
                            if let Some(Attribute { value, .. }) = &mut self.current_attribute {
                                value.push(character)
                            }
                        }
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(unquoted)-state
                State::AttributeValueUnquoted => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on_whitespace!() => {
                            // SPEC: Switch to the before attribute name state.
                            self.switch_to(State::BeforeAttributeName);
                        }
                        on!('&') => todo!(),
                        on!('>') => {
                            // SPEC: Switch to the data state.
                            self.switch_to(State::Data);
                            // SPEC: Emit the current tag token.
                            self.emit_current_token();
                        }
                        on_null!() => todo!(),
                        on_eof!() => todo!(),
                        on_anything_else!(character) => {
                            if let '"' | '\'' | '<' | '=' | '`' = character {
                                // FIXME Implement:
                                // SPEC: This is an unexpected-character-in-unquoted-attribute-value parse error. Treat it as per the "anything else" entry below.
                            }

                            // SPEC: Append the current input character to the current attribute's value.
                            if let Some(Attribute { value, .. }) = &mut self.current_attribute {
                                value.push(character)
                            }
                        }
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-value-(quoted)-state
                State::AfterAttributeValueQuoted => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on_whitespace!() => {
                            // SPEC: Switch to the before attribute name state.
                            self.switch_to(State::BeforeAttributeName);
                        }
                        on!('/') => {
                            // SPEC: Switch to the self-closing start tag state.
                            self.switch_to(State::SelfClosingStartTag);
                        }
                        on!('>') => {
                            // SPEC: Switch to the data state.
                            self.switch_to(State::Data);
                            // SPEC: Emit the current tag token.
                            self.emit_current_token();
                        }
                        on_eof!() => todo!(),
                        on_anything_else!() => {
                            // SPEC: This is a missing-whitespace-between-attributes parse error.
                            //       Reconsume in the before attribute name state.
                            self.reconsume_in(State::BeforeAttributeName);
                        }
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#self-closing-start-tag-state
                State::SelfClosingStartTag => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on!('>') => {
                            // SPEC: Set the self-closing flag of the current tag token.
                            if let Some(Token::StartTag { self_closing, .. }) =
                                &mut self.current_token
                            {
                                *self_closing = true
                            }
                            // SPEC: Emit the current tag token.
                            self.emit_current_token();
                            // SPEC: Switch to the data state.
                            self.switch_to(State::Data);
                        }
                        on_eof!() => todo!(),
                        on_anything_else!() => todo!(),
                    }
                }
                State::BogusComment => todo!(),
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#markup-declaration-open-state
                State::MarkupDeclarationOpen => {
                    // SPEC: Two U+002D HYPHEN-MINUS characters (-)
                    if self.next_characters_are_ascii_case_insensitive("--") {
                        // SPEC: Consume those two characters,
                        self.consume_characters("--");

                        // SPEC: create a comment token whose data is the empty string,
                        self.set_current_token(Token::Comment {
                            data: String::new(),
                        });

                        // SPEC: and switch to the comment start state.
                        self.switch_to(State::CommentStart);
                        continue;
                    }

                    // SPEC: ASCII case-insensitive match for the word "DOCTYPE"
                    if self.next_characters_are_ascii_case_insensitive("DOCTYPE") {
                        self.consume_characters("DOCTYPE");
                        self.switch_to(State::Doctype);
                        continue;
                    }

                    // SPEC: The string "[CDATA[" (the five uppercase letters "CDATA" with a U+005B LEFT SQUARE BRACKET character before and after)
                    // FIXME: Implement

                    // SPEC: Anything else
                    // FIXME: Implement
                    todo!()
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#comment-start-state
                State::CommentStart => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on!('-') => {
                            // SPEC: Switch to the comment start dash state.
                            self.switch_to(State::CommentStartDash);
                        }
                        on!('>') => {
                            // SPEC: This is an abrupt-closing-of-empty-comment parse error.

                            // SPEC: Switch to the data state.
                            self.switch_to(State::Data);

                            // SPEC: Emit the current comment token.
                            self.emit_current_token();
                        }
                        on_anything_else!() | None => {
                            // SPEC: Reconsume in the comment state.
                            self.reconsume_in(State::Comment);
                        }
                    }
                }
                State::CommentStartDash => todo!(),
                State::Comment => todo!(),
                State::CommentLessThanSign => todo!(),
                State::CommentLessThanSignBang => todo!(),
                State::CommentLessThanSignBangDash => todo!(),
                State::CommentLessThanSignBangDashDash => todo!(),
                State::CommentEndDash => todo!(),
                State::CommentEnd => todo!(),
                State::CommentEndBang => todo!(),
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#doctype-state
                State::Doctype => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on_whitespace!() => {
                            // SPEC: Switch to the before DOCTYPE name state.
                            self.switch_to(State::BeforeDoctypeName);
                            continue;
                        }
                        on!('>') => todo!(),
                        on_eof!() => todo!(),
                        on_anything_else!() => todo!(),
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#before-doctype-name-state
                State::BeforeDoctypeName => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on_whitespace!() => {
                            // SPEC: Ignore the character.
                            continue;
                        }
                        // FIXME: Implement ASCII upper alpha
                        on_null!() => todo!(),
                        on!('>') => todo!(),
                        on_eof!() => {
                            // FIXME: Implement
                            // SPEC: This is an eof-in-doctype parse error.

                            // SPEC: Create a new DOCTYPE token.
                            //       Emit the current token.
                            self.emit_token(Token::Doctype {
                                name: None,
                                public_identifier: None,
                                system_identifier: None,
                                // SPEC: Set its force-quirks flag to on.
                                force_quirks: true,
                            });
                            // SPEC: Emit an end-of-file token.
                            self.emit_token(Token::EndOfFile);
                        }
                        on_anything_else!(character) => {
                            // SPEC: Create a new DOCTYPE token.
                            self.set_current_token(Token::Doctype {
                                // SPEC: Set the token's name to the current input character.
                                name: Some(String::from(character)),
                                public_identifier: None,
                                system_identifier: None,
                                force_quirks: false,
                            });
                            // SPEC: Switch to the DOCTYPE name state.
                            self.switch_to(State::DoctypeName);
                        }
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#doctype-name-state
                State::DoctypeName => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on_whitespace!() => {
                            // SPEC: Switch to the after DOCTYPE name state.
                            self.switch_to(State::AfterDoctypeName);
                            continue;
                        }
                        on!('>') => {
                            // SPEC: Switch to the data state.
                            self.switch_to(State::Data);
                            // SPEC: Emit the current DOCTYPE token.
                            self.emit_current_token();
                            continue;
                        }
                        on_null!() => todo!(),
                        on_eof!() => todo!(),
                        on_anything_else!(character) => {
                            // SPEC: ASCII upper alpha
                            //          Append the lowercase version of the current input character
                            //          (add 0x0020 to the character's code point)
                            //          to the current DOCTYPE token's name.
                            let character = character.to_ascii_lowercase();

                            // SPEC: Append the current input character to the current DOCTYPE token's name.
                            if let Some(Token::Doctype {
                                name: Some(name), ..
                            }) = &mut self.current_token
                            {
                                name.push(character)
                            }
                        }
                    }
                }
                State::AfterDoctypeName => todo!(),
                State::AfterDoctypePublicKeyword => todo!(),
                State::BeforeDoctypePublicIdentifier => todo!(),
                State::DoctypePublicIdentifierDoubleQuoted => todo!(),
                State::DoctypePublicIdentifierSingleQuoted => todo!(),
                State::AfterDoctypePublicIdentifier => todo!(),
                State::BetweenDoctypePublicAndSystemIdentifiers => todo!(),
                State::AfterDoctypeSystemKeyword => todo!(),
                State::BeforeDoctypeSystemIdentifier => todo!(),
                State::DoctypeSystemIdentifierDoubleQuoted => todo!(),
                State::DoctypeSystemIdentifierSingleQuoted => todo!(),
                State::AfterDoctypeSystemIdentifier => todo!(),
                State::BogusDoctype => todo!(),
                State::CDataSection => todo!(),
                State::CDataSectionBracket => todo!(),
                State::CDataSectionEnd => todo!(),
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#character-reference-state
                State::CharacterReference => {
                    // SPEC: Set the temporary buffer to the empty string.
                    self.temporary_buffer.clear();
                    // SPEC: Append a U+0026 AMPERSAND (&) character to the temporary buffer.
                    self.temporary_buffer.push('&');
                    // SPEC: Consume the next input character:
                    self.consume_next_input_character();

                    match self.current_input_character {
                        ascii_alphanumeric!() => {
                            // SPEC: Reconsume in the named character reference state.
                            self.reconsume_in(State::NamedCharacterReference);
                        }
                        on!('#') => {
                            // SPEC: Append the current input character to the temporary buffer.
                            self.temporary_buffer
                                .push(self.current_input_character.unwrap());
                            // SPEC: Switch to the numeric character reference state.
                            self.switch_to(State::NumericCharacterReference);
                        }
                        on_anything_else!() | None => {
                            // SPEC: Flush code points consumed as a character reference.
                            self.flush_code_points_consumed_as_a_character_reference();
                            // SPEC: Reconsume in the return state.
                            self.reconsume_in_return_state();
                        }
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#named-character-reference-state
                State::NamedCharacterReference => {
                    self.temporary_named_character_references_buffer = NAMED_CHARACTER_REFERENCES
                        .map(|ncr| (ncr.0.to_string(), ncr.1))
                        .to_vec();

                    while self.temporary_named_character_references_buffer.len() > 1 {
                        self.consume_next_input_character();

                        // SPEC: Consume the maximum number of characters possible,
                        //       where the consumed characters are one of the identifiers
                        //       in the first column of the named character references table.
                        self.temporary_named_character_references_buffer
                            .retain_mut(|ncr| {
                                ncr.0.starts_with(self.current_input_character.unwrap())
                            });
                        for ncr in self.temporary_named_character_references_buffer.iter_mut() {
                            ncr.0.remove(0);
                        }
                        // SPEC: Append each character to the temporary buffer when it's consumed.
                        self.temporary_buffer
                            .push(self.current_input_character.unwrap());
                    }

                    if !self.temporary_named_character_references_buffer.is_empty() {
                        // SPEC: If there is a match

                        // SPEC: If the character reference was consumed as part of an attribute,
                        if self.current_attribute.is_some() {
                            // SPEC: and the last character matched is not a U+003B SEMICOLON character (;),
                            if !self.temporary_buffer.ends_with(';') {
                                // SPEC: and the next input character is either a U+003D EQUALS SIGN character (=)
                                //       or an ASCII alphanumeric, then, for historical reasons,
                                if let Some('=') | ascii_alphanumeric!() =
                                    self.next_input_character()
                                {
                                    // SPEC: flush code points consumed as a character reference
                                    self.flush_code_points_consumed_as_a_character_reference();
                                    // SPEC: and switch to the return state.
                                    self.switch_to_return_state();
                                }
                            }
                        }
                    } else {
                        // SPEC: Otherwise

                        // SPEC: Flush code points consumed as a character reference.
                        self.flush_code_points_consumed_as_a_character_reference();
                        // SPEC: Switch to the ambiguous ampersand state.
                        self.switch_to(State::AmbiguousAmpersand);
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#ambiguous-ampersand-state
                State::AmbiguousAmpersand => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        ascii_alphanumeric!(character) => {
                            // SPEC: If the character reference was consumed as part of an attribute,
                            if let Some(Attribute { value, .. }) = &mut self.current_attribute {
                                // SPEC: then append the current input character to the current attribute's value.
                                value.push(character);
                            } else {
                                // SPEC: Otherwise, emit the current input character as a character token.
                                self.emit_token(Token::Character { data: character });
                            }
                        }
                        on!(';') => {
                            // SPEC: This is an unknown-named-character-reference parse error.
                            // SPEC: Reconsume in the return state.
                            self.reconsume_in_return_state();
                        }
                        on_anything_else!() | None => {
                            // SPEC: Reconsume in the return state.
                            self.reconsume_in_return_state();
                        }
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#numeric-character-reference-state
                State::NumericCharacterReference => {
                    self.consume_next_input_character();

                    // SPEC: Set the character reference code to zero (0).
                    self.character_reference_code = 0;

                    match self.current_input_character {
                        Some(character @ 'x') | Some(character @ 'X') => {
                            // SPEC: Append the current input character to the temporary buffer.
                            self.temporary_buffer.push(character);
                            // SPEC: Switch to the hexadecimal character reference start state.
                            self.switch_to(State::HexadecimalCharacterReferenceStart);
                        }
                        on_anything_else!() | None => {
                            // SPEC: Reconsume in the decimal character reference start state.
                            self.reconsume_in(State::DecimalCharacterReferenceStart);
                        }
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#hexadecimal-character-reference-start-state
                State::HexadecimalCharacterReferenceStart => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on_ascii_hex_digit!() => {
                            // SPEC: Reconsume in the hexadecimal character reference state.
                            self.reconsume_in(State::HexadecimalCharacterReference);
                        }
                        on_anything_else!() | None => {
                            // SPEC: This is an absence-of-digits-in-numeric-character-reference parse error.

                            // SPEC: Flush code points consumed as a character reference.
                            self.flush_code_points_consumed_as_a_character_reference();
                            // SPEC: Reconsume in the return state.
                            self.reconsume_in_return_state();
                        }
                    }
                }
                State::DecimalCharacterReferenceStart => todo!(),
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#hexadecimal-character-reference-state
                State::HexadecimalCharacterReference => {
                    self.consume_next_input_character();

                    match self.current_input_character {
                        on_ascii_digit!(character) => {
                            // SPEC: Multiply the character reference code by 16.
                            self.character_reference_code *= 16;
                            // SPEC: Add a numeric version of the current input character
                            //      (subtract 0x0030 from the character's code point)
                            //      to the character reference code.
                            self.character_reference_code += character as u32 - 0x0030;
                        }
                        on_ascii_upper_hex_digit_alpha!(character) => {
                            // SPEC: Multiply the character reference code by 16.
                            self.character_reference_code *= 16;
                            // SPEC: Add a numeric version of the current input character as a hexadecimal digit
                            //       (subtract 0x0037 from the character's code point)
                            //       to the character reference code.
                            self.character_reference_code += character as u32 - 0x0037;
                        }
                        on_ascii_lower_hex_digit_alpha!(character) => {
                            // SPEC: Multiply the character reference code by 16.
                            self.character_reference_code *= 16;
                            // SPEC: Add a numeric version of the current input character as a hexadecimal digit
                            //       (subtract 0x0037 from the character's code point)
                            //       to the character reference code.
                            self.character_reference_code += character as u32 - 0x0057;
                        }
                        on!(';') => {
                            // SPEC: Switch to the numeric character reference end state.
                            self.switch_to(State::NumericCharacterReferenceEnd);
                        }
                        on_anything_else!() | None => {
                            // SPEC: This is a missing-semicolon-after-character-reference parse error.
                            // SPEC: Reconsume in the numeric character reference end state.
                            self.reconsume_in(State::NumericCharacterReferenceEnd);
                        }
                    }
                }
                State::DecimalCharacterReference => todo!(),
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#numeric-character-reference-end-state
                State::NumericCharacterReferenceEnd => match self.character_reference_code {
                    0x00 => todo!(),
                    surrogate_codepoint!() => todo!(),
                    noncharacter!() => todo!(),
                    0x10ffff.. => todo!(),
                    c @ 0x0d | c @ control_codepoint!() if c != whitespace_codepoint!() => todo!(),
                    _ => {
                        // SPEC: Set the temporary buffer to the empty string.
                        self.temporary_buffer.clear();
                        // SPEC: Append a code point equal to the character reference code to the temporary buffer.
                        if let Some(character) = char::from_u32(self.character_reference_code) {
                            self.temporary_buffer.push(character);
                        }
                        // SPEC: Flush code points consumed as a character reference.
                        self.flush_code_points_consumed_as_a_character_reference();
                        // SPEC: Switch to the return state.
                        self.switch_to_return_state();
                    }
                },
            }
        }

        self.tokens.last()
    }
}
