//! AVL木によるordered setの実装
//! # NOTE
//! - rangeの計算量が悪い
//! - (全体のk番目は取得できるが)指定した範囲のk番目を取得できない(逆も然り)

use std::{
    borrow::Borrow,
    cmp::Ordering,
    fmt::{Debug, Display},
    hash::Hash,
    mem::{swap, take},
    ops::{Bound, RangeBounds, RangeFull},
    ptr::NonNull,
};

#[derive(Clone)]
struct Node<T> {
    key: T,
    len: usize,
    height: i32,
    left: Link<T>,
    right: Link<T>,
}

type NodePtr<T> = NonNull<Node<T>>;
type Link<T> = Option<NodePtr<T>>;

impl<T> Node<T> {
    fn new(key: T) -> NodePtr<T> {
        let node = Self {
            key,
            len: 1,
            height: 1,
            left: None,
            right: None,
        };
        NonNull::from(Box::leak(Box::new(node)))
    }

    /// 部分木が更新された場合に呼ぶ
    /// 部分木の長さと高さを再計算する
    #[inline]
    fn fetch(&mut self) {
        self.len = node_len(self.left) + node_len(self.right) + 1;
        self.height = node_height(self.left).max(node_height(self.right)) + 1;
    }
}

#[inline]
fn free<T>(node: NodePtr<T>) {
    unsafe { drop(Box::from_raw(node.as_ptr())) };
}

#[inline]
fn node_len<T>(node: Link<T>) -> usize {
    node.map_or(0, |node| unsafe { node.as_ref() }.len)
}

#[inline]
fn node_height<T>(node: Link<T>) -> i32 {
    node.map_or(0, |node| unsafe { node.as_ref() }.height)
}

/// rootを根とした部分木を右回転させる
/// (左の子が存在している場合のみ呼び出す)
fn rotate_right<T>(root: &mut Link<T>) {
    *root = {
        unsafe {
            let mut root = root.unwrap();
            let mut left = root.as_ref().left.unwrap();
            root.as_mut().left = left.as_mut().right;
            root.as_mut().fetch();
            left.as_mut().right = Some(root);
            left.as_mut().fetch();
            Some(left)
        }
    };
}

/// rootを根とした部分木を左回転させる
/// (右の子が存在する場合のみ呼び出す)
fn rotate_left<T>(root: &mut Link<T>) {
    *root = {
        unsafe {
            let mut root = root.unwrap();
            let mut right = root.as_ref().right.unwrap();
            root.as_mut().right = right.as_mut().left;
            root.as_mut().fetch();
            right.as_mut().left = Some(root);
            right.as_mut().fetch();
            Some(right)
        }
    };
}

/// rootを根とした部分木を平衡させる
fn balance<T>(root: &mut Link<T>) {
    /// 左部分木と右部分木の高さの差
    /// 左部分木の高さ - 右部分木の高さ
    #[inline]
    fn diff_height<T>(node: Link<T>) -> i32 {
        node.map_or(0, |node| {
            let node = unsafe { node.as_ref() };
            node_height(node.left) - node_height(node.right)
        })
    }

    if root.is_none() {
        return;
    }

    let d = diff_height(*root);

    if d > 1 {
        // 左部分木が高い場合

        let left = &mut unsafe { root.unwrap().as_mut() }.left;
        if diff_height(*left) < 0 {
            rotate_left(left);
        }

        rotate_right(root);
    } else if d < -1 {
        // 右部分木が高い場合

        let right = &mut unsafe { root.unwrap().as_mut() }.right;
        if diff_height(*right) > 0 {
            rotate_right(right);
        }

        rotate_left(root);
    } else {
        unsafe { root.unwrap().as_mut() }.fetch();
    }
}

