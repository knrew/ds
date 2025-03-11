//! # AVLTreeMap
//! ## 未実装
//! - get_nth_mut
//! - iter_mut
//! - range
//! - index
//! ## NOTE
//! 今一旦Copyをつけてるけどあとで外す

use std::{
    cmp::Ordering,
    fmt::Debug,
    hash::Hash,
    mem::{swap, take},
    ptr::NonNull,
};

#[derive(Clone)]
pub struct AvlTreeMap<T, U> {
    root: Option<NonNull<Node<T, U>>>,
}

impl<T, U> AvlTreeMap<T, U> {
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

    pub fn insert(&mut self, key: T, value: U) -> bool
    where
        T: Ord,
    {
        fn insert<T: Ord, U>(node: &mut Option<NonNull<Node<T, U>>>, key: T, value: U) -> bool {
            if let Some(node) = node.map(|mut node| unsafe { node.as_mut() }) {
                match key.cmp(&node.key) {
                    Ordering::Equal => {
                        return false;
                    }
                    Ordering::Less => {
                        if !insert(&mut node.left, key, value) {
                            return false;
                        }
                    }
                    Ordering::Greater => {
                        if !insert(&mut node.right, key, value) {
                            return false;
                        }
                    }
                }
            } else {
                *node = Some(Node::new(key, value));
            }
            balance(node);
            true
        }
        insert(&mut self.root, key, value)
    }

    pub fn contains_key(&self, key: &T) -> bool
    where
        T: Ord,
    {
        let mut p = &self.root;
        while let Some(node) = p.map(|node| unsafe { node.as_ref() }) {
            match key.cmp(&node.key) {
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

    pub fn remove(&mut self, key: &T) -> bool
    where
        T: Ord + Copy,
    {
        fn remove<T: Ord + Copy, U>(node: &mut Option<NonNull<Node<T, U>>>, key: &T) -> bool {
            if let Some(x) = node {
                let x_ref = unsafe { x.as_mut() };
                match key.cmp(&x_ref.key) {
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
                            x_ref.key = unsafe { right.as_ref().key };
                            if remove(&mut x_ref.right, &x_ref.key) {
                                balance(node);
                                return true;
                            }
                        }
                    }
                    Ordering::Less => {
                        if remove(&mut x_ref.left, key) {
                            balance(node);
                            return true;
                        }
                    }
                    Ordering::Greater => {
                        if remove(&mut x_ref.right, key) {
                            balance(node);
                            return true;
                        }
                    }
                }
            }
            false
        }

        remove(&mut self.root, key)
    }

    pub fn get(&self, key: &T) -> Option<&U>
    where
        T: Ord,
    {
        let mut p = &self.root;
        while let Some(node) = p.map(|node| unsafe { node.as_ref() }) {
            match key.cmp(&node.key) {
                Ordering::Equal => {
                    return Some(&node.value);
                }
                Ordering::Less => {
                    p = &node.left;
                }
                Ordering::Greater => {
                    p = &node.right;
                }
            }
        }
        None
    }

    pub fn get_mut(&mut self, key: &T) -> Option<&mut U>
    where
        T: Ord,
    {
        let mut p = &mut self.root;
        while let Some(node) = p.map(|mut node| unsafe { node.as_mut() }) {
            match key.cmp(&node.key) {
                Ordering::Equal => {
                    return Some(&mut node.value);
                }
                Ordering::Less => {
                    p = &mut node.left;
                }
                Ordering::Greater => {
                    p = &mut node.right;
                }
            }
        }
        None
    }

    // /// 昇順でn番目の要素を取得する
    // pub fn get_nth(&self, mut n: usize) -> Option<(&T, &U)> {
    //     let mut cur = &self.root;
    //     while let Some(x) = cur.map(|x| unsafe { x.as_ref() }) {
    //         let left_len = Node::len(&x.left);
    //         if n == left_len {
    //             return Some((&x.key, &x.value));
    //         } else if n < left_len {
    //             cur = &x.left;
    //         } else {
    //             cur = &x.right;
    //             n -= left_len + 1;
    //         }
    //     }
    //     None
    // }

    // /// 降順でn番目の要素を取得する
    // pub fn get_nth_back(&self, mut n: usize) -> Option<(&T, &U)> {
    //     let mut cur = &self.root;
    //     while let Some(x) = cur.map(|x| unsafe { x.as_ref() }) {
    //         let right_len = Node::len(&x.right);
    //         if n == right_len {
    //             return Some((&x.key, &x.value));
    //         } else if n < right_len {
    //             cur = &x.right;
    //         } else {
    //             cur = &x.left;
    //             n -= right_len + 1;
    //         }
    //     }
    //     None
    // }

    /// NOTE: 2つのAVL木の要素数をN, Mに対してO(min(N+M)log N)
    pub fn append(&mut self, other: &mut Self)
    where
        T: Ord + Copy,
        U: Copy,
    {
        fn insert<T: Ord + Copy, U: Copy>(
            tree: &mut AvlTreeMap<T, U>,
            node: Option<NonNull<Node<T, U>>>,
        ) {
            if let Some(node) = node {
                insert(tree, unsafe { node.as_ref() }.left);
                insert(tree, unsafe { node.as_ref() }.right);
                tree.insert(unsafe { node.as_ref() }.key, unsafe { node.as_ref() }.value);
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
    pub fn split_off(&mut self, key: &T) -> Self
    where
        T: Ord + Copy,
        U: Copy,
    {
        fn insert<T: Ord + Copy, U: Copy>(
            node: &mut Option<NonNull<Node<T, U>>>,
            left: &mut AvlTreeMap<T, U>,
            right: &mut AvlTreeMap<T, U>,
            key: &T,
        ) {
            if let Some(mut node) = node.take() {
                insert(&mut unsafe { node.as_mut() }.left, left, right, key);
                insert(&mut unsafe { node.as_mut() }.right, left, right, key);
                let node_key = unsafe { node.as_ref() }.key;
                let node_value = unsafe { node.as_ref() }.value;
                if &node_key < key {
                    left.insert(node_key, node_value);
                } else {
                    right.insert(node_key, node_value);
                }
                unsafe { drop(Box::from_raw(node.as_ptr())) };
            }
        }

        let mut left = Self::new();
        let mut right = Self::new();
        insert(&mut self.root.take(), &mut left, &mut right, key);
        *self = left;
        right
    }

    pub fn iter(&self) -> Iter<'_, T, U> {
        Iter::new(&self.root)
    }
}

impl<T, U> Default for AvlTreeMap<T, U> {
    fn default() -> Self {
        AvlTreeMap { root: None }
    }
}

impl<T, U> Drop for AvlTreeMap<T, U> {
    fn drop(&mut self) {
        fn free<T, U>(node: &mut Option<NonNull<Node<T, U>>>) {
            if let Some(mut node) = node.take() {
                free(&mut unsafe { node.as_mut() }.left);
                free(&mut unsafe { node.as_mut() }.right);
                unsafe { drop(Box::from_raw(node.as_ptr())) };
            }
        }
        free(&mut self.root)
    }
}

impl<T: PartialEq, U: PartialEq> PartialEq for AvlTreeMap<T, U> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other)
    }
}

