//! Peers & Value Chain tab (Phase 2.3) — relative valuation, supply chain,
//! raw material sensitivity and reverse DCF.
//!
//! All figures are hardcoded mock data. Valuation multiples and percentages
//! are carried as display strings, so the mock holds no floating-point values.

use honba_kao_ui_core::{
    HfKpi, HfPanel, HfPanelHeader, HfTable, HfTableBody, HfTableCell, HfTableHeader,
    HfTableHeaderCell, HfTableRow,
};
use leptos::prelude::*;

struct Peer {
    name: &'static str,
    pe: &'static str,
    pb: &'static str,
    ev_ebitda: &'static str,
    roe: &'static str,
}

const PEERS: &[Peer] = &[
    Peer {
        name: "Peer Alpha",
        pe: "24.1x",
        pb: "3.8x",
        ev_ebitda: "14.2x",
        roe: "16.5%",
    },
    Peer {
        name: "Peer Beta",
        pe: "28.7x",
        pb: "4.6x",
        ev_ebitda: "17.9x",
        roe: "18.2%",
    },
    Peer {
        name: "Peer Gamma",
        pe: "21.3x",
        pb: "2.9x",
        ev_ebitda: "11.8x",
        roe: "13.4%",
    },
    Peer {
        name: "Peer Delta",
        pe: "31.0x",
        pb: "5.2x",
        ev_ebitda: "19.6x",
        roe: "19.0%",
    },
];

const SUPPLIERS: &[&str] = &[
    "Crude oil & petrochemical feedstock",
    "Industrial gas vendors",
    "Logistics & freight partners",
    "Packaging film makers",
];
const CUSTOMERS: &[&str] = &[
    "Retail distribution network",
    "Automotive OEMs",
    "Institutional buyers",
    "Export markets (EU, US)",
];

#[component]
pub fn ValueChainTab() -> impl IntoView {
    view! {
        <div class="value-chain-tab">
            <HfPanel>
                <HfPanelHeader title="Relative valuation" />
                <HfTable class="value-chain-tab__peers">
                    <HfTableHeader>
                        <HfTableRow>
                            <HfTableHeaderCell>"Company"</HfTableHeaderCell>
                            <HfTableHeaderCell class="tnum">"P/E"</HfTableHeaderCell>
                            <HfTableHeaderCell class="tnum">"P/B"</HfTableHeaderCell>
                            <HfTableHeaderCell class="tnum">"EV/EBITDA"</HfTableHeaderCell>
                            <HfTableHeaderCell class="tnum">"ROE"</HfTableHeaderCell>
                        </HfTableRow>
                    </HfTableHeader>
                    <HfTableBody>
                        {PEERS
                            .iter()
                            .map(|peer| {
                                view! {
                                    <HfTableRow>
                                        <HfTableCell>{peer.name}</HfTableCell>
                                        <HfTableCell class="tnum">{peer.pe}</HfTableCell>
                                        <HfTableCell class="tnum">{peer.pb}</HfTableCell>
                                        <HfTableCell class="tnum">{peer.ev_ebitda}</HfTableCell>
                                        <HfTableCell class="tnum">{peer.roe}</HfTableCell>
                                    </HfTableRow>
                                }
                            })
                            .collect_view()}
                    </HfTableBody>
                </HfTable>
                <p class="placeholder__note">"Mock data · trailing multiples"</p>
            </HfPanel>

            <div class="value-chain-tab__columns">
                <HfPanel>
                    <HfPanelHeader title="Suppliers" />
                    <ul class="value-chain-tab__list">
                        {SUPPLIERS.iter().map(|name| view! { <li>{*name}</li> }).collect_view()}
                    </ul>
                </HfPanel>
                <HfPanel>
                    <HfPanelHeader title="This company" />
                    <ul class="value-chain-tab__list">
                        <li>"Integrated manufacturing"</li>
                        <li>"Own distribution & retail"</li>
                        <li>"Digital channel & services"</li>
                    </ul>
                </HfPanel>
                <HfPanel>
                    <HfPanelHeader title="Customers" />
                    <ul class="value-chain-tab__list">
                        {CUSTOMERS.iter().map(|name| view! { <li>{*name}</li> }).collect_view()}
                    </ul>
                </HfPanel>
            </div>

            <HfPanel>
                <HfPanelHeader title="Raw material sensitivity" />
                <p class="value-chain-tab__line">
                    "Crude Oil +$10/bbl -> Operating margin -1.2%"
                </p>
                <p class="placeholder__note">"Mock sensitivity · illustrative"</p>
            </HfPanel>

            <HfPanel>
                <HfPanelHeader title="Reverse DCF" />
                <HfKpi label="Implied growth" delta="vs consensus 9.5%">
                    "11.2%"
                </HfKpi>
            </HfPanel>
        </div>
    }
}
