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
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption};
use near_sdk::json_types::ValidAccountId;
use near_sdk::{
    env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue,
};
use near_sdk::collections::{LookupMap, TreeMap, UnorderedSet};

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Vault {
    // mark tx already burn
    pub tx_burn: LookupMap<[u8; 32], bool>,
    // beacon committees
    pub beacons: TreeMap<u128, Vec<[u8; 64]>>,
}

const NEAR_ADDRESS: &str = "0000000000000000000000000000000000000000";

#[near_bindgen]
impl Vault {
    /// Initializes the beacon list
    #[init]
    pub fn new(
        beacons: Vec<[u8; 64]>,
        height: u128,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        assert!(beacons.len().eq(&0), "Invalid beacon list");
        let mut this = Self {
            tx_burn: LookupMap::new("tx_burn"),
            beacons: TreeMap::new("beacons")
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

    ) -> bool {

        true
    }


    /// getters

    /// get beacon list by height
    pub fn get_beacons(self, height: u128) -> Vec<[u8; 64]> {
        let get_height_key = self.beacons.lower(&height).unwrap();
        self.beacons.get(&get_height_key).unwrap()
    }

    /// check tx burn used
    pub fn get_tx_burn_used(self, tx_id: &[u8; 32]) -> bool {
        self.tx_burn.get(tx_id).unwrap_or_default()
    }
}
