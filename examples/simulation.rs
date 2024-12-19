//! Example on how to interact with a deployed `stylus-dorg-task` contract using defaults.
//! This example uses ethers-rs to instantiate the contract using a Solidity ABI.
//!
//! - User create new market.
//! - User swap base token for quote token.
//!

use dotenv::dotenv;
use ethers::{
    middleware::SignerMiddleware,
    prelude::abigen,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, U256},
};
use eyre::eyre;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::sync::Arc;

/// Your private key file path.
const PRIVATE_KEY_PATH: &str = "PRIVATE_KEY_PATH";

/// Stylus RPC endpoint url.
const RPC_URL: &str = "RPC_URL";

/// Deployed program addresses.
const STYLUS_CONTRACT_ADDRESS: &str = "STYLUS_CONTRACT_ADDRESS";
const BASE_TOKEN_ADDRESS: &str = "BASE_TOKEN_ADDRESS";
const QUOTE_TOKEN_ADDRESS: &str = "QUOTE_TOKEN_ADDRESS";

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Load environment variables
    dotenv().ok();

    // Get private key path
    let private_key_path = std::env::var(PRIVATE_KEY_PATH)
        .map_err(|_| eyre!("No {} env var set", PRIVATE_KEY_PATH))?;
    let private_key = read_secret_from_file(&private_key_path)?;

    // Get RPC connection URL
    let rpc_url = std::env::var(RPC_URL).map_err(|_| eyre!("No {} env var set", RPC_URL))?;

    // Get contract address
    let contract_address = std::env::var(STYLUS_CONTRACT_ADDRESS)
        .map_err(|_| eyre!("No {} env var set", STYLUS_CONTRACT_ADDRESS))?;
    let contract_address: Address = contract_address.parse()?;

    // Get base token address
    let base_token_address = std::env::var(BASE_TOKEN_ADDRESS)
        .map_err(|_| eyre!("No {} env var set", BASE_TOKEN_ADDRESS))?;
    let base_token_address: Address = base_token_address.parse()?;

    // Get quote token address
    let quote_token_address = std::env::var(QUOTE_TOKEN_ADDRESS)
        .map_err(|_| eyre!("No {} env var set", QUOTE_TOKEN_ADDRESS))?;
    let quote_token_address: Address = quote_token_address.parse()?;

    // Set up wallet.
    let wallet = LocalWallet::from_str(&private_key)?;

    // Set up rpc client.
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let chain_id = provider.get_chainid().await?.as_u64();
    let client = Arc::new(SignerMiddleware::new(
        provider,
        wallet.clone().with_chain_id(chain_id),
    ));

    // Define Solidity ABIs.
    abigen!(
        Erc20,
        r#"[
            function balanceOf(address) external view returns (uint256)
            function approve(address,uint256) external returns (bool)
            function transfer(address,uint256) external
            function transferFrom(address,address,uint256) external returns (bool)
        ]"#
    );

    abigen!(
        Contract,
        r#"[
            function initialize() external
            function createMarket(address,address,uint256) external returns (uint256)
            function swapBaseTokenForQuoteToken(address,address,uint256) external
            function fetchInitializationStatus() external view returns (bool)
            function fetchCurrentMarketIndex() external view returns (uint256)
            function fetchExchangeRate(address,address) external view returns (uint256)
            function fetchMarket(uint64) external view returns (address,address,uint256)
        ]"#
    );

    // Test amount
    let exchange_rate = 3;
    let base_amount = 10;
    let quote_amount = 30;

    // Set up contracts
    let contract = Contract::new(contract_address, client.clone());
    let base_token_contract = Erc20::new(base_token_address, client.clone());
    let quote_token_contract = Erc20::new(quote_token_address, client);

    // Initialize contract
    let initialization_status = contract.fetch_initialization_status().call().await?;
    println!("Initialization Status = {:?}", initialization_status);

    if !initialization_status {
        let pending_initialization_tx = contract.initialize();
        if let Some(initialization_receipt) = pending_initialization_tx.send().await?.await? {
            println!(
                "Initialized Contract Successfully With Transaction Hash: {:?}",
                initialization_receipt.transaction_hash
            );
        };
    } else {
        println!("Contract Already Initialized");
    }

    // Get current market index
    let current_market_index = contract.fetch_current_market_index().call().await?;

    // Check if market exist
    let market = contract
        .fetch_market(current_market_index.as_u64() - 1)
        .call()
        .await;

    if market.is_err() {
        // create market if it does not exist
        let pending_create_market_tx = contract.create_market(
            base_token_address,
            quote_token_address,
            U256::from(exchange_rate),
        );
        if let Some(create_market_receipt) = pending_create_market_tx
            .send()
            .await
            .unwrap()
            .await
            .unwrap()
        {
            println!(
                "Market Created Successfully With Transaction Hash: {:?}",
                create_market_receipt.transaction_hash
            );
        };

        // Assert index was updated
        assert_eq!(
            contract.fetch_current_market_index().call().await?,
            current_market_index + 1
        );

        // Get newly created market
        let (base_token, quote_token, rate) = contract
            .fetch_market(current_market_index.as_u64())
            .call()
            .await?;

        // Assert market was added
        assert_eq!(base_token_address, base_token);
        assert_eq!(quote_token_address, quote_token);
        assert_eq!(U256::from(exchange_rate), rate);
    }

    // Get contract token balances before swapping base for quote
    let initial_contract_base_token_balance_before_swap_base_for_quote = base_token_contract
        .balance_of(contract_address)
        .call()
        .await?;
    let initial_contract_quote_token_balance_before_swap_base_for_quote = quote_token_contract
        .balance_of(contract_address)
        .call()
        .await?;

    // Get user token balances before swapping base for quote
    let initial_user_base_token_balance_before_swap_base_for_quote = base_token_contract
        .balance_of(wallet.address())
        .call()
        .await?;
    let initial_user_quote_token_balance_before_swap_base_for_quote = quote_token_contract
        .balance_of(wallet.address())
        .call()
        .await?;

    // Approve contract to transfer base token
    let pending_approve_base_tx =
        base_token_contract.approve(contract_address, U256::from(base_amount));
    if let Some(approve_base_receipt) = pending_approve_base_tx.send().await?.await? {
        println!(
            "Approved Base Token Successfully With Transaction Hash: {:?}",
            approve_base_receipt.transaction_hash
        );
    }

    // Swap base token for quote token
    let pending_swap_base_for_quote_tx = contract.swap_base_token_for_quote_token(
        base_token_address,
        quote_token_address,
        U256::from(base_amount),
    );

    if let Some(swap_base_for_quote_receipt) = pending_swap_base_for_quote_tx.send().await?.await? {
        println!(
            "Swapped Base Token For Quote Token Successfully With Transaction Hash: {:?}",
            swap_base_for_quote_receipt.transaction_hash
        );
    }

    // Get contract token balance after swap base for quote
    let final_contract_base_token_balance_after_swap_base_for_quote = base_token_contract
        .balance_of(contract_address)
        .call()
        .await?;
    let final_contract_quote_token_balance_after_swap_base_for_quote = quote_token_contract
        .balance_of(contract_address)
        .call()
        .await?;

    // Get user token balances after swap base for quote
    let final_user_base_token_balance_before_swap_base_for_quote = base_token_contract
        .balance_of(wallet.address())
        .call()
        .await?;
    let final_user_quote_token_balance_before_swap_base_for_quote = quote_token_contract
        .balance_of(wallet.address())
        .call()
        .await?;

    // assert correct contract balance change
    assert_eq!(
        final_contract_base_token_balance_after_swap_base_for_quote,
        initial_contract_base_token_balance_before_swap_base_for_quote + U256::from(base_amount)
    );
    assert_eq!(
        final_contract_quote_token_balance_after_swap_base_for_quote,
        initial_contract_quote_token_balance_before_swap_base_for_quote - quote_amount
    );

    // assert correct user balance change
    assert_eq!(
        final_user_base_token_balance_before_swap_base_for_quote,
        initial_user_base_token_balance_before_swap_base_for_quote - base_amount
    );
    assert_eq!(
        final_user_quote_token_balance_before_swap_base_for_quote,
        initial_user_quote_token_balance_before_swap_base_for_quote + quote_amount
    );

    Ok(())
}

fn read_secret_from_file(fpath: &str) -> eyre::Result<String> {
    let f = std::fs::File::open(fpath)?;
    let mut buf_reader = BufReader::new(f);
    let mut secret = String::new();
    buf_reader.read_line(&mut secret)?;
    Ok(secret.trim().to_string())
}
