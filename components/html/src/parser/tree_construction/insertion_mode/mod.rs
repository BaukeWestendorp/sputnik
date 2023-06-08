use crate::{InsertionMode, Parser};

pub(crate) mod after_after_body;
pub(crate) mod after_body;
pub(crate) mod after_head;
pub(crate) mod before_head;
pub(crate) mod before_html;
pub(crate) mod in_body;
pub(crate) mod in_head;
pub(crate) mod initial;
pub(crate) mod text;

#[macro_export]
macro_rules! log_parser_error {
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

impl<'a> Parser<'a> {
    pub(crate) fn switch_insertion_mode_to(&self, insertion_mode: InsertionMode) {
        self.insertion_mode.set(insertion_mode)
    }
}
