use crate::errors::Error;
use soroban_sdk::{Address, Vec,};

/// Bounded transaction history for resource credits
#[derive(Clone, Debug)]
#[cfg_attr(feature = "std", derive(strumpack::Encode, strumpack::Decode))]
pub struct TransactionHistory {
    /// The credit ID this history belongs to
    pub credit_id: Address,
    /// Ordered list of transaction hashes (bounded by MAX_HISTORY)
    pub transactions: Vec<BytesN<32>>,
    /// Current number of transactions stored
    pub len: u32,
    /// Maximum number of transactions to keep in history
    max_history: u32,
}

impl TransactionHistory {
    /// Create new transaction history with a maximum size
    pub fn new(max_history: u32) -> Self {
        Self {
            credit_id: Address::random(),
            transactions: Vec::new(),
            len: 0,
            max_history,
        }
    }

    /// Add a transaction hash to the history
    ///
    /// Returns false if the history is full and the transaction was not added.
    pub fn add_transaction(&mut self, tx_hash: BytesN<32>) -> bool {
        if self.len >= self.max_history {
            return false; // History is full
        }
        self.transactions.push(tx_hash);
        self.len += 1;
        true
    }

    /// Get a transaction by index
    pub fn get_transaction(&self, index: u32) -> Option<BytesN<32>> {
        self.transactions.get(index).cloned()
    }

    /// Get the number of transactions in history
    pub fn length(&self) -> u32 {
        self.len
    }

    /// Check if the history is full
    pub fn is_full(&self) -> bool {
        self.len >= self.max_history
    }

    /// Clear the history
    pub fn clear(&mut self) {
        self.transactions.clear();
        self.len = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::BytesN;

    #[test]
    fn test_history_creation() {
        let history = TransactionHistory::new(10);
        assert_eq!(history.length(), 0);
        assert!(!history.is_full());
    }

    #[test]
    fn test_add_transaction() {
        let mut history = TransactionHistory::new(3);
        
        // Add 3 transactions (max)
        assert!(history.add_transaction(BytesN::from_array(&[0u8; 32])));
        assert!(history.add_transaction(BytesN::from_array(&[1u8; 32])));
        assert!(history.add_transaction(BytesN::from_array(&[2u8; 32])));
        
        // 4th should fail
        assert!(!history.add_transaction(BytesN::from_array(&[3u8; 32])));
        
        assert_eq!(history.length(), 3);
        assert!(history.is_full());
    }

    #[test]
    fn test_get_transaction() {
        let mut history = TransactionHistory::new(5);
        history.add_transaction(BytesN::from_array(&[0u8; 32]));
        history.add_transaction(BytesN::from_array(&[1u8; 32]));
        
        assert_eq!(
            history.get_transaction(0),
            Some(BytesN::from_array(&[0u8; 32]))
        );
        assert_eq!(
            history.get_transaction(1),
            Some(BytesN::from_array(&[1u8; 32]))
        );
        assert_eq!(history.get_transaction(2), None);
    }
}