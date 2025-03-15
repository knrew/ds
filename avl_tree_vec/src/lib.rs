use std::{
    fmt::Debug,
    ops::{Index, IndexMut},
    ptr::NonNull,
};

pub mod visualizer;

#[derive(Clone)]
struct Node<T> {
    value: T,
    len: usize,
    height: i32,
    left: Link<T>,
    right: Link<T>,
}

type NodePtr<T> = NonNull<Node<T>>;
type Link<T> = Option<NodePtr<T>>;

impl<T> Node<T> {
    fn new(value: T) -> NodePtr<T> {
        let node = Self {
            value,
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
        self.len = len(self.left) + len(self.right) + 1;
        self.height = height(self.left).max(height(self.right)) + 1;
    }
}

#[inline]
fn free<T>(node: NodePtr<T>) {
    unsafe { drop(Box::from_raw(node.as_ptr())) };
}

#[inline]
fn len<T>(node: Link<T>) -> usize {
    node.map_or(0, |node| unsafe { node.as_ref() }.len)
}

#[inline]
fn height<T>(node: Link<T>) -> i32 {
    node.map_or(0, |node| unsafe { node.as_ref() }.height)
}

/// 平衡係数
/// 左部分木と右部分木の高さの差
/// 左部分木の高さ - 右部分木の高さ
#[inline]
fn diff_height<T>(node: Link<T>) -> i32 {
    node.map_or(0, |node| {
        let node = unsafe { node.as_ref() };
        height(node.left) - height(node.right)
    })
}

/// 木を平衡して新たなrootを返す
fn balance<T>(mut root: Link<T>) -> Link<T> {
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

    if root.is_none() {
        return None;
    }

    let d = diff_height(root);

    if d > 1 {
        // 左部分木が高い場合

        let left = &mut unsafe { root.unwrap().as_mut() }.left;
        if diff_height(*left) < 0 {
            rotate_left(left);
        }

        rotate_right(&mut root);
    } else if d < -1 {
        // 右部分木が高い場合

        let right = &mut unsafe { root.unwrap().as_mut() }.right;
        if diff_height(*right) > 0 {
            rotate_right(right);
        }

        rotate_left(&mut root);
    } else {
        unsafe { root.unwrap().as_mut() }.fetch();
    }

    root
}

fn merge_with_root<T>(left: Link<T>, root: Link<T>, right: Link<T>) -> Link<T> {
    let d = height(left) - height(right);

    if d > 1 {
        unsafe {
            left.unwrap().as_mut().right =
                merge_with_root(left.unwrap().as_ref().right, root, right);
        }
        balance(left)
    } else if d < -1 {
        unsafe {
            right.unwrap().as_mut().left =
                merge_with_root(left, root, right.unwrap().as_ref().left);
        }
        balance(right)
    } else {
        unsafe {
            root.unwrap().as_mut().left = left;
            root.unwrap().as_mut().right = right;
        }
        balance(root)
    }
}

/// 2つの木をマージして新たなrootを返す
fn merge<T>(left: Link<T>, right: Link<T>) -> Link<T> {
    /// nodeの部分木のうち最も右のノードを削除して新たなrootと削除されたノードを返す
    fn remove_max<T>(mut node: Link<T>) -> (Link<T>, Link<T>) {
        let node_mut = unsafe { node.unwrap().as_mut() };
        if node_mut.right.is_some() {
            let (tmp, removed) = remove_max(node_mut.right);
            node_mut.right = tmp;
            node = balance(node);
            (node, removed)
        } else {
            let removed = node;
            node = node_mut.left;
            (node, removed)
        }
    }

    if left.is_none() {
        right
    } else if right.is_none() {
        left
    } else {
        let (left, removed) = remove_max(left);
        merge_with_root(left, removed, right)
    }
}

/// [0, index)の部分木と[index, n)の部分木に分割する
fn split<T>(node: Link<T>, index: usize) -> (Link<T>, Link<T>) {
    if node.is_none() {
        return (None, None);
    }

    let (left, right) = {
        let node_mut = unsafe { node.unwrap().as_mut() };
        let left = node_mut.left;
        let right = node_mut.right;
        node_mut.left = None;
        node_mut.right = None;
        (left, right)
    };

    let left_len = len(left);
    if index < left_len {
        let (x, y) = split(left, index);
        (x, merge_with_root(y, node, right))
    } else if index > left_len {
        let (x, y) = split(right, index - left_len - 1);
        (merge_with_root(left, node, x), y)
    } else {
        (left, merge_with_root(None, node, right))
    }
}

fn get<T>(node: Link<T>, index: usize) -> Link<T> {
    if let Some(raw_node) = node.map(|mut node| unsafe { node.as_mut() }) {
        let left = raw_node.left;
        let right = raw_node.right;
        let left_len = len(left);
        if index < left_len {
            get(left, index)
        } else if index > left_len {
            get(right, index - left_len - 1)
        } else {
            node
        }
    } else {
        None
    }
}

#[allow(unused)]
fn traverse<T>(
    node: Link<T>,
    mut preorder_f: impl FnMut(NodePtr<T>),
    mut inorder_f: impl FnMut(NodePtr<T>),
    mut post_order_f: impl FnMut(NodePtr<T>),
) {
    fn traverse<T>(
        node: Link<T>,
        preorder_f: &mut impl FnMut(NodePtr<T>),
        inorder_f: &mut impl FnMut(NodePtr<T>),
        post_order_f: &mut impl FnMut(NodePtr<T>),
    ) {
        if let Some(node) = node {
            let left = unsafe { node.as_ref() }.left;
            let right = unsafe { node.as_ref() }.right;
            preorder_f(node);
            traverse(left, preorder_f, inorder_f, post_order_f);
            inorder_f(node);
            traverse(right, preorder_f, inorder_f, post_order_f);
            post_order_f(node);
        }
    }

    traverse(node, &mut preorder_f, &mut inorder_f, &mut post_order_f);
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

#[derive(Clone)]
pub struct AvlTreeVec<T> {
    root: Link<T>,
}

impl<T> AvlTreeVec<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn len(&self) -> usize {
        len(self.root)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        Some(&unsafe { get(self.root, index)?.as_ref() }.value)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        Some(&mut unsafe { get(self.root, index)?.as_mut() }.value)
    }

    pub fn front(&self) -> Option<&T> {
        self.get(0)
    }

    pub fn back(&self) -> Option<&T> {
        self.get(self.len().checked_sub(1)?)
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.get_mut(0)
    }

    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.get_mut(self.len().checked_sub(1)?)
    }

