pub mod insertion_mode;
pub mod stack_of_open_elements;
pub mod tree_construction;

use std::cell::RefCell;

use dom::infrastructure::Namespace;
use dom::nodes::{Document, Element, ElementImpl};

use insertion_mode::{InsertionMode, Mode};
use stack_of_open_elements::StackOfOpenElements;
use tokenizer::{Token, Tokenizer};

pub(crate) const fn is_parser_whitespace(string: char) -> bool {
    if let '\t' | '\u{000a}' | '\u{000c}' | '\u{000d}' | '\u{0020}' = string {
        return true;
    }
    false
}

pub struct Parser<'a> {
    document: &'a Document,

    tokenizer: Tokenizer,

    insertion_mode: InsertionMode,
    stack_of_open_elements: RefCell<StackOfOpenElements<'a>>,
    head_element: RefCell<Option<&'a Element<'a>>>,
    foster_parenting: bool,
}

impl<'a> Parser<'a> {
    pub fn new(document: &'a Document, input: &'a str) -> Self {
        Self {
            document,
            tokenizer: Tokenizer::new(input),
            insertion_mode: InsertionMode::default(),
            stack_of_open_elements: RefCell::new(StackOfOpenElements::new()),
            head_element: RefCell::new(None),
            foster_parenting: false,
        }
    }

    pub fn current_node(&'a self) -> &'a Element<'a> {
        self.stack_of_open_elements.borrow().current_node()
    }

    pub fn process_token_according_to_rules_of_insertion_mode(&'a self, token: &mut Token) {
        self.insertion_mode.process_token(&self, token);
    }

    pub fn process_token_according_to_rules_of_parsing_in_foreign_content(
        &'a mut self,
        token: &mut Token,
    ) {
        todo!()
    }

    pub fn switch_to(&self, mode: Mode) {
        self.insertion_mode.mode.set(mode);
    }

    pub fn parse(&mut self) {
        while let Some(token) = self.tokenizer.next_token() {
            let mut token = token.clone();

            // FIXME: Implement other conditions
            if self.stack_of_open_elements.borrow().is_empty()
                || self
                    .stack_of_open_elements
                    .borrow()
                    .adjusted_current_node()
                    .namespace_uri()
                    == Some(Namespace::Html)
            {
                self.process_token_according_to_rules_of_insertion_mode(&mut token);
            } else {
                self.process_token_according_to_rules_of_parsing_in_foreign_content(&mut token);
            }
        }
    }
}
