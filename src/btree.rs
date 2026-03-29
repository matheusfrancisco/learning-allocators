const ORDER: usize = 4; // max keys per node = ORDER - 1 = 3

#[derive(Debug)]
struct BTreeNode {
    keys: Vec<i64>,
    children: Vec<u32>, // indices into Arena
    is_leaf: bool,
}

#[derive(Debug)]
struct Arena {
    nodes: Vec<BTreeNode>,
}

impl Arena {
    fn alloc(&mut self, node: BTreeNode) -> u32 {
        let idx = self.nodes.len() as u32;
        self.nodes.push(node);
        idx
    }
    fn get(&self, idx: u32) -> &BTreeNode {
        &self.nodes[idx as usize]
    }
    fn get_mut(&mut self, idx: u32) -> &mut BTreeNode {
        &mut self.nodes[idx as usize]
    }
}
// Arena representation
// [[Node 0] [Node 1] [Node 2] ...]

#[derive(Debug)]
struct BTree {
    arena: Arena,
    root: u32,
}

type BTreeNodeRef<'arena> = &'arena BTreeNode;

impl BTree {
    fn new(capacity: usize) -> Self {
        let mut arena = Arena {
            nodes: Vec::with_capacity(capacity),
        };
        // allocate an empty root leaf at index 0
        let root = arena.alloc(BTreeNode {
            keys: vec![],
            children: vec![],
            is_leaf: true,
        });
        BTree { arena, root }
    }

    pub fn search<'a>(&'a self, key: i64) -> Option<BTreeNodeRef<'a>> {
        let mut current = self.root;

        loop {
            let node = self.arena.get(current);

            println!("node {:?}", node);
            let mut left = 0;
            let mut right = node.keys.len();

            while left < right {
                let mid = left + (right - left) / 2;

                if node.keys[mid] == key {
                    if node.is_leaf && node.keys.len() > 1 {
                        // If it's a leaf with multiple keys,
                        // we want to return the node containing the key
                        // rather than just the key itself.
                        // This allows us to maintain the B-tree structure in the search result.
                        // or if it's an internal node, we also want to return the node itself.
                        // TODO(optional): we could also return the index of the key within the node, but for simplicity we just return the node.
                        return Some(node);
                    }
                    return Some(node); // exact match
                } else if node.keys[mid] < key {
                    left = mid + 1; // search right half
                } else {
                    right = mid; // search left half
                }
            }

            // Loop exited without finding key
            // `left` is now the insertion point = correct child index
            if node.is_leaf {
                return None;
            }

            current = node.children[left];
        }
    }

    pub fn search_2<'a>(&'a self, key: i64) -> Option<BTreeNodeRef<'a>> {
        let mut current = self.root;
        loop {
            let node = self.arena.get(current);

            match node.keys.binary_search(&key) {
                Ok(_) => return Some(node), // found exact key
                Err(pos) => {
                    if node.is_leaf {
                        return None; // not found
                    }
                    current = node.children[pos];
                }
            }
        }
    }

    fn insert(&mut self, val: i64) {
        let root_full = self.arena.get(self.root).keys.len() == ORDER - 1;
        if root_full {
            let new_root = BTreeNode {
                keys: vec![],
                children: vec![self.root],
                is_leaf: false,
            };
            let new_root_idx = self.arena.alloc(new_root);
            self.root = new_root_idx;
            self.split_child(self.root, 0);
        }
        self.insert_non_full(self.root, val);
    }

    fn split_child(&mut self, parent_idx: u32, child_pos: usize) {
        let mid = ORDER / 2 - 1; // index of median key = 1 for ORDER=4

        let child_idx = self.arena.get(parent_idx).children[child_pos];
        let (median, right_keys, right_children, is_leaf) = {
            let child = self.arena.get(child_idx);
            let median = child.keys[mid];
            let right_keys = child.keys[mid + 1..].to_vec();
            let right_children = if child.is_leaf {
                vec![]
            } else {
                child.children[mid + 1..].to_vec()
            };
            (median, right_keys, right_children, child.is_leaf)
        };
        let right_idx = self.arena.alloc(BTreeNode {
            keys: right_keys,
            children: right_children,
            is_leaf,
        });

        let parent = self.arena.get_mut(parent_idx);
        parent.keys.insert(child_pos, median);
        parent.children.insert(child_pos + 1, right_idx);
    }

    fn insert_non_full(&mut self, mut node_idx: u32, key: i64) {
        loop {
            let is_leaf = self.arena.get(node_idx).is_leaf;
            if is_leaf {
                // Binary search for insertion position
                let pos = {
                    let node = self.arena.get(node_idx);
                    let mut left = 0;
                    let mut right = node.keys.len();
                    while left < right {
                        let mid = left + (right - left) / 2;
                        if node.keys[mid] < key {
                            left = mid + 1;
                        } else {
                            right = mid;
                        }
                    }
                    left
                };
                self.arena.get_mut(node_idx).keys.insert(pos, key);
                return;
            }
            // Find child to descend into via binary search
            let pos = {
                let node = self.arena.get(node_idx);
                let mut left = 0;
                let mut right = node.keys.len();
                while left < right {
                    let mid = left + (right - left) / 2;
                    if node.keys[mid] < key {
                        left = mid + 1;
                    } else {
                        right = mid;
                    }
                }
                left
            };

            let child_idx = self.arena.get(node_idx).children[pos];
            // Pre-emptively split full child before descending
            if self.arena.get(child_idx).keys.len() == ORDER - 1 {
                self.split_child(node_idx, pos);
                // After split, median moved up — recheck which side to take
                let node = self.arena.get(node_idx);
                let new_pos = if key > node.keys[pos] { pos + 1 } else { pos };
                node_idx = node.children[new_pos];
            } else {
                node_idx = child_idx;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_btree_search() {
        let mut btree = BTree::new(10);

        for key in &[10, 20, 5, 15, 25, 28, 22, 27, 30] {
            btree.insert(*key);
        }
        assert!(btree.search(15).is_some());
        assert!(btree.search(22).is_some());

        let result = btree.search(30);
        match result {
            Some(node) => assert_eq!(node.keys, vec![22, 27, 30]),
            None => panic!("Key 30 should be found"),
        }

        assert!(btree.search(99).is_none()); // should not be found
    }
}
