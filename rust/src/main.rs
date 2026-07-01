
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

// this loads the wallet and checks if the wallet exists or not and then creates a new wallet
fn load_or_create_wallet(rpc: &Client, name: &str) -> bitcoincore_rpc::Result<()> {
    // first get the list of all wallets already loaded in the node
    let loaded_wallets = rpc.list_wallets()?;

    if loaded_wallets.contains(&name.to_string()) {
        // wallet is already loaded and ready, nothing to do
        println!("Wallet '{}' is already loaded.", name);
        return Ok(());
    }

    // try to load the wallet from disk
    match rpc.load_wallet(name) {
        Ok(_) => {
            // wallet was found on disk and loaded into memory
            println!("Wallet '{}' loaded.", name);
        }
        Err(_) => {
            // wallet does not exist at all so we create it fresh
            println!("Wallet '{}' not found, creating it now...", name);
            rpc.create_wallet(name, None, None, None, None)?;
            // new wallet ready to use
            println!("Wallet '{}' created.", name);
        }
    }
    Ok(())
}

fn main() -> bitcoincore_rpc::Result<()> {
    // code to connect the node to rpc 
    let rpc = Client::new(
        RPC_URL,
        Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
    )?;

    //run a check first to be sure it is running 
    let blockchain_info = rpc.get_blockchain_info()?;
    println!("Chain: {}", blockchain_info.chain);
    println!("Blocks: {}", blockchain_info.blocks);

    // use the helper function to create or load both wallets
    load_or_create_wallet(&rpc, "Miner")?;
    load_or_create_wallet(&rpc, "Trader")?;
    println!("Both wallets are ready!");

    // connect directly to the Miner wallet to generate a mining address
    let miner_rpc = Client::new(
        &format!("{}/wallet/Miner", RPC_URL),
        Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
    )?;

    // this is where all mined bitcoin will happen and where it will genereate the wallet and give it a name as "mining rewards"
    let mining_address = miner_rpc.get_new_address(Some("Mining Reward"), None)?;
    let mining_address = mining_address.require_network(bitcoincore_rpc::bitcoin::Network::Regtest).unwrap();
    println!("Mining address: {}", mining_address);

    // so we mine 101 blocks to make the first reward spendable
    println!("Mining blocks, please wait...");
    miner_rpc.generate_to_address(101, &mining_address)?;
    println!("101 blocks mined!");

    // check the bitcoin Miner wallet
    let balance = miner_rpc.get_balance(None, None)?;
    println!("Miner balance: {} BTC", balance);
    Ok(())
}