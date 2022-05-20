use crate::{errors::*, UnshieldRequest};
use near_sdk::{env};
use arrayref::{array_ref, array_refs};

pub const NEAR_ADDRESS: &str = "0000000000000000000000000000000000000000";
pub const LEN: usize = 1 + 1 + 32 + 32 + 32 + 32; // ignore last 32 bytes in instruction

pub fn verify_inst(
    unshield_info: UnshieldRequest, beacons: Vec<String>,
) -> (u8, u8, [u8; 20], [u8; 20], u128, [u8; 32]) {
    let inst = hex::decode(unshield_info.inst).unwrap_or_default();
    if inst.len() < LEN {
        panic!("{}", INVALID_INSTRUCTION)
    }
    let inst_ = array_ref![inst, 0, LEN];
    #[allow(clippy::ptr_offset_with_cast)]
    let (meta_type, shard_id, _, token, _, receiver_key, _, unshield_amount, tx_id) =
        array_refs![inst_, 1, 1, 12, 20, 12, 20, 24, 8, 32];
    let meta_type = u8::from_le_bytes(*meta_type);
    let shard_id = u8::from_le_bytes(*shard_id);
    let mut unshield_amount = u128::from(u64::from_be_bytes(*unshield_amount));

    if unshield_info.indexes.len() != unshield_info.signatures.len()
        || unshield_info.signatures.len() != unshield_info.vs.len()
    {
        panic!("{}", INVALID_KEY_AND_INDEX);
    }

    if beacons.len().eq(&0) {
        panic!("{}", INVALID_BEACON_LIST);
    }
    if unshield_info.signatures.len() <= beacons.len() * 2 / 3 {
        panic!("{}", INVALID_NUMBER_OF_SIGS);
    }

    let mut blk_data_bytes = unshield_info.blk_data.to_vec();
        blk_data_bytes.extend_from_slice(&unshield_info.inst_root);
        // Get double block hash from instRoot and other data
        let blk = env::keccak256_array(env::keccak256(blk_data_bytes.as_slice()).as_slice());

        // verify beacon signature
        for i in 0..unshield_info.indexes.len() {
            let (s_r, v) = (hex::decode(unshield_info.signatures[i].clone()).unwrap_or_default(), unshield_info.vs[i]);
            let index_beacon = unshield_info.indexes[i];
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
        let height_vec = append_at_top(unshield_info.height);
        let mut inst_vec = inst.to_vec();
        inst_vec.extend_from_slice(&height_vec);
        let inst_hash = env::keccak256_array(inst_vec.as_slice());
        if !instruction_in_merkle_tree(
            &inst_hash,
            &unshield_info.inst_root,
            &unshield_info.inst_paths,
            &unshield_info.inst_path_is_lefts
        ) {
            panic!("{}", INVALID_MERKLE_TREE);
        }

    (
        meta_type,
        shard_id,
        *token,
        *receiver_key,
        unshield_amount,
        *tx_id,
    )
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