    pub fn push_front(&mut self, value: T) {
        self.insert(0, value);
    }

    pub fn push_back(&mut self, value: T) {
        self.insert(self.len(), value);
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.remove(0)
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.remove(self.len().checked_sub(1)?)
    }

    pub fn insert(&mut self, index: usize, value: T) {
        assert!(index <= self.len());
        let new_node = Some(Node::new(value));
        let (left, right) = split(self.root.take(), index);
        self.root = merge_with_root(left, new_node, right);
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index < self.len() {
            let (left, right) = split(self.root.take(), index);
            let (removed, right) = split(right, 1);
            self.root = merge(left, right);
            let boxed = unsafe { Box::from_raw(removed?.as_ptr()) };
            Some(boxed.value)
        } else {
            None
        }
    }

    // TODO: 仮
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        let mut res = vec![];
        traverse_inorder(self.root, |node| unsafe {
            res.push(&node.as_ref().value);
        });
        res.into_iter()
    }
}

impl<T> Default for AvlTreeVec<T> {
    fn default() -> Self {
        Self { root: None }
    }
}

impl<T> Drop for AvlTreeVec<T> {
    fn drop(&mut self) {
        traverse_postorder(self.root, |node| free(node));
    }
}

impl<T> Index<usize> for AvlTreeVec<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T> IndexMut<usize> for AvlTreeVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl<T> Extend<T> for AvlTreeVec<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        iter.into_iter().for_each(|item| {
            self.push_back(item);
        });
    }
}

impl<'a, T: 'a + Copy> Extend<&'a T> for AvlTreeVec<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned());
    }
}

