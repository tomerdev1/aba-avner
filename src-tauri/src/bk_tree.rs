use std::collections::BTreeMap;

use crate::hashing::hamming_distance;

#[derive(Debug)]
struct Node {
    hash: u64,
    index: usize,
    children: BTreeMap<u32, Box<Node>>,
}

#[derive(Debug, Default)]
pub struct BKTree {
    root: Option<Box<Node>>,
}

impl BKTree {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn insert(&mut self, hash: u64, index: usize) {
        let node = Node {
            hash,
            index,
            children: BTreeMap::new(),
        };

        match self.root.as_mut() {
            None => self.root = Some(Box::new(node)),
            Some(root) => insert_node(root, node),
        }
    }

    pub fn search(&self, hash: u64, max_distance: u32, results: &mut Vec<usize>) {
        if let Some(root) = self.root.as_ref() {
            search_node(root, hash, max_distance, results);
        }
    }
}

fn insert_node(current: &mut Node, node: Node) {
    let distance = hamming_distance(current.hash, node.hash);
    if let Some(child) = current.children.get_mut(&distance) {
        insert_node(child, node);
    } else {
        current.children.insert(distance, Box::new(node));
    }
}

fn search_node(current: &Node, hash: u64, max_distance: u32, results: &mut Vec<usize>) {
    let distance = hamming_distance(current.hash, hash);
    if distance <= max_distance {
        results.push(current.index);
    }

    let min = distance.saturating_sub(max_distance);
    let max = distance.saturating_add(max_distance);
    for (_d, child) in current.children.range(min..=max) {
        search_node(child, hash, max_distance, results);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_returns_empty_when_tree_is_empty() {
        let tree = BKTree::new();
        let mut results = Vec::new();

        tree.search(0b1010, 2, &mut results);

        assert!(results.is_empty());
    }

    #[test]
    fn exact_match_is_returned() {
        let mut tree = BKTree::new();
        tree.insert(0b1010, 7);

        let mut results = Vec::new();
        tree.search(0b1010, 0, &mut results);

        assert_eq!(results, vec![7]);
    }

    #[test]
    fn matches_within_threshold_are_returned() {
        let mut tree = BKTree::new();
        tree.insert(0b0000, 1);
        tree.insert(0b0011, 2);
        tree.insert(0b1111, 3);

        let mut results = Vec::new();
        tree.search(0b0001, 2, &mut results);
        results.sort_unstable();

        assert_eq!(results, vec![1, 2]);
    }

    #[test]
    fn entries_outside_threshold_are_excluded() {
        let mut tree = BKTree::new();
        tree.insert(0b0000, 1);
        tree.insert(0b1111, 2);

        let mut results = Vec::new();
        tree.search(0b0001, 1, &mut results);

        assert_eq!(results, vec![1]);
    }

    #[test]
    fn duplicate_hashes_keep_all_indexes() {
        let mut tree = BKTree::new();
        tree.insert(0b1010, 3);
        tree.insert(0b1010, 4);
        tree.insert(0b1010, 5);

        let mut results = Vec::new();
        tree.search(0b1010, 0, &mut results);
        results.sort_unstable();

        assert_eq!(results, vec![3, 4, 5]);
    }

    #[test]
    fn oversized_threshold_does_not_overflow_search_bounds() {
        let mut tree = BKTree::new();
        tree.insert(0, 1);
        tree.insert(u64::MAX, 2);

        let mut results = Vec::new();
        tree.search(0, u32::MAX, &mut results);
        results.sort_unstable();

        assert_eq!(results, vec![1, 2]);
    }
}
