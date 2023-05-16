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
    input: String,
    state: State,
    tokens: Vec<Token>,
    insertion_point: Option<usize>,
    current_input_character: Option<char>,
    eof_emitted: bool,
    current_token: Option<Token>,
    current_attribute: Option<Attribute>,
}

impl Tokenizer {
    pub fn new(input: &str) -> Self {
        Self {
            input: String::from(input),
            state: State::Data,
            tokens: Vec::new(),
            insertion_point: None,
            current_input_character: None,
            eof_emitted: false,
            current_token: None,
            current_attribute: None,
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

        if token == Token::EndOfFile {
            self.eof_emitted = true
        }
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

    fn switch_to(&mut self, state: State) {
        self.state = state;
    }

    fn reconsume_and_switch_to(&mut self, state: State) {
        if let Some(insertion_point) = self.insertion_point {
            self.insertion_point = Some(insertion_point - 1);
        }
        self.switch_to(state);
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
            ($c:ident) => {
                Some($c)
            };
            (_) => {
                Some(_)
            };
        }

        macro_rules! ascii_alpha {
            () => {
                Some('a'..='z' | 'A'..='Z')
            };
        }

        loop {
            eprintln!("[{:?}] {:?}", self.state, self.current_input_character);

            if self.eof_emitted {
                break;
            }

            match self.state {
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#data-state
                State::Data => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        Some('&') => todo!(),
                        Some('<') => {
                            // SPEC: Switch to the tag open state.
                            self.switch_to(State::TagOpen)
                        }
                        null!() => todo!(),
                        eof!() => {
                            // SPEC: Emit an end-of-file token.
                            self.emit_token(Token::EndOfFile);
                        }
                        anything_else!(character) => {
                            // SPEC: Emit the current input character as a character token.
                            self.emit_token(Token::Character {
                                data: String::from(character),
                            });
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
                        Some('!') => {
                            // SPEC: Switch to the markup declaration open state.
                            self.switch_to(State::MarkupDeclarationOpen);
                            continue;
                        }
                        Some('/') => {
                            // SPEC: Switch to the end tag open state.
                            self.switch_to(State::EndTagOpen);
                        }
                        ascii_alpha!() => {
                            // SPEC: Create a new start tag token.
                            self.set_current_token(Token::StartTag {
                                // SPEC: Set its tag name to the empty string.
                                name: String::new(),
                                self_closing: false,
                                attributes: Vec::new(),
                            });
                            // SPEC: Reconsume in the tag name state.
                            self.reconsume_and_switch_to(State::TagName);
                        }
                        Some('?') => todo!(),
                        eof!() => todo!(),
                        anything_else!(_) => todo!(),
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#end-tag-open-state
                State::EndTagOpen => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        ascii_alpha!() => {
                            // SPEC: Create a new end tag token, set its tag name to the empty string.
                            self.set_current_token(Token::EndTag {
                                name: String::new(),
                                self_closing: false,
                                attributes: Vec::new(),
                            });
                            // SPEC: Reconsume in the tag name state.
                            self.reconsume_and_switch_to(State::TagName);
                        }
                        Some('>') => todo!(),
                        eof!() => todo!(),
                        anything_else!(_) => todo!(),
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#tag-name-state
                State::TagName => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        whitespace!() => {
                            // SPEC: Switch to the before attribute name state.
                            self.switch_to(State::BeforeAttributeName);
                        }
                        Some('/') => {
                            // SPEC: Switch to the self-closing start tag state.
                            self.switch_to(State::SelfClosingStartTag)
                        }
                        Some('>') => {
                            // SPEC: Switch to the data state.
                            self.switch_to(State::Data);
                            // SPEC: Emit the current tag token.
                            self.emit_current_token();
                        }
                        null!() => todo!(),
                        eof!() => todo!(),
                        anything_else!(character) => {
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
                        whitespace!() => {
                            // SPEC: Ignore the character
                            continue;
                        }
                        Some('/') | Some('>') | eof!() => {
                            // SPEC: Reconsume in the after attribute name state.
                            self.reconsume_and_switch_to(State::AfterAttributeName);
                        }
                        Some('=') => todo!(),
                        anything_else!(_) => {
                            // SPEC: Start a new attribute in the current tag token.
                            self.set_current_attribute(Attribute {
                                // Set that attribute name and value to the empty string.
                                name: String::new(),
                                value: String::new(),
                            });
                            // SPEC: Reconsume in the attribute name state.
                            self.reconsume_and_switch_to(State::AttributeName);
                        }
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state
                State::AttributeName => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        whitespace!() | Some('/') | Some('>') | eof!() => {
                            // SPEC: Reconsume in the after attribute name state.
                            self.reconsume_and_switch_to(State::AfterAttributeName);
                        }
                        Some('=') => {
                            // SPEC: Switch to the before attribute value state.
                            self.switch_to(State::BeforeAttributeValue);
                        }
                        null!() => todo!(),
                        anything_else!(character) => {
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
                        whitespace!() => {
                            // SPEC: Ignore the character
                            continue;
                        }
                        Some('/') => {
                            // SPEC: Switch to the self-closing start tag state.
                            self.switch_to(State::SelfClosingStartTag)
                        }
                        Some('=') => {
                            // SPEC: Emit the current tag token.
                            self.switch_to(State::BeforeAttributeValue)
                        }
                        Some('>') => {
                            // SPEC: Switch to the data state.
                            self.switch_to(State::Data);
                            // SPEC: Emit the current tag token.
                            self.emit_current_token();
                        }
                        eof!() => todo!(),
                        anything_else!(_) => {
                            // SPEC: Start a new attribute in the current tag token.
                            self.set_current_attribute(Attribute {
                                // SPEC: Set that attribute name and value to the empty string.
                                name: String::new(),
                                value: String::new(),
                            });
                            // SPEC: Reconsume in the attribute name state.
                            self.reconsume_and_switch_to(State::AttributeName);
                        }
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-value-state
                State::BeforeAttributeValue => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        whitespace!() => {
                            // SPEC: Ignore the character
                            continue;
                        }
                        Some('"') => {
                            // SPEC: Switch to the attribute value (double-quoted) state.
                            self.switch_to(State::AttributeValueDoubleQuoted);
                        }
                        Some('\'') => {
                            // SPEC: Switch to the attribute value (double-quoted) state.
                            self.switch_to(State::AttributeValueSingleQuoted);
                        }
                        Some('>') => todo!(),
                        anything_else!(_) => {
                            // SPEC: Reconsume in the attribute value (unquoted) state.
                            self.reconsume_and_switch_to(State::AttributeValueUnquoted);
                        }
                        eof!() => {}
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(double-quoted)-state
                State::AttributeValueDoubleQuoted => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        Some('"') => {
                            // SPEC: Switch to the after attribute value (quoted) state.
                            self.switch_to(State::AfterAttributeValueQuoted);
                        }
                        Some('&') => todo!(),
                        null!() => todo!(),
                        eof!() => todo!(),
                        anything_else!(character) => {
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
                        Some('\'') => {
                            // SPEC: Switch to the after attribute value (quoted) state.
                            self.switch_to(State::AfterAttributeValueQuoted);
                        }
                        Some('&') => todo!(),
                        null!() => todo!(),
                        eof!() => todo!(),
                        anything_else!(character) => {
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
                        whitespace!() => {
                            // SPEC: Switch to the before attribute name state.
                            self.switch_to(State::BeforeAttributeName);
                        }
                        Some('&') => todo!(),
                        Some('>') => {
                            // SPEC: Switch to the data state.
                            self.switch_to(State::Data);
                            // SPEC: Emit the current tag token.
                            self.emit_current_token();
                        }
                        null!() => todo!(),
                        eof!() => todo!(),
                        anything_else!(character) => {
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
                        whitespace!() => {
                            // SPEC: Switch to the before attribute name state.
                            self.switch_to(State::BeforeAttributeName);
                        }
                        Some('/') => {
                            // SPEC: Switch to the self-closing start tag state.
                            self.switch_to(State::SelfClosingStartTag);
                        }
                        Some('>') => {
                            // SPEC: Switch to the data state.
                            self.switch_to(State::Data);
                            // SPEC: Emit the current tag token.
                            self.emit_current_token();
                        }
                        eof!() => todo!(),
                        anything_else!(_) => todo!(),
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#self-closing-start-tag-state
                State::SelfClosingStartTag => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        Some('>') => {
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
                        eof!() => todo!(),
                        anything_else!(_) => todo!(),
                    }
                }
                State::BogusComment => todo!(),
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#markup-declaration-open-state
                State::MarkupDeclarationOpen => {
                    // FIXME: Fix spec comments
                    // FIXME: Implement --
                    if self.next_characters_are_ascii_case_insensitive("DOCTYPE") {
                        self.consume_characters("DOCTYPE");
                        self.switch_to(State::Doctype);
                        continue;
                    }
                    // FIXME: Implement [CDATA[
                    // FIXME: Anything else
                    todo!()
                }
                State::CommentStart => todo!(),
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
                        whitespace!() => {
                            // SPEC: Switch to the before DOCTYPE name state.
                            self.switch_to(State::BeforeDoctypeName);
                            continue;
                        }
                        Some('>') => todo!(),
                        eof!() => todo!(),
                        anything_else!(_) => todo!(),
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#before-doctype-name-state
                State::BeforeDoctypeName => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        whitespace!() => {
                            // SPEC: Ignore the character.
                            continue;
                        }
                        // FIXME: Implement ASCII upper alpha
                        null!() => todo!(),
                        Some('>') => todo!(),
                        eof!() => {
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
                        anything_else!(character) => {
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
                        whitespace!() => {
                            // SPEC: Switch to the after DOCTYPE name state.
                            self.switch_to(State::AfterDoctypeName);
                            continue;
                        }
                        Some('>') => {
                            // SPEC: Switch to the data state.
                            self.switch_to(State::Data);
                            // SPEC: Emit the current DOCTYPE token.
                            self.emit_current_token();
                            continue;
                        }
                        null!() => todo!(),
                        eof!() => todo!(),
                        anything_else!(character) => {
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
                State::CharacterReference => todo!(),
                State::NamedCharacterReference => todo!(),
                State::AmbiguousAmpersand => todo!(),
                State::NumericCharacterReference => todo!(),
                State::HexadecimalCharacterReferenceStart => todo!(),
                State::DecimalCharacterReferenceStart => todo!(),
                State::HexadecimalCharacterReference => todo!(),
                State::DecimalCharacterReference => todo!(),
                State::NumericCharacterReferenceEnd => todo!(),
            }
        }

        self.tokens.clone()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn tokenize() {
        use crate::*;

        let html = include_str!("test.html");
        let mut tokenizer = Tokenizer::new(html);
        let tokens = tokenizer.tokenize();

        eprintln!("--------- TAGS ---------");
        for token in tokens.iter() {
            eprintln!("{:?}", token);
        }
        eprintln!("------------------------");

        assert!(false);
    }
}
