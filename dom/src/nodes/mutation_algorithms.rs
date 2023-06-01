use super::{Node, NodeImpl, NodeType};

pub fn ensure_pre_insertion_validity<
    'a,
    N: NodeImpl<'a> + 'a,
    P: NodeImpl<'a> + 'a,
    C: NodeImpl<'a> + 'a,
>(
    node: &N,
    parent: &P,
    child: Option<&C>,
) {
    // FIXME: Implement
    eprintln!("FIXME: Skipped ensure pre-insertion validity");
}

pub fn pre_insert<'a, N: NodeImpl<'a> + 'a, P: NodeImpl<'a> + 'a, C: NodeImpl<'a> + 'a>(
    node: &'a N,
    parent: &'a P,
    child: Option<&'a C>,
) {
    ensure_pre_insertion_validity(node, parent, child);
    let mut reference_child = child;
    if let Some(r) = reference_child {
        if r.is_same_node(node) {
            reference_child = node.next_sibling();
        }
    }
    insert(node, parent, reference_child, None);
}

pub fn insert<'a, N: NodeImpl<'a> + 'a, P: NodeImpl<'a> + 'a, C: NodeImpl<'a> + 'a>(
    node: &'a N,
    parent: &'a P,
    child: Option<&'a C>,
    suppress_observers: Option<bool>,
) {
    let suppress_observers = match suppress_observers {
        Some(s) => s,
        None => false,
    };

    let nodes = match node.node_type() {
        NodeType::DocumentFragment => node.child_nodes().to_vec(),
        _ => vec![node],
    };
    let count = 0;
    if count == 0 {
        return;
    }
    if node.node_type() == NodeType::DocumentFragment {
        for child in node.child_nodes::<N>().to_vec().iter() {
            remove(*child, None);
        }
        // FIXME: 2. mutation_algorithms.rs:36:20
    }
    if let Some(_child) = child {
        // FIXME: 1. For each live range whose start node is parent and start offset is greater than child’s index, increase its start offset by count.
        // FIXME: 2. For each live range whose end node is parent and end offset is greater than child’s index, increase its end offset by count.
    }
    // let previous_sibling = match child {
    //     Some(child) => child.previous_sibling::<N>(),
    //     None => parent.last_child(),
    // };
    for node in nodes.iter() {
        parent.node_document().adopt(*node);
        if let Some(child) = child {
            parent.insert_before(*node, Some(child));
        } else {
            parent.append_child(*node);
        }
    }

    // FIXME 5. If parent’s root is a shadow root, and parent is a slot whose assigned nodes is the empty list, then run signal a slot change for parent.
    // FIXME 6. Run assign slottables for a tree with node’s root.
    // FIXME 7. For each shadow-including inclusive descendant inclusiveDescendant of node, in shadow-including tree order:
    if !suppress_observers {
        // FIXME: 8. If suppress observers flag is unset, then queue a tree mutation record for parent with nodes, « », previousSibling, and child.
    }
    // FIXME: 9. Run the children changed steps for parent.
}

pub fn append<'a, N: NodeImpl<'a> + 'a, P: NodeImpl<'a> + 'a>(node: &'a N, parent: &'a P) {
    pre_insert(node, parent, Option::<N>::None.as_ref());
}

pub fn replace<'a, N: NodeImpl<'a>>(child: &Node, node: &N, parent: &N) {
    todo!()
}

pub fn replace_all<'a, N: NodeImpl<'a>>(node: &N, parent: &N) {
    todo!()
}

pub fn pre_remove<'a, N: NodeImpl<'a>>(child: &N, parent: &N) {
    todo!()
}

pub fn remove<'a, N: NodeImpl<'a>>(node: &N, suppress_observers: Option<bool>) {
    todo!()
}
