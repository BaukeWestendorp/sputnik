use super::NodeImpl;

pub struct NodeList<'a, N: NodeImpl<'a>> {
    elements: Vec<&'a N>,
}

impl<'a, N: NodeImpl<'a>> NodeList<'a, N> {
    pub fn item(&self, index: usize) -> Option<&'a N> {
        self.elements.get(index).cloned()
    }

    pub fn length(&self) -> usize {
        self.elements.len()
    }

    // FIXME: Use From trait for this.
    pub fn to_vec(&self) -> Vec<&'a N> {
        self.elements.clone()
    }
}
