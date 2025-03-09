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
            if let Some(node) = node.map(|mut node| unsafe { node.as_mut() }) {
                match value.cmp(&node.value) {
                    Ordering::Equal => {
                        return false;
                    }
                    Ordering::Less => {
                        if !insert(&mut node.left, value) {
                            return false;
                        }
                    }
                    Ordering::Greater => {
                        if !insert(&mut node.right, value) {
                            return false;
                        }
                    }
                }
            } else {
                *node = Some(Node::new(value));
            }
            Node::balance(node);
            true
        }
        insert(&mut self.root, value)
    }

    pub fn contains(&mut self, value: &i32) -> bool {
        let mut p = &self.root;
        while let Some(node) = p.map(|node| unsafe { node.as_ref() }) {
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
        fn remove(node: &mut Option<NonNull<Node>>, value: &i32) -> bool {
            if let Some(x) = node {
                let x = unsafe { x.as_mut() };

                match value.cmp(&x.value) {
                    Ordering::Equal => {
                        if x.left.is_none() {
                            *node = x.right;
                            return true;
                        } else if x.right.is_none() {
                            *node = x.left;
                            return true;
                        } else {
                            let mut right = x.right.unwrap();
                            while let Some(left) = unsafe { right.as_ref().left } {
                                right = left;
                            }
                            x.value = unsafe { right.as_ref().value };
                            if remove(&mut x.right, &unsafe { right.as_ref().value }) {
                                Node::balance(node);
                                return true;
                            }
                        }
                    }
                    Ordering::Less => {
                        if remove(&mut x.left, value) {
                            Node::balance(node);
                            return true;
                        }
                    }
                    Ordering::Greater => {
                        if remove(&mut x.right, value) {
                            Node::balance(node);
                            return true;
                        }
                    }
                }
            }
            false
        }

        remove(&mut self.root, value)
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
        fn collect(node: &Option<NonNull<Node>>, v: &mut Vec<&i32>) {
            if let Some(node) = node.map(|node| unsafe { node.as_ref() }) {
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
        fn free(node: &mut Option<NonNull<Node>>) {
            if let Some(mut node) = node.take() {
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
        f.debug_set().entries(self.iter()).finish()
    }
}

struct Node {
    value: i32,
    len: usize,
    height: i32,
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

    fn height(node: &Option<NonNull<Node>>) -> i32 {
        node.map_or(0, |node| unsafe { node.as_ref() }.height)
    }

    /// 平衡
    fn balance(node: &mut Option<NonNull<Node>>) {
        /// 左部分木と右部分木の高さの差
        /// 左部分木の高さ - 右部分木の高さ
        #[inline]
        fn diff_height(node: &NonNull<Node>) -> i32 {
            let node = unsafe { node.as_ref() };
            Node::height(&node.left) - Node::height(&node.right)
        }

        /// 木を右回転させる
        fn rotate_right(root: &mut Option<NonNull<Node>>) {
            if let Some(x) = root {
                let mut y = unsafe { x.as_mut() }.left.unwrap();
                unsafe { x.as_mut() }.left = unsafe { y.as_mut() }.right;
                unsafe { x.as_mut() }.update();
                unsafe { y.as_mut() }.right = Some(*x);
                unsafe { y.as_mut() }.update();
                *root = Some(y);
            }
        }

        /// 木を左回転させる
        fn rotate_left(root: &mut Option<NonNull<Node>>) {
            if let Some(x) = root {
                let mut y = unsafe { x.as_mut() }.right.unwrap();
                unsafe { x.as_mut() }.right = unsafe { y.as_mut() }.left;
                unsafe { x.as_mut() }.update();
                unsafe { y.as_mut() }.left = Some(*x);
                unsafe { y.as_mut() }.update();
                *root = Some(y);
            }
        }

        if let Some(x) = node {
            let d = diff_height(&x);
            let x = unsafe { x.as_mut() };

            if d > 1 {
                // 左部分木が高い場合

                if diff_height(&x.left.unwrap()) < 0 {
                    rotate_left(&mut x.left);
                }

                rotate_right(node);
            } else if d < -1 {
                // 右部分木が高い場合

                if diff_height(&x.right.unwrap()) > 0 {
                    rotate_right(&mut x.right);
                }

                rotate_left(node);
            } else {
                x.update();
            }
        }
    }
}

#[cfg(debug_assertions)]
#[allow(unused)]
impl AVLTreeSet {
    fn visualize(&self) -> String {
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
        res
    }
}

#[allow(unused)]
fn main() {
    use rand::{rng, Rng};
    use std::collections::BTreeSet;
    let mut rng = rng();

    println!("AVLTreeSet");

    // let tree = AVLTreeSet::from([1, 2, 3, 4, 5]);
    // println!("{:?}", tree);
    // println!("{}", tree.visualize());

    // let tree = AVLTreeSet::from([3, 1, 4, 1, 5, 9, 2]);
    // println!("{:?}", tree);
    // println!("{}", tree.visualize());

    // let mut tree = AVLTreeSet::from([100, 50, 20, 10, 80, 90, 60, 95]);
    // println!("{:?}", tree);
    // println!("{}", tree.visualize());

    // let mut avl = AVLTreeSet::new();
    // for _ in 0..100 {
    //     avl.insert(rng.random_range(0..100));
    // }
    // println!("{}", avl.visualize());

    let mut avl = AVLTreeSet::new();
    let mut b = BTreeSet::new();
    for _ in 0..200 {
        // 0: insert
        // 1: contains
        // 2: remove
        let t = rng.random_range(0..3);
        let x = rng.random_range(0..100);
        match t {
            0 => {
                assert_eq!(b.insert(x), avl.insert(x));
            }
            1 => {
                assert_eq!(b.contains(&x), avl.contains(&x));
            }
            2 => {
                assert_eq!(b.remove(&x), avl.remove(&x));
            }
            _ => {}
        }
        assert_eq!(b.len(), avl.len());
        assert!(avl.iter().eq(b.iter()));
    }

    println!("{}", avl.visualize());
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
    fn test_remove() {
        let mut tree = AVLTreeSet::from([3, 1, 4, 1, 5]);
        assert!(!tree.remove(&10));
        assert!(tree.contains(&3));
        assert!(tree.remove(&3));
        assert!(!tree.contains(&3));
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
                let x = rng.random_range(-100..=100);
                match t {
                    0 => {
                        assert_eq!(b.insert(x), avl.insert(x));
                    }
                    1 => {
                        assert_eq!(b.contains(&x), avl.contains(&x));
                    }
                    2 => {
                        assert_eq!(b.remove(&x), avl.remove(&x));
                    }
                    _ => {}
                }
                assert_eq!(b.len(), avl.len());
                assert!(avl.iter().eq(b.iter()));
            }
        }
    }
}