#[allow(unused)]
fn traverse<T>(
    node: Link<T>,
    mut preorder_f: impl FnMut(NodePtr<T>),
    mut inorder_f: impl FnMut(NodePtr<T>),
    mut postorder_f: impl FnMut(NodePtr<T>),
) {
    fn traverse<T>(
        node: Link<T>,
        preorder_f: &mut impl FnMut(NodePtr<T>),
        inorder_f: &mut impl FnMut(NodePtr<T>),
        postorder_f: &mut impl FnMut(NodePtr<T>),
    ) {
        if let Some(node) = node {
            let left = unsafe { node.as_ref() }.left;
            let right = unsafe { node.as_ref() }.right;
            preorder_f(node);
            traverse(left, preorder_f, inorder_f, postorder_f);
            inorder_f(node);
            traverse(right, preorder_f, inorder_f, postorder_f);
            postorder_f(node);
        }
    }

    traverse(node, &mut preorder_f, &mut inorder_f, &mut postorder_f);
}

#[allow(unused)]
#[inline]
fn traverse_preorder<T>(node: Link<T>, f: impl FnMut(NodePtr<T>)) {
    traverse(node, f, |_| {}, |_| {});
}

#[allow(unused)]
#[inline]
fn traverse_inorder<T>(node: Link<T>, f: impl FnMut(NodePtr<T>)) {
    traverse(node, |_| {}, f, |_| {});
}

#[allow(unused)]
#[inline]
fn traverse_postorder<T>(node: Link<T>, f: impl FnMut(NodePtr<T>)) {
    traverse(node, |_| {}, |_| {}, f);
}

/// rootに新しいノードを挿入する
/// すでにnew_nodeと同じ値のノードが存在する場合は挿入せずnew_nodeのメモリを解放する
fn insert_node<T: Ord>(root: &mut Link<T>, new_node: NodePtr<T>) -> bool {
    fn insert<T: Ord>(root: &mut Link<T>, mut new_node: NodePtr<T>) -> bool {
        if let Some(node) = root.map(|mut node| unsafe { node.as_mut() }) {
            match unsafe { new_node.as_ref().key.borrow() }.cmp(&node.key) {
                Ordering::Equal => {
                    free(new_node);
                    return false;
                }
                Ordering::Less => {
                    if !insert(&mut node.left, new_node) {
                        return false;
                    }
                }
                Ordering::Greater => {
                    if !insert(&mut node.right, new_node) {
                        return false;
                    }
                }
            }
            balance(root);
        } else {
            unsafe {
                new_node.as_mut().left = None;
                new_node.as_mut().right = None;
            }
            *root = Some(new_node);
        }

        true
    }

    insert(root, new_node)
}

/// AVL木によるordered setの実装
#[derive(Clone)]
pub struct AvlTreeSet<T> {
    root: Link<T>,
}

impl<T> AvlTreeSet<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn len(&self) -> usize {
        node_len(self.root)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn insert(&mut self, key: T) -> bool
    where
        T: Ord,
    {
        insert_node(&mut self.root, Node::new(key))
    }

