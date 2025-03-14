//! AVL木によるordered setの実装
//! TODO: 不要なCopy, Defaultを外す

use std::{
    borrow::Borrow,
    cmp::Ordering,
    fmt::Debug,
    mem::{swap, take},
    ptr::NonNull,
};

pub mod visualizer;

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
    mut post_order_f: impl FnMut(NodePtr<T>),
) {
    fn traverse<T>(
        node: Link<T>,
        preorder_f: &mut impl FnMut(NodePtr<T>),
        inorder_f: &mut impl FnMut(NodePtr<T>),
        post_order_f: &mut impl FnMut(NodePtr<T>),
    ) {
        if let Some(node) = node {
            preorder_f(node);
            traverse(
                unsafe { node.as_ref() }.left,
                preorder_f,
                inorder_f,
                post_order_f,
            );
            inorder_f(node);
            traverse(
                unsafe { node.as_ref() }.right,
                preorder_f,
                inorder_f,
                post_order_f,
            );
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
pub struct AvlTreeSet<T> {
    root: Link<T>,
}

impl<T> AvlTreeSet<T> {
    pub fn new() -> Self {
        Self::default()
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
        fn insert<T: Ord>(root: &mut Link<T>, key: T) -> bool {
            if let Some(node) = root.map(|mut node| unsafe { node.as_mut() }) {
                match key.cmp(&node.key) {
                    Ordering::Equal => {
                        return false;
                    }
                    Ordering::Less => {
                        if !insert(&mut node.left, key) {
                            return false;
                        }
                    }
                    Ordering::Greater => {
                        if !insert(&mut node.right, key) {
                            return false;
                        }
                    }
                }
                balance(root);
            } else {
                *root = Some(Node::new(key));
            }

            true
        }

        insert(&mut self.root, key)
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
            let res = if let Some(node_ptr) = node {
                let node_mut = unsafe { node_ptr.as_mut() };

                match key.cmp(node_mut.key.borrow()) {
                    Ordering::Equal => {
                        if node_mut.left.is_none() {
                            let mut right = node_mut.right;
                            balance(node);
                            swap(node, &mut right);
                            right
                        } else if node_mut.right.is_none() {
                            let mut left = node_mut.left;
                            balance(node);
                            swap(node, &mut left);
                            left
                        } else {
                            unsafe {
                                let right = &mut node_mut.right;
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
                    Ordering::Less => remove(&mut node_mut.left, key),
                    Ordering::Greater => remove(&mut node_mut.right, key),
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
    pub fn get_nth(&self, mut n: usize) -> Option<&T> {
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
    pub fn get_nth_back(&self, mut n: usize) -> Option<&T> {
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

    /// NOTE: 2つのAVL木の要素数をN, Mに対してO(min(N+M)log N)
    pub fn append(&mut self, other: &mut Self)
    where
        T: Ord + Default,
    {
        fn insert<T: Ord + Default>(tree: &mut AvlTreeSet<T>, node: Link<T>) {
            traverse_preorder(node, |mut node| {
                // tree.insert(unsafe { node.as_ref() }.key);
                tree.insert(take(&mut unsafe { node.as_mut() }.key));
            });
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
    pub fn split_off(&mut self, value: &T) -> Self
    where
        T: Ord + Default,
    {
        fn insert_and_drop<T: Ord + Default>(
            node: Link<T>,
            left: &mut AvlTreeSet<T>,
            right: &mut AvlTreeSet<T>,
            value: &T,
        ) {
            traverse_postorder(node, |mut node| {
                // keyだけをこうしんするな
                let node_value = take(&mut unsafe { node.as_mut() }.key);
                if &node_value < value {
                    left.insert(node_value);
                } else {
                    right.insert(node_value);
                }
                free(node)
            });
        }

        let mut left = Self::new();
        let mut right = Self::new();
        insert_and_drop(self.root.take(), &mut left, &mut right, value);
        *self = left;
        right
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(&self.root)
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

impl<T: Ord> From<Vec<T>> for AvlTreeSet<T> {
    fn from(v: Vec<T>) -> Self {
        let mut tree = AvlTreeSet::new();
        v.into_iter().for_each(|key| {
            tree.insert(key);
        });
        tree
    }
}

impl<'a, T> IntoIterator for &'a AvlTreeSet<T> {
    type IntoIter = Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: Default> IntoIterator for AvlTreeSet<T> {
    type IntoIter = IntoIter<T>;
    type Item = T;

    fn into_iter(mut self) -> Self::IntoIter {
        IntoIter::new(self.root.take())
    }
}

impl<T: Ord, const N: usize> From<[T; N]> for AvlTreeSet<T> {
    fn from(v: [T; N]) -> Self {
        let mut tree = AvlTreeSet::new();
        v.into_iter().for_each(|key| {
            tree.insert(key);
        });
        tree
    }
}

impl<T: Debug> Debug for AvlTreeSet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

pub struct Iter<'a, T> {
    stack_left: Vec<&'a NodePtr<T>>,
    stack_right: Vec<&'a NodePtr<T>>,
}

impl<'a, T> Iter<'a, T> {
    fn new(root: &'a Link<T>) -> Self {
        let mut iter = Self {
            stack_left: vec![],
            stack_right: vec![],
        };
        iter.push_left(root);
        iter.push_right(root);
        iter
    }

    fn push_left(&mut self, mut node: &'a Link<T>) {
        while let Some(n) = node {
            self.stack_left.push(n);
            node = &unsafe { n.as_ref() }.left;
        }
    }

    fn push_right(&mut self, mut node: &'a Link<T>) {
        while let Some(n) = node {
            self.stack_right.push(n);
            node = &unsafe { n.as_ref() }.right;
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = unsafe { self.stack_left.pop()?.as_ref() };
        self.push_left(&node.right);
        Some(&node.key)
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let node = unsafe { self.stack_right.pop()?.as_ref() };
        self.push_right(&node.left);
        Some(&node.key)
    }
}

// TODO: ???
pub struct IntoIter<T> {
    iter: std::vec::IntoIter<NodePtr<T>>,
}

impl<T> IntoIter<T> {
    fn new(root: Link<T>) -> Self {
        let mut stack = vec![];
        traverse_inorder(root, |node| {
            stack.push(node);
        });
        IntoIter {
            iter: stack.into_iter(),
        }
    }
}

impl<T: Default> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let mut node = self.iter.next()?;
        // let res = unsafe { node.as_ref() }.key.take();
        let res = take(&mut unsafe { node.as_mut() }.key);
        free(node);
        Some(res)
    }
}

impl<T: Default> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let mut node = self.iter.next_back()?;
        // let res = unsafe { node.as_ref() }.key.take();
        let res = take(&mut unsafe { node.as_mut() }.key);
        free(node);
        Some(res)
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        for node in take(&mut self.iter) {
            free(node)
        }
    }
}

pub fn test() {
    use std::collections::BTreeSet;

    let q = [
        (1, 14),
        (0, 68),
        (1, 94),
        (0, 16),
        (0, 87),
        (0, 45),
        (1, 70),
        (1, 1),
        (0, 29),
        (0, 19),
        (0, 52),
        (1, 29),
    ];

    let i = 11;

    let mut avl = AvlTreeSet::new();
    let mut b = BTreeSet::new();

    for i in 0..i {
        let (t, x) = q[i];
        if t == 0 {
            avl.insert(x);
            b.insert(x);
        } else {
            avl.remove(&x);
            b.remove(&x);
        }
    }

    println!("{:?}", q);
    println!("b: {:?} | {}", b, b.len());
    println!("avl: {:?} | {}", avl, avl.len());
    avl.visualize();

    let (t, x) = q[i];
    if t == 0 {
        avl.insert(x);
        b.insert(x);
    } else {
        avl.remove(&x);
        b.remove(&x);
    }

    println!("{}", avl.iter().eq(b.iter()));
    println!("b: {:?} | {}", b, b.len());
    println!("avl: {:?} | {}", avl, avl.len());
    avl.visualize();

    traverse_inorder(avl.root, |node| unsafe {
        println!("{:?}: {:?}", node.as_ref().key, node.as_ref().len);
    });
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
                        let k = rng.random_range(0..100);
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
