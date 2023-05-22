use crate::arena::Ref;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct MutationRecord<'arena> {
    pub mutation_type: MutationType,
    pub target: Ref<'arena>,
    pub added_nodes: Vec<Ref<'arena>>,
    pub removed_nodes: Vec<Ref<'arena>>,
    pub previous_sibling: Option<Ref<'arena>>,
    pub next_sibling: Option<Ref<'arena>>,
    pub attribute_name: Option<String>,
    pub attribute_namespace: Option<String>,
    pub old_value: Option<String>,
}

impl<'arena> MutationRecord<'arena> {
    pub fn queue_tree_mutation_record(
        target: Ref<'arena>,
        added_nodes: Vec<Ref<'arena>>,
        removed_nodes: Vec<Ref<'arena>>,
        previous_sibling: Option<Ref<'arena>>,
        next_sibling: Option<Ref<'arena>>,
    ) {
        // SPEC: 1. Assert: either addedNodes or removedNodes is not empty.
        assert!(!(added_nodes.is_empty() && removed_nodes.is_empty()));

        // SPEC: 2. Queue a mutation record of "childList" for target with
        //          null, null, null, addedNodes, removedNodes, previousSibling, and nextSibling.
        Self::queue_mutation_record(
            target,
            MutationType::ChildList,
            None,
            None,
            None,
            added_nodes,
            removed_nodes,
            previous_sibling,
            next_sibling,
        );
    }

    pub fn queue_mutation_record(
        _target: Ref<'arena>,
        _mutation_type: MutationType,
        _name: Option<String>,
        _namespace: Option<String>,
        _old_value: Option<String>,
        _added_nodes: Vec<Ref<'arena>>,
        _removed_nodes: Vec<Ref<'arena>>,
        _previous_sibling: Option<Ref<'arena>>,
        _next_sibling: Option<Ref<'arena>>,
    ) {
        // FIXME: Implement
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum MutationType {
    Attributes,
    CharacterData,
    ChildList,
}