    pub fn contains(&self, key: &T) -> bool
    where
        T: Ord,
    {
        fn contains<T: Ord>(node: Link<T>, key: &T) -> bool {
            if let Some(node) = node.map(|node| unsafe { node.as_ref() }) {
                match key.cmp(&node.key) {
                    Ordering::Equal => true,
                    Ordering::Less => contains(node.left, key),
                    Ordering::Greater => contains(node.right, key),
                }
            } else {
                false
            }
        }
        contains(self.root, key)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Ord,
    {
        fn remove_min<T>(node: &mut Link<T>) -> Link<T> {
            unsafe {
                let left = &mut node.unwrap().as_mut().left;
                let left_left = &mut left.unwrap().as_mut().left;
                let res = if left_left.is_none() {
                    let res = *left;
                    node.unwrap().as_mut().left = left.unwrap().as_ref().right;
                    balance(node);
                    res
                } else {
                    remove_min(left)
                };
                balance(node);
                res
            }
        }

        fn remove<T, Q>(node: &mut Link<T>, key: &Q) -> Link<T>
        where
            T: Borrow<Q>,
            Q: Ord,
        {
            let res = if let Some(raw_node) = node.map(|mut node| unsafe { node.as_mut() }) {
                match key.cmp(raw_node.key.borrow()) {
                    Ordering::Equal => {
                        if raw_node.left.is_none() {
                            let mut right = raw_node.right;
                            // balance(node);
                            swap(node, &mut right);
                            right
                        } else if raw_node.right.is_none() {
                            let mut left = raw_node.left;
                            // balance(node);
                            swap(node, &mut left);
                            left
                        } else {
                            unsafe {
                                let right = &mut raw_node.right;
                                let right_left = &mut right.unwrap().as_mut().left;

                                let mut removed = if right_left.is_none() {
                                    right.unwrap().as_mut().left = node.unwrap().as_ref().left;
                                    *right
                                } else {
                                    let removed = remove_min(right);
                                    removed.unwrap().as_mut().left = node.unwrap().as_ref().left;
                                    removed.unwrap().as_mut().right = node.unwrap().as_ref().right;
                                    removed
                                };

                                swap(node, &mut removed);
                                removed
                            }
                        }
                    }
                    Ordering::Less => remove(&mut raw_node.left, key),
                    Ordering::Greater => remove(&mut raw_node.right, key),
                }
            } else {
                None
            };

            balance(node);
            res
        }

        remove(&mut self.root, key).map(|node| free(node)).is_some()
    }

    /// 昇順n番目の要素
    pub fn get_nth(&self, mut n: usize) -> Option<&T>
    where
        T: Ord,
    {
        let mut cur = &self.root;
        while let Some(node) = cur.map(|node| unsafe { node.as_ref() }) {
            let left_len = node_len(node.left);
            if n == left_len {
                return Some(&node.key);
            } else if n < left_len {
                cur = &node.left;
            } else {
                cur = &node.right;
                n -= left_len + 1;
            }
        }
        None
    }

    /// 降順n番目の要素
    pub fn get_nth_back(&self, mut n: usize) -> Option<&T>
    where
        T: Ord,
    {
        let mut cur = &self.root;
        while let Some(node) = cur.map(|node| unsafe { node.as_ref() }) {
            let right_len = node_len(node.right);
            if n == right_len {
                return Some(&node.key);
            } else if n < right_len {
                cur = &node.right;
            } else {
                cur = &node.left;
                n -= right_len + 1;
            }
        }
        None
    }

    // key以上最小の要素
    pub fn lower_bound(&self, key: &T) -> Option<&T>
    where
        T: Ord,
    {
        let mut cur = &self.root;
        let mut res = None;

        while let Some(node) = cur.map(|node| unsafe { node.as_ref() }) {
            match key.cmp(&node.key) {
                Ordering::Equal | Ordering::Less => {
                    res = cur.map(|node| &unsafe { node.as_ref() }.key);
                    cur = &node.left;
                }
                _ => cur = &node.right,
            }
        }

        res
    }

    pub fn range<'a, B>(&'a self, range: B) -> RangeIter<'a, T, B>
    where
        T: Ord,
        B: RangeBounds<T>,
    {
        RangeIter::new(&self.root, range)
    }

    /// NOTE: 2つのAVL木の要素数をN, Mに対してO(min(N+M)log N)
    pub fn append(&mut self, other: &mut Self)
    where
        T: Ord,
    {
        if self.len() < other.len() {
            swap(self, other);
        }

        traverse_postorder(other.root.take(), |node| {
            insert_node(&mut self.root, node);
        });
    }

    /// NOTE: AVL木の要素数Nに対してO(N log N)
    /// ↑本当？　もっと効率的な方法があるのでは
    /// AVL木の性質を利用したいが単純な分割では木のバランスが崩れる
    pub fn split_off(&mut self, key: &T) -> Self
    where
        T: Ord,
    {
        let mut left = None;
        let mut right = None;

        traverse_postorder(self.root.take(), |node| {
            match unsafe { node.as_ref() }.key.borrow().cmp(&key) {
                Ordering::Less => {
                    insert_node(&mut left, node);
                }
                _ => {
                    insert_node(&mut right, node);
                }
            }
        });

        *self = Self { root: left };
        Self { root: right }
    }

    pub fn iter(&self) -> RangeIter<'_, T, RangeFull>
    where
        T: Ord,
    {
        RangeIter::new(&self.root, ..)
    }
}

impl<T> Drop for AvlTreeSet<T> {
    fn drop(&mut self) {
        traverse_postorder(self.root, |node| free(node));
    }
}

