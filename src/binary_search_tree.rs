//! (平衡とは限らない)二分探索木(binary search tree)の実装
//! 任意のノードの値xに対して
//! そのノードの左部分木の要素はすべてxよりも小さく
//! 右部分木に含まれる要素はすべてxよりも大きい

use std::{cmp::Ordering, ptr::NonNull};

pub struct BinarySearchTree {
    root: Option<NonNull<Node>>,
}

impl BinarySearchTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn insert(&mut self, value: i32) -> bool {
        let mut node = &mut self.root;
        while let Some(p) = node {
            match value.cmp(unsafe { &p.as_ref().value }) {
                Ordering::Equal => {
                    return false;
                }
                Ordering::Less => {
                    node = unsafe { &mut p.as_mut().left };
                }
                Ordering::Greater => {
                    node = unsafe { &mut p.as_mut().right };
                }
            }
        }
        *node = Some(Node::new(value).as_ptr());
        true
    }

    pub fn contains(&self, value: &i32) -> bool {
        let mut node = &self.root;
        while let Some(p) = node {
            match value.cmp(unsafe { &p.as_ref().value }) {
                Ordering::Equal => {
                    return true;
                }
                Ordering::Less => {
                    node = unsafe { &p.as_ref().left };
                }
                Ordering::Greater => {
                    node = unsafe { &p.as_ref().right };
                }
            }
        }
        false
    }

    pub fn remove(&mut self, value: &i32) -> bool {
        // pの部分木のうち最小のノードを削除してそのノードを返す
        fn remove_min(mut p: &mut Option<NonNull<Node>>) -> Option<NonNull<Node>> {
            while unsafe { p.unwrap().as_ref().left.is_some() } {
                p = unsafe { &mut p.unwrap().as_mut().left };
            }
            let res = *p;
            *p = unsafe { p.unwrap().as_mut().right };
            res
        }

        let mut parent = &mut self.root;
        while parent.is_some() {
            match value.cmp(unsafe { &parent.unwrap().as_ref().value }) {
                Ordering::Equal => {
                    let mut rm_node = parent.unwrap();

                    match (unsafe { rm_node.as_ref().left }, unsafe {
                        rm_node.as_ref().right
                    }) {
                        (Some(_), Some(_)) => {
                            *parent = remove_min(unsafe { &mut rm_node.as_mut().right });
                            unsafe {
                                parent.unwrap().as_mut().right = rm_node.as_ref().right;
                                parent.unwrap().as_mut().left = rm_node.as_ref().left
                            };
                        }
                        (Some(left), None) => {
                            *parent = Some(left);
                        }
                        (None, Some(right)) => {
                            *parent = Some(right);
                        }
                        (None, None) => {
                            *parent = None;
                        }
                    }
                    unsafe { drop(Box::from_raw(rm_node.as_ptr())) };
                    return true;
                }
                Ordering::Less => {
                    parent = unsafe { &mut parent.unwrap().as_mut().left };
                }
                Ordering::Greater => {
                    parent = unsafe { &mut parent.unwrap().as_mut().right };
                }
            }
        }

        false
    }

    pub fn iter(&self) -> impl Iterator<Item = i32> + '_ {
        fn dfs(node: Option<NonNull<Node>>, v: &mut Vec<i32>) {
            if let Some(node) = node {
                dfs(unsafe { node.as_ref().left }, v);
                v.push(unsafe { node.as_ref().value });
                dfs(unsafe { node.as_ref().right }, v);
            }
        }
        let mut res = vec![];
        dfs(self.root, &mut res);
        res.into_iter()
    }
}

impl Default for BinarySearchTree {
    fn default() -> Self {
        Self { root: None }
    }
}

impl Drop for BinarySearchTree {
    fn drop(&mut self) {
        fn dfs(node: Option<NonNull<Node>>) {
            if let Some(node) = node {
                dfs(unsafe { node.as_ref().left });
                dfs(unsafe { node.as_ref().right });
                unsafe { drop(Box::from_raw(node.as_ptr())) };
            }
        }
        dfs(self.root);
    }
}

