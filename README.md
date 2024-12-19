# Stylus dOrg technical assessment

The project implements a fixed-cost token sales contract.

- User can create new market.
- Market stores the base token, quote token and exchange rate.
- User can swap base token for quote token.
- User can swap quote token for base token.

The program is ABI-equivalent with Solidity is shown below:

```solidity
// SPDX-License-Identifier: MIT-OR-APACHE-2.0
pragma solidity ^0.8.23;

interface IContract {
    function initialize() external;

    function createMarket(address base_token, address quote_token, uint256 exchange_rate) external returns (uint256);

    function swapBaseTokenForQuoteToken(address base_token, address quote_token, uint256 base_amount) external;

    function swapQuoteTokenForBaseToken(address base_token, address quote_token, uint256 quote_amount) external;

    function fetchExchangeRate(address base_token, address quote_token) external returns (uint256);

    function fetchMarket(uint64 market_index) external returns (address, address, uint256);

    error AlreadyInitialized();

    error MarketExists();

    error BaseTokenCanNotBeZeroAddress();

    error QuoteTokenCanNotBeZeroAddress();

    error ExchangeRateCanNotBeZero();

    error AmountCanNotBeZero();

    error DivisionUnderflow();

    error MultiplicationOverflow();

    error BaseTokenTransferFailed();

    error QuoteTokenTransferFailed();

    error OutOfBOundIndex();
}
```

## Token Contracts

### Base Token

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract BaseToken is ERC20 {
    constructor() ERC20("BaseToken", "BTN") {
        _mint(msg.sender, 1000000 * 10 ** decimals());
    }
}
```

### Quote Token

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract QuoteToken is ERC20 {
    constructor() ERC20("QuoteToken", "QTN") {
        _mint(msg.sender, 1000000 * 10 ** decimals());
    }
}
```

## Development

