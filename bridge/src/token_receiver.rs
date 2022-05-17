use crate::*;
use crate::errors::*;
use near_sdk::{PromiseOrValue, Balance, serde_json};
use near_sdk::json_types::{U128};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;

pub const VIRTUAL_ACC: &str = "@";

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
        _sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.assert_contract_running();
        let token_in = env::predecessor_account_id();
        assert!(msg.is_empty(), INVALID_MESSAGE);
        // instant swap
        let message =
            serde_json::from_str::<TokenReceiverMessage>(&msg).expect(ERR28_WRONG_MSG_FORMAT);
        match message {
            TokenReceiverMessage::Deposit {
                incognito_address
            } => {
                env::log(
                    format!(
                        "{} {} {}",
                        incognito_address, token_in, amount.0
                    ).as_bytes(),
                );
                // Even if send tokens fails, we don't return funds back to sender.
                PromiseOrValue::Value(U128(0))
            }
        }
    }
}