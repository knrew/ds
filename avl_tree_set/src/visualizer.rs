use std::ptr::NonNull;

use crate::{AvlTreeSet, Node};

impl AvlTreeSet {
    #[allow(unused)]
    pub fn visualize(&self) {
        fn visualize(
            node: &Option<NonNull<Node>>,
            prefix: &str,
            is_root: bool,
            is_last: bool,
            res: &mut String,
        ) {
            if let Some(node) = node.map(|node| unsafe { node.as_ref() }) {
                if is_root {
                    *res += &format!("{}\n", node.value);
                } else {
                    *res += &format!(
                        "{}{}{}\n",
                        prefix,
                        if is_last { "└── " } else { "├── " },
                        node.value
                    );
                }

                let new_prefix = if is_root {
                    String::new()
                } else {
                    format!("{}{}", prefix, if is_last { "    " } else { "│   " })
                };

                visualize(&node.right, &new_prefix, false, node.left.is_none(), res);
                visualize(&node.left, &new_prefix, false, true, res);
            }
        }

        let mut res = String::new();
        visualize(&self.root, "", true, true, &mut res);
        println!("{}", res);
    }
}
