use dom::infrastructure::Namespace;
use dom::nodes::{Element, ElementImpl, NodeImpl};
use tokenizer::Token;

use crate::Parser;

pub struct AdjustedInsertionLocation<'a> {
    parent: &'a Element<'a>,
    after: Option<&'a Element<'a>>,
}

pub fn appropriate_place_for_inserting_node<'a>(
    parser: &'a Parser<'a>,
    override_target: Option<&'a Element<'a>>,
) -> AdjustedInsertionLocation<'a> {
    let target = match override_target {
        Some(override_target) => override_target,
        None => parser.current_node(),
    };
    let adjusted_insertion_location = match parser.foster_parenting {
        true => {
            todo!()
        }
        false => AdjustedInsertionLocation {
            parent: target,
            after: None,
        },
    };
    if adjusted_insertion_location.parent.tag_name() == "template" {
        todo!()
    }

    adjusted_insertion_location
}

pub fn create_element_for_token<'a, P: NodeImpl<'a>>(
    namespace: Namespace,
    intended_parent: &'a P,
) -> Element<'a> {
    todo!()
}

pub fn insert_foreign_element<'a, E: ElementImpl<'a>>(
    parser: &'a Parser<'a>,
    token: &Token,
    namespace: Namespace,
) -> &'a E {
    let adjusted_insertion_location = appropriate_place_for_inserting_node(parser, None);
    let element = create_element_for_token(namespace, adjusted_insertion_location.parent);
    todo!()
}

pub fn insert_html_element<'a>(parser: &'a Parser<'a>, token: &Token) -> &'a Element<'a> {
    insert_foreign_element(parser, token, Namespace::Html)
}

pub fn insert_character(character: char) {
    todo!()
}
