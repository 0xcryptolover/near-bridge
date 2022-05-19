use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{serde_json, PromiseOrValue, env};
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
        amount: U128,
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
                env::log_str(
                    format!(
                        "{} {} {}",
                        incognito_address, token_in, amount.0
                    ).as_str());
                // Even if send tokens fails, we don't return funds back to sender.
                PromiseOrValue::Value(U128(0))
            }
        }
    }
}