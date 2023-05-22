use std::rc::Rc;

use crate::node::Node;

pub trait AbstractRange<'a> {
    fn start_container(&'a mut self) -> &'a Node {
        todo!()
    }

    fn start_offset(&mut self) -> usize {
        todo!()
    }

    fn end_container(&'a mut self) -> &'a Node {
        todo!()
    }

    fn end_offset(&mut self) -> usize {
        todo!()
    }

    fn collapsed(&mut self) -> bool {
        todo!()
    }
}

pub struct Range {}
impl<'a> AbstractRange<'a> for Range {}

impl Range {
    pub fn live_ranges() -> Vec<Range> {
        todo!()
    }

    pub fn set_start(&mut self, _node: Rc<Node>, _offset: usize) {
        todo!()
    }

    pub fn set_end(&mut self, _node: Rc<Node>, _offset: usize) {
        todo!()
    }

    pub fn set_start_before(&mut self, _node: Rc<Node>) {
        todo!()
    }

    pub fn set_start_after(&mut self, _node: Rc<Node>) {
        todo!()
    }

    pub fn set_end_before(&mut self, _node: Rc<Node>) {
        todo!()
    }

    pub fn set_end_after(&mut self, _node: Rc<Node>) {
        todo!()
    }

    pub fn collapse(&mut self, _to_start: bool) {
        todo!()
    }

    pub fn select_node(&mut self, _node: Rc<Node>) {
        todo!()
    }

    pub fn select_node_contents(&mut self, _node: Rc<Node>) {
        todo!()
    }
}
