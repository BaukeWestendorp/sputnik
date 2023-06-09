#[derive(Debug, Clone, PartialEq)]
pub struct TreeNode<T>
where
    T: PartialEq,
{
    pub(crate) index: usize,
    pub value: T,
    pub(crate) parent: Option<usize>,
    pub(crate) children: Vec<usize>,
}

impl<T> TreeNode<T>
where
    T: PartialEq,
{
    pub fn new(index: usize, value: T) -> Self {
        Self {
            index,
            value,
            parent: None,
            children: vec![],
        }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn parent(&self) -> Option<usize> {
        self.parent
    }

    pub fn children(&self) -> &[usize] {
        self.children.as_ref()
    }
}