impl<T> Default for AvlTreeSet<T> {
    fn default() -> Self {
        Self { root: None }
    }
}

impl<T: Ord + PartialEq> PartialEq for AvlTreeSet<T> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<T: Ord + Eq> Eq for AvlTreeSet<T> {}

impl<T: Ord + PartialOrd> PartialOrd for AvlTreeSet<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<T: Ord> Ord for AvlTreeSet<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<T: Ord + Hash> Hash for AvlTreeSet<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.iter().for_each(|item| item.hash(state));
    }
}

impl<'a, T: Ord> IntoIterator for &'a AvlTreeSet<T> {
    type IntoIter = RangeIter<'a, T, RangeFull>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> IntoIterator for AvlTreeSet<T> {
    type IntoIter = IntoIter<T>;
    type Item = T;

    fn into_iter(mut self) -> Self::IntoIter {
        IntoIter::new(self.root.take())
    }
}

impl<T: Ord> Extend<T> for AvlTreeSet<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        iter.into_iter().for_each(|x| {
            self.insert(x);
        });
    }
}

impl<'a, T: 'a + Ord + Copy> Extend<&'a T> for AvlTreeSet<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned());
    }
}

impl<T: Ord> FromIterator<T> for AvlTreeSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut res = Self::new();
        res.extend(iter);
        res
    }
}

impl<T: Ord> From<Vec<T>> for AvlTreeSet<T> {
    fn from(v: Vec<T>) -> Self {
        Self::from_iter(v)
    }
}

impl<T: Ord, const N: usize> From<[T; N]> for AvlTreeSet<T> {
    fn from(v: [T; N]) -> Self {
        Self::from_iter(v)
    }
}

impl<T: Ord + Debug> Debug for AvlTreeSet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

// TODO: ???
pub struct IntoIter<T> {
    iter: std::vec::IntoIter<NodePtr<T>>,
}

impl<T> IntoIter<T> {
    fn new(root: Link<T>) -> Self {
        let mut stack = Vec::with_capacity(node_len(root));
        traverse_inorder(root, |node| {
            stack.push(node);
        });
        IntoIter {
            iter: stack.into_iter(),
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.iter.next()?;
        let boxed = unsafe { Box::from_raw(node.as_ptr()) };
        Some(boxed.key)
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let node = self.iter.next_back()?;
        let boxed = unsafe { Box::from_raw(node.as_ptr()) };
        Some(boxed.key)
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        for node in take(&mut self.iter) {
            free(node)
        }
    }
}

// TODO: 全部走査しているので効率が悪い
pub struct RangeIter<'a, T, B> {
    stack_left: Vec<&'a NodePtr<T>>,
    stack_right: Vec<&'a NodePtr<T>>,
    range: B,
}

impl<'a, T: Ord, B: RangeBounds<T>> RangeIter<'a, T, B> {
    fn new(root: &'a Link<T>, range: B) -> Self {
        let mut res = Self {
            stack_left: vec![],
            stack_right: vec![],
            range,
        };
        res.push_left(root);
        res.push_right(root);
        res
    }

    fn push_left(&mut self, mut node: &'a Link<T>) {
        while let Some(node_ptr) = node {
            self.stack_left.push(node_ptr);
            node = &unsafe { node_ptr.as_ref() }.left;
        }
    }

    fn push_right(&mut self, mut node: &'a Link<T>) {
        while let Some(node_ptr) = node {
            self.stack_right.push(node_ptr);
            node = &unsafe { node_ptr.as_ref() }.right;
        }
    }

    fn in_range(&self, key: &T) -> bool {
        match self.range.start_bound() {
            Bound::Included(start) if key < start => return false,
            Bound::Excluded(start) if key <= start => return false,
            _ => {}
        }

        match self.range.end_bound() {
            Bound::Included(end) if key > end => return false,
            Bound::Excluded(end) if key >= end => return false,
            _ => {}
        }

        true
    }
}

impl<'a, T: Ord, B: RangeBounds<T>> Iterator for RangeIter<'a, T, B> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let node = unsafe { self.stack_left.pop()?.as_ref() };
            self.push_left(&node.right);
            if self.in_range(&node.key) {
                return Some(&node.key);
            }
        }
    }
}