Install [Rust](https://www.rust-lang.org/tools/install), and then install the Stylus CLI tool with Cargo

```bash
cargo install --force cargo-stylus cargo-stylus-check
```

Add the `wasm32-unknown-unknown` build target to your Rust compiler:

```bash
rustup target add wasm32-unknown-unknown
```

You should now have it available as a Cargo subcommand:

```bash
cargo stylus --help
```

Then, clone the project:

```bash
git clone https://github.com/cenwadike/stylus-dorg-task && cd stylus-dorg-task
```

### Testnet Information

All testnet information, including faucets and RPC endpoints can be found [here](https://docs.arbitrum.io/stylus/reference/testnet-information).

### ABI Export

You can export the Solidity ABI for your program by using the `cargo stylus` tool as follows:

```bash
cargo stylus export-abi
```

which outputs:

```solidity
// SPDX-License-Identifier: MIT-OR-APACHE-2.0
pragma solidity ^0.8.23;

interface IContract {
    function initialize() external;

    function createMarket(address base_token, address quote_token, uint256 exchange_rate) external returns (uint256);

    function swapBaseTokenForQuoteToken(address base_token, address quote_token, uint256 base_amount) external;

    function fetchInitializationStatus() external view returns (bool);

    function fetchCurrentMarketIndex() external view returns (uint256);

    function fetchExchangeRate(address base_token, address quote_token) external view returns (uint256);

    function fetchMarket(uint64 market_index) external view returns (address, address, uint256);

    error AlreadyInitialized();

    error MarketExists();

    error BaseTokenCanNotBeZeroAddress();

    error QuoteTokenCanNotBeZeroAddress();

    error ExchangeRateCanNotBeZero();

    error AmountCanNotBeZero();

    error DivisionUnderflow();

    error MultiplicationOverflow();

    error BaseTokenTransferFailed();

    error QuoteTokenTransferFailed();

    error OutOfBoundIndex();
}
```

Exporting ABIs uses a feature that is enabled by default in your Cargo.toml:

```toml
[features]
export-abi = ["stylus-sdk/export-abi"]
```

## Set up

### Deploying Market contract

You can use the `cargo stylus` command to also deploy your program to the Stylus testnet. We can use the tool to first check
our program compiles to valid WASM for Stylus and will succeed a deployment onchain without transacting. By default, this will use the Stylus testnet public RPC endpoint. See here for [Stylus testnet information](https://docs.arbitrum.io/stylus/reference/testnet-information)

```bash
cargo stylus check
```

If successful, you should see:

```bash
Finished release [optimized] target(s) in 1.88s
Reading WASM file at stylus-hello-world/target/wasm32-unknown-unknown/release/stylus-hello-world.wasm
Compressed WASM size: 8.9 KB
Program succeeded Stylus onchain activation checks with Stylus version: 1
```

Next, we can estimate the gas costs to deploy and activate our program before we send our transaction. Check out the [cargo-stylus](https://github.com/OffchainLabs/cargo-stylus) README to see the different wallet options for this step:

```bash
cargo stylus deploy \
  --private-key-path=<PRIVKEY_FILE_PATH> \
  --estimate-gas
```

You will then see the estimated gas cost for deploying before transacting:

```bash
Deploying program to address e43a32b54e48c7ec0d3d9ed2d628783c23d65020
Estimated gas for deployment: 1874876
```

The above only estimates gas for the deployment tx by default. To estimate gas for activation, first deploy your program using `--mode=deploy-only`, and then run `cargo stylus deploy` with the `--estimate-gas` flag, `--mode=activate-only`, and specify `--activate-program-address`.


Here's how to deploy:

```bash
cargo stylus deploy \
  --private-key-path=<PRIVKEY_FILE_PATH>
```

The CLI will send 2 transactions to deploy and activate your program onchain.

```bash
Compressed WASM size: 8.9 KB
Deploying program to address 0x457b1ba688e9854bdbed2f473f7510c476a3da09
Estimated gas: 1973450
Submitting tx...
Confirmed tx 0x42db…7311, gas used 1973450
Activating program at address 0x457b1ba688e9854bdbed2f473f7510c476a3da09
Estimated gas: 14044638
Submitting tx...
Confirmed tx 0x0bdb…3307, gas used 14044638
```

### Deployment Token Contract

To set up the project you need to deploy the following contract:

- dOrg task contract (fixed-cost token sales contract).
- base token [contract](#base-token).
- quote token [contract](#quote-token).

### Funding

To prepare fixed-cost token sales contract, you need to fund the
contract address with sufficient amount of base token and quote token

## Calling The Program

This project includes an rust script of how to call and transact with the program in Rust using [ethers-rs](https://github.com/gakonst/ethers-rs) under the `scripts/task.rs`. However, your programs are also Ethereum ABI equivalent if using the Stylus SDK. **They can be called and transacted with using any other Ethereum tooling.**

By using the program address from your deployment step above, and your wallet, you can attempt to call the counter program and increase its value in storage:

```rs
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

// New contract instance.
let contract = Contract::new(contract_address, client);

// Fetch initialization status
let initialization_status = contract.fetchInitializationStatus().call().await;

// Initialize contract
let pending_initialization_tx = contract.initialize();
if let Some(initialization_receipt) = pending_initialization_tx.send().await?.await? {
    println!(
        "Initialized Contract Successfully With Receipt = {:?}",
        initialization_receipt
    );
}

// Create market
let pending_create_market_tx =
    contract.createMarket(base_token_address, quote_token_address, exchange_rate);
if let Some(create_market_receipt) = pending_create_market_tx.send().await?.await? {
    println!(
        "Market Created Successfully With Receipt = {:?}",
        create_market_receipt
    );
}

// Fetch current market index
let current_market_index = contract.fetchCurrentMarketIndex().call().await;

// Fetch market
let (base_token, quote_token, rate) = contract.fetchMarket(current_market_index).call().await;

// Swap base token for quote token
let pending_swap_base_for_quote_tx =
    contract.swapBaseTokenForQuoteToken(base_token_address, quote_token_address, base_amount);
if let Some(swap_base_for_quote_receipt) = pending_swap_base_for_quote_tx.send().await?.await? {
    println!(
        "Swapped Base Token For Quote Token Successfully With Receipt = {:?}",
        swap_base_for_quote_receipt
    );
}
```

Add private-key.txt file containing private key or an arbitrum testnet account.

Before running, set the following env vars or place them in a `.env` file (see: [.env.example](./.env.example)) in this project:

```sh
RPC_URL=https://sepolia-rollup.arbitrum.io/rpc
STYLUS_CONTRACT_ADDRESS=<the onchain address of your deployed program>
BASE_TOKEN_ADDRESS=<the onchain address of your deployed base token>
QUOTE_TOKEN_ADDRESS=<the onchain address of your deployed quote token>
PRIVATE_KEY_PATH=<the file path for your private key to transact with>
```

Next, run:

```sh
cargo run --example simulation --target=<YOUR_ARCHITECTURE>
```

Where you can find `YOUR_ARCHITECTURE` by running `rustc -vV | grep host`. For M1 Apple computers, for example, this is `aarch64-apple-darwin` and for most Linux x86 it is `x86_64-unknown-linux-gnu`

## Build Options

By default, the cargo stylus tool will build your project for WASM using sensible optimizations, but you can control how this gets compiled by seeing the full README for [cargo stylus](https://github.com/OffchainLabs/cargo-stylus). If you wish to optimize the size of your compiled WASM, see the different options available [here](https://github.com/OffchainLabs/cargo-stylus/blob/main/OPTIMIZING_BINARIES.md).

## Peeking Under the Hood

The [stylus-sdk](https://github.com/OffchainLabs/stylus-sdk-rs) contains many features for writing Stylus programs in Rust. It also provides helpful macros to make the experience for Solidity developers easier. These macros expand your code into pure Rust code that can then be compiled to WASM. If you want to see what the `stylus-hello-world` boilerplate expands into, you can use `cargo expand` to see the pure Rust code that will be deployed onchain.

First, run `cargo install cargo-expand` if you don't have the subcommand already, then:

```bash
cargo expand --all-features --release --target=<YOUR_ARCHITECTURE>
```

Where you can find `YOUR_ARCHITECTURE` by running `rustc -vV | grep host`. For M1 Apple computers, for example, this is `aarch64-apple-darwin`.

## License

This project is fully open source, including an Apache-2.0 or MIT license at your choosing under your own copyright.
