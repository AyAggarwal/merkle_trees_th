#![allow(unused_imports)]
use crate::errors::errors::MerkleError;
use crate::utils::index::{left_child_index, parent_index};
use hex;
use num_bigint::BigUint;
use num_traits::FromPrimitive;
use sha3::{Digest, Sha3_256};
///backbone MerkleTree struct using Vec
pub struct MerkleTree {
    nodes: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum Direction {
    Left,
    Right,
}

pub struct ProofStep {
    direction: Direction,
    sibling: String,
}

impl MerkleTree {
    /// returns the root of the tree
    pub fn root(&self) -> String {
        return self.nodes[0].clone();
    }

    // returns the number of leaves in the tree
    pub fn num_leaves(&self) -> usize {
        return self.nodes.len() / 2 + 1;
    }

    // bool indicating if the current index is the left child
    fn is_left_child(&self, index: usize) -> bool {
        index % 2 == 1
    }

    /// Given `depth` (one indexed) and `initial_leaf`, constructs a merkle tree with leaf values as initial_leaf.
    ///
    /// # Arguments
    ///
    /// * `depth` - The depth of the tree. Ex: depth 20 creates tree with level 0 to level 19.
    /// * `initial_leaf` - value to be assinged to the leaves. must be 32 bit hex string starting with '0x'
    ///
    /// # Returns
    ///
    /// * A new MerkleTree
    pub fn new(depth: usize, initial_leaf: &str) -> Result<Self, MerkleError> {
        if depth > 30 {
            return Err(MerkleError::MaxDepthExceeded);
        }
        //adjusting example in spec, which is one-indexed
        //i.e. depth(20) == 0 to 19,
        let depth = depth - 1;

        let string_to_decode = &initial_leaf[2..];

        if string_to_decode.len() != 64 {
            return Err(MerkleError::InvalidBytes);
        }

        let leaf_count = 1 << depth;
        let total_nodes = 2 * leaf_count - 1;
        let mut nodes = vec![String::with_capacity(64); total_nodes];
        let mut hasher = Sha3_256::new();
        let mut current_hash: [u8; 32];
        let mut current_hash_string = String::from(initial_leaf);

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
            nodes[i] = current_hash_string.clone();
        }

        // build up
        for d in (0..depth).rev() {
            let mut concatenated_hash = [0u8; 64];
            concatenated_hash[..32].copy_from_slice(&current_hash);
            concatenated_hash[32..].copy_from_slice(&current_hash);
            hasher.update(&concatenated_hash);
            current_hash = hasher.finalize_reset().into();
            current_hash_string = format!("0x{}", hex::encode(current_hash));

            let start_idx = (1 << d) - 1;
            let end_idx = (1 << (d + 1)) - 1;
            for i in start_idx..end_idx {
                nodes[i] = current_hash_string.clone();
            }
        }

        Ok(MerkleTree { nodes })
    }

    /// Sets a new leaf value and re-calculates the merkle root.
    ///
    /// # Arguments
    ///
    /// * `leaf_index` - The 0 indexed leaf to set.
    /// * `value` - The new value for the leaf. Must be 32 bit hex string starting with `0x`
    ///
    /// # Returns
    ///
    /// * Result indicating success or error
    pub fn set(&mut self, leaf_index: usize, value: &str) -> Result<(), MerkleError> {
        let leaf_count = self.num_leaves();
        if leaf_index >= leaf_count {
            return Err(MerkleError::InvalidIndex);
        }

        let array_index = self.nodes.len() - leaf_count + leaf_index;

        self.nodes[array_index] = value.to_string();

        let mut hasher = Sha3_256::new();
        let mut curr_index = parent_index(array_index);
        while let Some(index) = curr_index {
            let left_child_hash = hex::decode(&self.nodes[left_child_index(index)][2..])
                .map_err(|e| MerkleError::EncodeError(e))?;
            let right_child_hash = hex::decode(&self.nodes[left_child_index(index) + 1][2..])
                .map_err(|e| MerkleError::EncodeError(e))?;

            let mut concatenated_hash: Vec<u8> = Vec::new();
            concatenated_hash.extend(&left_child_hash);
            concatenated_hash.extend(&right_child_hash);

            hasher.update(&concatenated_hash);
            self.nodes[index] = format!("0x{}", hex::encode(hasher.finalize_reset()));
            curr_index = parent_index(index);
        }
        Ok(())
    }

    /// Constructs a proof out of `ProofStep` objects, which can be used verify the proof.
    /// Records direction and sibling all the way to the root to prove inclusion of a leaf.
    ///
    /// # Arguments
    ///
    /// * `leaf_index` - 0 indexed leaf you want to construct a proof for.
    ///
    /// # Returns
    ///
    /// * `Vec<ProofStep>` containing the proof steps to be verified.
    pub fn proof(&self, leaf_index: usize) -> Vec<ProofStep> {
        let mut proof_steps = Vec::new();

        let mut index = leaf_index + self.nodes.len() - self.num_leaves();
        while let Some(parent_index) = parent_index(index) {
            let sibling_index = if self.is_left_child(index) {
                index + 1
            } else {
                index - 1
            };

            let step = ProofStep {
                direction: if self.is_left_child(index) {
                    Direction::Left
                } else {
                    Direction::Right
                },
                sibling: self.nodes[sibling_index].clone(),
            };

            proof_steps.push(step);

            // Move up the tree
            index = parent_index;
        }
        proof_steps
    }

