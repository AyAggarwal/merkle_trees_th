use crate::errors::errors::MerkleError;
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

    pub fn root(&self) -> String {
        let root = hex::encode(&self.nodes[0]);
        return format!("0x{}", root);
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