impl<'a, T: Ord, B: RangeBounds<T>> DoubleEndedIterator for RangeIter<'a, T, B> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let node = unsafe { self.stack_right.pop()?.as_ref() };
            self.push_right(&node.left);
            if self.in_range(&node.key) {
                return Some(&node.key);
            }
        }
    }
}

/// デバッグ用
#[allow(unused)]
fn visualize<T: Display>(root: Link<T>) {
    fn visualize<T: Display>(
        node: Link<T>,
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

            visualize(node.right, &new_prefix, false, node.left.is_none(), res);
            visualize(node.left, &new_prefix, false, true, res);
        }
    }

    let mut res = String::new();
    visualize(root, "", true, true, &mut res);
    println!("{}", res);
}

impl<T: Display> AvlTreeSet<T> {
    #[allow(unused)]
    pub fn visualize(&self) {
        visualize(self.root);
    }
}

#[cfg(test)]
mod tests {

    use super::AvlTreeSet;

    #[test]
    fn test_avl_tree_set_insert_and_contains() {
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
    fn test_avl_tree_set_remove() {
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
    fn test_avl_tree_set_get_nth() {
        let tree = AvlTreeSet::from([1, 3, 5, 7, 9]);
        assert_eq!(tree.get_nth(0), Some(&1));
        assert_eq!(tree.get_nth(1), Some(&3));
        assert_eq!(tree.get_nth(2), Some(&5));
        assert_eq!(tree.get_nth(3), Some(&7));
        assert_eq!(tree.get_nth(4), Some(&9));
        assert_eq!(tree.get_nth(5), None);
    }

    #[test]
    fn test_avl_tree_set_get_nth_back() {
        let tree = AvlTreeSet::from([2, 4, 6, 8, 10]);
        assert_eq!(tree.get_nth_back(0), Some(&10));
        assert_eq!(tree.get_nth_back(1), Some(&8));
        assert_eq!(tree.get_nth_back(2), Some(&6));
        assert_eq!(tree.get_nth_back(3), Some(&4));
        assert_eq!(tree.get_nth_back(4), Some(&2));
        assert_eq!(tree.get_nth_back(5), None);
    }

    #[test]
    fn test_avl_tree_set_append() {
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
    fn test_avl_tree_set_split_off() {
        let mut tree1 = AvlTreeSet::from([1, 2, 3, 4, 5, 6]);
        let tree2 = tree1.split_off(&4);
        assert!(tree1.iter().copied().eq([1, 2, 3]));
        assert!(tree2.iter().copied().eq([4, 5, 6]));

        let mut tree1 = AvlTreeSet::from([2, 4, 6, 8, 10]);
        let tree2 = tree1.split_off(&5);
        assert!(tree1.iter().copied().eq([2, 4]));
        assert!(tree2.iter().copied().eq([6, 8, 10]));

        let mut tree1 = AvlTreeSet::from([1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let tree2 = tree1.split_off(&5);
        assert!(tree1.iter().copied().eq([1, 2, 3, 4]));
        assert!(tree2.iter().copied().eq([5, 6, 7, 8, 9]));

        let mut tree = AvlTreeSet::from([10, 20, 30, 40, 50]);
        let tree2 = tree.split_off(&25);
        assert!(tree.iter().copied().eq([10, 20]));
        assert!(tree2.iter().copied().eq([30, 40, 50]));

        let mut tree1 = AvlTreeSet::new();
        let tree2 = tree1.split_off(&10);
        assert!(tree1.is_empty());
        assert!(tree2.is_empty());
    }

    #[test]
    fn test_avl_tree_set_range() {
        let st = AvlTreeSet::from([1, 2, 3, 4, 5]);
        assert!(st.range(2..5).copied().eq([2, 3, 4]));
        assert!(st.range(2..5).rev().copied().eq([4, 3, 2]));

        let st = AvlTreeSet::from([1, 3, 5]);
        assert!(st.range(2..3).copied().eq([]));
        assert!(st.range(2..3).rev().copied().eq([]));

        let st = AvlTreeSet::from([1, 2, 3]);
        assert!(st.range(1..4).copied().eq([1, 2, 3]));
        assert!(st.range(1..4).rev().copied().eq([3, 2, 1]));

        let st = AvlTreeSet::from([2, 4, 6, 8]);
        assert!(st.range(3..7).copied().eq([4, 6]));
        assert!(st.range(3..7).rev().copied().eq([6, 4]));

        let st = AvlTreeSet::from([1, 3, 5, 7]);
        assert!(st.range(2..6).copied().eq([3, 5]));
        assert!(st.range(2..6).rev().copied().eq([5, 3]));

        let st = AvlTreeSet::from([10, 20, 30]);
        assert!(st.range(40..50).copied().eq([]));
        assert!(st.range(40..50).rev().copied().eq([]));

        let st = AvlTreeSet::from([5, 10, 15]);
        assert!(st.range(10..11).copied().eq([10]));
        assert!(st.range(10..11).rev().copied().eq([10]));

        let st = AvlTreeSet::from([2, 4, 6, 8, 10]);
        assert!(st.range(..).copied().eq([2, 4, 6, 8, 10]));
        assert!(st.range(..=8).copied().eq([2, 4, 6, 8]));
        assert!(st.range(4..).copied().eq([4, 6, 8, 10]));

        let st = AvlTreeSet::from([2, 4, 6, 8, 10, 12, 14, 16]);
        assert!(st.range(3..11).copied().eq([4, 6, 8, 10]));
        assert!(st.range(3..11).rev().copied().eq([10, 8, 6, 4]));
        assert!(st.range(4..7).copied().eq([4, 6]));
    }

    #[test]
    fn test_avl_tree_set_range_nth() {
        let st = AvlTreeSet::from([2, 4, 6, 8, 10, 12, 14, 16]);
        assert_eq!(st.range(3..9).nth(0), Some(&4));
        assert_eq!(st.range(3..9).nth(1), Some(&6));
        assert_eq!(st.range(3..9).nth(2), Some(&8));
        assert_eq!(st.range(3..9).nth(3), None);
        assert_eq!(st.range(3..11).nth(3), Some(&10));

        assert_eq!(st.range(6..16).nth_back(0), Some(&14));
        assert_eq!(st.range(6..16).nth_back(1), Some(&12));
        assert_eq!(st.range(6..16).nth_back(2), Some(&10));
        assert_eq!(st.range(6..16).nth_back(3), Some(&8));
        assert_eq!(st.range(6..16).nth_back(4), Some(&6));
    }

    #[test]
    fn test_avl_tree_set_lower_bound() {
        let st = AvlTreeSet::from([2, 4, 6, 8, 10, 12, 14, 16]);
        assert_eq!(st.lower_bound(&4), Some(&4));
        assert_eq!(st.lower_bound(&9), Some(&10));
        assert_eq!(st.lower_bound(&1), Some(&2));
        assert_eq!(st.lower_bound(&17), None);
    }

    #[allow(deprecated)]
    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_avl_tree_set_random() {
        use rand::{thread_rng, Rng};
        use std::collections::BTreeSet;

        let mut rng = thread_rng();

        for _ in 0..5 {
            let mut avl = AvlTreeSet::new();
            let mut b = BTreeSet::new();
            for _ in 0..1000 {
                // 0: insert
                // 1: contains
                // 2: remove
                // 3: nth
                let t = rng.gen_range(0..4);
                match t {
                    0 => {
                        let x = rng.gen_range(-100..=100);
                        assert_eq!(b.insert(x), avl.insert(x));
                    }
                    1 => {
                        let x = rng.gen_range(-100..=100);
                        assert_eq!(b.contains(&x), avl.contains(&x));
                    }
                    2 => {
                        let x = rng.gen_range(-100..=100);
                        assert_eq!(b.remove(&x), avl.remove(&x));
                    }
                    3 => {
                        let k = rng.gen_range(0..100);
                        assert_eq!(b.iter().nth(k), avl.get_nth(k));
                        assert_eq!(b.iter().nth_back(k), avl.get_nth_back(k));
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
