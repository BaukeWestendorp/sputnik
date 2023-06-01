use tokenizer::Token;

use crate::tree_construction::creation_insertion::insert_html_element;
use crate::{is_parser_whitespace, Parser};

pub fn handle_before_head<'a>(parser: &'a Parser<'a>, token: &Token) {
    match token {
        Token::Character { data } if is_parser_whitespace(*data) => {
            return;
        }
        Token::Comment { .. } => todo!(),
        Token::Doctype { .. } => todo!(),
        Token::StartTag { name, .. } if name == "html" => todo!(),
        Token::StartTag { name, .. } if name == "head" => todo!(),
        Token::EndTag { name, .. }
            if name == "head" || name == "body" || name == "html" || name == "br" =>
        {
            todo!()
        }
        Token::EndTag { .. } => todo!(),
        _ => {
            let element = insert_html_element(
                parser,
                &Token::StartTag {
                    name: "head".to_string(),
                    self_closing: false,
                    self_closing_acknowledged: false,
                    attributes: vec![],
                },
            );

            *parser.head_element.borrow_mut() = Some(element);
        }
    }
}
