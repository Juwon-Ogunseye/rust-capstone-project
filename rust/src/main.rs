#![allow(unused)]
use bitcoin::hex::DisplayHex;
use bitcoincore_rpc::bitcoin::Amount;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use serde::Deserialize;
use serde_json::json;
use std::fs::File;
use std::io::Write;

const RPC_URL: &str = "http://127.0.0.1:18443";
const RPC_USER: &str = "alice";
const RPC_PASS: &str = "password";

// this part check if wallet exist then if it is not existing it will create
fn load_or_create_wallet(bitcoin_node: &Client, wallet_name: &str) -> bitcoincore_rpc::Result<()> {
    // check the node and then load the wallet in the node
    let list_of_loaded_wallets = bitcoin_node.list_wallets()?;

    if list_of_loaded_wallets.contains(&wallet_name.to_string()) {
        // i will remove this created it to check if a wallet it ready to use
        println!("Wallet '{}' is already loaded.", wallet_name);
        return Ok(());
    }

    // move wallet to memmory. let me use some concept from keg videos
    match bitcoin_node.load_wallet(wallet_name) {
        Ok(_) => {
            println!("Wallet '{}' loaded from disk.", wallet_name);
        }
        Err(_) => {
            // if a wallet does not exist it will create new one
            println!("Wallet '{}' not found, creating it now...", wallet_name);
            bitcoin_node.create_wallet(wallet_name, None, None, None, None)?;

            println!("Wallet '{}' created successfully.", wallet_name);
        }
    }
    Ok(())
}

fn main() -> bitcoincore_rpc::Result<()> {
    // connect to regtest node using rpc url
    let connection_to_bitcoin_node = Client::new(
        RPC_URL,
        Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
    )?;

    // get the blockchain details usingthe get_blockchain command
    let blockchain_details = connection_to_bitcoin_node.get_blockchain_info()?;
    println!("Chain: {}", blockchain_details.chain);
    println!("Blocks: {}", blockchain_details.blocks);

    //load miner and traders wallet here
    load_or_create_wallet(&connection_to_bitcoin_node, "Miner")?;
    load_or_create_wallet(&connection_to_bitcoin_node, "Trader")?;
    println!("Both wallets are ready!");

    // connect to the miner wallet so we can load new address
    let connection_to_miner_wallet = Client::new(
        &format!("{}/wallet/Miner", RPC_URL),
        Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
    )?;

    // create new address called mining reward
    let miner_receiving_address =
        connection_to_miner_wallet.get_new_address(Some("Mining Reward"), None)?;
    let miner_receiving_address = miner_receiving_address
        .require_network(bitcoincore_rpc::bitcoin::Network::Regtest)
        .unwrap();
    println!("Miner receiving address: {}", miner_receiving_address);

    // mine 101 block for 50 btc spendable reward
    println!("Mining blocks, please wait...");
    connection_to_miner_wallet.generate_to_address(101, &miner_receiving_address)?;
    println!("101 blocks mined!");

    // print balance of miner
    let miner_wallet_balance = connection_to_miner_wallet.get_balance(None, None)?;
    println!("Miner balance: {}", miner_wallet_balance.to_btc());

    let connection_to_trader_wallet = Client::new(
        &format!("{}/wallet/Trader", RPC_URL),
        Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
    )?;

    // create new lable address so that miners will send 20 btc inside
    let trader_receiving_address =
        connection_to_trader_wallet.get_new_address(Some("Received"), None)?;
    let trader_receiving_address = trader_receiving_address
        .require_network(bitcoincore_rpc::bitcoin::Network::Regtest)
        .unwrap();
    println!("Trader receiving address: {}", trader_receiving_address);

    let sent_transaction_id = connection_to_miner_wallet.send_to_address(
        &trader_receiving_address,
        Amount::from_btc(20.0).unwrap(),
        None,
        None,
        None,
        None,
        None,
        None,
    )?;
    println!("Transaction sent! ID: {}", sent_transaction_id);

    // Fetch the unconfirmed transaction from the mempool before it is confirmed
    let unconfirmed_transaction =
        connection_to_bitcoin_node.get_mempool_entry(&sent_transaction_id)?;
    let transaction_fee = unconfirmed_transaction.fees.base;
    println!("Transaction fee: {}", transaction_fee.to_btc());

    // mine block and move it from mempool to blockchain
    connection_to_miner_wallet.generate_to_address(1, &miner_receiving_address)?;
    println!("1 block mined to confirm the transaction!");

    let confirmed_transaction =
        connection_to_bitcoin_node.get_raw_transaction_info(&sent_transaction_id, None)?;

    let confirmed_block_hash = confirmed_transaction.blockhash.unwrap();
    let confirmed_block_details =
        connection_to_bitcoin_node.get_block_info(&confirmed_block_hash)?;
    let confirmed_block_height = confirmed_block_details.height;

    let first_input = &confirmed_transaction.vin[0];
    let previous_transaction_id = first_input.txid.unwrap();
    let previous_transaction_output_index = first_input.vout.unwrap();
    let previous_transaction =
        connection_to_bitcoin_node.get_raw_transaction_info(&previous_transaction_id, None)?;
    let previous_transaction_output =
        &previous_transaction.vout[previous_transaction_output_index as usize];
    let miner_input_address = previous_transaction_output
        .script_pub_key
        .address
        .clone()
        .unwrap()
        .assume_checked()
        .to_string();
    let miner_amount_spent = previous_transaction_output.value;

    let mut trader_output_address = String::new();
    let mut trader_amount_received = Amount::ZERO;
    let mut miner_change_address = String::new();
    let mut miner_change_amount = Amount::ZERO;

    for output in &confirmed_transaction.vout {
        let output_address = output
            .script_pub_key
            .address
            .clone()
            .unwrap()
            .assume_checked()
            .to_string();
        if output.value == Amount::from_btc(20.0).unwrap() {
            // This output is 20 BTC so it belongs to the Trader
            trader_output_address = output_address;
            trader_amount_received = output.value;
        } else {
            // This output is the leftover change going back to the Miner also
            miner_change_address = output_address;
            miner_change_amount = output.value;
        }
    }

    // all transaction i will print to know if there is any error
    println!("Transaction ID: {}", sent_transaction_id);
    println!("Miner input address: {}", miner_input_address);
    println!("Miner amount spent: {}", miner_amount_spent.to_btc());
    println!("Trader output address: {}", trader_output_address);
    println!(
        "Trader amount received: {}",
        trader_amount_received.to_btc()
    );
    println!("Miner change address: {}", miner_change_address);
    println!("Miner change amount: {}", miner_change_amount.to_btc());
    println!("Transaction fee: {}", transaction_fee.to_btc());
    println!("Block height: {}", confirmed_block_height);
    println!("Block hash: {}", confirmed_block_hash);

    let mut output_file = File::create("../out.txt")?;
    writeln!(output_file, "{}", sent_transaction_id)?;
    writeln!(output_file, "{}", miner_input_address)?;
    writeln!(output_file, "{}", miner_amount_spent.to_btc())?;
    writeln!(output_file, "{}", trader_output_address)?;
    writeln!(output_file, "{}", trader_amount_received.to_btc())?;
    writeln!(output_file, "{}", miner_change_address)?;
    writeln!(output_file, "{}", miner_change_amount.to_btc())?;
    writeln!(output_file, "{}", transaction_fee.to_btc())?;
    writeln!(output_file, "{}", confirmed_block_height)?;
    writeln!(output_file, "{}", confirmed_block_hash)?;
    println!("out.txt written successfully!");

    Ok(())
}
