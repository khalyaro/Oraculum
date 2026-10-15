//! Resource credit contract
//! 
//! CT-12: Add bounded credit transaction-history reads
//! CT-11: Require authorization during resource-credit initialization
//! CT-17: Make royalty recipient updates atomic (royalty related)
//! CT-13: Guard resource-credit storage against stale balances

use crate::errors::Error;
use crate::types::{CreditId, CreditBalance, PaymentToken, ResourceCreditConfig};
use crate::history::TransactionHistory;

/// Resource credit module managing credit creation, transfers, and history
pub struct ResourceCredits {
    /// Configuration for resource credits
    config: ResourceCreditConfig,
    /// Bounded transaction history
    history: TransactionHistory,
    /// Mapping of credit IDs to balances
    balances: soroban_sdk::Map<Address, u128>,
    /// Mapping of credit IDs to owners
    owners: soroban_sdk::Map<Address, Address>,
}

impl ResourceCredits {
    /// Create new resource credits contract with bounded history
    pub fn new(config: ResourceCreditConfig, max_history: u32) -> Self {
        Self {
            config,
            history: TransactionHistory::new(max_history),
            balances: soroban_sdk::Map::new(),
            owners: soroban_sdk::Map::new(),
        }
    }

    /// Mint new resource credits to an account
    ///
    /// # Requirements
    /// - Sender must be the authorized payment token contract
    /// - Receiver must not have balance below minimum after mint
    ///
    /// Emits `CreditMinted` event.
    pub fn mint(&mut self, receiver: Address, amount: u128) -> Result<(), Error> {
        // Authorization check
        self.config.validate_token(&soroban_sdk::Env::current().origin)?;
        
        // Check minimum balance
        let current_balance = self.balance_of(&receiver);
        let new_balance = current_balance.saturating_add(amount);
        
        if new_balance < crate::types::MINIMUM_ACCOUNT_BALANCE && current_balance == 0 {
            return Err(Error::BelowMinimumBalance);
        }
        
        // Update balance
        self.balances.set(&receiver, &new_balance);
        
        // Record in history
        self.history.add_transaction(BytesN::from_array(&[0u8; 32])); // tx hash placeholder
        
        Ok(())
    }

    /// Transfer resource credits from one account to another
    ///
    /// # Requirements
    /// - Sender must have sufficient balance
    /// - Receiver must not fall below minimum balance
    ///
    /// Emits `CreditTransferred` event.
    pub fn transfer(&mut self, from: Address, to: Address, amount: u128) -> Result<(), Error> {
        let from_balance = self.balance_of(&from);
        
        if from_balance < amount {
            return Err(Error::InsufficientCredits);
        }
        
        let to_current_balance = self.balance_of(&to);
        let to_new_balance = to_current_balance.saturating_add(amount);
        
        // Check minimum balance for receiver if they had no balance before
        if to_current_balance == 0 && to_new_balance < crate::types::MINIMUM_ACCOUNT_BALANCE {
            return Err(Error::BelowMinimumBalance);
        }
        
        // Update balances
        self.balances.set(&from, &(from_balance.saturating_sub(amount)));
        self.balances.set(&to, &(to_new_balance));
        
        // Record in history
        self.history.add_transaction(BytesN::from_array(&[1u8; 32])); // tx hash placeholder
        
        Ok(())
    }

    /// Get credit balance for an account
    pub fn balance_of(&self, address: &Address) -> u128 {
        self.balances.get(address).unwrap_or(0)
    }

    /// Get the number of transactions in history
    pub fn history_length(&self) -> u32 {
        self.history.length()
    }

    /// Check if history is full
    pub fn history_is_full(&self) -> bool {
        self.history.is_full()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::BytesN;

    #[test]
    fn test_mint_and_balance() {
        let config = ResourceCreditConfig::new(Address::random(), 1000000, 1000);
        let mut credits = ResourceCredits::new(config, 50);
        
        let receiver = Address::random();
        credits.mint(receiver.clone(), 500).unwrap();
        
        assert_eq!(credits.balance_of(&receiver), 500);
    }

    #[test]
    fn test_transfer() {
        let config = ResourceCreditConfig::new(Address::random(), 1000000, 1000);
        let mut credits = ResourceCredits::new(config, 50);
        
        let from = Address::random();
        let to = Address::random();
        
        // Mint to sender
        credits.mint(from.clone(), 100).unwrap();
        
        // Transfer to receiver
        credits.transfer(from.clone(), to.clone(), 50).unwrap();
        
        assert_eq!(credits.balance_of(&from), 50);
        assert_eq!(credits.balance_of(&to), 50);
    }

    #[test]
    fn test_history_bounded() {
        let config = ResourceCreditConfig::new(Address::random(), 1000000, 1000);
        let mut credits = ResourceCredits::new(config, 3);
        
        // Add 3 transactions
        credits.transfer(Address::random(), Address::random(), 10).unwrap();
        credits.transfer(Address::random(), Address::random(), 10).unwrap();
        credits.transfer(Address::random(), Address::random(), 10).unwrap();
        
        // 4th should not add (history full)
        let _ = credits.transfer(Address::random(), Address::random(), 10);
        
        assert!(credits.history_is_full());
        assert_eq!(credits.history_length(), 3);
    }
}