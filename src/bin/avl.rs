//! AVLTreeによるsetの実装

use std::{cmp::Ordering, fmt::Debug, ptr::NonNull};

#[derive(Clone)]
pub struct AVLTreeSet {
    root: Option<NonNull<Node>>,
}

impl AVLTreeSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn len(&self) -> usize {
        Node::len(&self.root)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn insert(&mut self, value: i32) -> bool {
        fn insert(node: &mut Option<NonNull<Node>>, value: i32) -> bool {
            if let Some(node) = node {
                match value.cmp(&unsafe { node.as_ref() }.value) {
                    Ordering::Equal => {
                        return false;
                    }
                    Ordering::Less => {
                        if !insert(&mut unsafe { node.as_mut() }.left, value) {
                            return false;
                        }
                    }
                    Ordering::Greater => {
                        if !insert(&mut unsafe { node.as_mut() }.right, value) {
                            return false;
                        }
                    }
                }
            } else {
                *node = Some(Node::new(value));
            }
            // balance(node);
            true
        }

        insert(&mut self.root, value)
    }

    pub fn contains(&mut self, value: &i32) -> bool {
        let mut p = &self.root;
        while let Some(node) = p {
            let node = unsafe { node.as_ref() };
            match value.cmp(&node.value) {
                Ordering::Equal => {
                    return true;
                }
                Ordering::Less => {
                    p = &node.left;
                }
                Ordering::Greater => {
                    p = &node.right;
                }
            }
        }
        false
    }

    pub fn remove(&mut self, value: &i32) -> bool {
        todo!()
    }

    /// 昇順でk番目の要素を取得する
    pub fn get_kth(&self, index: usize) -> Option<&i32> {
        todo!()
    }

    /// 昇順でk番目の要素を取得する
    pub fn get_kth_mut(&self, index: usize) -> Option<&i32> {
        todo!()
    }

    pub fn iter(&self) -> impl Iterator<Item = &i32> + '_ {
        // 再帰的に要素を収集する
        // 結果は昇順になる
        fn collect(root: &Option<NonNull<Node>>, v: &mut Vec<&i32>) {
            if let Some(node) = root {
                let node = unsafe { node.as_ref() };
                collect(&node.left, v);
                v.push(&node.value);
                collect(&node.right, v);
            }
        }
        let mut res = vec![];
        collect(&self.root, &mut res);
        res.into_iter()
    }
}

impl Default for AVLTreeSet {
    fn default() -> Self {
        AVLTreeSet { root: None }
    }
}

impl Drop for AVLTreeSet {
    fn drop(&mut self) {
        // 再帰的にメモリを解放する
        fn free(root: &mut Option<NonNull<Node>>) {
            if let Some(mut node) = root.take() {
                free(&mut unsafe { node.as_mut() }.left);
                free(&mut unsafe { node.as_mut() }.right);
                unsafe { drop(Box::from_raw(node.as_ptr())) };
            }
        }
        free(&mut self.root)
    }
}

impl From<Vec<i32>> for AVLTreeSet {
    fn from(v: Vec<i32>) -> Self {
        let mut res = Self::new();
        v.into_iter().for_each(|e| {
            res.insert(e);
        });
        res
    }
}

impl<const N: usize> From<[i32; N]> for AVLTreeSet {
    fn from(v: [i32; N]) -> Self {
        let mut res = Self::new();
        v.into_iter().for_each(|e| {
            res.insert(e);
        });
        res
    }
}

impl Debug for AVLTreeSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

struct Node {
    value: i32,
    len: usize,
    height: usize,
    left: Option<NonNull<Node>>,
    right: Option<NonNull<Node>>,
}

impl Node {
    fn new(value: i32) -> NonNull<Node> {
        let node = Node {
            value,
            len: 1,
            height: 1,
            left: None,
            right: None,
        };
        NonNull::from(Box::leak(Box::new(node)))
    }

    fn update(&mut self) {
        self.len = Node::len(&self.left) + Node::len(&self.right) + 1;
        self.height = Node::height(&self.left).max(Node::height(&self.right)) + 1;
    }

    fn len(node: &Option<NonNull<Node>>) -> usize {
        node.map_or(0, |node| unsafe { node.as_ref() }.len)
    }

    fn height(node: &Option<NonNull<Node>>) -> usize {
        node.map_or(0, |node| unsafe { node.as_ref() }.height)
    }
}

