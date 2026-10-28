//! Wallet / funds API namespace. Mirrors `web/src/lib/api/wallet.ts`.

use crate::shared::data::demo_data;
use crate::shared::types::AccountInfo;

pub fn get_account() -> AccountInfo {
    demo_data::demo_account()
}

pub fn get_balances() -> Vec<(&'static str, f64)> {
    vec![
        ("INR", 620_000.00),
        ("Collateral (securities)", 1_222_650.00),
        ("Blocked margin", 84_500.00),
    ]
}
