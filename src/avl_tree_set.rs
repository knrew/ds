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

    pub fn contains(&self, value: &i32) -> bool {
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
                let x_ref = unsafe { x.as_mut() };
                match value.cmp(&x_ref.value) {
                    Ordering::Equal => {
                        if x_ref.left.is_none() {
                            let right = x_ref.right;
                            unsafe { drop(Box::from_raw(x.as_ptr())) };
                            *node = right;
                            return true;
                        } else if x_ref.right.is_none() {
                            let left = x_ref.left;
                            unsafe { drop(Box::from_raw(x.as_ptr())) };
                            *node = left;
                            return true;
                        } else {
                            let mut right = x_ref.right.unwrap();
                            while let Some(left) = unsafe { right.as_ref().left } {
                                right = left;
                            }
                            x_ref.value = unsafe { right.as_ref().value };
                            if remove(&mut x_ref.right, &unsafe { right.as_ref().value }) {
                                Node::balance(node);
                                return true;
                            }
                        }
                    }
                    Ordering::Less => {
                        if remove(&mut x_ref.left, value) {
                            Node::balance(node);
                            return true;
                        }
                    }
                    Ordering::Greater => {
                        if remove(&mut x_ref.right, value) {
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

    /// 昇順でn番目の要素を取得する
    pub fn get_nth(&self, mut n: usize) -> Option<&i32> {
        let mut cur = &self.root;
        while let Some(x) = cur.map(|x| unsafe { x.as_ref() }) {
            let left_len = Node::len(&x.left);
            if n == left_len {
                return Some(&x.value);
            } else if n < left_len {
                cur = &x.left;
            } else {
                cur = &x.right;
                n -= left_len + 1;
            }
        }

        None
    }

    pub fn append(&mut self, other: &mut Self) {
        let root = Node::merge(self.root.take(), other.root.take());
        *self = Self { root }
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter::new(&self.root)
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

impl<'a> IntoIterator for &'a AVLTreeSet {
    type IntoIter = Iter<'a>;
    type Item = &'a i32;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for AVLTreeSet {
    type IntoIter = IntoIter;
    type Item = i32;

    fn into_iter(self) -> Self::IntoIter {
        fn collect(node: Option<NonNull<Node>>, v: &mut Vec<i32>) {
            if let Some(node) = node {
                collect(unsafe { node.as_ref() }.left, v);
                v.push(unsafe { node.as_ref() }.value);
                collect(unsafe { node.as_ref() }.right, v);
            }
        }
        let mut res = vec![];
        collect(self.root, &mut res);
        IntoIter {
            iter: res.into_iter(),
        }
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

pub struct Iter<'a> {
    stack_left: Vec<&'a NonNull<Node>>,
    stack_right: Vec<&'a NonNull<Node>>,
}

impl<'a> Iter<'a> {
    fn new(root: &'a Option<NonNull<Node>>) -> Self {
        let mut iter = Self {
            stack_left: vec![],
            stack_right: vec![],
        };
        iter.push_left(root);
        iter.push_right(root);
        iter
    }

    fn push_left(&mut self, mut node: &'a Option<NonNull<Node>>) {
        while let Some(n) = node {
            self.stack_left.push(n);
            node = &unsafe { n.as_ref() }.left;
        }
    }

    fn push_right(&mut self, mut node: &'a Option<NonNull<Node>>) {
        while let Some(n) = node {
            self.stack_right.push(n);
            node = &unsafe { n.as_ref() }.right;
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a i32;

    fn next(&mut self) -> Option<Self::Item> {
        let node = unsafe { self.stack_left.pop()?.as_ref() };
        self.push_left(&node.right);
        Some(&node.value)
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let node = unsafe { self.stack_right.pop()?.as_ref() };
        self.push_right(&node.left);
        Some(&node.value)
    }
}

pub struct IntoIter {
    iter: std::vec::IntoIter<i32>,
}

impl Iterator for IntoIter {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl DoubleEndedIterator for IntoIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
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

    fn merge_with_root(
        mut left: Option<NonNull<Node>>,
        mut root: NonNull<Node>,
        mut right: Option<NonNull<Node>>,
    ) -> NonNull<Node> {
        let d = Node::height(&left) - Node::height(&right);

        if d.abs() <= 1 {
            unsafe {
                root.as_mut().left = left;
                root.as_mut().right = right;
                root.as_mut().update();
            }
            root
        } else if d > 0 {
            unsafe {
                left.unwrap().as_mut().right = Some(Node::merge_with_root(
                    left.unwrap().as_mut().right,
                    root,
                    right,
                ));
            }
            Node::balance(&mut left);
            left.unwrap()
        } else {
            unsafe {
                right.unwrap().as_mut().left = Some(Node::merge_with_root(
                    left,
                    root,
                    right.unwrap().as_mut().left,
                ));
            }
            Node::balance(&mut right);
            right.unwrap()
        }
    }

    fn merge(
        mut left: Option<NonNull<Node>>,
        right: Option<NonNull<Node>>,
    ) -> Option<NonNull<Node>> {
        if left.is_none() {
            right
        } else if right.is_none() {
            left
        } else {
            let (l, removed) = Node::remove_rightest(left);
            left = l;
            Some(Node::merge_with_root(left, removed.unwrap(), right))
        }
    }

    fn remove_rightest(
        mut node: Option<NonNull<Node>>,
    ) -> (Option<NonNull<Node>>, Option<NonNull<Node>>) {
        unsafe {
            if node.unwrap().as_ref().right.is_some() {
                let (x, removed) = Node::remove_rightest(node.unwrap().as_ref().right);
                node.unwrap().as_mut().right = x;
                Node::balance(&mut node);
                (node, removed)
            } else {
                let removed = node;
                (node.unwrap().as_ref().left, removed)
            }
        }
    }
}

impl AVLTreeSet {
    #[cfg(debug_assertions)]
    #[allow(unused)]
    pub fn visualize(&self) -> String {
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

#[cfg(test)]
mod tests {
    use super::AVLTreeSet;

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
        let mut tree = AVLTreeSet::from([52, 73, 63, 27, 44, 94, 31, 82, 70, 37]);
        assert!(tree
            .iter()
            .copied()
            .eq([27, 31, 37, 44, 52, 63, 70, 73, 82, 94]));
        assert!(tree.remove(&44));
        assert!(tree.remove(&52));
        assert!(tree.remove(&63));
        assert!(!tree.remove(&100));
        assert!(tree.remove(&82));
        assert!(!tree.remove(&44));
        assert!(tree.iter().copied().eq([27, 31, 37, 70, 73, 94]));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
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
                // 3: nth
                let t = rng.random_range(0..4);
                match t {
                    0 => {
                        let x = rng.random_range(-100..=100);
                        assert_eq!(b.insert(x), avl.insert(x));
                    }
                    1 => {
                        let x = rng.random_range(-100..=100);
                        assert_eq!(b.contains(&x), avl.contains(&x));
                    }
                    2 => {
                        let x = rng.random_range(-100..=100);
                        assert_eq!(b.remove(&x), avl.remove(&x));
                    }
                    3 => {
                        let k = rng.random_range(0..1000);
                        assert_eq!(b.iter().nth(k), avl.get_nth(k));
                    }
                    _ => {}
                }
                assert_eq!(b.len(), avl.len());
                assert!(avl.iter().eq(b.iter()));
                assert!(avl.iter().rev().eq(b.iter().rev()));
            }
            assert!(avl.into_iter().eq(b.into_iter()));
        }
    }
}
