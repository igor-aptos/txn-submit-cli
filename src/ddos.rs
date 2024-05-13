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

const CONTRACT_ADDRESS: &str = "0x66398cf97d29fd3825f65b37cb2773268e5438d37e20777e6a98261da0cf1f1e";

fn ddos_module_id() -> ModuleId {
    ModuleId::new(
        AccountAddress::from_str_strict(CONTRACT_ADDRESS).unwrap(),
        ident_str!("ddos_coin").to_owned(),
    )
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateUserCounterEventStruct {
    user_counter: AccountAddress,
}

pub async fn register_user_counter(
    account: &LocalAccount,
    client: &Client,
    txn_factory: TransactionFactory,
) -> Result<AccountAddress> {
    let ddos_contract = ddos_module_id();

    let new_user_counter_txn = account.sign_with_transaction_builder(
        txn_factory.entry_function(EntryFunction::new(
            ddos_contract.clone(),
            ident_str!("new_user_counter").to_owned(),
            vec![],
            vec![],
        )),
    );
    // info!("new_user_counter txn: {:?}", new_user_counter_txn);

    let output = client
        .submit_and_wait_bcs(&new_user_counter_txn)
        .await?
        .into_inner();
    assert!(output.info.status().is_success(), "{:?}", output);
    info!("new_user_counter txn: {:?}", output.info);
    let create_user_counter_event: CreateUserCounterEventStruct = search_single_event_data(
        &output.events,
        &format!("{}::CreateUserCounterEvent", ddos_contract),
    )?;
    let user_counter_address = create_user_counter_event.user_counter;

    // let register_user_counter_txn = account.sign_with_transaction_builder(
    //     txn_factory.entry_function(EntryFunction::new(
    //         ddos_contract.clone(),
    //         ident_str!("register_user_counter").to_owned(),
    //         vec![],
    //         vec![
    //             bcs::to_bytes(&user_counter_address).unwrap()
    //         ],
    //     )),
    // );
    // let output = client
    //     .submit_and_wait_bcs(&register_user_counter_txn)
    //     .await?
    //     .into_inner();
    // assert!(output.info.status().is_success(), "{:?}", output);

    Ok(user_counter_address)
}

pub struct DdosIncrementSignedTransactionBuilder {
    ddos_contract: ModuleId,
    user_counter_address: AccountAddress,
}

impl DdosIncrementSignedTransactionBuilder {
    pub fn new(
        user_counter_address: AccountAddress,
    ) -> Self {
        Self {
            ddos_contract: ddos_module_id(),
            user_counter_address,
        }
    }
}

impl SignedTransactionBuilder<()> for DdosIncrementSignedTransactionBuilder {
    fn build(
        &self,
        data: &(),
        account: &LocalAccount,
        txn_factory: &TransactionFactory,
    ) -> SignedTransaction {
        let txn = account.sign_with_transaction_builder(
            txn_factory.entry_function(EntryFunction::new(
                self.ddos_contract.clone(),
                ident_str!("increment_user_counter").to_owned(),
                vec![],
                vec![
                    bcs::to_bytes(&self.user_counter_address).unwrap(),
                ],
            )),
        );
        // println!("{:?}", txn);
        txn
    }

    fn success_output(&self, data: &(), txn_out: &Option<TransactionOnChainData>) -> String {
        let (status, sender) = match txn_out {
            Some(txn_out) => ("success".to_string(), txn_out.transaction.try_as_signed_user_txn().unwrap().sender().to_standard_string()),
            None => ("failure".to_string(), "".to_string()),
        };
        format!(
            "{}\t{}\t{}",
            sender,
            self.user_counter_address.to_standard_string(),
            status
        )
    }
}
