//! # AVLTreeSet
//! ## 未実装
//! - get_nth_mut
//! - iter_mut
//! - range

pub mod visualizer;

use std::{
    cmp::Ordering,
    fmt::Debug,
    hash::Hash,
    mem::{swap, take},
    ptr::NonNull,
};

#[derive(Clone)]
pub struct AvlTreeSet {
    root: Option<NonNull<Node>>,
}

impl AvlTreeSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn len(&self) -> usize {
        node_len(&self.root)
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
            balance_node(node);
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
                                balance_node(node);
                                return true;
                            }
                        }
                    }
                    Ordering::Less => {
                        if remove(&mut x_ref.left, value) {
                            balance_node(node);
                            return true;
                        }
                    }
                    Ordering::Greater => {
                        if remove(&mut x_ref.right, value) {
                            balance_node(node);
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
            let left_len = node_len(&x.left);
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

    /// 降順でn番目の要素を取得する
    pub fn get_nth_back(&self, mut n: usize) -> Option<&i32> {
        let mut cur = &self.root;
        while let Some(x) = cur.map(|x| unsafe { x.as_ref() }) {
            let right_len = node_len(&x.right);
            if n == right_len {
                return Some(&x.value);
            } else if n < right_len {
                cur = &x.right;
            } else {
                cur = &x.left;
                n -= right_len + 1;
            }
        }
        None
    }

    /// NOTE: 2つのAVL木の要素数をN, Mに対してO(min(N+M)log N)
    pub fn append(&mut self, other: &mut AvlTreeSet) {
        fn insert(tree: &mut AvlTreeSet, node: Option<NonNull<Node>>) {
            if let Some(node) = node {
                insert(tree, unsafe { node.as_ref() }.left);
                insert(tree, unsafe { node.as_ref() }.right);
                tree.insert(unsafe { node.as_ref() }.value);
            }
        }

        if self.len() >= other.len() {
            insert(self, take(other).root);
        } else {
            insert(other, take(self).root);
            swap(self, other);
        }
    }

    /// NOTE: AVL木の要素数Nに対してO(Nlog N)
    /// ↑本当？　もっと効率的な方法があるのでは
    /// 実装も要改善
    pub fn split_off(&mut self, value: &i32) -> AvlTreeSet {
        fn insert(
            node: &mut Option<NonNull<Node>>,
            left: &mut AvlTreeSet,
            right: &mut AvlTreeSet,
            value: &i32,
        ) {
            if let Some(mut node) = node.take() {
                insert(&mut unsafe { node.as_mut() }.left, left, right, value);
                insert(&mut unsafe { node.as_mut() }.right, left, right, value);
                let node_value = unsafe { node.as_ref() }.value;
                if &node_value < value {
                    left.insert(node_value);
                } else {
                    right.insert(node_value);
                }
                unsafe { drop(Box::from_raw(node.as_ptr())) };
            }
        }

        let mut left = Self::new();
        let mut right = Self::new();
        insert(&mut self.root.take(), &mut left, &mut right, value);
        *self = left;
        right
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter::new(&self.root)
    }
}

impl Default for AvlTreeSet {
    fn default() -> Self {
        AvlTreeSet { root: None }
    }
}

impl Drop for AvlTreeSet {
    fn drop(&mut self) {
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

impl PartialEq for AvlTreeSet {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other)
    }
}

impl Eq for AvlTreeSet {}

impl PartialOrd for AvlTreeSet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl Ord for AvlTreeSet {
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other)
    }
}

impl Hash for AvlTreeSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.iter().for_each(|item| item.hash(state));
    }
}

impl<'a> IntoIterator for &'a AvlTreeSet {
    type IntoIter = Iter<'a>;
    type Item = &'a i32;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for AvlTreeSet {
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

impl From<Vec<i32>> for AvlTreeSet {
    fn from(v: Vec<i32>) -> Self {
        let mut res = Self::new();
        v.into_iter().for_each(|e| {
            res.insert(e);
        });
        res
    }
}

impl<const N: usize> From<[i32; N]> for AvlTreeSet {
    fn from(v: [i32; N]) -> Self {
        let mut res = Self::new();
        v.into_iter().for_each(|e| {
            res.insert(e);
        });
        res
    }
}

impl Debug for AvlTreeSet {
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
        self.len = node_len(&self.left) + node_len(&self.right) + 1;
        self.height = node_height(&self.left).max(node_height(&self.right)) + 1;
    }
}

