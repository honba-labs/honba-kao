//! Single option/futures leg used by the multi-leg payoff engine.
//!
//! A leg encodes one side of a derivatives position: its instrument, strike,
//! premium, quantity, and whether the investor is long or short the contract.
//! All quantities are expressed in absolute units (lots × lot size).
//!
//! All monetary fields use `rust_decimal::Decimal` per the Zero Floats policy
//! (restructure.md Section 5.2 / AGENTS.md Section 2.3).

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Option right: call or put.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptionType {
    #[default]
    Call,
    Put,
}

/// Position side: long (holder) or short (writer).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    #[default]
    Long,
    Short,
}

/// One leg of a multi-leg derivatives strategy.
///
/// `premium` is the price per unit at which the leg was opened (paid when long,
/// received when short). `quantity` is the total number of underlying units
/// covered by the leg (lots × lot size).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Leg {
    /// Underlying symbol (e.g. "NIFTY24NOV22500CE").
    pub symbol: String,
    /// Option type for options legs; unused for futures.
    pub option_type: OptionType,
    /// Strike price of the contract.
    pub strike: Decimal,
    /// Premium per unit at entry.
    pub premium: Decimal,
    /// Number of underlying units covered.
    pub quantity: Decimal,
    /// Long (holder) or short (writer).
    pub side: Side,
}

impl Leg {
    /// Intrinsic value per unit at a given spot price.
    pub fn intrinsic_value(&self, spot: Decimal) -> Decimal {
        match self.option_type {
            OptionType::Call => (spot - self.strike).max(Decimal::ZERO),
            OptionType::Put => (self.strike - spot).max(Decimal::ZERO),
        }
    }

    /// Per-unit P&L of this leg at a given spot price.
    ///
    /// For a long leg: intrinsic value minus premium paid.
    /// For a short leg: premium received minus intrinsic value.
    pub fn pnl_per_unit(&self, spot: Decimal) -> Decimal {
        let intrinsic = self.intrinsic_value(spot);
        match self.side {
            Side::Long => intrinsic - self.premium,
            Side::Short => self.premium - intrinsic,
        }
    }

    /// Total P&L of this leg at a given spot price.
    pub fn pnl(&self, spot: Decimal) -> Decimal {
        self.pnl_per_unit(spot) * self.quantity
    }
}
