use std::cell::RefCell;

use dom::node::NodeRef;

use super::stack_of_open_elements::StackOfOpenElements;

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
    pub fn reconstruct_if_any(&self, stack_of_open_elements: &'a StackOfOpenElements<'a>) {
        let elements = self.elements.borrow_mut();

        // If there are no entries in the list of active formatting elements, then there is nothing to reconstruct; stop this algorithm.
        if elements.is_empty() {
            return;
        }

        // If the last (most recently added) entry in the list of active formatting elements is a marker, or if it is an element that is in the stack of open elements, then there is nothing to reconstruct; stop this algorithm.
        match elements.last().unwrap() {
            ActiveFormattingElement::Marker => return,
            ActiveFormattingElement::Element(element)
                if stack_of_open_elements.contains(element) =>
            {
                return;
            }
            _ => {}
        }

        todo!();

        // FIXME: Let entry be the last (most recently added) element in the list of active formatting elements.
        // FIXME: Rewind: If there are no entries before entry in the list of active formatting elements, then jump to the step labeled create.
        // FIXME: Let entry be the entry one earlier than entry in the list of active formatting elements.
        // FIXME: If entry is neither a marker nor an element that is also in the stack of open elements, go to the step labeled rewind.
        // FIXME: Advance: Let entry be the element one later than entry in the list of active formatting elements.
        // FIXME: Create: Insert an HTML element for the token for which the element entry was created, to obtain new element.
        // FIXME: Replace the entry for entry in the list with an entry for new element.
        // FIXME: If the entry for new element in the list of active formatting elements is not the last entry in the list, return to the step labeled advance.
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
