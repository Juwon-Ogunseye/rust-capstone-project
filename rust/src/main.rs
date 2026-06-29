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

// this loads the wallet and check if the wallet exist or not and then create a new wallet 
fn load_or_create_wallet(rpc: &Client, name: &str) -> bitcoincore_rpc::Result<()> {
    match rpc.load_wallet(name) {
        Ok(_) => {
            // wallet found and then added to memory 
            println!("Wallet '{}' loaded.", name);
        }
        Err(e) => {
            // Loading failed which means the wallet does not exist yet so we create it
            // ✍️ YOUR VERSION: _______________________________________________
            println!("Wallet '{}' not found, creating it now...", name);
            rpc.create_wallet(name, None, None, None, None)?;
            // new wallet ready to use
            println!("Wallet '{}' created.", name);
        }
    }
    Ok(())
}

fn main() -> bitcoincore_rpc::Result<()> {
    // connecting to bitcoin node via the rpc
    let rpc = Client::new(
        RPC_URL,
        Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
    )?;

    // get the blockchain details and check the node if it connects 
    let blockchain_info = rpc.get_blockchain_info()?;
    println!("Chain: {}", blockchain_info.chain);
    println!("Blocks: {}", blockchain_info.blocks);
    // use the helper function to create wallet 
    load_or_create_wallet(&rpc, "Miner")?;
    load_or_create_wallet(&rpc, "Trader")?;

    println!("Both wallets are ready!");

    Ok(())
}