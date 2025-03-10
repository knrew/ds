//! Boxを使った単方向連結リスト(singly-linked list)の実装

use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct SinglyLinkedList {
    head: Option<Box<Node>>,
}

impl SinglyLinkedList {
    /// 空の連結リストを作成
    pub fn new() -> Self {
        Self { head: None }
    }

    /// 先頭に挿入
    pub fn push_front(&mut self, value: i32) {
        let mut new_node = Node::new(value);
        new_node.next = std::mem::replace(&mut self.head, None);
        self.head = Some(Box::new(new_node));
    }

    /// 末尾に挿入
    pub fn push_back(&mut self, value: i32) {
        let mut node = &mut self.head;
        while let Some(next) = node {
            node = &mut next.next;
        }
        *node = Some(Box::new(Node::new(value)));
    }

    /// 先頭からindex番目の位置にvalueを挿入する
    pub fn insert(&mut self, index: usize, value: i32) {
        if index == 0 {
            self.push_front(value);
        } else if let Some(next) = node_nth_next_mut(&mut self.head, index - 1) {
            let mut new_node = Node::new(value);
            new_node.next = std::mem::replace(&mut next.next, None);
            next.next = Some(Box::new(new_node));
        } else {
            panic!("out of bounds");
        }
    }

    /// 先頭からindex番目の要素を削除する
    pub fn remove(&mut self, index: usize) {
        if index == 0 {
            if let Some(next) = &mut self.head {
                self.head = std::mem::take(&mut next.next);
            } else {
                panic!("out of bounds");
            }
        } else if let Some(next) = node_nth_next_mut(&mut self.head, index - 1) {
            if let Some(mut node) = next.next.take() {
                next.next = std::mem::take(&mut node.next);
            } else {
                panic!("out of bounds");
            }
        } else {
            panic!("out of bounds");
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = i32> + '_ {
        std::iter::successors(self.head.as_ref(), |node| node.next.as_ref()).map(|node| node.value)
    }
}

impl Index<usize> for SinglyLinkedList {
    type Output = i32;
    fn index(&self, index: usize) -> &Self::Output {
        if let Some(node) = node_nth_next(&self.head, index) {
            &node.value
        } else {
            panic!("out of bounds");
        }
    }
}

impl IndexMut<usize> for SinglyLinkedList {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if let Some(node) = node_nth_next_mut(&mut self.head, index) {
            &mut node.value
        } else {
            panic!("out of bounds");
        }
    }
}

impl From<Vec<i32>> for SinglyLinkedList {
    fn from(v: Vec<i32>) -> Self {
        let mut list = SinglyLinkedList::new();
        for e in v {
            list.push_back(e);
        }
        list
    }
}

impl FromIterator<i32> for SinglyLinkedList {
    fn from_iter<I: IntoIterator<Item = i32>>(iter: I) -> Self {
        let mut list = SinglyLinkedList::new();
        for e in iter {
            list.push_back(e);
        }
        list
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    value: i32,
    next: Option<Box<Node>>,
}

impl Node {
    pub fn new(value: i32) -> Self {
        Node { value, next: None }
    }
}

/// nodeのn個先のノードを取得する
pub fn node_nth_next(mut node: &Option<Box<Node>>, n: usize) -> &Option<Box<Node>> {
    for _ in 0..n {
        if let Some(next) = node {
            node = &next.next;
        } else {
            panic!("out of bounds");
        }
    }
    node
}

/// nodeのn個先のノードを取得する
pub fn node_nth_next_mut(mut node: &mut Option<Box<Node>>, n: usize) -> &mut Option<Box<Node>> {
    for _ in 0..n {
        if let Some(next) = node {
            node = &mut next.next;
        } else {
            panic!("out of bounds");
        }
    }
    node
}

#[cfg(test)]
mod tests {
    use super::SinglyLinkedList;

    #[test]
    fn test_push_back() {
        let mut list = SinglyLinkedList::new();
        assert!(list.iter().eq([]));
        list.push_back(4);
        assert!(list.iter().eq([4]));
        list.push_back(3);
        list.push_back(1);
        list.push_back(5);
        assert!(list.iter().eq([4, 3, 1, 5]));
    }

    #[test]
    fn test_push_front() {
        let mut list = SinglyLinkedList::new();
        list.push_front(1);
        assert!(list.iter().eq([1]));
        list.push_front(10);
        assert!(list.iter().eq([10, 1]));
        list.push_front(2);
        assert!(list.iter().eq([2, 10, 1]));
    }

    #[test]
    fn test_from() {
        let list = SinglyLinkedList::from(vec![3, 1, 4, 1, 5]);
        assert!(list.iter().eq([3, 1, 4, 1, 5]));
        let list = SinglyLinkedList::from_iter(0..10);
        assert!(list.iter().eq([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]));
    }

    #[test]
    fn test_index() {
        let mut list = SinglyLinkedList::from(vec![3, 1, 4, 1, 5]);
        assert_eq!(list[0], 3);
        assert_eq!(list[1], 1);
        assert_eq!(list[2], 4);
        assert_eq!(list[3], 1);
        assert_eq!(list[4], 5);

        list[2] = 9;
        list[4] = 30;
        assert_eq!(list[2], 9);
        assert_eq!(list[4], 30);
        assert!(list.iter().eq([3, 1, 9, 1, 30]));
    }

    #[test]
    #[should_panic]
    fn test_index_out_of_bounds() {
        let list = SinglyLinkedList::from(vec![3, 1, 4, 1, 5]);
        let _ = list[10];
    }

    #[test]
    #[should_panic]
    fn test_index_mut_out_of_bounds() {
        let mut list = SinglyLinkedList::from(vec![3, 1, 4, 1, 5]);
        list[6] = 30;
    }

    #[test]
    fn test_insert() {
        let mut list = SinglyLinkedList::from(vec![3, 1, 4, 1, 5]);
        list.insert(2, 10);
        assert!(list.iter().eq([3, 1, 10, 4, 1, 5]));
        list.insert(6, 30);
        assert!(list.iter().eq([3, 1, 10, 4, 1, 5, 30]));
    }

    #[test]
    #[should_panic]
    fn test_insert_index_out_of_bounds() {
        let mut list = SinglyLinkedList::from(vec![3, 1, 4, 1, 5]);
        list.insert(6, 80);
    }

    #[test]
    fn test_remove() {
        let mut list = SinglyLinkedList::from(vec![3, 1, 4, 1, 5]);
        list.remove(0);
        assert!(list.iter().eq([1, 4, 1, 5]));
        list.remove(2);
        assert!(list.iter().eq([1, 4, 5]));
    }

    #[test]
    #[should_panic]
    fn test_remove_index_out_of_bounds() {
        let mut list = SinglyLinkedList::from(vec![3, 1, 4, 1, 5]);
        list.remove(5);
    }
}
