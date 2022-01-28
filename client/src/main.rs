use std::str::FromStr;

use clap::{
    crate_description, crate_name, crate_version, App, AppSettings, Arg, ArgMatches, SubCommand,
};

use solana_clap_utils::input_parsers::value_of;
use solana_clap_utils::input_validators::{is_amount, is_valid_pubkey};
use solana_multisig::{Account, Transaction, MAX_SIGNERS, MIN_SIGNERS};
use solana_program::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

use solana_multisig_cli::client::*;
use solana_multisig_cli::error;
use solana_multisig_cli::utils::*;

fn main() -> anyhow::Result<()> {
    let app_matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("create-account")
                .about("Create a new multisig account")
                .arg(
                    Arg::with_name("threshold")
                        .validator(is_signers_number_valid)
                        .value_name("THRESHOLD")
                        .takes_value(true)
                        .index(1)
                        .required(true)
                        .help(&format!(
                            "The minimum number of signers required \
                            to allow the operation. [{} <= N <= {}]",
                            MIN_SIGNERS, MAX_SIGNERS,
                        )),
                )
                .arg(
                    Arg::with_name("owners")
                        .value_name("OWNERS")
                        .validator(is_valid_pubkey)
                        .takes_value(true)
                        .index(2)
                        .required(true)
                        .min_values(MIN_SIGNERS as u64)
                        .max_values(MAX_SIGNERS as u64)
                        .help(&format!(
                            "The public keys for each of the N \
                            signing members of this account. [{} <= N <= {}]",
                            MIN_SIGNERS, MAX_SIGNERS,
                        )),
                ),
        )
        .subcommand(
            SubCommand::with_name("create-transaction")
                .about("Create a new multisig transaction")
                .arg(
                    Arg::with_name("recipient")
                        .validator(is_valid_pubkey)
                        .value_name("RECIPIENT")
                        .takes_value(true)
                        .index(1)
                        .required(true)
                        .help("Recipient address"),
                )
                .arg(
                    Arg::with_name("amount")
                        .value_name("AMOUNT")
                        .validator(is_amount)
                        .takes_value(true)
                        .index(2)
                        .required(true)
                        .help("Amount to transfer"),
                ),
        )
        .subcommand(
            SubCommand::with_name("approve")
                .about("Approve multisig transaction")
                .arg(
                    Arg::with_name("multisig")
                        .validator(is_valid_pubkey)
                        .value_name("MULTISIG")
                        .takes_value(true)
                        .index(1)
                        .required(true)
                        .help("Multisig address"),
                ),
        )
        .get_matches();

    let connection = establish_connection()?;
    println!(
        "Connected to remote solana node running version ({}).",
        connection.get_version()?
    );

    let payer = get_payer()?;

    let (sub_command, sub_matches) = app_matches.subcommand();

    let _ = match (sub_command, sub_matches) {
        ("create-account", Some(arg_matches)) => {
            let threshold =
                value_of::<u32>(arg_matches, "threshold").ok_or(error::Error::InvalidThreshold)?;
            let owners = pubkeys_of_multiple_signers(arg_matches, "owners")?
                .ok_or(error::Error::InvalidOwners)?;

            if threshold < owners.len() as u32 {
                return Err(error::Error::InvalidOwnersNumber.into());
            }

            create_account(&payer, &payer, threshold, owners, &connection)?
        }
        ("create-transaction", Some(arg_matches)) => {
            let recipient = Pubkey::from_str(
                value_of::<String>(arg_matches, "recipient")
                    .ok_or(error::Error::InvalidThreshold)?
                    .as_str(),
            )?;

            let amount =
                value_of::<u64>(arg_matches, "amount").ok_or(error::Error::InvalidAmount)?;

            let transaction = Keypair::new();

            create_transaction(
                &payer,
                &payer,
                &transaction,
                &recipient,
                amount,
                &connection,
            )?
        }
        ("approve", Some(arg_matches)) => {
            let multisig = Pubkey::from_str(
                value_of::<String>(arg_matches, "multisig")
                    .ok_or(error::Error::InvalidThreshold)?
                    .as_str(),
            )?;

            let multisig_info = connection.get_account(&multisig)?;
            let multisig_data = Account::unpack(&multisig_info.data)?;

            let mut need_to_approve = Vec::new();

            for pending_transaction in multisig_data.pending_transactions {
                let pending_transaction_info = connection.get_account(&pending_transaction)?;
                let pending_transaction_data =
                    Transaction::unpack_unchecked(&pending_transaction_info.data)?;

                for (signer, is_signed) in pending_transaction_data.signers {
                    if signer == payer.pubkey() && !is_signed {
                        need_to_approve
                            .push((pending_transaction, pending_transaction_data.recipient));
                        break;
                    }
                }
            }

            for (transaction, recipient) in need_to_approve {
                approve_transaction(&payer, &multisig, &transaction, &recipient, &connection)?;
            }
        }
        _ => {}
    };

    Ok(())
}

fn is_signers_number_valid(string: String) -> Result<(), String> {
    let v = u8::from_str(&string).map_err(|e| e.to_string())? as usize;
    if v < MIN_SIGNERS {
        Err(format!("must be at least {}", MIN_SIGNERS))
    } else if v > MAX_SIGNERS {
        Err(format!("must be at most {}", MAX_SIGNERS))
    } else {
        Ok(())
    }
}

fn pubkeys_of_multiple_signers(
    matches: &ArgMatches<'_>,
    name: &str,
) -> anyhow::Result<Option<Vec<Pubkey>>> {
    if let Some(pubkey_matches) = matches.values_of(name) {
        let mut pubkeys: Vec<Pubkey> = vec![];
        for signer in pubkey_matches {
            pubkeys.push(Pubkey::from_str(signer)?);
        }
        Ok(Some(pubkeys))
    } else {
        Ok(None)
    }
}