fn node_len(node: &Option<NonNull<Node>>) -> usize {
    node.map_or(0, |node| unsafe { node.as_ref() }.len)
}

fn node_height(node: &Option<NonNull<Node>>) -> i32 {
    node.map_or(0, |node| unsafe { node.as_ref() }.height)
}

/// nodeを根とする部分木を平衡する
fn balance_node(node: &mut Option<NonNull<Node>>) {
    /// 左部分木と右部分木の高さの差
    /// 左部分木の高さ - 右部分木の高さ
    #[inline]
    fn diff_height(node: &NonNull<Node>) -> i32 {
        let node = unsafe { node.as_ref() };
        node_height(&node.left) - node_height(&node.right)
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

#[cfg(test)]
mod tests {
    use super::AvlTreeSet;

    #[test]
    fn test_insert_and_contains() {
        let mut tree = AvlTreeSet::new();
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
        let mut tree = AvlTreeSet::from([52, 73, 63, 27, 44, 94, 31, 82, 70, 37]);
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
    fn test_get_nth() {
        let tree = AvlTreeSet::from([1, 3, 5, 7, 9]);
        assert_eq!(tree.get_nth(0), Some(&1));
        assert_eq!(tree.get_nth(1), Some(&3));
        assert_eq!(tree.get_nth(2), Some(&5));
        assert_eq!(tree.get_nth(3), Some(&7));
        assert_eq!(tree.get_nth(4), Some(&9));
        assert_eq!(tree.get_nth(5), None);
    }

    #[test]
    fn test_get_nth_back() {
        let tree = AvlTreeSet::from([2, 4, 6, 8, 10]);
        assert_eq!(tree.get_nth_back(0), Some(&10));
        assert_eq!(tree.get_nth_back(1), Some(&8));
        assert_eq!(tree.get_nth_back(2), Some(&6));
        assert_eq!(tree.get_nth_back(3), Some(&4));
        assert_eq!(tree.get_nth_back(4), Some(&2));
        assert_eq!(tree.get_nth_back(5), None);
    }

    #[test]
    fn test_append() {
        let mut tree1 = AvlTreeSet::from([1, 3, 5]);
        let mut tree2 = AvlTreeSet::from([2, 4, 6]);
        tree1.append(&mut tree2);
        assert!(tree1.iter().copied().eq([1, 2, 3, 4, 5, 6]));
        assert!(tree2.is_empty());

        let mut tree1 = AvlTreeSet::new();
        let mut tree2 = AvlTreeSet::new();
        tree1.append(&mut tree2);
        assert!(tree1.is_empty());
        assert!(tree2.is_empty());
        tree1.insert(10);
        tree1.append(&mut AvlTreeSet::new());
        assert!(tree1.iter().copied().eq([10]));
        assert!(tree2.is_empty());

        let mut tree1 = AvlTreeSet::new();
        let mut tree2 = AvlTreeSet::from([7, 8]);
        tree1.append(&mut tree2);
        assert!(tree1.iter().copied().eq([7, 8]));
        assert!(tree2.is_empty());

        let mut tree1 = AvlTreeSet::from([2, 4, 6]);
        let mut tree2 = AvlTreeSet::from([3, 4, 5]);
        tree1.append(&mut tree2);
        assert!(tree1.iter().copied().eq([2, 3, 4, 5, 6]));
        assert!(tree2.is_empty());
    }

    #[test]
    fn test_split_off() {
        let mut tree1 = AvlTreeSet::from([1, 2, 3, 4, 5, 6]);
        let tree2 = tree1.split_off(&4);
        assert!(tree1.iter().copied().eq([1, 2, 3]));
        assert!(tree2.iter().copied().eq([4, 5, 6]));

        let mut tree1 = AvlTreeSet::from([2, 4, 6, 8, 10]);
        let tree2 = tree1.split_off(&5);
        assert!(tree1.iter().copied().eq([2, 4]));
        assert!(tree2.iter().copied().eq([6, 8, 10]));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_random() {
        use rand::{rng, Rng};
        use std::collections::BTreeSet;

        let mut rng = rng();

        for _ in 0..5 {
            let mut avl = AvlTreeSet::new();
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
