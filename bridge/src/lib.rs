/*!
Near - Incognito bridge implementation with JSON serialization.
NOTES:
  - Shield / Unshield features: move tokens forth and back between Near and Incognito
  - Swap beacon
*/

mod internal;
mod token_receiver;
mod errors;

use std::cmp::Ordering;
use std::convert::TryFrom;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, BorshStorageKey, PanicOnDefault, ext_contract, PromiseResult, AccountId, PromiseOrValue, Gas, Promise};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::collections::{LookupMap, TreeMap};
use crate::errors::*;
use arrayref::{array_refs, array_ref};
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::json_types::U128;

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

// define the methods we'll use on ContractB
#[ext_contract(ext_ft)]
pub trait FtContract {
    fn ft_metadata(&self) -> FungibleTokenMetadata;
    fn ft_balance_of(&mut self, account_id: AccountId) -> U128;
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

// define methods we'll use as callbacks on ContractA
#[ext_contract(ext_self)]
pub trait VaultContract {
    fn fallback_deposit(
        &self,
        incognito_address: String,
        account: AccountId,
        token: AccountId,
        amount: u128,
    );
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
        let total_native = env::account_balance();
        if total_native.checked_div(1e15 as u128).unwrap_or_default().cmp(&(u64::MAX as u128)) == Ordering::Greater {
            panic!("{}", VALUE_EXCEEDED);
        }

        // extract near amount from deposit transaction
        let amount = env::attached_deposit().checked_div(1e15 as u128).unwrap_or(0);
        env::log_str(format!(
            "{} {} {}",
            incognito_address, NEAR_ADDRESS.to_string(), amount
        ).as_str());
    }

    /// withdraw tokens
    ///
    /// submit burn proof to receive token
    pub fn withdraw(
        &mut self,
        unshield_info: UnshieldRequest
    ) -> Promise {
        let inst = hex::decode(unshield_info.inst).unwrap_or_default();
        if inst.len() < LEN {
            panic!("{}", INVALID_INSTRUCTION)
        }
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
        if (meta_type != 157 && meta_type != 158) || shard_id != 1 {
            panic!("{}", INVALID_METADATA);
        }

        if unshield_info.indexes.len() != unshield_info.signatures.len() ||
            unshield_info.signatures.len() != unshield_info.vs.len() {
            panic!("{}", INVALID_KEY_AND_INDEX);
        }

        let beacons = self.get_beacons(unshield_info.height);
        if beacons.len().eq(&0) {
            panic!("{}", INVALID_BEACON_LIST);
        }
        if unshield_info.signatures.len() <= beacons.len() * 2 / 3 {
            panic!("{}", INVALID_NUMBER_OF_SIGS);
        }

        // check tx burn used
        if self.tx_burn.get(tx_id).unwrap_or_default() {
            panic!("{}", INVALID_TX_BURN);
        }
        self.tx_burn.insert(tx_id, &true);

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
        let height_vec = self.append_at_top(unshield_info.height);
        let mut inst_vec = inst.to_vec();
        inst_vec.extend_from_slice(&height_vec);
        let inst_hash = env::keccak256_array(inst_vec.as_slice());
        if !self.instruction_in_merkle_tree(
            &inst_hash,
            &unshield_info.inst_root,
            &unshield_info.inst_paths,
            &unshield_info.inst_path_is_lefts
        ) {
            panic!("{}", INVALID_MERKLE_TREE);
        }
        // todo: update account and token address
        let account: AccountId = AccountId::try_from(hex::encode(receiver_key)).unwrap();

        if hex::encode(token) == NEAR_ADDRESS {
            Promise::new(account).transfer(unshield_amount)
        } else {
            // todo: update account and token address
            let token: AccountId = AccountId::try_from(hex::encode(token)).unwrap();
            ext_ft::ft_transfer(
                account,
                U128(unshield_amount),
                None,
                token,
                0,
                Gas(5_000_000_000),
            ).into()
        }
    }

    pub fn swap_beacon_committee(
        &mut self,
    ) {}


    /// getters

    /// get beacon list by height
    pub fn get_beacons(&self, height: u128) -> Vec<String> {
        let get_height_key = self.beacons.lower(&(height + 1)).unwrap();
        self.beacons.get(&get_height_key).unwrap()
    }

    /// check tx burn used
    pub fn get_tx_burn_used(self, tx_id: &[u8; 32]) -> bool {
        self.tx_burn.get(tx_id).unwrap_or_default()
    }

    /// fallbacks
    pub fn fallback_deposit(&self, incognito_addr: String, account: AccountId, token: AccountId, amount: u128) -> PromiseOrValue<U128> {
        assert_eq!(env::promise_results_count(), 2, "This is a callback method");

        // handle the result from the second cross contract call this method is a callback for
        let token_meta_data: FungibleTokenMetadata = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => panic!("{:?}", b"Unable to make comparison"),
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<FungibleTokenMetadata>(&result)
                .unwrap()
                .into(),
        };

        // handle the result from the first cross contract call this method is a callback for
        let vault_acc_balance: u128 = match env::promise_result(1) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => panic!("{:?}", b"Unable to make comparison"),
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<U128>(&result)
                .unwrap()
                .into(),
        };

        if vault_acc_balance.cmp(&(u64::MAX as u128)) == Ordering::Greater {
            return ext_ft::ft_transfer(
                    account,
                    U128(amount),
                    None,
                    token,
                    0,
                    Gas(5_000_000_000),
                ).into();
        }

        let mut emit_amount = amount;
        if token_meta_data.decimals > 9 {
            emit_amount = amount.checked_div(u128::pow(10, (token_meta_data.decimals - 9) as u32)).unwrap_or_default();
        }

        env::log_str(
            format!(
                "{} {} {}",
                incognito_addr, token, emit_amount
            ).as_str());

        PromiseOrValue::Value(U128(0))
    }
}
