use std::rc::Rc;

use crate::node::Node;

pub struct LiveNodeList {
    collection: Vec<Rc<Node>>,
    root: Rc<Node>,
    filter: Rc<dyn Fn(Rc<Node>) -> bool>,
}

// SPEC: If a collection is live, then the attributes and methods on that object must operate on the actual underlying data,
//       not a snapshot of the data.
//       When a collection is created, a filter and a root are associated with it.
//       The collection then represents a view of the subtree rooted at the collection’s root,
//       containing only nodes that match the given filter. The view is linear.
//       In the absence of specific requirements to the contrary,
//       the nodes within the collection must be sorted in tree order.
impl LiveNodeList {
    pub fn new<F: Fn(Rc<Node>) -> bool + 'static>(root: Rc<Node>, filter: F) -> Self {
        Self {
            collection: Vec::new(),
            root,
            filter: Rc::new(filter),
        }
    }

    // SPEC: The length attribute must return the number of nodes represented by the collection.
    pub fn length(&self) -> usize {
        self.collection.len()
    }

    // SPEC: The item(index) method must return the indexth node in the collection.
    //       If there is no indexth node in the collection, then the method must return null.
    pub fn item(&self, index: usize) -> Option<Rc<Node>> {
        match self.collection.get(index) {
            Some(node) => Some(node.clone()),
            None => None,
        }
    }

    fn collection(&self) -> Vec<Rc<Node>> {
        let mut nodes = Vec::<Rc<Node>>::new();
        self.root.for_each_in_inclusive_subtree(|node: Rc<Node>| {
            if (self.filter.clone())(node.clone()) {
                nodes.push(node);
            }
            true
        });

        nodes
    }
}

impl From<LiveNodeList> for Vec<Rc<Node>> {
    fn from(value: LiveNodeList) -> Self {
        value.collection()
    }
}
