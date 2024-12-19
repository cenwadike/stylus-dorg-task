//!
//! Stylus dOrg tech assessment
//!
//! The following contract implements a fixed-cost token sales contract.
//!
//! - User can create new market.
//! - Market stores the base token, quote token and exchange rate.
//! - User can swap base token for quote token.
//!
//! The program is ABI-equivalent with Solidity, which means you can call it from both Solidity and Rust.
//! To do this, run `cargo stylus export-abi`.
//!
//! Note: this code is a technical task and has not been audited.
//!

// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

/// Import items from the SDK. The prelude contains common traits and macros.
use alloy_sol_types::sol;
use stylus_sdk::{
    alloy_primitives::{Address, U256, U64},
    call::Call,
    contract::address,
    evm, function_selector, msg,
    prelude::*,
};

// Define some persistent storage using the Solidity ABI.
// `Contract` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct Contract {
        // Initialization status
        bool initialized;
        // Market index
        uint64 market_index;
        // Maps market index to Market data.
        mapping(uint64 => Market) markets;
        // Maps base token and quote token address to market index.
        mapping(address => mapping(address => uint64)) indexes;
    }

    // Market consist of base token, quote token and market rate
    pub struct Market {
        address base_token;     // eg. ETH in ETH/USDT
        address quote_token;    // eg. USDT in ETH/USDT
        uint256 exchange_rate;  // eg. ETH/USDT exchange is 3500
    }
}

// Declare Erc20 interface
sol_interface! {
    interface IErc20 {
        function balanceOf(address) external view returns (uint256);
        function approve(address,uint256) external returns (bool);
        function transfer(address,uint256) external;
        function transferFrom(address,address,uint256) external returns (bool);
    }
}

