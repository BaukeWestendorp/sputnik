use std::cell::Cell;

use named_character_references::{NamedCharacterReference, NAMED_CHARACTER_REFERENCES};

mod named_character_references;

include!("macros.rs");

macro_rules! log_current_token {
    ($state:expr, $current_token:expr) => {
        if std::env::var("TOKENIZER_LOGGING").is_ok() {
            eprintln!(
                "\x1b[34m[Tokenizer::State::{:?}] {:?}\x1b[0m",
                $state, $current_token
            );
        }
    };
}

#[allow(unused)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
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

#[derive(PartialEq, Eq, Debug, Clone)]
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
        self_closing_acknowledged: Cell<bool>,
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
    pub fn acknowledge_self_closing_flag_if_set(&self) {
        if let Token::StartTag {
            self_closing_acknowledged,
            self_closing,
            ..
        } = self
        {
            if *self_closing {
                self_closing_acknowledged.set(true);
            }
        } else {
            panic!("Tried to acknowledge non-StarTag token!");
        }
    }

    pub fn tag_name(&self) -> Option<String> {
        match self {
            Token::StartTag { name, .. } => Some(name.clone()),
            Token::EndTag { name, .. } => Some(name.clone()),
            _ => None,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Tokenizer {
    input: String,
    state: State,
    return_state: Option<State>,
    temporary_buffer: String,
    temporary_named_character_references_buffer: Vec<(String, NamedCharacterReference<'static>)>,
    last_start_tag_name: Option<String>,
    tokens: Vec<Token>,
    insertion_point: Option<usize>,
    current_input_character: Option<char>,
    token_emitted: bool,
    current_building_token: Option<Token>,
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
            last_start_tag_name: None,
            tokens: Vec::new(),
            insertion_point: None,
            current_input_character: None,
            token_emitted: false,
            current_building_token: None,
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
        if let Token::StartTag { name, .. } = &token {
            self.last_start_tag_name = Some(name.to_owned());
        }

        self.tokens.push(token);
        self.token_emitted = true;
    }

    fn create_new_token(&mut self, token: Token) {
        self.current_building_token = Some(token);
    }

    fn push_current_attribute_to_current_tag(&mut self) {
        if let Some(current_attribute) = self.current_attribute.clone() {
            if let Some(Token::StartTag { attributes, .. }) = &mut self.current_building_token {
                attributes.push(current_attribute)
            }
            self.current_attribute = None
        }
    }

    fn emit_current_token(&mut self) {
        // If we have prepared an attribute, add it to the current tag.
        self.push_current_attribute_to_current_tag();

        if let Some(current_token) = self.current_building_token.clone() {
            self.emit_token(current_token);
        }
        self.current_building_token = None
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

    pub fn state(&self) -> State {
        self.state
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

    pub fn set_insertion_point(&mut self, insertion_point: Option<usize>) {
        self.insertion_point = insertion_point
    }

    pub fn current_token(&self) -> Option<&Token> {
        self.tokens.last()
    }

    pub fn current_end_tag_token_is_an_appropriate_end_tag_token(&self) -> bool {
        assert!(matches!(
            self.current_building_token,
            Some(Token::EndTag { .. })
        ), "Current token is not an EndTag. This is needed when checking for the appropriate EndTag!");

        if let Some(Token::EndTag { name, .. }) = &self.current_building_token {
            return self.last_start_tag_name.clone().is_some_and(|f| f == *name);
        }
        false
    }

    pub fn next_token(&mut self) -> Option<&Token> {
        self.token_emitted = false;

        if self.current_token() == Some(&Token::EndOfFile) {
            return None;
        }

        loop {
            log_current_token!(self.state, self.current_token());

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
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#rcdata-state
                State::RcData => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on!('&') => {
                            // SPEC: Set the return state to the RCDATA state.
                            self.set_return_state(State::RcData);

                            // SPEC: Switch to the character reference state.
                            self.switch_to(State::CharacterReference);
                        }
                        on!('<') => {
                            // SPEC: Switch to the RCDATA less-than sign state.
                            self.switch_to(State::RcDataLessThanSign);
                        }
                        on_null!() => {
                            // SPEC: This is an unexpected-null-character parse error.

                            // SPEC: Emit a U+FFFD REPLACEMENT CHARACTER character token.
                            self.emit_token(Token::Character { data: '\u{FFFD}' });
                        }
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
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#rawtext-state
                State::RawText => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on!('<') => {
                            // SPEC: Switch to the RAWTEXT less-than sign state.
                            self.switch_to(State::RawTextLessThanSign);
                        }
                        on_null!() => {
                            // SPEC: This is an unexpected-null-character parse error.

                            // SPEC: Emit a U+FFFD REPLACEMENT CHARACTER character token.
                            self.emit_token(Token::Character { data: '\u{FFFD}' });
                        }
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
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#script-data-state
                State::ScriptData => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on!('<') => {
                            // SPEC: Switch to the script data less-than sign state.
                            self.switch_to(State::ScriptDataLessThanSign);
                        }
                        on_null!() => {
                            // SPEC: This is an unexpected-null-character parse error.

                            // SPEC: Emit a U+FFFD REPLACEMENT CHARACTER character token.
                            self.emit_token(Token::Character { data: '\u{FFFD}' });
                        }
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
                            self.create_new_token(Token::StartTag {
                                // SPEC: Set its tag name to the empty string.
                                name: String::new(),
                                self_closing: false,
                                self_closing_acknowledged: Cell::new(false),
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
                            self.create_new_token(Token::EndTag {
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
                            match &mut self.current_building_token {
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
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#rcdata-less-than-sign-state
                State::RcDataLessThanSign => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on!('/') => {
                            // SPEC: Set the temporary buffer to the empty string.
                            self.temporary_buffer.clear();

                            // SPEC: Switch to the RCDATA end tag open state.
                            self.switch_to(State::RcDataEndTagOpen);
                        }
                        on_anything_else!() | None => {
                            // SPEC: Emit a U+003C LESS-THAN SIGN character token.
                            self.emit_token(Token::Character { data: '<' });
                            // SPEC: Reconsume in the RCDATA state.
                            self.reconsume_in(State::RcData);
                        }
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#rcdata-end-tag-open-state
                State::RcDataEndTagOpen => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on_ascii_alpha!() => {
                            // SPEC: Create a new end tag token, set its tag name to the empty string.
                            self.create_new_token(Token::EndTag {
                                name: String::new(),
                                self_closing: false,
                                attributes: Vec::new(),
                            });

                            // SPEC: Reconsume in the RCDATA end tag name state.
                            self.reconsume_in(State::RcDataEndTagName);
                        }
                        on_anything_else!() | None => {
                            // SPEC: Emit a U+003C LESS-THAN SIGN character token
                            self.emit_token(Token::Character { data: '<' });
                            // SPEC: and a U+002F SOLIDUS character token.
                            self.emit_token(Token::Character { data: '/' });
                            // SPEC: Reconsume in the RCDATA state.
                            self.reconsume_in(State::RcData);
                        }
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#rcdata-end-tag-name-state
                State::RcDataEndTagName => {
                    macro_rules! anything_else {
                        () => {
                            // SPEC: Emit a U+003C LESS-THAN SIGN character token,
                            self.emit_token(Token::Character { data: '<' });

                            // SPEC: a U+002F SOLIDUS character token,
                            self.emit_token(Token::Character { data: '/' });

                            // SPEC: and a character token for each of the characters in the temporary buffer
                            //       (in the order they were added to the buffer).
                            for character in self.temporary_buffer.clone().chars() {
                                self.emit_token(Token::Character { data: character });
                            }

                            // SPEC: Reconsume in the RCDATA state.
                            self.reconsume_in(State::RcData);
                        };
                    }

                    self.consume_next_input_character();
                    match self.current_input_character {
                        on_whitespace!() => {
                            // SPEC: If the current end tag token is an appropriate end tag token,
                            //       then switch to the before attribute name state.
                            if self.current_end_tag_token_is_an_appropriate_end_tag_token() {
                                self.switch_to(State::BeforeAttributeName);
                            } else {
                                // SPEC: Otherwise, treat it as per the "anything else" entry below.
                                anything_else!();
                            }
                        }
                        on!('/') => {
                            // SPEC: If the current end tag token is an appropriate end tag token,
                            //       then switch to the self-closing start tag state.
                            if self.current_end_tag_token_is_an_appropriate_end_tag_token() {
                                self.switch_to(State::SelfClosingStartTag);
                            } else {
                                // SPEC: Otherwise, treat it as per the "anything else" entry below.
                                anything_else!();
                            }
                        }
                        on!('>') => {
                            // SPEC: If the current end tag token is an appropriate end tag token,
                            //       then switch to the data state and emit the current tag token.
                            if self.current_end_tag_token_is_an_appropriate_end_tag_token() {
                                self.switch_to(State::Data);
                                self.emit_current_token();
                            } else {
                                // SPEC: Otherwise, treat it as per the "anything else" entry below.
                                anything_else!();
                            }
                        }
                        on_ascii_upper_alpha!() => {
                            todo!()
                        }
                        on_ascii_lower_alpha!(character) => {
                            // SPEC: Append the current input character to the current tag token's tag name.
                            if let Some(Token::StartTag { name, .. }) =
                                &mut self.current_building_token
                            {
                                name.push(character)
                            }
                            if let Some(Token::EndTag { name, .. }) =
                                &mut self.current_building_token
                            {
                                name.push(character);
                            }

                            // SPEC: Append the current input character to the temporary buffer.
                            self.temporary_buffer.push(character);
                        }
                        on_anything_else!() | None => {
                            anything_else!();
                        }
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#rawtext-less-than-sign-state
                State::RawTextLessThanSign => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on!('/') => {
                            // SPEC: Set the temporary buffer to the empty string.
                            self.temporary_buffer.clear();
                            // SPEC: Switch to the RAWTEXT end tag open state.
                            self.switch_to(State::RawTextEndTagOpen);
                        }
                        on_anything_else!() | None => {
                            // SPEC: Emit a U+003C LESS-THAN SIGN character token.
                            self.emit_token(Token::Character { data: '<' });
                            // SPEC: Reconsume in the RAWTEXT state.
                            self.reconsume_in(State::RawText);
                        }
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#rawtext-end-tag-open-state
                State::RawTextEndTagOpen => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on_ascii_alpha!() => {
                            // SPEC: Create a new end tag token, set its tag name to the empty string.
                            self.create_new_token(Token::EndTag {
                                name: String::new(),
                                self_closing: false,
                                attributes: vec![],
                            });
                            // SPEC: Reconsume in the RAWTEXT end tag name state.
                            self.reconsume_in(State::RawTextEndTagName);
                        }
                        on_anything_else!() | None => {
                            // SPEC: Emit a U+003C LESS-THAN SIGN character token.
                            self.emit_token(Token::Character { data: '<' });
                            // SPEC: and a U+002F SOLIDUS character token
                            self.emit_token(Token::Character { data: '/' });
                            // SPEC: Reconsume in the RAWTEXT state.
                            self.reconsume_in(State::RawText);
                        }
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#rawtext-end-tag-name-state
                State::RawTextEndTagName => {
                    macro_rules! anything_else {
                        () => {
                            // SPEC: Emit a U+003C LESS-THAN SIGN character token,
                            self.emit_token(Token::Character { data: '<' });

                            // SPEC: a U+002F SOLIDUS character token,
                            self.emit_token(Token::Character { data: '/' });

                            // SPEC: and a character token for each of the characters in the temporary buffer
                            //       (in the order they were added to the buffer).
                            for character in self.temporary_buffer.clone().chars() {
                                self.emit_token(Token::Character { data: character });
                            }

                            // SPEC: Reconsume in the script data state.
                            self.reconsume_in(State::RawText);
                        };
                    }

                    self.consume_next_input_character();
                    match self.current_input_character {
                        on_whitespace!() => {
                            // SPEC: If the current end tag token is an appropriate end tag token,
                            if self.current_end_tag_token_is_an_appropriate_end_tag_token() {
                                // SPEC: then switch to the before attribute name state.
                                self.switch_to(State::BeforeAttributeName);
                            } else {
                                // SPEC: Otherwise, treat it as per the "anything else" entry below.
                                anything_else!();
                            }
                        }
                        on!('/') => {
                            // SPEC: If the current end tag token is an appropriate end tag token,
                            if self.current_end_tag_token_is_an_appropriate_end_tag_token() {
                                // SPEC: then switch to the self-closing start tag state.
                                self.switch_to(State::SelfClosingStartTag);
                            } else {
                                // SPEC: Otherwise, treat it as per the "anything else" entry below.
                                anything_else!();
                            }
                        }
                        on!('>') => {
                            // SPEC: If the current end tag token is an appropriate end tag token,
                            if self.current_end_tag_token_is_an_appropriate_end_tag_token() {
                                // SPEC: then switch to the data state
                                self.switch_to(State::Data);
                                // SPEC: and emit the current tag token.
                                self.emit_current_token();
                            } else {
                                // SPEC: Otherwise, treat it as per the "anything else" entry below.
                                anything_else!();
                            }
                        }
                        on_ascii_upper_alpha!() => {
                            todo!();
                        }
                        on_ascii_lower_alpha!(character) => {
                            // SPEC: Append the current input character to the current tag token's tag name.
                            if let Some(Token::StartTag { name, .. }) =
                                &mut self.current_building_token
                            {
                                name.push(character);
                            }
                            if let Some(Token::EndTag { name, .. }) =
                                &mut self.current_building_token
                            {
                                name.push(character);
                            }

                            // SPEC: Append the current input character to the temporary buffer.
                            self.temporary_buffer.push(character);
                        }
                        on_anything_else!() | None => {
                            anything_else!();
                        }
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#script-data-less-than-sign-state
                State::ScriptDataLessThanSign => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on!('/') => {
                            // SPEC: Set the temporary buffer to the empty string.
                            self.temporary_buffer.clear();
                            // SPEC: Switch to the script data end tag open state.
                            self.switch_to(State::ScriptDataEndTagOpen);
                        }
                        on!('!') => {
                            // SPEC: Switch to the script data escape start state.
                            self.switch_to(State::ScriptDataEscapeStart);
                            // Emit a U+003C LESS-THAN SIGN character token
                            self.emit_token(Token::Character { data: '<' });
                            // and a U+0021 EXCLAMATION MARK character token.
                            self.emit_token(Token::Character { data: '!' });
                        }
                        on_anything_else!() | None => {
                            // SPEC: Emit a U+003C LESS-THAN SIGN character token.
                            self.emit_token(Token::Character { data: '<' });
                            // Reconsume in the script data state.
                            self.reconsume_in(State::ScriptData);
                        }
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-open-state
                State::ScriptDataEndTagOpen => {
                    self.consume_next_input_character();
                    match self.current_input_character {
                        on_ascii_alpha!() => {
                            // SPEC: Create a new end tag token, set its tag name to the empty string.
                            self.create_new_token(Token::EndTag {
                                name: String::new(),
                                self_closing: false,
                                attributes: vec![],
                            });
                            // SPEC: Reconsume in the script data end tag name state.
                            self.reconsume_in(State::ScriptDataEndTagName);
                        }
                        on_anything_else!() | None => {
                            // SPEC: Emit a U+003C LESS-THAN SIGN character token
                            self.emit_token(Token::Character { data: '<' });
                            // and a U+002F SOLIDUS character token.
                            self.emit_token(Token::Character { data: '/' });
                            // Reconsume in the script data state.
                            self.reconsume_in(State::ScriptData);
                        }
                    }
                }
                // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-name-state
                State::ScriptDataEndTagName => {
                    macro_rules! anything_else {
                        () => {
                            // SPEC: Emit a U+003C LESS-THAN SIGN character token,
                            self.emit_token(Token::Character { data: '<' });

                            // SPEC: a U+002F SOLIDUS character token,
                            self.emit_token(Token::Character { data: '/' });

                            // SPEC: and a character token for each of the characters in the temporary buffer
                            //       (in the order they were added to the buffer).
                            for character in self.temporary_buffer.clone().chars() {
                                self.emit_token(Token::Character { data: character });
                            }

                            // SPEC: Reconsume in the script data state.
                            self.reconsume_in(State::ScriptData);
                        };
                    }

                    self.consume_next_input_character();
                    match self.current_input_character {
                        on_whitespace!() => {
                            // SPEC: If the current end tag token is an appropriate end tag token,
                            if self.current_end_tag_token_is_an_appropriate_end_tag_token() {
                                // SPEC: then switch to the before attribute name state.
                                self.switch_to(State::BeforeAttributeName);
                            } else {
                                // SPEC: Otherwise, treat it as per the "anything else" entry below.
                                anything_else!();
                            }
                        }
                        on!('/') => {
                            // SPEC: If the current end tag token is an appropriate end tag token,
                            if self.current_end_tag_token_is_an_appropriate_end_tag_token() {
                                // SPEC: then switch to the self-closing start tag state.
                                self.switch_to(State::SelfClosingStartTag);
                            } else {
                                // SPEC: Otherwise, treat it as per the "anything else" entry below.
                                anything_else!();
                            }
                        }
                        on!('>') => {
                            // SPEC: If the current end tag token is an appropriate end tag token,
                            if self.current_end_tag_token_is_an_appropriate_end_tag_token() {
                                // SPEC: then switch to the data state
                                self.switch_to(State::Data);
                                // SPEC: and emit the current tag token.
                                self.emit_current_token();
                            } else {
                                // SPEC: Otherwise, treat it as per the "anything else" entry below.
                                anything_else!();
                            }
                        }
                        on_ascii_upper_alpha!() => {
                            todo!();
                        }
                        on_ascii_lower_alpha!(character) => {
                            // SPEC: Append the current input character to the current tag token's tag name.
                            if let Some(Token::StartTag { name, .. }) =
                                &mut self.current_building_token
                            {
                                name.push(character);
                            }
                            if let Some(Token::EndTag { name, .. }) =
                                &mut self.current_building_token
                            {
                                name.push(character);
                            }

                            // SPEC: Append the current input character to the temporary buffer.
                            self.temporary_buffer.push(character);
                        }
                        on_anything_else!() | None => {
                            anything_else!();
                        }
                    }
                }
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
                                // SPEC: This is an unexpected-character-in-attribute-name parse error.
                                // FIXME: Implement

                                // Treat it as per the "anything else" entry below.
                                // FIXME: Implement
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
                                // SPEC: This is an unexpected-character-in-unquoted-attribute-value parse error.
                                // FIXME: Implement

                                // SPEC: Treat it as per the "anything else" entry below.
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
                                &mut self.current_building_token
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
                        self.create_new_token(Token::Comment {
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
                            // SPEC: This is an eof-in-doctype parse error.
                            // FIXME: Implement

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
                            self.create_new_token(Token::Doctype {
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
                            }) = &mut self.current_building_token
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
