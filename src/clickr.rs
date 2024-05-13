use anyhow::Result;
use aptos_experimental_bulk_txn_submit::{event_lookup::{get_burn_token_addr, get_mint_token_addr, search_single_event_data}, workloads::{rand_string, SignedTransactionBuilder}};
use aptos_logger::info;
use aptos_sdk::{
    move_types::{account_address::AccountAddress, ident_str, language_storage::ModuleId},
    rest_client::{aptos_api_types::TransactionOnChainData, Client},
    transaction_builder::TransactionFactory,
    types::{
        transaction::{EntryFunction, SignedTransaction},
        LocalAccount,
    },
};
use serde::{Deserialize, Serialize};

const CONTRACT_ADDRESS: &str = "0xff9659c0da82a6701e5641584a05ca03576bed4c994ab677dd6d12fe679f6615";

fn clickr_module_id() -> ModuleId {
    ModuleId::new(
        AccountAddress::from_str_strict(CONTRACT_ADDRESS).unwrap(),
        ident_str!("clickr").to_owned(),
    )
}

pub struct ClickrPlaySignedTransactionBuilder {
    clickr_contract: ModuleId,
}

impl ClickrPlaySignedTransactionBuilder {
    pub fn new() -> Self {
        Self {
            clickr_contract: clickr_module_id(),
        }
    }
}

impl SignedTransactionBuilder<()> for ClickrPlaySignedTransactionBuilder {
    fn build(
        &self,
        data: &(),
        account: &LocalAccount,
        txn_factory: &TransactionFactory,
    ) -> SignedTransaction {
        let txn = account.sign_with_transaction_builder(
            txn_factory.entry_function(EntryFunction::new(
                self.clickr_contract.clone(),
                ident_str!("play").to_owned(),
                vec![],
                vec![],
            )),
        );
        txn
    }

    fn success_output(&self, data: &(), txn_out: &Option<TransactionOnChainData>) -> String {
        let (status, sender) = match txn_out {
            Some(txn_out) => ("success".to_string(), txn_out.transaction.try_as_signed_user_txn().unwrap().sender().to_standard_string()),
            None => ("failure".to_string(), "".to_string()),
        };
        format!(
            "{}\t{}",
            sender,
            status
        )
    }
}