// Declare events and error types
sol! {
    // Events for the Contract
    event Initialized();
    event MarketCreated(address indexed base_token, address indexed quote_token, uint256 exchange_rate);
    event SwappedQuoteTokenForBaseToken(address indexed base_token, address indexed quote_token, uint256 amount_in, uint256 amount_out);
    event SwappedBaseTokenForQuoteToken(address indexed base_token, address indexed quote_token, uint256 amount_in, uint256 amount_out);

    // Error types for the Contract
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

/// Represents the ways methods may fail.
#[derive(SolidityError)]
pub enum ContractError {
    AlreadyInitialized(AlreadyInitialized),
    MarketExists(MarketExists),
    BaseTokenCanNotBeZeroAddress(BaseTokenCanNotBeZeroAddress),
    QuoteTokenCanNotBeZeroAddress(QuoteTokenCanNotBeZeroAddress),
    ExchangeRateCanNotBeZero(ExchangeRateCanNotBeZero),
    AmountCanNotBeZero(AmountCanNotBeZero),
    DivisionUnderflow(DivisionUnderflow),
    MultiplicationOverflow(MultiplicationOverflow),
    BaseTokenTransferFailed(BaseTokenTransferFailed),
    QuoteTokenTransferFailed(QuoteTokenTransferFailed),
    OutOfBoundIndex(OutOfBoundIndex),
}

/// Declare that `Contract` is a contract with the following external methods.
#[public]
impl Contract {
    /// Initialize contract.
    pub fn initialize(&mut self) -> Result<(), ContractError> {
        // Ensure contract has not being initialized
        if self.initialized.get() {
            return Err(ContractError::AlreadyInitialized(AlreadyInitialized {}));
        }

        // Set initialized.
        self.initialized.set(true);

        // Initialize market index
        self.market_index.set(U64::from(1));

        // Emit event
        evm::log(Initialized {});

        Ok(())
    }

    /// Create new market.
    ///
    /// Return market index.
    pub fn create_market(
        &mut self,
        base_token: Address,
        quote_token: Address,
        exchange_rate: U256,
    ) -> Result<U256, ContractError> {
        // Ensures rate is not 0
        if exchange_rate == U256::from(0) {
            return Err(ContractError::ExchangeRateCanNotBeZero(
                ExchangeRateCanNotBeZero {},
            ));
        }

        // Ensures base token address is not a zero address
        if base_token == Address::ZERO {
            return Err(ContractError::BaseTokenCanNotBeZeroAddress(
                BaseTokenCanNotBeZeroAddress {},
            ));
        }

        // Ensures quote token address is not a zero address
        if quote_token == Address::ZERO {
            return Err(ContractError::QuoteTokenCanNotBeZeroAddress(
                QuoteTokenCanNotBeZeroAddress {},
            ));
        }

        // Get current market index
        let mut current_market_index = self.market_index.get();

        // Ensure market does not exist
        let mut base_token_map = self.indexes.setter(base_token);
        let quote_token_map = base_token_map.setter(quote_token);
        let market_index = quote_token_map.get();

        // Return error if market exists
        if !market_index.is_zero() {
            return Err(ContractError::MarketExists(MarketExists {}));
        }

        // Create new market in storage
        let mut market = self.markets.setter(current_market_index);
        market.base_token.set(base_token);
        market.quote_token.set(quote_token);
        market.exchange_rate.set(exchange_rate);

        // Map (base_token_address, quote_token_address) => market_index
        let mut base_token_map = self.indexes.setter(base_token);
        let mut quote_token_map = base_token_map.setter(quote_token);
        quote_token_map.set(current_market_index);

        // Set new market index
        current_market_index = current_market_index + U64::from(1);
        self.market_index.set(current_market_index);

        // Emit event
        evm::log(MarketCreated {
            base_token,
            quote_token,
            exchange_rate,
        });

        // Return market index
        Ok(U256::from(current_market_index))
    }

    /// Swap base token for quote token.
    pub fn swap_base_token_for_quote_token(
        &mut self,
        base_token: Address,
        quote_token: Address,
        base_amount: U256,
    ) -> Result<(), ContractError> {
        // Ensures amount is not 0
        if base_amount == U256::from(0) {
            return Err(ContractError::AmountCanNotBeZero(AmountCanNotBeZero {}));
        }

        // Ensures base token address is not a zero address
        if base_token == Address::ZERO {
            return Err(ContractError::BaseTokenCanNotBeZeroAddress(
                BaseTokenCanNotBeZeroAddress {},
            ));
        }

        // Ensures quote token address is not a zero address
        if quote_token == Address::ZERO {
            return Err(ContractError::QuoteTokenCanNotBeZeroAddress(
                QuoteTokenCanNotBeZeroAddress {},
            ));
        }

        // Get market from base token and quote token
        let mut base_token_map = self.indexes.setter(base_token);
        let quote_token_map = base_token_map.setter(quote_token);
        let market_index = quote_token_map.get();

        // Get market.
        let market = self.markets.get(market_index);

        // Get market rate.
        let exchange_rate = market.exchange_rate.get();

        // Calculate quote token amount.
        let quote_amount = base_amount.checked_mul(exchange_rate);

        // Return overflow error.
        if quote_amount.is_none() {
            return Err(ContractError::MultiplicationOverflow(
                MultiplicationOverflow {},
            ));
        }

        // Safely unwrap quote amount.
        let quote_amount = quote_amount.unwrap();

        // Transfer base token from user.
        let base_token_contract = IErc20::new(market.base_token.get());
        let _ =
            base_token_contract.transfer_from(Call::new(), msg::sender(), address(), base_amount);

        // Transfer quote token transfer to user.
        let quote_token_contract = IErc20::new(market.quote_token.get());
        let _ = quote_token_contract.transfer(Call::new(), msg::sender(), quote_amount);

        // Emit event.
        evm::log(SwappedBaseTokenForQuoteToken {
            base_token,
            quote_token,
            amount_in: base_amount,
            amount_out: quote_amount,
        });

        Ok(())
    }

    /// Fetch initialization status.
    pub fn fetch_initialization_status(&self) -> Result<bool, ContractError> {
        Ok(self.initialized.get())
    }

    /// Fetch current market index.
    pub fn fetch_current_market_index(&self) -> Result<U256, ContractError> {
        Ok(U256::from(self.market_index.get()))
    }

    /// Fetch exchange rate.
    pub fn fetch_exchange_rate(
        &self,
        base_token: Address,
        quote_token: Address,
    ) -> Result<U256, ContractError> {
        // Get market from base token and quote token.
        let base_token_map = self.indexes.getter(base_token);
        let quote_token_map = base_token_map.getter(quote_token);
        let market_index = quote_token_map.get();

        // Get market.
        let market = self.markets.get(market_index);

        // Get exchange rate.
        let exchange_rate = market.exchange_rate.get();

        Ok(exchange_rate)
    }

    /// Fetch market.
    /// Useful for pagination.
    ///
    /// Return market (base_token, quote_token, exchange_rate).
    pub fn fetch_market(
        &self,
        market_index: u64,
    ) -> Result<(Address, Address, U256), ContractError> {
        // Ensure index is valid.
        if U64::from(market_index).ge(&self.market_index.get())
            || U64::from(market_index).eq(&U64::from(0))
        {
            return Err(ContractError::OutOfBoundIndex(OutOfBoundIndex {}));
        }

        // Get market.
        let market = self.markets.get(U64::from(market_index));

        Ok((
            market.base_token.get(),
            market.quote_token.get(),
            market.exchange_rate.get(),
        ))
    }
}
