use crate::{errors::*, InteractRequest};
use near_sdk::{env};

pub const WITHDRAW_METADATA: u8 = 158;
pub const SWAP_BEACON_METADATA: u8 = 159;
pub const BURN_METADATA: u8 = 160;

pub const NEAR_ADDRESS: &str = "0000000000000000000000000000000000000001";
pub const WITHDRAW_INST_LEN: usize = 1 + 1 + 1 + 64 + 1 + 64 + 32 + 32; // ignore last 64 bytes in instruction
pub const SWAP_COMMITTEE_INST_LEN: usize = 1 + 1 + 32 + 32 + 32;

pub fn verify_inst(
    request_info: &InteractRequest, beacons: Vec<String>,
) {
    if request_info.indexes.len() != request_info.signatures.len()
        || request_info.signatures.len() != request_info.vs.len()
    {
        panic!("{}", INVALID_KEY_AND_INDEX);
    }

    if beacons.len().eq(&0) {
        panic!("{}", INVALID_BEACON_LIST);
    }
    if request_info.signatures.len() <= beacons.len() * 2 / 3 {
        panic!("{}", INVALID_NUMBER_OF_SIGS);
    }

    let mut blk_data_bytes = request_info.blk_data.to_vec();
        blk_data_bytes.extend_from_slice(&request_info.inst_root);
        // Get double block hash from instRoot and other data
        let blk = env::keccak256_array(env::keccak256(blk_data_bytes.as_slice()).as_slice());

        // verify beacon signature
        for i in 0..request_info.indexes.len() {
            let (s_r, v) = (hex::decode(request_info.signatures[i].clone()).unwrap_or_default(), request_info.vs[i]);
            let index_beacon = request_info.indexes[i];
            let beacon_key = beacons[index_beacon as usize].clone();
            let recover_key = env::ecrecover(
                &blk,
                s_r.as_slice(),
                v,
                false,
            ).unwrap();
            if !hex::encode(recover_key).eq(beacon_key.as_str()) {
                panic!("{}", INVALID_BEACON_SIGNATURE);
            }
        }
        // append block height to instruction
        let height_vec = append_at_top(request_info.height);
        let mut inst_vec = hex::decode(&request_info.inst).unwrap_or_default();
        inst_vec.extend_from_slice(&height_vec);
        let inst_hash = env::keccak256_array(inst_vec.as_slice());
        if !instruction_in_merkle_tree(
            &inst_hash,
            &request_info.inst_root,
            &request_info.inst_paths,
            &request_info.inst_path_is_lefts
        ) {
            panic!("{}", INVALID_MERKLE_TREE);
        }
}

fn append_at_top(input: u128) -> Vec<u8>  {
    let mut  input_vec = input.to_be_bytes().to_vec();
    for _ in 0..24 {
        input_vec.insert(0, 0);
    }

    input_vec
}

fn instruction_in_merkle_tree(
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
        build_root = env::keccak256_array(&temp[..]);
    }
    build_root == *root
}