impl From<Vec<i32>> for BinarySearchTree {
    fn from(v: Vec<i32>) -> Self {
        let mut res = Self::new();
        v.into_iter().for_each(|e| {
            res.insert(e);
        });
        res
    }
}

impl<const N: usize> From<[i32; N]> for BinarySearchTree {
    fn from(v: [i32; N]) -> Self {
        let mut res = Self::new();
        v.into_iter().for_each(|e| {
            res.insert(e);
        });
        res
    }
}

struct Node {
    value: i32,
    left: Option<NonNull<Node>>,
    right: Option<NonNull<Node>>,
}

impl Node {
    fn new(value: i32) -> Self {
        Node {
            value,
            left: None,
            right: None,
        }
    }

    fn as_ptr(self) -> NonNull<Self> {
        NonNull::from(Box::leak(Box::new(self)))
    }
}

#[cfg(test)]
mod tests {
    use super::BinarySearchTree;

    #[test]
    fn test_insert_and_contains() {
        let mut tree = BinarySearchTree::new();
        assert!(!tree.contains(&10));
        assert!(!tree.contains(&6));
        assert!(tree.insert(10));
        assert!(tree.contains(&10));
        assert!(tree.insert(6));
        assert!(tree.contains(&6));
        assert!(!tree.insert(6));
    }

    #[test]
    fn test_iter() {
        let mut tree = BinarySearchTree::new();
        assert!(tree.iter().eq([]));
        tree.insert(3);
        tree.insert(1);
        tree.insert(4);
        tree.insert(1);
        tree.insert(5);
        assert!(tree.iter().eq([1, 3, 4, 5]));
    }

    #[test]
    fn test_remove() {
        let mut tree = BinarySearchTree::from([3, 1, 4, 1, 5]);
        assert!(!tree.remove(&10));
        assert!(tree.contains(&3));
        assert!(tree.remove(&3));
        assert!(!tree.contains(&3));

        let mut tree = BinarySearchTree::from(vec![9, 5, 1, 7, 3, 8]);
        tree.remove(&5);
        assert!(tree.iter().eq([1, 3, 7, 8, 9]));

        let mut tree = BinarySearchTree::from([100, 50, 20, 10, 80, 90, 60, 95]);
        tree.remove(&10);
        assert!(tree.iter().eq([20, 50, 60, 80, 90, 95, 100]));

        let mut tree = BinarySearchTree::from([100, 50, 20, 10, 80, 90, 60, 95]);
        tree.remove(&95);
        assert!(tree.iter().eq([10, 20, 50, 60, 80, 90, 100]));

        let mut tree = BinarySearchTree::from([100, 50, 20, 10, 80, 90, 60, 95]);
        tree.remove(&80);
        assert!(tree.iter().eq([10, 20, 50, 60, 90, 95, 100]));

        let mut tree = BinarySearchTree::from([100, 50, 20, 10, 80, 90, 60, 95]);
        tree.remove(&50);
        assert!(tree.iter().eq([10, 20, 60, 80, 90, 95, 100]));
    }

    #[test]
    fn test_random() {
        use rand::{rng, Rng};
        use std::collections::BTreeSet;

        let mut rng = rng();

        for _ in 0..5 {
            let mut st = BTreeSet::new();
            let mut tree = BinarySearchTree::new();
            for _ in 0..1000 {
                // クエリ
                // 0: insert
                // 1: remove
                // 2: contains
                let t = rng.random_range(0..3);
                let x = rng.random();
                match t {
                    0 => {
                        assert_eq!(st.insert(x), tree.insert(x));
                    }
                    1 => {
                        assert_eq!(st.remove(&x), tree.remove(&x));
                    }
                    2 => {
                        assert_eq!(st.contains(&x), tree.contains(&x));
                    }
                    _ => {}
                }
                assert!(tree.iter().eq(st.iter().copied()));
            }
        }
    }
}
