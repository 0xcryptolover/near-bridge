use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{serde_json, env, PromiseOrValue, Gas};
use near_sdk::AccountId;
use near_sdk::json_types::U128;

use crate::errors::*;
use crate::*;

/// Message parameters to receive via token function call.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(untagged)]
enum TokenReceiverMessage {
    Deposit {
        incognito_address: String
    },
}

#[near_bindgen]
impl FungibleTokenReceiver for Vault {
    /// Callback on receiving tokens by this contract.
    /// `msg` format is either "" for deposit or `TokenReceiverMessage`.
    #[allow(unreachable_code)]
    fn ft_on_transfer(
        &mut self,
        _sender_id: AccountId,
        value: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let token_in = env::predecessor_account_id();
        if msg.is_empty() {
            panic!("{}", INVALID_MESSAGE)
        }
        // shield request
        let message =
            serde_json::from_str::<TokenReceiverMessage>(&msg).expect(ERR28_WRONG_MSG_FORMAT);
        match message {
            TokenReceiverMessage::Deposit {
                incognito_address
            } => {
                let amount = value.0;
                ext_ft::ft_metadata(
                    token_in.clone(),
                    0,
                    Gas(5_000_000_000_000),          // gas to attach
                )
                .and(ext_ft::ft_balance_of(
                    env::current_account_id().clone(),
                    token_in.clone(),
                    0,
                    Gas(5_000_000_000_000),         // gas to attach
                ))
                .then(ext_self::fallback_deposit(
                    incognito_address,
                    _sender_id,
                    token_in,
                    amount,
                    env::current_account_id().clone(),
                    0,
                    Gas(5_000_000_000_000),          // gas to attach to the callback
                )).into()
            }
        }
    }
}