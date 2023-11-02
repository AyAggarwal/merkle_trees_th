use crate::utils::index;
use crate::{errors::errors::MerkleError, utils::index::left_child_index};
use hex;
use sha3::{Digest, Sha3_256};
pub struct MerkleTree {
    nodes: Vec<[u8; 32]>,
}

impl MerkleTree {
    pub fn new(depth: usize, initial_leaf: &str) -> Result<Self, MerkleError> {
        if depth > 30 {
            return Err(MerkleError::MaxDepthExceeded);
        }
        //adjusting example in spec, which is one-indexed
        //i.e. depth(20) == 0 to 19,
        let depth = depth - 1;

        let string_to_decode;
        if &initial_leaf[..2] == "0x" {
            string_to_decode = &initial_leaf[2..];
        } else {
            string_to_decode = initial_leaf;
        }

        if string_to_decode.len() != 64 {
            return Err(MerkleError::InvalidBytes);
        }

        let leaf_count = 1 << depth;
        let total_nodes = 2 * leaf_count - 1;
        let mut nodes = vec![[0u8; 32]; total_nodes];
        let mut hasher = Sha3_256::new();
        let mut current_hash: [u8; 32];

        let initial_leaf_bytes;
        match hex::decode(string_to_decode) {
            Ok(bytes) => {
                initial_leaf_bytes = bytes;
            }
            Err(e) => return Err(MerkleError::EncodeError(e)),
        }

        current_hash = match initial_leaf_bytes.try_into() {
            Ok(bytes) => bytes,
            Err(_) => return Err(MerkleError::InvalidBytes),
        };

        for i in (total_nodes - leaf_count)..total_nodes {
            nodes[i] = current_hash;
        }

        // build up
        for d in (0..depth).rev() {
            let mut concatenated_hash = [0u8; 64];
            concatenated_hash[..32].copy_from_slice(&current_hash);
            concatenated_hash[32..].copy_from_slice(&current_hash);
            hasher.update(&concatenated_hash);
            current_hash = hasher.finalize_reset().into();

            let start_idx = (1 << d) - 1;
            let end_idx = (1 << (d + 1)) - 1;
            for i in start_idx..end_idx {
                nodes[i] = current_hash;
            }
        }

        Ok(MerkleTree { nodes })
    }

    pub fn set(&mut self, leaf_index: usize, value: &str) -> Result<(), MerkleError> {
        let leaf_count = self.num_leaves();
        if leaf_index >= leaf_count {
            return Err(MerkleError::InvalidIndex);
        }

        let array_index = self.nodes.len() - leaf_count + leaf_index;

        let string_to_decode;
        if &value[..2] == "0x" {
            string_to_decode = &value[2..];
        } else {
            string_to_decode = value;
        }

        let new_leaf_bytes;
        match hex::decode(string_to_decode) {
            Ok(bytes) => {
                new_leaf_bytes = bytes;
            }
            Err(e) => return Err(MerkleError::EncodeError(e)),
        }

        let current_leaf: [u8; 32];
        current_leaf = match new_leaf_bytes.try_into() {
            Ok(bytes) => bytes,
            Err(_) => return Err(MerkleError::InvalidBytes),
        };

        self.nodes[array_index] = current_leaf;

        let mut hasher = Sha3_256::new();
        let mut curr_index = index::parent_index(array_index);
        while let Some(index) = curr_index {
            let mut concatenated_hash = [0u8; 64];
            concatenated_hash[..32].copy_from_slice(&self.nodes[left_child_index(index)]);
            concatenated_hash[32..].copy_from_slice(&self.nodes[left_child_index(index) + 1]);
            hasher.update(&concatenated_hash);
            self.nodes[index] = hasher.finalize_reset().into();
            curr_index = index::parent_index(index);
            println!("INDEX {}", index);
        }
        Ok(())
    }

    pub fn root(&self) -> String {
        let root = hex::encode(&self.nodes[0]);
        return format!("0x{}", root);
    }

    pub fn num_leaves(&self) -> usize {
        return self.nodes.len() / 2 + 1;
    }
}

#[test]
fn test_merkle_tree_depth_20() {
    let initial_leaf = "0xabababababababababababababababababababababababababababababababab";
    let tree = MerkleTree::new(20, initial_leaf).unwrap();
    assert_eq!(
        tree.root(),
        "0xd4490f4d374ca8a44685fe9471c5b8dbe58cdffd13d30d9aba15dd29efb92930"
    );
}

#[test]
fn test_merkle_tree_full() {
    let initial_leaf = "0xabababababababababababababababababababababababababababababababab";
    let tree = MerkleTree::new(3, initial_leaf).unwrap();
    for i in 3..7 {
        assert_eq!(
            hex::encode(tree.nodes[i]),
            "abababababababababababababababababababababababababababababababab"
        )
    }
    for i in 1..3 {
        assert_eq!(
            hex::encode(tree.nodes[i]),
            "699fc94ff1ec83f1abf531030e324003e7758298281645245f7c698425a5e0e7"
        )
    }
    assert_eq!(
        tree.root(),
        "0xa2422433244a1da24b3c4db126dcc593666f98365403e6aaf07fae011c824f09"
    );
}

#[test]
fn test_merkle_tree_set() {
    let initial_leaf = "0xabababababababababababababababababababababababababababababababab";
    let mut tree = MerkleTree::new(2, initial_leaf).unwrap();
    assert_eq!(tree.nodes.len(), 3);
    assert_eq!(
        tree.root(),
        "0x699fc94ff1ec83f1abf531030e324003e7758298281645245f7c698425a5e0e7"
    );
    tree.set(
        0,
        "0xabababababababababababababababababababababababababababababababcd",
    )
    .unwrap();
    assert_eq!(
        hex::encode(tree.nodes[1]),
        "abababababababababababababababababababababababababababababababcd"
    );
    assert_eq!(
        tree.root(),
        "0x1e69d9c46ca7065c20d7a9bb407f574fd92fd862f69cb96e1146926c8f198e81"
    )
}

#[test]
fn test_merkle_tree_set_higher_depth() {
    let initial_leaf = "0xabababababababababababababababababababababababababababababababcd";
    let mut tree = MerkleTree::new(10, initial_leaf).unwrap();
    for i in 0..tree.num_leaves() {
        tree.set(
            i,
            "0xabababababababababababababababababababababababababababababababab",
        )
        .unwrap();
    }
    assert_eq!(
        tree.root(),
        "0xc795494aa662dd012c5de6c52f0ab28ee9135fe846074d62bb7807cf98742fd9"
    )
}
