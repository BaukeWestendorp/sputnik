use dom::arena::Ref;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct ListOfActiveFormattingElements<'arena> {
    elements: Vec<ActiveFormattingElement<'arena>>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum ActiveFormattingElement<'arena> {
    Marker,
    Element(Ref<'arena>),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum Position {
    End,
    LastMarkerOrElseStart,
}

// FIXME: This should probably inherit from Vec or something.
impl<'arena> ListOfActiveFormattingElements<'arena> {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub(crate) fn remove(&mut self, element: Ref<'arena>) {
        if let Some(index) = self
            .elements
            .iter()
            .position(|e| *e == ActiveFormattingElement::Element(element))
        {
            self.elements.remove(index);
        }
    }

    #[allow(unused)]
    pub fn number_of_elements_after_last_marker() -> usize {
        todo!()
    }

    pub fn last_marker_index(&self) -> Option<usize> {
        self.elements
            .iter()
            .rev()
            .position(|element| matches!(element, ActiveFormattingElement::Marker))
    }

    #[allow(unused)]
    pub fn last_marker(&self) -> Option<ActiveFormattingElement<'arena>> {
        self.elements
            .iter()
            .rev()
            .find(|element| matches!(element, ActiveFormattingElement::Marker))
            .copied()
    }

    pub fn last_element_that_is_between_index_and_has_tag_name(
        &self,
        start_index: usize,
        end_index: usize,
        tag_name: &str,
    ) -> Option<ActiveFormattingElement<'arena>> {
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

    pub fn contains(&self, element: ActiveFormattingElement<'arena>) -> bool {
        self.elements.contains(&element)
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

    pub fn insert_marker_at_end(&mut self) {
        self.elements.push(ActiveFormattingElement::Marker);
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
