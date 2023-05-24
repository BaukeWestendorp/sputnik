use dom::arena::NodeRef;
use dom::node::Node;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct ListOfActiveFormattingElements<'a> {
    elements: Vec<ActiveFormattingElement<'a>>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum ActiveFormattingElement<'a> {
    Marker,
    Element(NodeRef<'a>),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum Position {
    End,
    LastMarkerOrElseStart,
}

// FIXME: This should probably inherit from Vec or something.
impl<'a> ListOfActiveFormattingElements<'a> {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    // SPECLINK: https://html.spec.whatwg.org/multipage/parsing.html#push-onto-the-list-of-active-formatting-elements
    pub fn push_element(&mut self, element: NodeRef<'a>) {
        // FIXME: Implement Noah's Ark clause.
        self.elements
            .push(ActiveFormattingElement::Element(element));
    }

    pub fn insert_marker_at_end(&mut self) {
        self.elements.push(ActiveFormattingElement::Marker);
    }

    pub fn remove(&mut self, element: NodeRef<'a>) {
        if let Some(index) = self
            .elements
            .iter()
            .position(|e| *e == ActiveFormattingElement::Element(element))
        {
            self.elements.remove(index);
        }
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn last_marker_index(&self) -> Option<usize> {
        self.elements
            .iter()
            .rev()
            .position(|element| matches!(element, ActiveFormattingElement::Marker))
    }

    pub fn last_element_that_is_between_index_and_has_tag_name(
        &self,
        start_index: usize,
        end_index: usize,
        tag_name: &str,
    ) -> Option<ActiveFormattingElement<'a>> {
        self.elements
            .iter()
            .rev()
            .enumerate()
            .find(|(i, element)| {
                let range = start_index..end_index;
                if let ActiveFormattingElement::Element(element) = element {
                    return range.contains(i) && element.element_tag_name() == Some(tag_name);
                }
                false
            })
            .map(|element| *element.1)
    }

    pub fn contains(&self, target: ActiveFormattingElement<'a>) -> bool {
        self.elements
            .iter()
            .find(|element| {
                if matches!(target, ActiveFormattingElement::Marker) {
                    return matches!(element, ActiveFormattingElement::Marker);
                }

                if let ActiveFormattingElement::Element(element) = element {
                    if let ActiveFormattingElement::Element(target) = target {
                        return Node::are_same(element, target);
                    }
                }
                false
            })
            .is_some()
    }

    pub fn contains_element_between(&self, start: Position, end: Position, tag_name: &str) -> bool {
        if let Some(start) = self.index_from_position(start) {
            if let Some(end) = self.index_from_position(end) {
                for i in start..end {
                    if let Some(ActiveFormattingElement::Element(element)) = self.elements.get(i) {
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
                .iter()
                .rev()
                .enumerate()
                .find(|(_, element)| matches!(element, ActiveFormattingElement::Marker))
                .map(|element| element.0),
        }
    }
}
