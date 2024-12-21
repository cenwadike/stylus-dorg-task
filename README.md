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

    function createMarket(address base_token, address quote_token, uint256 exchange_rate, uint256 base_amount, uint256 quote_amount) external returns (uint256);

    function swapBaseTokenForQuoteToken(address base_token, address quote_token, uint256 base_amount) external;

    function swapQuoteTokenForBaseToken(address base_token, address quote_token, uint256 quote_amount) external;

    function fetchInitializationStatus() external view returns (bool);

    function fetchCurrentMarketIndex() external view returns (uint256);

    function fetchExchangeRate(address base_token, address quote_token) external view returns (uint256);

    function fetchMarketId(address base_token, address quote_token) external view returns (uint256);

    function fetchMarketByTokens(address base_token, address quote_token) external view returns (address, address, uint256);

    function fetchMarketById(uint64 market_index) external view returns (address, address, uint256);

    error AlreadyInitialized();

    error MarketExists();

    error BaseTokenCanNotBeZeroAddress();

    error QuoteTokenCanNotBeZeroAddress();

    error ExchangeRateCanNotBeZero();

    error AmountCanNotBeZero();

    error IncorrectBaseAmount();

    error IncorrectQuoteAmount();

    error BaseTokenTransferFailed();

    error QuoteTokenTransferFailed();

    error DivisionUnderflow();

    error MultiplicationOverflow();

    error OutOfBoundIndex();
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

Run the command below to deploy:

```bash
cargo stylus deploy \
  --private-key-path=<PRIVKEY_FILE_PATH> --endpoint=<RPC_ENDPOINT>
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
user address with sufficient amount of quote token.

## Calling The Program

This project includes an rust script of how to call and transact with the program in Rust using [ethers-rs](https://github.com/gakonst/ethers-rs) under the `examples/simulation.rs`.

By using the program address from your deployment step above, and your wallet, you can attempt to call the program:

Add **private-key.txt** file containing private key of an arbitrum testnet account.

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

You will see an output like so:

```bash
     Running `target/aarch64-apple-darwin/debug/examples/simulation`
Initialized Contract Successfully With Signature: https://sepolia.arbiscan.io/tx/0x091fd535cb39955593fdb3163000ec2f67a1db35f3fcbcc853d1173fd4cec6b6
Approved Base Token Successfully With Signature: https://sepolia.arbiscan.io/tx/0x00208830b581725d802dad2a6278e3f07b38b4a90507f76270ac05e03224f0b9
Approved Quote Token Successfully With Signature: https://sepolia.arbiscan.io/tx/0xfe1bdbef8b52947e782b41b4c0b3ea426797fba9ee0b0b360f3e656dd0bdb447
Market Created Successfully With Signature: https://sepolia.arbiscan.io/tx/0x453b01cfe275430d684a199d7658c581d3d1b53c4b7560d007f1f28a3690dfcf
Approved Base Token Successfully With Signature: https://sepolia.arbiscan.io/tx/0x85e9c08cc0fe654f10cb06f662d1f0a15e86df148dcd33388b03f86b3aad5456
Swapped Base Token For Quote Token Successfully With Signature: https://sepolia.arbiscan.io/tx/0xb4c52f0fd98b67375c824f4277f3446be2af1ff985d773c5bd6aef8fb3a2fa9e
Approved Quote Token Successfully With Signature: https://sepolia.arbiscan.io/tx/0xa99eab597cfa127472efa68e8a2c4d67314600f090b1e01d8fb25322158a9383
Swapped Base Token For Quote Token Successfully With Signature: https://sepolia.arbiscan.io/tx/0x58b8d23a734745ff059c5d4d7c782a8298010c92bfbb8cb32025564c0542a639
```

## License

This project is fully open source, including an Apache-2.0 or MIT license at your choosing under your own copyright.
