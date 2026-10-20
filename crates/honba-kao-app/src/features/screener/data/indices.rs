//! Market indices registry — the port of the labs app's `core/indices.ts`
//! (`MARKET_INDICES` + `instrumentBelongsToIndex` heuristics).

use crate::features::screener::data::instruments::{Instrument, by_symbol};

/// One index definition (`Nifty 50`, `S&P 500`, …).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IndexItem {
    pub code: &'static str,
    pub name: &'static str,
    pub country: &'static str,
    /// Short badge shown on index chips.
    pub badge: &'static str,
}

pub const MARKET_INDICES: &[IndexItem] = &[
    IndexItem {
        code: "NIFTY",
        name: "Nifty 50",
        country: "IN",
        badge: "50",
    },
    IndexItem {
        code: "SENSEX",
        name: "BSE Sensex",
        country: "IN",
        badge: "BSE",
    },
    IndexItem {
        code: "BANKNIFTY",
        name: "Nifty Bank",
        country: "IN",
        badge: "BANK",
    },
    IndexItem {
        code: "CNXFINANCE",
        name: "Nifty Financial Services",
        country: "IN",
        badge: "FIN",
    },
    IndexItem {
        code: "CNX500",
        name: "Nifty 500",
        country: "IN",
        badge: "500",
    },
    IndexItem {
        code: "NIFTYJR",
        name: "Nifty Next 50",
        country: "IN",
        badge: "N50",
    },
    IndexItem {
        code: "CNXAUTO",
        name: "Nifty Auto",
        country: "IN",
        badge: "AUTO",
    },
    IndexItem {
        code: "CNXPHARMA",
        name: "Nifty Pharma",
        country: "IN",
        badge: "PHARMA",
    },
    IndexItem {
        code: "CNXPSUBANK",
        name: "Nifty PSU Bank",
        country: "IN",
        badge: "PSU",
    },
    IndexItem {
        code: "CNXIT",
        name: "Nifty IT",
        country: "IN",
        badge: "IT",
    },
    IndexItem {
        code: "CNXMETAL",
        name: "Nifty Metal",
        country: "IN",
        badge: "METAL",
    },
    IndexItem {
        code: "CNXFMCG",
        name: "Nifty FMCG",
        country: "IN",
        badge: "FMCG",
    },
    IndexItem {
        code: "CNXREALTY",
        name: "Nifty Realty",
        country: "IN",
        badge: "REALTY",
    },
    IndexItem {
        code: "NIFTYFINSRV25_50",
        name: "Nifty Financial Services 25/50",
        country: "IN",
        badge: "F25",
    },
    IndexItem {
        code: "SPX",
        name: "S&P 500",
        country: "US",
        badge: "500",
    },
    IndexItem {
        code: "NDX",
        name: "NASDAQ 100",
        country: "US",
        badge: "100",
    },
    IndexItem {
        code: "DJI",
        name: "Dow Jones Industrial",
        country: "US",
        badge: "30",
    },
    IndexItem {
        code: "RUT",
        name: "Russell 2000",
        country: "US",
        badge: "2K",
    },
    IndexItem {
        code: "SOX",
        name: "PHLX Semiconductor",
        country: "US",
        badge: "SOX",
    },
    IndexItem {
        code: "XLK",
        name: "Technology Select",
        country: "US",
        badge: "XLK",
    },
    IndexItem {
        code: "XLF",
        name: "Financial Select",
        country: "US",
        badge: "XLF",
    },
    IndexItem {
        code: "XLE",
        name: "Energy Select",
        country: "US",
        badge: "ENG",
    },
    IndexItem {
        code: "XLV",
        name: "Health Care Select",
        country: "US",
        badge: "XLV",
    },
    IndexItem {
        code: "N225",
        name: "Nikkei 225",
        country: "JP",
        badge: "225",
    },
    IndexItem {
        code: "TOPIX",
        name: "TOPIX Index",
        country: "JP",
        badge: "TPX",
    },
    IndexItem {
        code: "JPX400",
        name: "JPX-Nikkei 400",
        country: "JP",
        badge: "400",
    },
    IndexItem {
        code: "FTSE100",
        name: "FTSE 100",
        country: "UK",
        badge: "100",
    },
    IndexItem {
        code: "FTSE250",
        name: "FTSE 250",
        country: "UK",
        badge: "250",
    },
    IndexItem {
        code: "FTSEALL",
        name: "FTSE All-Share",
        country: "UK",
        badge: "ALL",
    },
];

/// Index codes known as tradeable instruments (the search "Index" category).
pub fn is_index_symbol(symbol: &str) -> bool {
    let upper = symbol.to_uppercase();
    MARKET_INDICES.iter().any(|index| index.code == upper)
}

/// Indices offered for one market, excluding the ones not backed by the
/// bundled universe (`NIFTY`/`SENSEX` rows exist as index instruments).
pub fn indices_for_market(country: &str) -> Vec<&'static IndexItem> {
    MARKET_INDICES
        .iter()
        .filter(|index| index.country == country)
        .collect()
}

fn industry_contains(instrument: &Instrument, needle: &str) -> bool {
    instrument.industry.to_lowercase().contains(needle)
}

fn symbol_in(instrument: &Instrument, symbols: &[&str]) -> bool {
    symbols.iter().any(|symbol| *symbol == instrument.symbol)
}