impl<T> FromIterator<T> for AvlTreeVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut res = Self::new();
        res.extend(iter);
        res
    }
}

impl<T> From<Vec<T>> for AvlTreeVec<T> {
    fn from(v: Vec<T>) -> Self {
        Self::from_iter(v)
    }
}

impl<T, const N: usize> From<[T; N]> for AvlTreeVec<T> {
    fn from(v: [T; N]) -> Self {
        Self::from_iter(v)
    }
}

impl<T: Debug> Debug for AvlTreeVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::AvlTreeVec;

    #[test]
    fn test_push_back() {
        let mut tree = AvlTreeVec::new();
        tree.push_back(3);
        tree.push_back(1);
        tree.push_back(4);
        tree.push_back(1);
        tree.push_back(5);
        assert_eq!(tree[0], 3);
        assert_eq!(tree[1], 1);
        assert_eq!(tree[2], 4);
        assert_eq!(tree[3], 1);
        assert_eq!(tree[4], 5);
        assert!(tree.iter().copied().eq([3, 1, 4, 1, 5]));
        assert_eq!(tree.len(), 5);
    }

    #[test]
    fn test_push_front() {
        let mut tree = AvlTreeVec::new();
        tree.push_front(3);
        tree.push_front(1);
        tree.push_front(4);
        tree.push_front(1);
        tree.push_front(5);
        assert_eq!(tree[0], 5);
        assert_eq!(tree[1], 1);
        assert_eq!(tree[2], 4);
        assert_eq!(tree[3], 1);
        assert_eq!(tree[4], 3);
        assert!(tree.iter().copied().eq([5, 1, 4, 1, 3]));
        assert_eq!(tree.len(), 5);
    }

    #[test]
    fn test_pop_back() {
        let mut tree = AvlTreeVec::from([3, 1, 4, 1, 5]);
        assert_eq!(tree.back(), Some(&5));
        assert_eq!(tree.pop_back(), Some(5));
        assert_eq!(tree.back(), Some(&1));
        assert_eq!(tree.pop_back(), Some(1));
        assert_eq!(tree.back(), Some(&4));
        assert_eq!(tree.pop_back(), Some(4));
        assert_eq!(tree.back(), Some(&1));
        assert_eq!(tree.pop_back(), Some(1));
        assert_eq!(tree.back(), Some(&3));
        assert_eq!(tree.pop_back(), Some(3));
        assert_eq!(tree.back(), None);
        assert_eq!(tree.pop_back(), None);
        assert!(tree.is_empty());
    }

    #[test]
    fn test_pop_front() {
        let mut tree = AvlTreeVec::from([3, 1, 4, 1, 5]);
        assert_eq!(tree.front(), Some(&3));
        assert_eq!(tree.pop_front(), Some(3));
        assert_eq!(tree.front(), Some(&1));
        assert_eq!(tree.pop_front(), Some(1));
        assert_eq!(tree.front(), Some(&4));
        assert_eq!(tree.pop_front(), Some(4));
        assert_eq!(tree.front(), Some(&1));
        assert_eq!(tree.pop_front(), Some(1));
        assert_eq!(tree.front(), Some(&5));
        assert_eq!(tree.pop_front(), Some(5));
        assert_eq!(tree.front(), None);
        assert_eq!(tree.pop_front(), None);
        assert!(tree.is_empty());
    }

    #[test]
    fn test_get_mut() {
        let mut tree = AvlTreeVec::from([3, 1, 4, 1, 5]);
        tree.get_mut(0).map(|item| *item = 2);
        tree.get_mut(1).map(|item| *item = 7);
        tree.get_mut(2).map(|item| *item = 1);
        tree.get_mut(3).map(|item| *item = 8);
        tree.get_mut(4).map(|item| *item = 2);
        tree.get_mut(5).map(|item| *item = 8);
        assert!(tree.iter().copied().eq([2, 7, 1, 8, 2]));

        tree[0] = 9;
        tree[1] = 9;
        tree[2] = 8;
        tree[3] = 2;
        tree[4] = 4;
        assert!(tree.iter().copied().eq([9, 9, 8, 2, 4]));
    }
}
