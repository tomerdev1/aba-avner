use crate::bk_tree::BKTree;
use crate::domain::ImageEntry;
use crate::hashing::hamming_distance;
use std::collections::HashMap;
use std::convert::Infallible;

pub fn group_by_similarity(entries: &[ImageEntry], threshold: u32) -> Vec<Vec<ImageEntry>> {
    group_by_similarity_with_progress(entries, threshold, |_| {})
}

pub fn group_by_similarity_with_progress<F>(
    entries: &[ImageEntry],
    threshold: u32,
    mut on_progress: F,
) -> Vec<Vec<ImageEntry>>
where
    F: FnMut(usize),
{
    match try_group_by_similarity_with_progress_and_cancellation(
        entries,
        threshold,
        || Ok::<(), Infallible>(()),
        |idx| {
            on_progress(idx);
        },
    ) {
        Ok(groups) => groups,
        Err(never) => match never {},
    }
}

pub fn try_group_by_similarity_with_progress_and_cancellation<F, C, E>(
    entries: &[ImageEntry],
    threshold: u32,
    mut check_cancellation: C,
    mut on_progress: F,
) -> std::result::Result<Vec<Vec<ImageEntry>>, E>
where
    F: FnMut(usize),
    C: FnMut() -> std::result::Result<(), E>,
{
    let mut tree = BKTree::new();
    let mut uf = UnionFind::new(entries.len());

    for (idx, entry) in entries.iter().enumerate() {
        check_cancellation()?;
        let mut matches = Vec::new();
        tree.search(entry.hash, threshold, &mut matches);

        for &other_idx in &matches {
            let other = &entries[other_idx];
            if hamming_distance(entry.hash, other.hash) <= threshold {
                uf.union(idx, other_idx);
            }
        }

        tree.insert(entry.hash, idx);
        on_progress(idx + 1);
    }

    let mut groups: HashMap<usize, Vec<ImageEntry>> = HashMap::new();
    let mut group_order = Vec::new();
    for (idx, entry) in entries.iter().enumerate() {
        let root = uf.find(idx);
        let bucket = groups.entry(root).or_insert_with(|| {
            group_order.push(root);
            Vec::new()
        });
        bucket.push(entry.clone());
    }

    Ok(group_order
        .into_iter()
        .filter_map(|root| groups.remove(&root))
        .collect())
}

#[derive(Debug)]
struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<u8>,
}

impl UnionFind {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
            rank: vec![0; size],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            let root = self.find(self.parent[x]);
            self.parent[x] = root;
        }
        self.parent[x]
    }

    fn union(&mut self, a: usize, b: usize) {
        let mut root_a = self.find(a);
        let mut root_b = self.find(b);
        if root_a == root_b {
            return;
        }

        let rank_a = self.rank[root_a];
        let rank_b = self.rank[root_b];
        if rank_a < rank_b {
            std::mem::swap(&mut root_a, &mut root_b);
        }

        self.parent[root_b] = root_a;
        if rank_a == rank_b {
            self.rank[root_a] += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::path::PathBuf;

    fn entry(name: &str, hash: u64) -> ImageEntry {
        ImageEntry {
            path: PathBuf::from(name),
            hash,
            size_bytes: 0,
        }
    }

    fn sort_groups(groups: Vec<Vec<ImageEntry>>) -> Vec<Vec<String>> {
        let mut normalized: Vec<Vec<String>> = groups
            .into_iter()
            .map(|group| {
                let mut names: Vec<String> = group
                    .into_iter()
                    .map(|entry| entry.path.display().to_string())
                    .collect();
                names.sort();
                names
            })
            .collect();
        normalized.sort();
        normalized
    }

    #[test]
    fn identical_hashes_form_one_group() {
        let entries = vec![entry("a.png", 42), entry("b.png", 42)];

        let groups = sort_groups(group_by_similarity(&entries, 0));

        assert_eq!(groups, vec![vec!["a.png".to_string(), "b.png".to_string()]]);
    }

    #[test]
    fn transitive_similarity_groups_correctly() {
        let entries = vec![
            entry("a.png", 0b0000),
            entry("b.png", 0b0001),
            entry("c.png", 0b0011),
        ];

        let groups = sort_groups(group_by_similarity(&entries, 1));

        assert_eq!(
            groups,
            vec![vec![
                "a.png".to_string(),
                "b.png".to_string(),
                "c.png".to_string()
            ]]
        );
    }

    #[test]
    fn unrelated_hashes_stay_in_separate_groups() {
        let entries = vec![
            entry("a.png", 0b0000),
            entry("b.png", 0b1111),
            entry("c.png", 0b0011),
        ];

        let groups = sort_groups(group_by_similarity(&entries, 1));

        assert_eq!(
            groups,
            vec![
                vec!["a.png".to_string()],
                vec!["b.png".to_string()],
                vec!["c.png".to_string()]
            ]
        );
    }

    #[test]
    fn empty_input_returns_no_groups() {
        let groups = group_by_similarity(&[], 5);
        assert!(groups.is_empty());
    }

    #[test]
    fn multiple_clusters_with_internal_transitivity_stay_separate() {
        let entries = vec![
            entry("a1.png", 0b0000),
            entry("a2.png", 0b0001),
            entry("a3.png", 0b0011),
            entry("b1.png", 0b1100),
            entry("b2.png", 0b1110),
            entry("b3.png", 0b1111),
        ];

        let groups = sort_groups(group_by_similarity(&entries, 1));

        assert_eq!(
            groups,
            vec![
                vec![
                    "a1.png".to_string(),
                    "a2.png".to_string(),
                    "a3.png".to_string()
                ],
                vec![
                    "b1.png".to_string(),
                    "b2.png".to_string(),
                    "b3.png".to_string()
                ]
            ]
        );
    }

    #[test]
    fn zero_threshold_only_groups_exact_hash_matches() {
        let entries = vec![
            entry("exact-a.png", 0b0101),
            entry("exact-b.png", 0b0101),
            entry("near.png", 0b0100),
        ];

        let groups = sort_groups(group_by_similarity(&entries, 0));

        assert_eq!(
            groups,
            vec![
                vec!["exact-a.png".to_string(), "exact-b.png".to_string()],
                vec!["near.png".to_string()]
            ]
        );
    }

    #[test]
    fn group_order_follows_first_encountered_member() {
        let entries = vec![
            entry("a.png", 0b1111),
            entry("b.png", 0b0000),
            entry("c.png", 0b1111),
            entry("d.png", 0b0010),
        ];

        let groups = group_by_similarity(&entries, 0);
        let representative_names: Vec<String> = groups
            .iter()
            .filter_map(|group| group.first())
            .map(|entry| entry.path.display().to_string())
            .collect();

        assert_eq!(representative_names, vec!["a.png", "b.png", "d.png"]);
    }

    #[test]
    fn cancellable_grouping_stops_before_processing_remaining_entries() {
        let entries = vec![
            entry("a.png", 0b0000),
            entry("b.png", 0b0001),
            entry("c.png", 0b0011),
        ];
        let mut progress = Vec::new();
        let cancelled = Cell::new(false);

        let result = try_group_by_similarity_with_progress_and_cancellation(
            &entries,
            1,
            || {
                if cancelled.get() {
                    Err(())
                } else {
                    Ok(())
                }
            },
            |idx| {
                progress.push(idx);
                if idx == 1 {
                    cancelled.set(true);
                }
            },
        );

        assert!(result.is_err());
        assert_eq!(progress, vec![1]);
    }
}