    /// Given a `proof` and leaf_value, calculates and returns the root.
    ///
    /// # Arguments
    ///
    /// * `proof` - `Vec<ProofStep>` containing the proof steps to be verified.
    /// * `leaf_value` - The value of the leaf you want to verify proof for. Must be 32 bit hex string with `0x` prefix.
    ///
    /// # Returns
    ///
    /// * Result containing the root of the tree or Error.
    pub fn verify(proof: &Vec<ProofStep>, leaf_value: String) -> Result<String, MerkleError> {
        let mut hasher = Sha3_256::new();
        let mut current_value = leaf_value;

        for step in proof.iter() {
            let mut concatenated: Vec<u8> = Vec::new();
            let sibling_hash =
                hex::decode(&step.sibling[2..]).map_err(|e| MerkleError::EncodeError(e))?;
            let current_hash =
                hex::decode(&current_value[2..]).map_err(|e| MerkleError::EncodeError(e))?;
            match step.direction {
                Direction::Right => {
                    concatenated.extend(&sibling_hash);
                    concatenated.extend(&current_hash);
                }
                Direction::Left => {
                    concatenated.extend(&current_hash);
                    concatenated.extend(&sibling_hash);
                }
            }
            hasher.update(concatenated);
            current_value = format!("0x{}", hex::encode(hasher.finalize_reset()));
        }

        Ok(current_value)
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
            &tree.nodes[i],
            "0xabababababababababababababababababababababababababababababababab"
        )
    }
    for i in 1..3 {
        assert_eq!(
            &tree.nodes[i],
            "0x699fc94ff1ec83f1abf531030e324003e7758298281645245f7c698425a5e0e7"
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
        &tree.nodes[1],
        "0xabababababababababababababababababababababababababababababababcd"
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

#[test]
fn test_proof() {
    let initial_leaf = "0x0000000000000000000000000000000000000000000000000000000000000000";
    let mut tree = MerkleTree::new(5, initial_leaf).unwrap();
    let num_leaves = tree.num_leaves();

    let multiplier = BigUint::parse_bytes(
        b"1111111111111111111111111111111111111111111111111111111111111111",
        16,
    )
    .expect("Failed to parse hex string to BigInt");

    for i in 0..num_leaves {
        let product = BigUint::from_usize(i).unwrap() * &multiplier;
        let value = format!("0x{:064x}", product);
        tree.set(i, &value).unwrap();
    }

    assert_eq!(
        tree.root(),
        "0x57054e43fa56333fd51343b09460d48b9204999c376624f52480c5593b91eff4"
    );

    let proof = tree.proof(3);

    let expected_proof = vec![
        ProofStep {
            direction: Direction::Right,
            sibling: "0x2222222222222222222222222222222222222222222222222222222222222222"
                .to_string(),
        },
        ProofStep {
            direction: Direction::Right,
            sibling: "0x35e794f1b42c224a8e390ce37e141a8d74aa53e151c1d1b9a03f88c65adb9e10"
                .to_string(),
        },
        ProofStep {
            direction: Direction::Left,
            sibling: "0x26fca7737f48fa702664c8b468e34c858e62f51762386bd0bddaa7050e0dd7c0"
                .to_string(),
        },
        ProofStep {
            direction: Direction::Left,
            sibling: "0xe7e11a86a0c1d8d8624b1629cb58e39bb4d0364cb8cb33c4029662ab30336858"
                .to_string(),
        },
    ];

    assert_eq!(proof.len(), expected_proof.len());
    for i in 0..proof.len() {
        assert_eq!(proof[i].direction, expected_proof[i].direction);
        assert_eq!(proof[i].sibling, expected_proof[i].sibling);
    }
}

#[test]
fn test_verify() {
    let initial_leaf = "0x0000000000000000000000000000000000000000000000000000000000000000";
    let mut tree = MerkleTree::new(5, initial_leaf).unwrap();
    let num_leaves = tree.num_leaves();

    let multiplier = BigUint::parse_bytes(
        b"1111111111111111111111111111111111111111111111111111111111111111",
        16,
    )
    .expect("Failed to parse hex string to BigInt");

    for i in 0..num_leaves {
        let product = BigUint::from_usize(i).unwrap() * &multiplier;
        let value = format!("0x{:064x}", product);
        tree.set(i, &value).unwrap();
    }

    let leaf_5_bigint = multiplier * BigUint::from(5u32);
    let leaf_5_string = format!("0x{:064x}", leaf_5_bigint);

    let root = tree.root();
    let proof = tree.proof(5);

    assert_eq!(MerkleTree::verify(&proof, leaf_5_string).unwrap(), root);
}
