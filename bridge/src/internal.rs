use crate::*;
use near_sdk::serde::{Deserialize, Serialize};

impl Vault {

    /// verify merkle tree in instruction
    pub(crate) fn instruction_in_merkle_tree(
        &mut self,
        leaf: &[u8; 32],
        root: &[u8; 32],
        paths: &Vec<[u8; 32]>,
        path_lefts: &Vec<bool>
    ) -> bool {
        if paths.len() != path_lefts.len() {
            return false;
        }
        let mut build_root = leaf.clone();
        let mut temp;
        for i in 0..paths.len() {
            if path_lefts[i] {
                temp = paths[i][..].to_vec();
                temp.extend_from_slice(&build_root[..]);
            } else if paths[i] == [0; 32] {
                temp = build_root[..].to_vec();
                temp.extend_from_slice(&build_root[..]);
            } else {
                temp = build_root[..].to_vec();
                temp.extend_from_slice(&paths[i][..]);
            }
            build_root = <[u8; 32]>::try_from(env::keccak256(&temp[..]).as_slice());
        }
        build_root == *root
    }

    pub(crate) fn append_at_top(&mut self, input: u128) -> Vec<u8>  {
        let mut  input_vec = input.to_be_bytes().to_vec();
        for _ in 0..24 {
            input_vec.insert(0, 0);
        }

        input_vec
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct UnshieldRequest {
    // instruction in bytes
    pub inst: String,
    // beacon height
    pub height: u128,
    // inst paths to build merkle tree
    pub inst_paths: Vec<[u8; 32]>,
    // inst path indicator
    pub inst_path_is_lefts: Vec<bool>,
    // instruction root
    pub inst_root: [u8; 32],
    // blkData
    pub blk_data: [u8; 32],
    // signature index
    pub indexes: Vec<u8>,
    // signatures
    pub signatures: Vec<String>,
    // v value
    pub vs: Vec<u8>
}