use dom::infrastructure::Namespace;
use dom::nodes::{append, Element, ElementImpl};
use tokenizer::Token;

use crate::insertion_mode::Mode;
use crate::{is_parser_whitespace, Parser};

pub fn handle_before_html<'a>(parser: &'a Parser<'a>, token: &Token) {
    macro_rules! anything_else {
        () => {
            let element =
                Element::create(parser.document, "html", Some(Namespace::Html), None, None);
            append(&element, parser.document);

            parser.stack_of_open_elements.borrow_mut().push(element);
            parser.switch_to(Mode::BeforeHead);
        };
    }

    match token {
        Token::Doctype { .. } => todo!(),
        Token::Comment { .. } => todo!(),
        Token::Character { data } if is_parser_whitespace(*data) => {
            return;
        }
        Token::StartTag { name, .. } if name == "html" => todo!(),
        Token::EndTag { name, .. }
            if name == "head" || name == "body" || name == "html" || name == "br" =>
        {
            anything_else!();
        }
        Token::EndTag { .. } => todo!(),
        _ => {
            anything_else!();
        }
    }
}
