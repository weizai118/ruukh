//! Representation of a list of nodes in VDOM.

use dom::DOMPatch;
use std::fmt::{self, Display, Formatter};
use wasm_bindgen::prelude::JsValue;
use web_api::*;
use {KeyedVNodes, VNode};

/// The representation of a list of vnodes in the vtree.
#[derive(Debug)]
pub struct VList(Vec<KeyedVNodes>);

impl VList {
    /// Constructor to create a list of VNodes.
    pub fn new(list: Vec<KeyedVNodes>) -> VList {
        VList(list)
    }
}

impl From<VList> for VNode {
    fn from(list: VList) -> VNode {
        VNode::List(list)
    }
}

impl Display for VList {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for vnode in self.0.iter() {
            write!(f, "{}", vnode)?;
        }
        Ok(())
    }
}

impl DOMPatch for VList {
    type Node = Node;

    fn render_walk(&mut self, parent: Self::Node, next: Option<Self::Node>) -> Result<(), JsValue> {
        let mut next = next;
        for vnode in self.0.iter_mut().rev() {
            vnode.render_walk(parent.clone(), next)?;
            next = vnode.node();
        }
        Ok(())
    }

    fn patch(
        &mut self,
        old: Option<Self>,
        parent: Self::Node,
        next: Option<Self::Node>,
    ) -> Result<(), JsValue> {
        let mut next = next;
        if let Some(mut old) = old {
            let old_len = old.0.len();
            for (index, vnode) in self.0.iter_mut().enumerate().rev() {
                let old = if index < old_len {
                    Some(old.0.remove(index))
                } else {
                    None
                };
                vnode.patch(old, parent.clone(), next)?;
                next = vnode.node();
            }
            old.remove(parent)?;
        } else {
            for vnode in self.0.iter_mut().rev() {
                vnode.patch(None, parent.clone(), next)?;
                next = vnode.node();
            }
        }
        Ok(())
    }

    fn remove(self, parent: Self::Node) -> Result<(), JsValue> {
        for vnode in self.0 {
            vnode.remove(parent.clone())?;
        }
        Ok(())
    }

    fn node(&self) -> Option<Node> {
        self.0.get(0).and_then(|first| first.node())
    }
}

#[cfg(test)]
mod test {
    use super::VList;
    use velement::VElement;
    use vtext::VText;
    use KeyedVNodes;

    #[test]
    fn should_display_a_list_of_vnodes() {
        let list = VList::new(vec![
            KeyedVNodes::unkeyed(VText::text("First of the node")),
            KeyedVNodes::unkeyed(VElement::childless("input", vec![])),
        ]);
        assert_eq!(format!("{}", list), "First of the node<input>");
    }
}

#[cfg(test)]
pub mod wasm_test {
    use dom::*;
    use prelude::*;
    use wasm_bindgen_test::*;
    use web_api::*;

    fn container() -> Element {
        html_document.create_element("div").unwrap()
    }

    #[wasm_bindgen_test]
    fn should_patch_container_with_list_of_vnodes() {
        let mut list = VList::new(vec![
            KeyedVNodes::unkeyed(VText::text("Hello World!")),
            KeyedVNodes::unkeyed(VElement::childless("div", vec![])),
        ]);
        let div = container();
        list.patch(None, div.clone().into(), None)
            .expect("To patch div");

        assert_eq!(div.inner_html(), "Hello World!<div></div>");
    }

    #[wasm_bindgen_test]
    fn should_patch_container_with_updated_list() {
        let mut list = VList::new(vec![
            KeyedVNodes::unkeyed(VText::text("Hello World!")),
            KeyedVNodes::unkeyed(VElement::childless("div", vec![])),
        ]);
        let div = container();
        list.patch(None, div.clone().into(), None)
            .expect("To patch div");

        assert_eq!(div.inner_html(), "Hello World!<div></div>");

        let mut new_list = VList::new(vec![
            KeyedVNodes::unkeyed(VElement::childless("div", vec![])),
            KeyedVNodes::unkeyed(VText::text("Hello World!")),
            KeyedVNodes::unkeyed(VText::text("How are you?")),
        ]);
        new_list
            .patch(Some(list), div.clone().into(), None)
            .expect("To patch div");

        assert_eq!(div.inner_html(), "<div></div>Hello World!How are you?");
    }
}
