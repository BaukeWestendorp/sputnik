use std::cell::Cell;

pub mod parser_algorithms;
pub mod parser_entry_points;
pub mod parsing_results;
pub mod token_streams;
pub mod types;

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

pub struct Parser {
    position: Cell<isize>,
}