/// Membership heuristic — faithful port of `instrumentBelongsToIndex`.
pub fn instrument_belongs_to_index(instrument: &Instrument, index_code: &str) -> bool {
    let code = index_code.to_uppercase();
    let country_market_cap = instrument.market_cap;

    if instrument.country == "IN" {
        match code.as_str() {
            "CNX500" => true,
            "NIFTY" => {
                instrument.market_cap_tier == "mega" || country_market_cap >= 1_500_000_000_000.0
            }
            "SENSEX" => {
                instrument.market_cap_tier == "mega" || country_market_cap >= 1_800_000_000_000.0
            }
            "NIFTYJR" => instrument.market_cap_tier == "large",
            "BANKNIFTY" => {
                instrument.sector == "Financials"
                    && (industry_contains(instrument, "bank")
                        || symbol_in(
                            instrument,
                            &[
                                "HDFCBANK",
                                "ICICIBANK",
                                "SBIN",
                                "KOTAKBANK",
                                "AXISBANK",
                                "INDUSINDBK",
                                "BANDHANBNK",
                                "FEDERALBNK",
                                "PNB",
                                "BANKBARODA",
                            ],
                        ))
            }
            "CNXPSUBANK" => {
                instrument.symbol == "SBIN"
                    || instrument.industry.to_lowercase().contains("public")
                    || symbol_in(
                        instrument,
                        &["SBIN", "PNB", "BANKBARODA", "CANBK", "UNIONBANK", "INDIANB"],
                    )
            }
            "CNXFINANCE" | "NIFTYFINSRV25_50" => instrument.sector == "Financials",
            "CNXAUTO" => {
                instrument.sector == "Automotive"
                    || industry_contains(instrument, "auto")
                    || symbol_in(
                        instrument,
                        &[
                            "MARUTI",
                            "TATAMOTORS",
                            "M&M",
                            "BAJAJ-AUTO",
                            "HEROMOTOCO",
                            "EICHERMOT",
                            "TVSMOTOR",
                            "HEROMOTORS",
                            "OLAELEC",
                        ],
                    )
            }
            "CNXPHARMA" => {
                instrument.sector == "Healthcare"
                    || industry_contains(instrument, "pharma")
                    || industry_contains(instrument, "health")
                    || symbol_in(
                        instrument,
                        &[
                            "SUNPHARMA",
                            "DRREDDY",
                            "CIPLA",
                            "DIVISLAB",
                            "APOLLOHOSP",
                            "BIOCON",
                            "LUPIN",
                            "MANKIND",
                            "CUPID",
                        ],
                    )
            }
            "CNXIT" => {
                instrument.sector == "Technology"
                    || industry_contains(instrument, "it")
                    || industry_contains(instrument, "software")
                    || symbol_in(
                        instrument,
                        &[
                            "TCS",
                            "INFY",
                            "HCLTECH",
                            "WIPRO",
                            "TECHM",
                            "LTIM",
                            "PERSISTENT",
                            "COFORGE",
                            "POLICYBZR",
                        ],
                    )
            }
            "CNXMETAL" => {
                instrument.sector == "Materials"
                    || industry_contains(instrument, "metal")
                    || industry_contains(instrument, "steel")
                    || industry_contains(instrument, "mining")
                    || symbol_in(
                        instrument,
                        &[
                            "TATASTEEL",
                            "JSWSTEEL",
                            "HINDALCO",
                            "COALINDIA",
                            "VEDL",
                            "NMDC",
                            "SAIL",
                        ],
                    )
            }
            "CNXFMCG" => {
                instrument.sector == "Consumer Defensive"
                    || industry_contains(instrument, "fmcg")
                    || industry_contains(instrument, "food")
                    || symbol_in(
                        instrument,
                        &[
                            "ITC",
                            "HINDUNILVR",
                            "NESTLEIND",
                            "BRITANNIA",
                            "DABUR",
                            "MARICO",
                            "GODREJCP",
                            "TATACONSUM",
                            "SSRETAIL",
                        ],
                    )
            }
            "CNXREALTY" => {
                instrument.sector == "Real Estate"
                    || industry_contains(instrument, "realty")
                    || industry_contains(instrument, "real estate")
                    || symbol_in(
                        instrument,
                        &[
                            "DLF",
                            "GODREJPROP",
                            "OBEROIRLTY",
                            "PHOENIXLTD",
                            "RAYMONDREL",
                            "MACROTECH",
                        ],
                    )
            }
            _ => false,
        }
    } else if instrument.country == "US" {
        match code.as_str() {
            "SPX" => true,
            "NDX" => instrument.exchange == "NASDAQ",
            "DJI" => instrument.market_cap_tier == "mega",
            "RUT" => instrument.market_cap_tier == "mid" || instrument.market_cap_tier == "small",
            "XLK" => instrument.sector == "Technology",
            "XLF" => instrument.sector == "Financials",
            "XLE" => instrument.sector == "Energy",
            "XLV" => instrument.sector == "Healthcare",
            "SOX" => {
                instrument.industry.to_lowercase().contains("semiconductor")
                    || symbol_in(
                        instrument,
                        &["NVDA", "AMD", "INTC", "AVGO", "QCOM", "TXN", "MU"],
                    )
            }
            _ => false,
        }
    } else {
        // JP / UK have heuristic-free Fallthrough in the labs app, matched by
        // universe rows with the same code when they exist.
        by_symbol(&code).map_or(true, |row| row.symbol.to_uppercase() != code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nifty_membership_follows_the_labs_thresholds() {
        let tcs = by_symbol("TCS").unwrap();
        assert!(instrument_belongs_to_index(tcs, "NIFTY"));
        assert!(instrument_belongs_to_index(tcs, "CNXIT"));
        assert!(!instrument_belongs_to_index(tcs, "CNXPHARMA"));
    }

    #[test]
    fn us_industry_indices_match() {
        let nvda = by_symbol("NVDA").unwrap();
        assert!(instrument_belongs_to_index(nvda, "SOX"));
        assert!(!instrument_belongs_to_index(nvda, "XLF"));
    }
}