// rootを根とする部分木を平衡にする
fn balance(node: &mut Option<NonNull<Node>>) {
    if let Some(node) = node.as_mut() {
        let left_height = Node::height(&unsafe { node.as_ref() }.left);
        let right_height = Node::height(&unsafe { node.as_ref() }.right);
        if left_height > right_height + 1 {
            // 左部分木が高い場合

            // 二重回転
            let left = unsafe { node.as_ref().left.unwrap().as_ref() };
            if Node::height(&left.left) < Node::height(&left.right) {
                // rotate_left(&mut unsafe { node.as_mut().left });
            }

            rotate_right(node);
        } else if right_height > left_height + 1 {
            // 右部分木が高い場合

            // 二重回転
            let right = unsafe { node.as_ref().right.unwrap().as_ref() };
            if Node::height(&right.right) < Node::height(&right.left) {
                rotate_right(&mut unsafe { node.as_mut().right.unwrap() });
            }

            // rotate_left(node);
        } else {
            // すでに平衡である場合
            unsafe { node.as_mut() }.update();
        }
    }
}

/// 木を右回転させる
/// 回転の方向は[Wikipedia](https://ja.wikipedia.org/wiki/%E6%9C%A8%E3%81%AE%E5%9B%9E%E8%BB%A2)準拠
fn rotate_right(node: &mut NonNull<Node>) {
    //TODO:
    unsafe {
        let p = node.as_mut();
        assert!(p.left.is_some());
        let left = p.left.unwrap();
        p.left = p.right;
        p.right = p.left.unwrap().as_mut().right;
        // p.left.unwrap().as_mut().right = Some(*node);
        // p.left.unwrap().as_mut().update();
        // *node = left;
        // node.as_mut().update();
    }
}

/// 木を左回転させる
/// 回転の方向は[Wikipedia](https://ja.wikipedia.org/wiki/%E6%9C%A8%E3%81%AE%E5%9B%9E%E8%BB%A2)準拠
// fn rotate_left(node: &mut Option<NonNull<Node>>) {
//     unsafe {
//         assert!(node.is_some());
//         let p = node.take().unwrap().as_mut();
//         assert!(p.right.is_some());
//         let right = p.right.take().unwrap();
//         p.right = p.left;
//         p.left = p.right.unwrap().as_mut().left;
//         p.right.unwrap().as_mut().left = *node;
//         p.right.unwrap().as_mut().update();
//         *node = Some(right);
//         node.unwrap().as_mut().update();
//     }
// }

#[allow(unused)]
fn visualize(tree: &AVLTreeSet) -> String {
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
    visualize(&tree.root, "", true, true, &mut res);
    res
}

#[allow(unused)]
fn main() {
    use std::collections::BTreeSet;

    println!("AVLTreeSet");

    let tree = AVLTreeSet::from([1, 2, 3, 4, 5]);
    println!("{:?}", tree);
    println!("{}", visualize(&tree));

    let tree = AVLTreeSet::from([3, 1, 4, 1, 5, 9, 2]);
    println!("{:?}", tree);
    println!("{}", visualize(&tree));

    let mut tree = AVLTreeSet::from([100, 50, 20, 10, 80, 90, 60, 95]);
    println!("{:?}", tree);
    println!("{}", visualize(&tree));
}

#[allow(unused)]
#[cfg(test)]
mod tests {
    use crate::AVLTreeSet;

    #[test]
    fn test_insert_and_contains() {
        let mut tree = AVLTreeSet::new();
        assert!(!tree.contains(&3));
        assert!(!tree.contains(&1));
        assert!(!tree.contains(&4));
        assert!(!tree.contains(&5));
        assert!(!tree.contains(&100));
        assert!(tree.insert(3));
        assert!(tree.insert(1));
        assert!(tree.insert(4));
        assert!(!tree.insert(1));
        assert!(tree.insert(5));
        assert!(tree.contains(&3));
        assert!(tree.contains(&1));
        assert!(tree.contains(&4));
        assert!(tree.contains(&5));
        assert!(!tree.contains(&100));
    }

    #[test]
    fn test_random() {
        use rand::{rng, Rng};
        use std::collections::BTreeSet;

        let mut rng = rng();

        for _ in 0..5 {
            let mut avl = AVLTreeSet::new();
            let mut b = BTreeSet::new();
            for _ in 0..1000 {
                // 0: insert
                // 1: contains
                // 2: remove
                let t = rng.random_range(0..3);
                let x = rng.random();
                match t {
                    0 => {
                        assert_eq!(b.insert(x), avl.insert(x));
                    }
                    1 => {
                        assert_eq!(b.contains(&x), avl.contains(&x));
                    }
                    2 => {
                        // assert_eq!(b.remove(&x), avl.remove(&x));
                    }
                    _ => {}
                }
                assert!(avl.iter().eq(b.iter()));
            }
        }
    }
}
