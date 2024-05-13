use anyhow::{Context, Result, bail};
use aptos_logger::{Level, Logger};
use aptos_sdk::{
    transaction_builder::TransactionFactory,
    move_types::account_address::AccountAddress,
};    
use aptos_transaction_emitter_lib::{
    emitter::account_minter::prompt_yes,
    Cluster, ClusterArgs,
};
use clap::{Parser, Subcommand};

use aptos_experimental_bulk_txn_submit::{coordinator::{execute_txn_list, TransactionFactoryArgs}};
use ddos::{register_user_counter, DdosIncrementSignedTransactionBuilder};
use clickr::ClickrPlaySignedTransactionBuilder;
use std::time::Duration;

const MAX_SUBMIT_BATCH: usize = 10;
fn default_poll_interval() -> Duration {
    Duration::from_secs_f32(0.1)
}

mod clickr;
mod ddos;

#[derive(Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    command: TxnSubmitCommand,
}

#[derive(Subcommand, Debug)]
enum TxnSubmitCommand {
    Submit(Submit),
    // ReturnWorkerFunds(ReturnWorkerFunds),
}

#[derive(Parser, Debug)]
pub struct Submit {
    #[clap(flatten)]
    pub cluster_args: ClusterArgs,

    #[clap(flatten)]
    pub transaction_factory_args: TransactionFactoryArgs,

    #[clap(subcommand)]
    work_args: WorkTypeSubcommand,
    
    // #[clap(flatten)]
    // submit_args: SubmitArgs,

    // #[clap(flatten)]
    // pub accounts_args: AccountsArgs,

}

#[derive(Subcommand, Debug)]
pub enum WorkTypeSubcommand {
    DdosIncrement(DdosArgs),
    ClickrPlay(ClickrArgs)
    // ReturnWorkerFunds,
}

#[derive(Parser, Debug)]
pub struct DdosArgs {
    #[clap(long)]
    num_txns: usize,

    #[clap(long)]
    counter_address: Option<AccountAddress>
}

#[derive(Parser, Debug)]
pub struct ClickrArgs {
    #[clap(long)]
    num_txns: usize,
}


#[tokio::main]
pub async fn main() -> Result<()> {
    Logger::builder().level(Level::Info).build();

    let args = Args::parse();

    match args.command {
        TxnSubmitCommand::Submit(args) => create_work_and_execute(args).await,
    }
}

async fn create_work_and_execute(args: Submit) -> Result<()> {
    let cluster = Cluster::try_from_cluster_args(&args.cluster_args)
        .await
        .context("Failed to build cluster")?;
    let coin_source_account = cluster
        .load_coin_source_account(&cluster.random_instance().rest_client())
        .await?;
    let clients = cluster
        .all_instances()
        .map(|i| i.rest_client())
        .collect::<Vec<_>>();

    match &args.work_args {
        WorkTypeSubcommand::DdosIncrement(ddos_args) => {
            let work = (0..ddos_args.num_txns).map(|_| ()).collect::<Vec<()>>();

            let client = &cluster.random_instance().rest_client();

            let txn_factory = args.transaction_factory_args.with_init_params(
                TransactionFactory::new(cluster.chain_id));

            let user_counter_address = match ddos_args.counter_address {
                Some(addr) => addr,
                None => register_user_counter(
                    &coin_source_account,
                    client, 
                    txn_factory.clone(),
                ).await?,
            };
            
            let builder =
                DdosIncrementSignedTransactionBuilder::new(user_counter_address);

            if !prompt_yes(&format!("About to submit {} transactions and spend {} APT. Continue?", work.len(), work.len() as f32 * args.transaction_factory_args.octas_per_workload_transaction as f32 / 1e8 )) {
                bail!("User aborted")
            }

            let results = execute_txn_list(
                vec![coin_source_account],
                clients,
                work,
                MAX_SUBMIT_BATCH,
                default_poll_interval(),
                args.transaction_factory_args.with_params(TransactionFactory::new(cluster.chain_id)),
                builder,
            ).await?;

            Ok(())
        },
        WorkTypeSubcommand::ClickrPlay(clickr_args) => {
            let work = (0..clickr_args.num_txns).map(|_| ()).collect::<Vec<()>>();

            let client = &cluster.random_instance().rest_client();
            
            let builder = ClickrPlaySignedTransactionBuilder::new();

            if !prompt_yes(&format!("About to submit {} transactions and spend {} APT. Continue?", work.len(), work.len() as f32 * args.transaction_factory_args.octas_per_workload_transaction as f32 / 1e8 )) {
                bail!("User aborted")
            }

            let results = execute_txn_list(
                vec![coin_source_account],
                clients,
                work,
                MAX_SUBMIT_BATCH,
                default_poll_interval(),
                args.transaction_factory_args.with_params(TransactionFactory::new(cluster.chain_id)),
                builder,
            ).await?;

            Ok(())
        },
    }
}
