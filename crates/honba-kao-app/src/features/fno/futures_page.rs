//! Futures term structure and cost of carry (Phase 3.4a) at `/fno/futures`.
//!
//! Mock data only. Futures prices are whole paise (`i64`). Basis is
//! futures minus spot. Annualized basis is integer centi-percent:
//! `basis / spot * 365 / days * 100`.

use super::fmt::{fmt_centi_pct, fmt_paise, fmt_paise_signed};
use honba_kao_ui_core::{
    HfKpi, HfPageHeader, HfPanel, HfPanelHeader, HfTable, HfTableBody, HfTableCell, HfTableHeader,
    HfTableHeaderCell, HfTableRow,
};
use leptos::prelude::*;

/// Underlying spot in paise (24,850.00).
const SPOT_PAISE: i64 = 2_485_000;

struct Expiry {
    label: &'static str,
    /// Calendar days from the mock trade date (2026-10-26).
    days: i64,
    futures_paise: i64,
    /// Share of this month's OI rolled into the next expiry, in centi-percent.
    /// `None` for the farthest expiry, which has no next month.
    rollover_centi_pct: Option<i64>,
}

const EXPIRIES: [Expiry; 3] = [
    Expiry {
        label: "24-Nov-2026",
        days: 29,
        futures_paise: 2_489_025,
        rollover_centi_pct: Some(7_180),
    },
    Expiry {
        label: "29-Dec-2026",
        days: 64,
        futures_paise: 2_497_550,
        rollover_centi_pct: Some(3_640),
    },
    Expiry {
        label: "26-Jan-2027",
        days: 92,
        futures_paise: 2_507_500,
        rollover_centi_pct: None,
    },
];

/// Annualized basis in centi-percent for a basis of `basis_paise` over `days`.
fn annualized_centi_pct(basis_paise: i64, days: i64) -> i64 {
    if days <= 0 {
        return 0;
    }
    basis_paise * 365 * 10_000 / (SPOT_PAISE * days)
}

#[component]
pub fn FuturesPage() -> impl IntoView {
    let near = &EXPIRIES[0];
    let near_carry = annualized_centi_pct(near.futures_paise - SPOT_PAISE, near.days);

    view! {
        <div class="console fno">
            <HfPageHeader title="Futures" subtitle="NIFTY · Derivatives · mock data" />

            <div class="fno__kpis">
                <HfKpi label="Cost of Carry">{format!("{} p.a.", fmt_centi_pct(near_carry))}</HfKpi>
            </div>

            <HfPanel class="fno__futures">
                <HfPanelHeader title="Futures Term Structure" />
                <div class="fno__toolbar">
                    <span class="placeholder__note">
                        {format!("Spot {} · Basis = Futures − Spot · mock data", fmt_paise(SPOT_PAISE))}
                    </span>
                </div>

                <div class="fno__scroll">
                    <HfTable class="fno__table">
                        <HfTableHeader>
                            <HfTableRow>
                                <HfTableHeaderCell>"Expiry"</HfTableHeaderCell>
                                <HfTableHeaderCell class="tnum">"Futures Price"</HfTableHeaderCell>
                                <HfTableHeaderCell class="tnum">"Basis (₹)"</HfTableHeaderCell>
                                <HfTableHeaderCell class="tnum">"Annualized Basis"</HfTableHeaderCell>
                                <HfTableHeaderCell class="tnum">"Rollover %"</HfTableHeaderCell>
                            </HfTableRow>
                        </HfTableHeader>
                        <HfTableBody>
                            {EXPIRIES
                                .iter()
                                .map(|e| {
                                    let basis = e.futures_paise - SPOT_PAISE;
                                    let ann = annualized_centi_pct(basis, e.days);
                                    let rollover = match e.rollover_centi_pct {
                                        Some(v) => fmt_centi_pct(v),
                                        None => "—".to_string(),
                                    };
                                    view! {
                                        <HfTableRow>
                                            <HfTableCell>{e.label}</HfTableCell>
                                            <HfTableCell class="tnum">{fmt_paise(e.futures_paise)}</HfTableCell>
                                            <HfTableCell class="tnum fno__pos">{fmt_paise_signed(basis)}</HfTableCell>
                                            <HfTableCell class="tnum">{fmt_centi_pct(ann)}</HfTableCell>
                                            <HfTableCell class="tnum">{rollover}</HfTableCell>
                                        </HfTableRow>
                                    }
                                })
                                .collect_view()}
                        </HfTableBody>
                    </HfTable>
                </div>
            </HfPanel>
        </div>
    }
}
