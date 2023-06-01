use dom::nodes::Element;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct StackOfOpenElements<'a> {
    elements: Vec<Element<'a>>,
}

impl<'a> StackOfOpenElements<'a> {
    pub fn new() -> Self {
        Self { elements: vec![] }
    }

    pub fn push(&mut self, element: Element<'a>) {
        self.elements.push(element);
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn current_node(&'a self) -> &'a Element<'a> {
        self.elements
            .last()
            .expect("Should exist, as it's only empty if the parser has finished")
    }

    pub fn adjusted_current_node(&'a self) -> &'a Element<'a> {
        // FIXME: For now let's just make it the current node.
        self.current_node()
    }

    pub fn has_element_in_specific_scope(&self, target_node: &Element, list: &[&str]) -> bool {
        todo!()
    }

    pub fn has_element_in_scope(&self, element: &str) -> bool {
        todo!()
    }

    pub fn has_element_in_list_item_scope(&self, element: &str) -> bool {
        todo!()
    }

    pub fn has_element_in_button_scope(&self, element: &str) -> bool {
        todo!()
    }

    pub fn has_element_in_table_scope(&self, element: &str) -> bool {
        todo!()
    }

    pub fn has_element_in_select_scope(&self, element: &str) -> bool {
        todo!()
    }
}
