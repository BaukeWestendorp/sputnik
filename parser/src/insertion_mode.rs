use std::cell::Cell;

use tokenizer::Token;

use crate::tree_construction::rules::{handle_before_head, handle_before_html, handle_initial};
use crate::Parser;

#[allow(unused)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
pub enum Mode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    InHeadNoscript,
    AfterHead,
    InBody,
    Text,
    InTable,
    InTableText,
    InCaption,
    InColumnGroup,
    InTableBody,
    InRow,
    InCell,
    InSelect,
    InSelectInTable,
    InTemplate,
    AfterBody,
    InFrameset,
    AfterFrameset,
    AfterAfterBody,
    AfterAfterFrameset,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct InsertionMode {
    pub mode: Cell<Mode>,
}

impl InsertionMode {
    pub fn process_token<'a>(&self, parser: &'a Parser<'a>, token: &Token) {
        match self.mode.get() {
            Mode::Initial => handle_initial(parser, token),
            Mode::BeforeHtml => handle_before_html(parser, token),
            Mode::BeforeHead => handle_before_head(parser, token),
            _ => todo!("Insertion mode '{:?}'", self.mode.get()),
        }
    }
}

impl Default for InsertionMode {
    fn default() -> Self {
        Self {
            mode: Cell::new(Mode::Initial),
        }
    }
}