impl<T: Eq, U: Eq> Eq for AvlTreeMap<T, U> {}

impl<T: PartialOrd, U: PartialOrd> PartialOrd for AvlTreeMap<T, U> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T: Ord, U: Ord> Ord for AvlTreeMap<T, U> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other)
    }
}

impl<T: Hash, U: Hash> Hash for AvlTreeMap<T, U> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.iter().for_each(|item| item.hash(state));
    }
}

impl<'a, T, U> IntoIterator for &'a AvlTreeMap<T, U> {
    type IntoIter = Iter<'a, T, U>;
    type Item = (&'a T, &'a U);

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: Copy, U: Copy> IntoIterator for AvlTreeMap<T, U> {
    type IntoIter = IntoIter<T, U>;
    type Item = (T, U);

    fn into_iter(self) -> Self::IntoIter {
        fn collect<T: Copy, U: Copy>(node: Option<NonNull<Node<T, U>>>, v: &mut Vec<(T, U)>) {
            if let Some(node) = node {
                collect(unsafe { node.as_ref() }.left, v);
                v.push((unsafe { node.as_ref() }.key, unsafe { node.as_ref() }.value));
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

impl<T: Ord, U> From<Vec<(T, U)>> for AvlTreeMap<T, U> {
    fn from(v: Vec<(T, U)>) -> Self {
        let mut res = Self::new();
        v.into_iter().for_each(|(k, v)| {
            res.insert(k, v);
        });
        res
    }
}

impl<T: Ord, U, const N: usize> From<[(T, U); N]> for AvlTreeMap<T, U> {
    fn from(v: [(T, U); N]) -> Self {
        let mut res = Self::new();
        v.into_iter().for_each(|(k, v)| {
            res.insert(k, v);
        });
        res
    }
}

impl<T: Debug, U: Debug> Debug for AvlTreeMap<T, U> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

pub struct Iter<'a, T, U> {
    stack_left: Vec<&'a NonNull<Node<T, U>>>,
    stack_right: Vec<&'a NonNull<Node<T, U>>>,
}

impl<'a, T, U> Iter<'a, T, U> {
    fn new(root: &'a Option<NonNull<Node<T, U>>>) -> Self {
        let mut iter = Self {
            stack_left: vec![],
            stack_right: vec![],
        };
        iter.push_left(root);
        iter.push_right(root);
        iter
    }

    fn push_left(&mut self, mut node: &'a Option<NonNull<Node<T, U>>>) {
        while let Some(n) = node {
            self.stack_left.push(n);
            node = &unsafe { n.as_ref() }.left;
        }
    }

    fn push_right(&mut self, mut node: &'a Option<NonNull<Node<T, U>>>) {
        while let Some(n) = node {
            self.stack_right.push(n);
            node = &unsafe { n.as_ref() }.right;
        }
    }
}

impl<'a, T, U> Iterator for Iter<'a, T, U> {
    type Item = (&'a T, &'a U);

    fn next(&mut self) -> Option<Self::Item> {
        let node = unsafe { self.stack_left.pop()?.as_ref() };
        self.push_left(&node.right);
        Some((&node.key, &node.value))
    }
}

impl<'a, T, U> DoubleEndedIterator for Iter<'a, T, U> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let node = unsafe { self.stack_right.pop()?.as_ref() };
        self.push_right(&node.left);
        Some((&node.key, &node.value))
    }
}

pub struct IntoIter<T, U> {
    iter: std::vec::IntoIter<(T, U)>,
}

impl<T, U> Iterator for IntoIter<T, U> {
    type Item = (T, U);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<T, U> DoubleEndedIterator for IntoIter<T, U> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

struct Node<T, U> {
    key: T,
    value: U,
    len: usize,
    height: i32,
    left: Option<NonNull<Node<T, U>>>,
    right: Option<NonNull<Node<T, U>>>,
}

impl<T, U> Node<T, U> {
    fn new(key: T, value: U) -> NonNull<Self> {
        let node = Self {
            key,
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

    fn len(node: &Option<NonNull<Self>>) -> usize {
        node.map_or(0, |node| unsafe { node.as_ref() }.len)
    }

    fn height(node: &Option<NonNull<Self>>) -> i32 {
        node.map_or(0, |node| unsafe { node.as_ref() }.height)
    }
}

/// 平衡
fn balance<T, U>(node: &mut Option<NonNull<Node<T, U>>>) {
    /// 左部分木と右部分木の高さの差
    /// 左部分木の高さ - 右部分木の高さ
    #[inline]
    fn diff_height<T, U>(node: &NonNull<Node<T, U>>) -> i32 {
        let node = unsafe { node.as_ref() };
        Node::height(&node.left) - Node::height(&node.right)
    }

    /// 木を右回転させる
    fn rotate_right<T, U>(root: &mut Option<NonNull<Node<T, U>>>) {
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
    fn rotate_left<T, U>(root: &mut Option<NonNull<Node<T, U>>>) {
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

#[cfg(debug_assertions)]
use std::fmt::Display;
impl<T: Display, U> AvlTreeMap<T, U> {
    #[cfg(debug_assertions)]
    #[allow(unused)]
    pub fn visualize(&self) -> String {
        fn visualize<T: Display, U>(
            node: &Option<NonNull<Node<T, U>>>,
            prefix: &str,
            is_root: bool,
            is_last: bool,
            res: &mut String,
        ) {
            if let Some(node) = node.map(|node| unsafe { node.as_ref() }) {
                if is_root {
                    *res += &format!("{}\n", node.key);
                } else {
                    *res += &format!(
                        "{}{}{}\n",
                        prefix,
                        if is_last { "└── " } else { "├── " },
                        node.key
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
    use super::AvlTreeMap;

    #[test]
    fn test_insert_and_contains() {
        let mut tree = AvlTreeMap::new();
        assert!(!tree.contains_key(&3));
        assert!(!tree.contains_key(&1));
        assert!(!tree.contains_key(&4));
        assert!(!tree.contains_key(&5));
        assert!(!tree.contains_key(&100));
        assert!(tree.insert(3, 2));
        assert!(tree.insert(1, 7));
        assert!(tree.insert(4, 1));
        assert!(!tree.insert(1, 8));
        assert!(tree.insert(5, 2));
        assert!(tree.contains_key(&3));
        assert!(tree.contains_key(&1));
        assert!(tree.contains_key(&4));
        assert!(tree.contains_key(&5));
        assert!(!tree.contains_key(&100));
    }

    #[test]
    fn test_remove() {
        // let mut tree = AvlTreeMap::from([52, 73, 63, 27, 44, 94, 31, 82, 70, 37]);
        // assert!(tree
        //     .iter()
        //     .copied()
        //     .eq([27, 31, 37, 44, 52, 63, 70, 73, 82, 94]));
        // assert!(tree.remove(&44));
        // assert!(tree.remove(&52));
        // assert!(tree.remove(&63));
        // assert!(!tree.remove(&100));
        // assert!(tree.remove(&82));
        // assert!(!tree.remove(&44));
        // assert!(tree.iter().copied().eq([27, 31, 37, 70, 73, 94]));
    }

    #[test]
    fn test_get_nth() {
        // let tree = AvlTreeMap::from([1, 3, 5, 7, 9]);
        // assert_eq!(tree.get_nth(0), Some(&1));
        // assert_eq!(tree.get_nth(1), Some(&3));
        // assert_eq!(tree.get_nth(2), Some(&5));
        // assert_eq!(tree.get_nth(3), Some(&7));
        // assert_eq!(tree.get_nth(4), Some(&9));
        // assert_eq!(tree.get_nth(5), None);
    }

    #[test]
    fn test_get_nth_back() {
        // let tree = AvlTreeMap::from([2, 4, 6, 8, 10]);
        // assert_eq!(tree.get_nth_back(0), Some(&10));
        // assert_eq!(tree.get_nth_back(1), Some(&8));
        // assert_eq!(tree.get_nth_back(2), Some(&6));
        // assert_eq!(tree.get_nth_back(3), Some(&4));
        // assert_eq!(tree.get_nth_back(4), Some(&2));
        // assert_eq!(tree.get_nth_back(5), None);
    }

    #[test]
    fn test_append() {
        // let mut tree1 = AvlTreeMap::from([1, 3, 5]);
        // let mut tree2 = AvlTreeMap::from([2, 4, 6]);
        // tree1.append(&mut tree2);
        // assert!(tree1.iter().copied().eq([1, 2, 3, 4, 5, 6]));
        // assert!(tree2.is_empty());

        // let mut tree1 = AvlTreeMap::new();
        // let mut tree2 = AvlTreeMap::new();
        // tree1.append(&mut tree2);
        // assert!(tree1.is_empty());
        // assert!(tree2.is_empty());
        // tree1.insert(10);
        // tree1.append(&mut AvlTreeMap::new());
        // assert!(tree1.iter().copied().eq([10]));
        // assert!(tree2.is_empty());

        // let mut tree1 = AvlTreeMap::new();
        // let mut tree2 = AvlTreeMap::from([7, 8]);
        // tree1.append(&mut tree2);
        // assert!(tree1.iter().copied().eq([7, 8]));
        // assert!(tree2.is_empty());

        // let mut tree1 = AvlTreeMap::from([2, 4, 6]);
        // let mut tree2 = AvlTreeMap::from([3, 4, 5]);
        // tree1.append(&mut tree2);
        // assert!(tree1.iter().copied().eq([2, 3, 4, 5, 6]));
        // assert!(tree2.is_empty());
    }

    #[test]
    fn test_split_off() {
        // let mut tree1 = AvlTreeMap::from([1, 2, 3, 4, 5, 6]);
        // let tree2 = tree1.split_off(&4);
        // assert!(tree1.iter().copied().eq([1, 2, 3]));
        // assert!(tree2.iter().copied().eq([4, 5, 6]));

        // let mut tree1 = AvlTreeMap::from([2, 4, 6, 8, 10]);
        // let tree2 = tree1.split_off(&5);
        // assert!(tree1.iter().copied().eq([2, 4]));
        // assert!(tree2.iter().copied().eq([6, 8, 10]));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_random() {
        // use rand::{rng, Rng};
        // use std::collections::BTreeSet;

        // let mut rng = rng();

        // for _ in 0..5 {
        //     let mut avl = AvlTreeMap::new();
        //     let mut b = BTreeSet::new();
        //     for _ in 0..1000 {
        //         // 0: insert
        //         // 1: contains
        //         // 2: remove
        //         // 3: nth
        //         let t = rng.random_range(0..4);
        //         match t {
        //             0 => {
        //                 let x = rng.random_range(-100..=100);
        //                 assert_eq!(b.insert(x), avl.insert(x));
        //             }
        //             1 => {
        //                 let x = rng.random_range(-100..=100);
        //                 assert_eq!(b.contains(&x), avl.contains_key(&x));
        //             }
        //             2 => {
        //                 let x = rng.random_range(-100..=100);
        //                 assert_eq!(b.remove(&x), avl.remove(&x));
        //             }
        //             3 => {
        //                 let k = rng.random_range(0..1000);
        //                 assert_eq!(b.iter().nth(k), avl.get_nth(k));
        //             }
        //             _ => {}
        //         }
        //         assert_eq!(b.len(), avl.len());
        //         assert!(avl.iter().eq(b.iter()));
        //         assert!(avl.iter().rev().eq(b.iter().rev()));
        //     }
        //     assert!(avl.into_iter().eq(b.into_iter()));
        // }
    }
}
