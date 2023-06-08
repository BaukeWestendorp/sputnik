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
