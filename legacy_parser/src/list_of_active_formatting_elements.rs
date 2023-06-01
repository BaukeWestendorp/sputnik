use dom::arena::NodeRef;

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

    pub fn first_index_of(&self, target: NodeRef<'a>) -> Option<usize> {
        self.elements
            .iter()
            .position(|e| *e == ActiveFormattingElement::Element(target))
    }

    pub fn replace(&mut self, target: NodeRef<'a>, replacement: NodeRef<'a>) {
        if let Some(index) = self
            .elements
            .iter()
            .position(|e| *e == ActiveFormattingElement::Element(target))
        {
            self.elements[index] = ActiveFormattingElement::Element(replacement);
        }
    }

    pub fn insert(&mut self, index: usize, element: NodeRef<'a>) {
        self.elements
            .insert(index, ActiveFormattingElement::Element(element));
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

    pub fn last_element_with_tag_name_before_marker(&self, tag_name: &str) -> Option<NodeRef<'a>> {
        for element in self.elements.iter().rev() {
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
        self.elements.iter().any(|element| {
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
