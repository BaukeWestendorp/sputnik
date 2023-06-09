use std::error::Error;

use super::node::{Node, NodeRef};

// 4.2.3. Mutation algorithms
// https://dom.spec.whatwg.org/#mutation-algorithms
impl<'a> Node<'a> {
    // https://dom.spec.whatwg.org/#concept-node-ensure-pre-insertion-validity
    pub fn ensure_pre_insertion_validity(
        _node: NodeRef<'a>,
        _parent: NodeRef<'a>,
        _child: Option<NodeRef<'a>>,
    ) -> Result<(), Box<dyn Error>> {
        eprintln!("FIXME: Skipped pre-insertion validity check!");
        Ok(())
    }

    // https://dom.spec.whatwg.org/#concept-node-pre-insert
    pub fn pre_insert(
        node: NodeRef<'a>,
        parent: NodeRef<'a>,
        child: Option<NodeRef<'a>>,
    ) -> NodeRef<'a> {
        // 1. Ensure pre-insertion validity of node into parent before child.
        // FIXME: Propogate error.
        Node::ensure_pre_insertion_validity(node, parent, child).unwrap();

        // 2. Let referenceChild be child.
        let mut reference_child = child;

        // 3. If referenceChild is node, then set referenceChild to node’s next sibling.
        if reference_child == Some(node) {
            reference_child = node.next_sibling();
        }

        // 4. Insert node into parent before referenceChild.
        Node::insert(node, parent, reference_child, false);

        // 5. Return node.
        node
    }

    pub fn insert(
        node: NodeRef<'a>,
        parent: NodeRef<'a>,
        child: Option<NodeRef<'a>>,
        _suppress_observers: bool,
    ) {
        // 1. Let nodes be node’s children, if node is a DocumentFragment node; otherwise « node ».
        let mut nodes = Vec::new();
        match node.is_document_fragment() {
            true => {
                for child in node.children.borrow().iter() {
                    nodes.push(*child);
                }
            }
            false => nodes.push(node),
        }

        // 2. Let count be nodes’s size.
        // 3. If count is 0, then return.
        if nodes.is_empty() {
            return;
        }

        // FIXME: 4. If node is a DocumentFragment node, then:

        // 5. If child is non-null, then:
        if let Some(_child) = child {
            // FIXME: 5.1. For each live range whose start node is parent and start offset is greater than child’s index, increase its start offset by count.
            // FIXME: 5.2. For each live range whose end node is parent and end offset is greater than child’s index, increase its end offset by count.
        }

        // 6. Let previousSibling be child’s previous sibling or parent’s last child if child is null.
        let _previous_sibling = match child {
            Some(child) => child.previous_sibling(),
            None => parent.last_child(),
        };

        // 7. For each node in nodes, in tree order:
        for node in nodes.iter() {
            // 7.1. Adopt node into parent’s node document.
            parent.node_document().adopt(node);

            if let Some(child) = child {
                // 7.3. Otherwise, insert node into parent’s children before child’s index.
                parent.children.borrow_mut().insert(child.index(), node);

                node.previous_sibling.set(child.previous_sibling());
                node.next_sibling.set(Some(child));
                if let Some(child_previous_sibling) = child.previous_sibling.get() {
                    child_previous_sibling.next_sibling.set(Some(node));
                }
                if parent.first_child.get() == Some(child) {
                    parent.first_child.set(Some(node));
                }

                child.previous_sibling.set(Some(node));

                node.parent.set(Some(parent));
            } else {
                // 7.2. If child is null, then append node to parent’s children.
                parent.children.borrow_mut().push(node);

                if parent.last_child().is_some() {
                    parent.last_child.set(Some(node));
                }
                node.previous_sibling.set(parent.last_child());
                node.parent.set(Some(parent));
                parent.last_child.set(Some(node));
                if parent.first_child().is_none() {
                    parent.first_child.set(parent.last_child());
                }
            }

            // FIXME: 7.4. If parent is a shadow host whose shadow root’s slot assignment is "named" and node is a slottable, then assign a slot for node.
            // FIXME: 7.5. If parent’s root is a shadow root, and parent is a slot whose assigned nodes is the empty list, then run signal a slot change for parent.
            // FIXME: 7.6. Run assign slottables for a tree with node’s root.
            // FIXME: 7.7. For each shadow-including inclusive descendant inclusiveDescendant of node, in shadow-including tree order:
        }

        // FIXME: 8. If suppress observers flag is unset, then queue a tree mutation record for parent with nodes, « », previousSibling, and child.
        // FIXME: 9. Run the children changed steps for parent.
    }

    // https://dom.spec.whatwg.org/#concept-node-append
    pub fn append(node: NodeRef<'a>, parent: NodeRef<'a>, _suppress_observers: bool) {
        Node::pre_insert(node, parent, None);
    }

    // https://dom.spec.whatwg.org/#concept-node-remove
    pub fn remove(_node: NodeRef<'a>, _suppress_observers: bool) {
        todo!()
    }
}
