#![cfg(feature = "test-bpf")]

use solana_program::{program_pack::Pack, pubkey::Pubkey};
use solana_program_test::*;
use solana_sdk::account::ReadableAccount;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;

use solana_multisig::*;

fn program_test() -> ProgramTest {
    ProgramTest::new("solana_multisig", id(), processor!(Processor::process))
}

#[tokio::test]
async fn test_create_multisig_account() {
    let owner = Keypair::new();
    let (multisig_address, _) = get_associated_address_and_bump_seed(&owner.pubkey(), &id());

    let (mut banks_client, funder, recent_blockhash) = program_test().start().await;

    let rent = banks_client.get_rent().await.unwrap();
    let expected_multisig_account_balance = rent.minimum_balance(solana_multisig::Account::LEN);

    // Multisig account does not exist
    assert_eq!(
        banks_client
            .get_account(multisig_address)
            .await
            .expect("get_account"),
        None,
    );

    let custodian_address = Pubkey::new_unique();

    let mut transaction = Transaction::new_with_payer(
        &[solana_multisig::create_associated_account(
            &funder.pubkey(),
            &owner.pubkey(),
            solana_multisig::MultisigInstruction::CreateAccount {
                threshold: 1,
                owners: vec![custodian_address],
            }
            .pack()
            .expect("pack"),
        )],
        Some(&funder.pubkey()),
    );
    transaction.sign(&[&funder, &owner], recent_blockhash);
    banks_client
        .process_transaction(transaction)
        .await
        .expect("process_transaction");

    // Multisig account now exists
    let multisig_account = banks_client
        .get_account(multisig_address)
        .await
        .expect("get_account")
        .expect("associated_account not none");

    assert_eq!(multisig_account.owner, id());
    assert_eq!(multisig_account.data.len(), solana_multisig::Account::LEN);
    assert_eq!(multisig_account.lamports, expected_multisig_account_balance);

    let multisig_account_data = Account::unpack(multisig_account.data()).expect("unpack");
    assert_eq!(multisig_account_data.is_initialized, true);
    assert_eq!(multisig_account_data.threshold, 1);
    assert_eq!(multisig_account_data.owners.len(), 1);
    assert_eq!(
        *multisig_account_data.owners.first().expect("custodian"),
        custodian_address
    );
    assert_eq!(multisig_account_data.pending_transactions.len(), 0);
    assert_eq!(multisig_account_data.frozen_amount, 0);
}
