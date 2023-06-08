#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    EndOfFile,

    Ident {
        value: String,
    },
    Function {
        value: String,
    },
    AtKeyword {
        value: String,
    },
    Hash {
        value: String,
        hash_type: HashType,
    },
    String {
        value: String,
    },
    BadString,
    Url {
        value: String,
    },
    BadUrl,
    Delim {
        value: char,
    },
    Number {
        value: f32,
        number_type: NumberType,
    },
    Percentage {
        value: f32,
    },
    Dimension {
        value: f32,
        number_type: NumberType,
        unit: String,
    },
    Whitespace,
    Cdo,
    Cdc,
    Colon,
    Semicolon,
    Comma,
    LeftSquareBracket,
    RightSquareBracket,
    LeftParenthesis,
    RightParenthesis,
    LeftCurlyBracket,
    RightCurlyBracket,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberType {
    Integer,
    Number,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HashType {
    Id,
    Unrestricted,
}
