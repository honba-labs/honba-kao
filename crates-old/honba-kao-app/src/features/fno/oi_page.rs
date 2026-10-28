//! Open Interest analysis (Phase 3.4a) at `/fno/oi`.
//!
//! Mock data only: no F&O feed or backend is wired. OI is in contracts and
//! change-in-OI is signed. Bars are scaled to the largest OI on the page.

use super::fmt::{fmt_signed, group_int};
use honba_kao_ui_core::{
    HfPageHeader, HfPanel, HfPanelHeader, HfTable, HfTableBody, HfTableCell, HfTableHeader,
    HfTableHeaderCell, HfTableRow,
};
use leptos::prelude::*;

/// Underlying spot in whole rupees.
const SPOT: i64 = 24_850;
/// Strike spacing in whole rupees.
const STRIKE_STEP: i64 = 50;
/// Eleven strikes, ATM - 5 through ATM + 5.
const HALF_SPAN: i64 = 5;
const ROWS: usize = (2 * HALF_SPAN + 1) as usize;

// Mock OI and change-in-OI, indexed from the lowest strike (ATM - 5) upward.
const CALL_OI: [i64; ROWS] = [
    182_400, 215_300, 264_900, 318_700, 402_100, 388_500, 441_200, 296_800, 512_600, 238_400,
    176_900,
];
const PUT_OI: [i64; ROWS] = [
    96_500, 121_800, 188_300, 276_400, 512_700, 412_900, 302_600, 208_200, 143_800, 97_400, 61_200,
];
const CALL_OI_CHG: [i64; ROWS] = [
    -4_200, 3_150, 12_800, -2_400, 27_900, 35_600, -8_100, 6_300, 74_900, -3_700, 2_050,
];
const PUT_OI_CHG: [i64; ROWS] = [
    5_600, -1_900, 9_400, 18_200, 61_300, 15_700, -4_500, 7_800, 11_200, -6_300, 3_400,
];

#[derive(Clone, Copy, Debug)]
struct OiRow {
    strike: i64,
    is_atm: bool,
    call_oi: i64,
    put_oi: i64,
    call_chg: i64,
    put_chg: i64,
}

fn build_rows() -> Vec<OiRow> {
    (0..ROWS)
        .map(|i| {
            let offset = i as i64 - HALF_SPAN;
            OiRow {
                strike: SPOT + offset * STRIKE_STEP,
                is_atm: offset == 0,
                call_oi: CALL_OI[i],
                put_oi: PUT_OI[i],
                call_chg: CALL_OI_CHG[i],
                put_chg: PUT_OI_CHG[i],
            }
        })
        .collect()
}

/// Width in percent of `value` against `max`, clamped to 0..=100.
fn bar_pct(value: i64, max: i64) -> String {
    let pct = if max <= 0 {
        0
    } else {
        (value.max(0) * 100 / max).clamp(0, 100)
    };
    format!("{pct}%")
}

/// Colour class for a change-in-OI value.
fn chg_class(value: i64) -> &'static str {
    if value > 0 {
        "tnum fno__pos"
    } else if value < 0 {
        "tnum fno__neg"
    } else {
        "tnum"
    }
}

#[component]
pub fn OiPage() -> impl IntoView {
    let rows = build_rows();
    let max_oi = rows
        .iter()
        .flat_map(|r| [r.call_oi, r.put_oi])
        .max()
        .unwrap_or(1);

    view! {
        <div class="console fno">
            <HfPageHeader title="Open Interest" subtitle="NIFTY · Derivatives · mock data" />

            <HfPanel class="fno__oi">
                <HfPanelHeader title="OI by Strike" />
                <div class="fno__toolbar">
                    <span class="placeholder__note">
                        "Contracts · ATM ± 5 strikes · bars scaled to max OI · mock data"
                    </span>
                </div>

                <div class="fno__scroll">
                    <HfTable class="fno__table">
                        <HfTableHeader>
                            <HfTableRow>
                                <HfTableHeaderCell class="tnum">"Strike"</HfTableHeaderCell>
                                <HfTableHeaderCell class="tnum">"Call OI"</HfTableHeaderCell>
                                <HfTableHeaderCell class="tnum">"Put OI"</HfTableHeaderCell>
                                <HfTableHeaderCell class="tnum">"Call OI Chg"</HfTableHeaderCell>
                                <HfTableHeaderCell class="tnum">"Put OI Chg"</HfTableHeaderCell>
                                <HfTableHeaderCell>"OI Profile"</HfTableHeaderCell>
                            </HfTableRow>
                        </HfTableHeader>
                        <HfTableBody>
                            {rows
                                .into_iter()
                                .map(|r| {
                                    let class = if r.is_atm { "fno__row--atm" } else { "" };
                                    view! {
                                        <HfTableRow class=class>
                                            <HfTableCell class="tnum fno__strike">
                                                {group_int(r.strike)}
                                            </HfTableCell>
                                            <HfTableCell class="tnum">{group_int(r.call_oi)}</HfTableCell>
                                            <HfTableCell class="tnum">{group_int(r.put_oi)}</HfTableCell>
                                            <HfTableCell class=chg_class(r.call_chg)>
                                                {fmt_signed(r.call_chg)}
                                            </HfTableCell>
                                            <HfTableCell class=chg_class(r.put_chg)>
                                                {fmt_signed(r.put_chg)}
                                            </HfTableCell>
                                            <HfTableCell>
                                                <div class="fno__bar-track">
                                                    <div
                                                        class="fno__bar fno__bar--call"
                                                        style=format!("width: {}", bar_pct(r.call_oi, max_oi))
                                                    ></div>
                                                    <div
                                                        class="fno__bar fno__bar--put"
                                                        style=format!("width: {}", bar_pct(r.put_oi, max_oi))
                                                    ></div>
                                                </div>
                                            </HfTableCell>
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
