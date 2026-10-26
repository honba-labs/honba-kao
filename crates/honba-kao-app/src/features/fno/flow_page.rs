//! Options flow tape (Phase 3.4a) at `/fno/flow`.
//!
//! Mock prints only; no exchange or vendor feed is wired. Premium is per
//! unit in whole paise. The tag marks large prints ("Block") and multi-venue
//! sweeps ("Sweep"); blank means neither.

use super::fmt::{fmt_paise, group_int};
use honba_kao_ui_core::{
    HfPageHeader, HfPanel, HfPanelHeader, HfTable, HfTableBody, HfTableCell, HfTableHeader,
    HfTableHeaderCell, HfTableRow,
};
use leptos::prelude::*;

/// One mock print on the tape.
struct Print {
    time: &'static str,
    strike: i64,
    is_call: bool,
    is_buy: bool,
    qty: i64,
    premium_paise: i64,
    tag: &'static str,
}

const PRINTS: [Print; 8] = [
    Print {
        time: "09:17:02",
        strike: 24_850,
        is_call: true,
        is_buy: true,
        qty: 2_400,
        premium_paise: 14_200,
        tag: "Sweep",
    },
    Print {
        time: "09:24:38",
        strike: 25_000,
        is_call: true,
        is_buy: true,
        qty: 4_800,
        premium_paise: 9_650,
        tag: "Block",
    },
    Print {
        time: "09:31:15",
        strike: 24_700,
        is_call: false,
        is_buy: true,
        qty: 1_650,
        premium_paise: 6_130,
        tag: "",
    },
    Print {
        time: "10:02:44",
        strike: 24_800,
        is_call: false,
        is_buy: false,
        qty: 3_900,
        premium_paise: 18_400,
        tag: "Sweep",
    },
    Print {
        time: "10:15:09",
        strike: 25_100,
        is_call: true,
        is_buy: false,
        qty: 1_200,
        premium_paise: 2_875,
        tag: "",
    },
    Print {
        time: "11:40:26",
        strike: 24_900,
        is_call: true,
        is_buy: true,
        qty: 6_000,
        premium_paise: 11_900,
        tag: "Block",
    },
    Print {
        time: "12:08:51",
        strike: 24_600,
        is_call: false,
        is_buy: false,
        qty: 2_250,
        premium_paise: 4_510,
        tag: "",
    },
    Print {
        time: "13:33:19",
        strike: 24_750,
        is_call: false,
        is_buy: true,
        qty: 8_100,
        premium_paise: 27_300,
        tag: "Sweep",
    },
];

/// Class for the flow tag badge; blank tags get no badge.
fn tag_class(tag: &str) -> &'static str {
    match tag {
        "Block" => "fno__tag fno__tag--block",
        "Sweep" => "fno__tag fno__tag--sweep",
        _ => "",
    }
}

#[component]
pub fn FlowPage() -> impl IntoView {
    view! {
        <div class="console fno">
            <HfPageHeader title="Options Flow" subtitle="NIFTY · Derivatives · mock data" />

            <HfPanel class="fno__flow">
                <HfPanelHeader title="Unusual Prints" counter=PRINTS.len() />
                <div class="fno__toolbar">
                    <span class="placeholder__note">"Mock prints · qty in contracts · premium ₹ per unit"</span>
                </div>

                <div class="fno__scroll">
                    <HfTable class="fno__table">
                        <HfTableHeader>
                            <HfTableRow>
                                <HfTableHeaderCell>"Time"</HfTableHeaderCell>
                                <HfTableHeaderCell class="tnum">"Strike"</HfTableHeaderCell>
                                <HfTableHeaderCell>"Type"</HfTableHeaderCell>
                                <HfTableHeaderCell>"Side"</HfTableHeaderCell>
                                <HfTableHeaderCell class="tnum">"Qty"</HfTableHeaderCell>
                                <HfTableHeaderCell class="tnum">"Premium"</HfTableHeaderCell>
                                <HfTableHeaderCell>"Tag"</HfTableHeaderCell>
                            </HfTableRow>
                        </HfTableHeader>
                        <HfTableBody>
                            {PRINTS
                                .iter()
                                .map(|p| {
                                    let side_class = if p.is_buy {
                                        "fno__badge fno__badge--buy"
                                    } else {
                                        "fno__badge fno__badge--sell"
                                    };
                                    let side = if p.is_buy { "Buy" } else { "Sell" };
                                    let kind = if p.is_call { "Call" } else { "Put" };
                                    view! {
                                        <HfTableRow>
                                            <HfTableCell class="tnum">{p.time}</HfTableCell>
                                            <HfTableCell class="tnum fno__strike">{group_int(p.strike)}</HfTableCell>
                                            <HfTableCell>{kind}</HfTableCell>
                                            <HfTableCell><span class=side_class>{side}</span></HfTableCell>
                                            <HfTableCell class="tnum">{group_int(p.qty)}</HfTableCell>
                                            <HfTableCell class="tnum">{fmt_paise(p.premium_paise)}</HfTableCell>
                                            <HfTableCell>
                                                {(!p.tag.is_empty())
                                                    .then(|| view! { <span class=tag_class(p.tag)>{p.tag}</span> })}
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
