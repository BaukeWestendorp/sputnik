use dom::nodes::{append, Document};
use tokenizer::Token;

use crate::insertion_mode::Mode;
use crate::{is_parser_whitespace, Parser};

pub fn handle_initial(parser: &Parser<'_>, token: &Token) {
    match token {
        Token::Character { data } if is_parser_whitespace(*data) => {
            return;
        }
        Token::Comment { .. } => todo!(),
        Token::Doctype { .. } => {
            // FIXME: Implement spec
            append(&Document::new(), parser.document)
        }
        _ => {
            // FIXME: If the document is not an iframe srcdoc document, then this is a parse error; if the parser cannot change the mode flag is false, set the Document to quirks mode.
            parser.switch_to(Mode::BeforeHtml);
        }
    }
}
