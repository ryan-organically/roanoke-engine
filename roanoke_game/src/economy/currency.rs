//! Dual Currency System
//!
//! Wampum (WPM) - Utility currency, earned through play
//! Tobacco (TBC) - Premium currency, deflationary store of value

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Player wallet containing both currencies
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Wallet {
    /// Wampum - the utility currency
    pub wampum: u64,

    /// Tobacco - the premium currency
    pub tobacco: u64,

    /// Lifetime earnings
    pub lifetime_wampum_earned: u64,
    pub lifetime_tobacco_earned: u64,

    /// Lifetime spending
    pub lifetime_wampum_spent: u64,
    pub lifetime_tobacco_spent: u64,

    /// Transaction history (last 100)
    #[serde(default)]
    pub transaction_history: Vec<CurrencyTransaction>,
}

impl Wallet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add wampum to wallet
    pub fn add_wampum(&mut self, amount: u64) {
        self.wampum = self.wampum.saturating_add(amount);
        self.lifetime_wampum_earned = self.lifetime_wampum_earned.saturating_add(amount);
    }

    /// Spend wampum (returns false if insufficient)
    pub fn spend_wampum(&mut self, amount: u64) -> bool {
        if self.wampum >= amount {
            self.wampum -= amount;
            self.lifetime_wampum_spent = self.lifetime_wampum_spent.saturating_add(amount);
            true
        } else {
            false
        }
    }

    /// Add tobacco to wallet
    pub fn add_tobacco(&mut self, amount: u64) {
        self.tobacco = self.tobacco.saturating_add(amount);
        self.lifetime_tobacco_earned = self.lifetime_tobacco_earned.saturating_add(amount);
    }

    /// Spend tobacco (returns false if insufficient)
    pub fn spend_tobacco(&mut self, amount: u64) -> bool {
        if self.tobacco >= amount {
            self.tobacco -= amount;
            self.lifetime_tobacco_spent = self.lifetime_tobacco_spent.saturating_add(amount);
            true
        } else {
            false
        }
    }

    /// Get net worth in wampum-equivalent
    pub fn net_worth(&self, tbc_to_wpm_rate: u64) -> u64 {
        self.wampum + (self.tobacco * tbc_to_wpm_rate)
    }

    /// Record a transaction
    pub fn record_transaction(&mut self, tx: CurrencyTransaction) {
        self.transaction_history.push(tx);
        // Keep last 100
        if self.transaction_history.len() > 100 {
            self.transaction_history.remove(0);
        }
    }

    /// Transfer wampum with transaction recording
    pub fn transfer_wampum(&mut self, amount: u64, tx_type: TransactionType, description: &str) -> bool {
        if self.spend_wampum(amount) {
            self.record_transaction(CurrencyTransaction {
                transaction_type: tx_type,
                wampum_amount: -(amount as i64),
                tobacco_amount: 0,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                description: description.to_string(),
            });
            true
        } else {
            false
        }
    }

    /// Receive wampum with transaction recording
    pub fn receive_wampum(&mut self, amount: u64, tx_type: TransactionType, description: &str) {
        self.add_wampum(amount);
        self.record_transaction(CurrencyTransaction {
            transaction_type: tx_type,
            wampum_amount: amount as i64,
            tobacco_amount: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            description: description.to_string(),
        });
    }
}

/// Currency conversion rates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRates {
    /// How much WPM for 1 TBC
    pub tbc_to_wpm: u64,

    /// Minimum WPM per TBC (floor)
    pub tbc_floor: u64,

    /// Conversion tax (burned, deflationary)
    pub conversion_tax_percent: f32,
}

impl Default for ExchangeRates {
    fn default() -> Self {
        Self {
            tbc_to_wpm: 1000, // 1 TBC = 1000 WPM
            tbc_floor: 500,   // Never below 500 WPM per TBC
            conversion_tax_percent: 5.0, // 5% burned on conversion
        }
    }
}

impl ExchangeRates {
    /// Convert wampum to tobacco
    pub fn wpm_to_tbc(&self, wampum: u64) -> (u64, u64) {
        let before_tax = wampum / self.tbc_to_wpm;
        let tax = (before_tax as f32 * (self.conversion_tax_percent / 100.0)) as u64;
        let after_tax = before_tax.saturating_sub(tax);
        (after_tax, tax)
    }

    /// Convert tobacco to wampum
    pub fn tbc_to_wpm(&self, tobacco: u64) -> (u64, u64) {
        let before_tax = tobacco * self.tbc_to_wpm;
        let tax = (before_tax as f32 * (self.conversion_tax_percent / 100.0)) as u64;
        let after_tax = before_tax.saturating_sub(tax);
        (after_tax, tax)
    }

    /// Update exchange rate based on market conditions
    pub fn update_rate(&mut self, new_rate: u64) {
        self.tbc_to_wpm = new_rate.max(self.tbc_floor);
    }
}

/// Currency transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyTransaction {
    pub transaction_type: TransactionType,
    pub wampum_amount: i64,   // Positive = gained, negative = spent
    pub tobacco_amount: i64,
    pub timestamp: u64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionType {
    LootDrop,
    NpcSale,
    NpcPurchase,
    PlayerTrade,
    CurrencyConversion,
    QuestReward,
    RepairCost,
    FastTravel,
    MarketplaceFee,
    Sacrifice,
    CraftingCost,
    SkillTraining,
}

/// Economy statistics for tracking and balancing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EconomyStats {
    /// Total wampum in circulation
    pub total_wampum_supply: u64,

    /// Total tobacco in circulation
    pub total_tobacco_supply: u64,

    /// Wampum burned (sinks)
    pub total_wampum_burned: u64,

    /// Tobacco burned (sinks)
    pub total_tobacco_burned: u64,

    /// Average wampum per player
    pub avg_wampum_per_player: u64,

    /// Velocity (transactions per day)
    pub daily_transaction_volume: u64,
}
