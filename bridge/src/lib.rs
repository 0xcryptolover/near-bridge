/*!
Near - Incognito bridge implementation with JSON serialization.
NOTES:
  - Shield / Unshield features: move tokens forth and back between Near and Incognito
  - Swap beacon
*/

mod internal;
mod token_receiver;
mod errors;

use std::any::Any;
use std::convert::TryFrom;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption};
use near_sdk::json_types::ValidAccountId;
use near_sdk::{
    env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue,
};
use near_sdk::collections::{LookupMap, TreeMap, UnorderedSet};
use secp256k1::{Message, Secp256k1, Signature, PublicKey};
use crate::errors::{INVALID_BEACON_LIST, INVALID_BEACON_SIGNATURE, INVALID_INSTRUCTION, INVALID_KEY_AND_INDEX, INVALID_MERKLE_TREE, INVALID_METADATA, INVALID_NUMBER_OF_SIGS, INVALID_TX_BURN};
use crate::internal::UnshieldRequest;
use arrayref::{array_refs, array_ref};

near_sdk::setup_alloc!();

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    Transaction,
    BeaconHeight,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Vault {
    // mark tx already burn
    pub tx_burn: LookupMap<[u8; 32], bool>,
    // beacon committees
    pub beacons: TreeMap<u128, Vec<String>>,
}

const NEAR_ADDRESS: &str = "0000000000000000000000000000000000000000";
const LEN: usize = 1 + 1 + 32 + 32 + 32 + 32; // ignore last 32 bytes in instruction

#[near_bindgen]
impl Vault {
    /// Initializes the beacon list
    #[init]
    pub fn new(
        beacons: Vec<String>,
        height: u128,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        assert!(beacons.len().eq(&0), "Invalid beacon list");
        let mut this = Self {
            tx_burn: LookupMap::new(StorageKey::Transaction), 
            beacons: TreeMap::new(StorageKey::BeaconHeight)
        };
        // insert beacon height and list in tree
        this.beacons.insert(&height, &beacons);

        this
    }

    /// shield native token
    ///
    /// receive token from users and generate proof
    /// validate proof on Incognito side and mint corresponding token
    #[payable]
    pub fn deposit(
        &mut self,
        incognito_address: String,
    ) {
        // extract near amount from deposit transaction
        let amount = env::attached_deposit();
        env::log(
            format!(
                "{} {} {}",
                incognito_address, NEAR_ADDRESS.to_string(), amount
            ).as_bytes(),
        );
    }

    /// withdraw tokens
    ///
    /// submit burn proof to receive token
    pub fn withdraw(
        &mut self,
        unshield_info: UnshieldRequest
    ) -> bool {
        let inst = hex::decode(unshield_info.inst).unwrap_or_default();
        assert!(inst.len() < LEN, INVALID_INSTRUCTION);
        let inst_ = array_ref![inst, 0, LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (
            meta_type,
            shard_id,
            _,
            token,
            _,
            receiver_key,
            _,
            unshield_amount,
            tx_id,
        ) = array_refs![
             inst_,
             1,
             1,
             12,
             20,
             12,
             20,
             24,
             8,
             32
        ];
        let meta_type = u8::from_le_bytes(*meta_type);
        let shard_id = u8::from_le_bytes(*shard_id);
        let mut unshield_amount = u128::from(u64::from_be_bytes(*unshield_amount));

        // validate metatype and key provided
        assert!((meta_type != 157 && meta_type != 158) || shard_id != 1, INVALID_METADATA);

        // verify beacon signature
        assert!(unshield_info.indexes.len() != unshield_info.signatures.len() ||
            unshield_info.signatures.len() != unshield_info.vs.len(),
            INVALID_KEY_AND_INDEX
        );

        let beacons = self.get_beacons(unshield_info.height);
        assert!(beacons.len().eq(&0), INVALID_BEACON_LIST);
        assert!(unshield_info.signatures.len() <= beacons.len() * 2 / 3, INVALID_NUMBER_OF_SIGS);

        // check tx burn used
        assert!(self.tx_burn.get(tx_id).unwrap_or_default(), INVALID_TX_BURN);
        self.tx_burn.insert(tx_id, &true);

        let mut blk_data_bytes = unshield_info.blk_data.to_vec();
        blk_data_bytes.extend_from_slice(&unshield_info.inst_root);
        // Get double block hash from instRoot and other data
        let blk = env::keccak256(env::keccak256(blk_data_bytes.as_slice()).as_slice());
        let ecdsa_verification = Secp256k1::verification_only();

        for i in 0..unshield_info.indexes.len() {
            let (s_r, v) = (hex::decode(unshield_info.signatures[i].clone()).unwrap_or_default(), unshield_info.vs[i]);
            let index_beacon = unshield_info.indexes[i];
            let beacon_key = beacons[index_beacon as usize].clone();
            let msg = Message::from_slice(blk.as_slice());
            let beacon_key_byte = hex::decode(beacon_key).unwrap_or_default();
            let pub_key = PublicKey::from_slice(&beacon_key_byte).unwrap();
            let signature = Signature::from_compact(s_r.as_slice()).unwrap();
            assert!(ecdsa_verification.verify(&msg.unwrap(), &signature, &pub_key).is_err(),
                INVALID_BEACON_SIGNATURE
            );
        }
        // append block height to instruction
        let height_vec = self.append_at_top(unshield_info.height);
        let mut inst_vec = inst.to_vec();
        inst_vec.extend_from_slice(&height_vec);
        let inst_hash = <[u8; 32]>::try_from(env::keccak256(inst_vec.as_slice())).unwrap();
        assert!(!self.instruction_in_merkle_tree(
            &inst_hash,
            &unshield_info.inst_root,
            &unshield_info.inst_paths,
            &unshield_info.inst_path_is_lefts
        ), INVALID_MERKLE_TREE);

        // todo: transfer token to users.


        true
    }

    pub fn swap_beacon_commitee(
        &mut self,
    ) {}


    /// getters

    /// get beacon list by height
    pub fn get_beacons(&self, height: u128) -> Vec<String> {
        let get_height_key = self.beacons.lower(&height).unwrap();
        self.beacons.get(&get_height_key).unwrap()
    }

    /// check tx burn used
    pub fn get_tx_burn_used(self, tx_id: &[u8; 32]) -> bool {
        self.tx_burn.get(tx_id).unwrap_or_default()
    }
}
