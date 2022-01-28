use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;

use crate::error::Result;
use crate::utils;

/// Establishes a RPC connection with the solana cluster configured by
/// `solana config set --url <URL>`. Information about what cluster
/// has been configured is gleened from the solana config file
/// `~/.config/solana/cli/config.yml`.
pub fn establish_connection() -> Result<RpcClient> {
    let rpc_url = utils::get_rpc_url()?;
    Ok(RpcClient::new_with_commitment(
        rpc_url,
        CommitmentConfig::confirmed(),
    ))
}

pub fn create_account(
    payer: &Keypair,
    wallet: &Keypair,
    threshold: u32,
    owners: Vec<Pubkey>,
    connection: &RpcClient,
) -> Result<()> {
    let mut transaction = Transaction::new_with_payer(
        &[solana_multisig::create_associated_account(
            &payer.pubkey(),
            &wallet.pubkey(),
            solana_multisig::MultisigInstruction::CreateAccount { threshold, owners }
                .pack()
                .expect("pack"),
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[payer, wallet], connection.get_latest_blockhash()?);

    connection.send_and_confirm_transaction(&transaction)?;

    Ok(())
}

pub fn create_transaction(
    payer: &Keypair,
    wallet: &Keypair,
    transaction: &Keypair,
    recipient: &Pubkey,
    amount: u64,
    connection: &RpcClient,
) -> Result<()> {
    let mut transaction = Transaction::new_with_payer(
        &[solana_multisig::create_transaction(
            &payer.pubkey(),
            &wallet.pubkey(),
            &transaction.pubkey(),
            recipient,
            solana_multisig::MultisigInstruction::CreateTransaction { amount }
                .pack()
                .expect("pack"),
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[payer, wallet], connection.get_latest_blockhash()?);

    connection.send_and_confirm_transaction(&transaction)?;

    Ok(())
}

pub fn approve_transaction(
    payer: &Keypair,
    multisig: &Pubkey,
    transaction: &Pubkey,
    recipient: &Pubkey,
    connection: &RpcClient,
) -> Result<()> {
    let mut transaction = Transaction::new_with_payer(
        &[solana_multisig::approve_transaction(
            &payer.pubkey(),
            multisig,
            transaction,
            recipient,
            solana_multisig::MultisigInstruction::ApproveTransaction
                .pack()
                .expect("pack"),
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[payer], connection.get_latest_blockhash()?);

    connection.send_and_confirm_transaction(&transaction)?;

    Ok(())
}
