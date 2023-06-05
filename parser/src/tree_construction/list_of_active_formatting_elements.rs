use std::cell::RefCell;

use crate::types::NodeRef;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ActiveFormattingElement<'a> {
    Marker,
    Element(NodeRef<'a>),
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Position {
    End,
    LastMarkerOrElseStart,
}

pub struct ListOfActiveFormattingElements<'a> {
    elements: RefCell<Vec<ActiveFormattingElement<'a>>>,
}

impl<'a> ListOfActiveFormattingElements<'a> {
    pub fn new() -> Self {
        Self {
            elements: RefCell::new(vec![]),
        }
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#reconstruct-the-active-formatting-elements
    pub fn reconstruct_if_any(&self) {
        // FIXME: Implement
        eprintln!("FIXME: Skip reconstructing active formatting elements!")
    }

    //  https://html.spec.whatwg.org/multipage/parsing.html#push-onto-the-list-of-active-formatting-elements
    pub fn push_element(&self, element: NodeRef<'a>) {
        // FIXME: Implement Noah's Ark clause.
        self.elements
            .borrow_mut()
            .push(ActiveFormattingElement::Element(element));
    }

    pub fn first_index_of(&self, target: NodeRef<'a>) -> Option<usize> {
        self.elements
            .borrow()
            .iter()
            .position(|e| *e == ActiveFormattingElement::Element(target))
    }

    pub fn replace(&self, target: NodeRef<'a>, replacement: NodeRef<'a>) {
        if let Some(index) = self
            .elements
            .borrow()
            .iter()
            .position(|e| *e == ActiveFormattingElement::Element(target))
        {
            self.elements.borrow_mut()[index] = ActiveFormattingElement::Element(replacement);
        }
    }

    pub fn insert(&self, index: usize, element: NodeRef<'a>) {
        self.elements
            .borrow_mut()
            .insert(index, ActiveFormattingElement::Element(element));
    }

    pub fn remove(&self, element: NodeRef<'a>) {
        let mut elements = self.elements.borrow_mut();
        if let Some(index) = elements
            .iter()
            .position(|e| *e == ActiveFormattingElement::Element(element))
        {
            elements.remove(index);
        }
    }

    pub fn len(&self) -> usize {
        self.elements.borrow().len()
    }

    pub fn last_element_with_tag_name_before_marker(&self, tag_name: &str) -> Option<NodeRef<'a>> {
        for element in self.elements.borrow().iter().rev() {
            if matches!(element, ActiveFormattingElement::Marker) {
                break;
            }
            if let ActiveFormattingElement::Element(element) = element {
                if element.is_element_with_tag(tag_name) {
                    return Some(*element);
                }
            }
        }
        None
    }

    pub fn contains(&self, target: NodeRef<'a>) -> bool {
        self.elements.borrow().iter().any(|element| {
            if let ActiveFormattingElement::Element(element) = element {
                return *element == target;
            }
            false
        })
    }

    pub fn contains_element_between(&self, start: Position, end: Position, tag_name: &str) -> bool {
        if let Some(start) = self.index_from_position(start) {
            if let Some(end) = self.index_from_position(end) {
                for i in start..end {
                    if let Some(ActiveFormattingElement::Element(element)) =
                        self.elements.borrow().get(i)
                    {
                        if element.is_element_with_one_of_tags(&[tag_name]) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn index_from_position(&self, position: Position) -> Option<usize> {
        match position {
            Position::End => Some(self.len().saturating_sub(1)),
            Position::LastMarkerOrElseStart => self
                .elements
                .borrow()
                .iter()
                .rev()
                .enumerate()
                .find(|(_, element)| matches!(element, ActiveFormattingElement::Marker))
                .map(|element| element.0),
        }
    }
}
