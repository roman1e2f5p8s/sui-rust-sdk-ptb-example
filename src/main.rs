use std::{
    time::Duration,
    str::FromStr,
};

use sui_sdk::{
    SuiClientBuilder,
    SUI_TESTNET_URL,
    rpc_types::SuiTransactionBlockResponseOptions,
    wallet_context::WalletContext,
};
use sui_types::{
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::{
        Transaction,
        TransactionData,
    },
    quorum_driver_types::ExecuteTransactionRequestType,
    Identifier,
    MOVE_STDLIB_PACKAGE_ID,
};
use sui_keys::keystore::{
    AccountKeystore, 
    FileBasedKeystore,
};
use sui_config::{
    sui_config_dir,
    SUI_CLIENT_CONFIG,
    SUI_KEYSTORE_FILENAME,
};
use shared_crypto::intent::Intent;

const SEP: &str = 
    "--------------------------------------------------------------------";
const RPC: &str = SUI_TESTNET_URL;


#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    println!("{}\n", SEP);


    // Build Sui client -------------------------------------------------------
    print!("- Building Sui client...");
    let sui_client= SuiClientBuilder::default()
        .build(RPC)
        .await?;
    println!("done! --------------------------------------");
    println!("- Sui testnet version: {}", sui_client.api_version());
    println!("{}\n", SEP);
 

    // Get active Sui address -------------------------------------------------
    print!("- Getting active Sui address...");
    let wallet_config = sui_config_dir()?.join(SUI_CLIENT_CONFIG);
    let timeout = Duration::from_secs(30);
    let max_concurrent_requests = None;
    let mut wallet = WalletContext::new(
        &wallet_config,
        Some(timeout),
        max_concurrent_requests,
        )?;
    let sender = wallet.active_address()?;
    println!("done! -------------------------------");
    println!("- Sui active address:\n- {}", sender);
    println!("{}\n", SEP);


    // Find gas coin ----------------------------------------------------------
    print!("- Finding gas coin...");
    let coins = sui_client
        .coin_read_api()
        .get_coins(
            sender,
            None,
            None,
            None
        )
        .await?;
    let gas_coin = coins
        .data
        .into_iter()
        .next()
        .unwrap();
    let gas_payment = 
        vec![gas_coin.object_ref()];
    println!("done! -----------------------------------------");
    println!("- Gas coin object ID:\n- {}", gas_coin.coin_object_id);
    println!("{}\n", SEP);


    // Get gas price ----------------------------------------------------------
    print!("- Getting gas price...");
    let gas_budget = 500_000_000; // 0.5 Sui
    let gas_price = sui_client
        .read_api()
        .get_reference_gas_price()
        .await?;
    println!("done! ----------------------------------------");
    println!("- Gas price:\n- {} MIST", gas_price);
    println!("{}\n", SEP);

    // Create PTB -------------------------------------------------------------
    print!("- Creating PTB...");
    let mut ptb = 
        ProgrammableTransactionBuilder::new();

    let package = MOVE_STDLIB_PACKAGE_ID;
    let module = Identifier::from_str("address")?;
    let function = Identifier::from_str("length")?;
    let type_arguments = vec![];
    let call_args = vec![];
    let _res = ptb.programmable_move_call(
        package,
        module,
        function,
        type_arguments,
        call_args
        );

    // Finalize building ptb
    let pt = ptb.finish();
    println!("done! ---------------------------------------------");
    println!("- Programmable TX:\n- {:#?}", pt);
    println!("{}\n", SEP);

    // Create TX data that will be sent to the network
    let tx_data = TransactionData::new_programmable(
        sender,
        gas_payment,
        pt,
        gas_budget,
        gas_price
    );


    // Sign transaction -------------------------------------------------------
    print!("- Signing TX...");
    let keystore = FileBasedKeystore::new(
        &sui_config_dir()?
        .join(SUI_KEYSTORE_FILENAME)
    )?; 
    let signature = keystore
        .sign_secure(
            &sender,
            &tx_data,
            Intent::sui_transaction()
        )?;
    println!("done! -----------------------------------------------");
    println!("{}\n", SEP);
    

    // Execute transaction ----------------------------------------------------
    print!("- Executing TX...");
    let tx_response = sui_client
        .quorum_driver_api()
        .execute_transaction_block(
            Transaction::from_data(
                tx_data,
                vec![signature]
            ),
            SuiTransactionBlockResponseOptions::full_content(),
            Some(ExecuteTransactionRequestType::WaitForLocalExecution)
        )
        .await?;
    println!("done! ---------------------------------------------");
    println!("- TX digest:\n- {}", tx_response.digest.to_string());
    println!("{}", SEP);

    Ok(())
}
