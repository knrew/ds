//! ポインタ(NonNull)を使った単方向連結リスト(singly-linked list)の実装

use std::{iter::successors, mem::replace, ptr::NonNull};

pub struct SinglyLinkedList {
    head: Option<NonNull<Node>>,
}

impl SinglyLinkedList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_front(&mut self, value: i32) {
        let mut new_node = Node::new(value);
        new_node.next = replace(&mut self.head, None);
        self.head = Some(new_node.as_ptr());
    }

    pub fn push_back(&mut self, value: i32) {
        let mut node = &mut self.head;
        while let Some(next) = node {
            node = &mut unsafe { next.as_mut() }.next;
        }
        *node = Some(Node::new(value).as_ptr());
    }

    pub fn iter(&self) -> impl Iterator<Item = i32> + '_ {
        successors(self.head, |node| unsafe { node.as_ref() }.next)
            .map(|node| unsafe { node.as_ref() }.value)
    }
}

impl Default for SinglyLinkedList {
    fn default() -> Self {
        Self { head: None }
    }
}

impl Drop for SinglyLinkedList {
    fn drop(&mut self) {
        if let Some(first) = self.head {
            let mut next = Some(first);
            while let Some(node) = next {
                next = unsafe { (*node.as_ptr()).next };
                unsafe { drop(Box::from_raw(node.as_ptr())) };
            }
        }
    }
}

struct Node {
    value: i32,
    next: Option<NonNull<Node>>,
}

impl Node {
    fn new(value: i32) -> Self {
        Self { value, next: None }
    }

    fn as_ptr(self) -> NonNull<Self> {
        NonNull::from(Box::leak(Box::new(self)))
    }
}

impl Default for Node {
    fn default() -> Self {
        Self {
            value: 0,
            next: None,
        }
    }
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
}
