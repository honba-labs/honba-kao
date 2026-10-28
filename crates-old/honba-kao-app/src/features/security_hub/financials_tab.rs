//! Financials tab (Phase 2.2) — three transposed statement tables.
//!
//! Periods run across the columns and line items down the rows. The
//! Annual / Quarterly toggle swaps which period set is shown and the growth
//! column switches between YoY (annual) and QoQ (quarterly).
//!
//! All figures are hardcoded mock data in ₹ Cr. Values are whole crores, so
//! the mock itself carries no floating-point amounts; growth percentages are
//! derived for display only.

use honba_kao_ui_core::{
    HfPLNumber, HfPLNumberFormat, HfPanel, HfPanelHeader, HfTable, HfTableBody, HfTableCell,
    HfTableHeader, HfTableHeaderCell, HfTableRow, HfToggleGroup, HfToggleGroupItem,
};
use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Period {
    Annual,
    Quarterly,
}

impl Period {
    fn labels(self) -> [&'static str; 4] {
        match self {
            Period::Annual => ["FY23", "FY24", "FY25", "FY26"],
            Period::Quarterly => ["Q2 FY26", "Q3 FY26", "Q4 FY26", "Q1 FY27"],
        }
    }

    fn growth_label(self) -> &'static str {
        match self {
            Period::Annual => "YoY %",
            Period::Quarterly => "QoQ %",
        }
    }
}

/// One line item. `annual` and `quarterly` each hold four periods, oldest first.
struct Row {
    label: &'static str,
    annual: [i64; 4],
    quarterly: [i64; 4],
}

struct Statement {
    title: &'static str,
    rows: &'static [Row],
}

const INCOME_STATEMENT: Statement = Statement {
    title: "Income Statement",
    rows: &[
        Row {
            label: "Revenue",
            annual: [41200, 46800, 52300, 58900],
            quarterly: [14100, 14800, 15300, 15900],
        },
        Row {
            label: "Gross profit",
            annual: [17800, 20600, 23400, 26900],
            quarterly: [6000, 6350, 6700, 7100],
        },
        Row {
            label: "Operating expenses",
            annual: [9900, 11100, 12700, 14300],
            quarterly: [3200, 3300, 3500, 3650],
        },
        Row {
            label: "Operating income",
            annual: [7900, 9500, 10700, 12600],
            quarterly: [2800, 3050, 3200, 3450],
        },
        Row {
            label: "Net income",
            annual: [5300, 6400, 7200, 8650],
            quarterly: [1850, 2050, 2180, 2400],
        },
    ],
};

const BALANCE_SHEET: Statement = Statement {
    title: "Balance Sheet",
    rows: &[
        Row {
            label: "Cash & equivalents",
            annual: [6200, 7400, 8100, 9300],
            quarterly: [7900, 8150, 9300, 9650],
        },
        Row {
            label: "Receivables",
            annual: [5400, 6100, 6900, 7800],
            quarterly: [6500, 6650, 7800, 8000],
        },
        Row {
            label: "Total assets",
            annual: [78000, 88500, 99200, 111400],
            quarterly: [103100, 106300, 111400, 114900],
        },
        Row {
            label: "Total liabilities",
            annual: [42000, 47600, 52800, 58100],
            quarterly: [55200, 56400, 58100, 59300],
        },
        Row {
            label: "Shareholders' equity",
            annual: [36000, 40900, 46400, 53300],
            quarterly: [47900, 49900, 53300, 55600],
        },
    ],
};

const CASH_FLOW: Statement = Statement {
    title: "Cash Flow",
    rows: &[
        Row {
            label: "Cash from operations",
            annual: [7400, 8600, 9800, 11200],
            quarterly: [2500, 2700, 2950, 3300],
        },
        Row {
            label: "Capital expenditure",
            annual: [-2900, -3300, -3900, -4500],
            quarterly: [-950, -1000, -1150, -1200],
        },
        Row {
            label: "Free cash flow",
            annual: [4500, 5300, 5900, 6700],
            quarterly: [1550, 1700, 1800, 2100],
        },
        Row {
            label: "Dividends paid",
            annual: [-1200, -1400, -1600, -1800],
            quarterly: [-400, -420, -460, -500],
        },
        Row {
            label: "Net change in cash",
            annual: [900, 1200, 700, 1200],
            quarterly: [400, 250, 1150, 350],
        },
    ],
};

const STATEMENTS: [&Statement; 3] = [&INCOME_STATEMENT, &BALANCE_SHEET, &CASH_FLOW];

/// Growth between the last two periods, as a percentage of the prior period's
/// absolute value. `None` when the prior period is zero.
fn growth_pct(values: [i64; 4]) -> Option<f64> {
    let (prev, last) = (values[2], values[3]);
    if prev == 0 {
        None
    } else {
        Some((last - prev) as f64 / prev.unsigned_abs() as f64 * 100.0)
    }
}

/// Format whole crores with thousands separators; negatives in accounting
/// parentheses.
fn format_cr(value: i64) -> String {
    let digits = value.unsigned_abs().to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(c);
    }
    if value < 0 {
        format!("({grouped})")
    } else {
        grouped
    }
}

#[component]
pub fn FinancialsTab() -> impl IntoView {
    let period = RwSignal::new(Period::Annual);

    view! {
        <div class="financials-tab">
            <div class="financials-tab__toolbar">
                <HfToggleGroup default_value="annual">
                    <HfToggleGroupItem
                        value="annual"
                        on_select=Callback::new(move |_| period.set(Period::Annual))
                    >
                        "Annual"
                    </HfToggleGroupItem>
                    <HfToggleGroupItem
                        value="quarterly"
                        on_select=Callback::new(move |_| period.set(Period::Quarterly))
                    >
                        "Quarterly"
                    </HfToggleGroupItem>
                </HfToggleGroup>
                <span class="placeholder__note">"₹ Cr · mock data"</span>
            </div>

            {STATEMENTS
                .iter()
                .map(|statement| {
                    view! { <StatementTable statement=*statement period=period /> }
                })
                .collect_view()}
        </div>
    }
}

#[component]
fn StatementTable(statement: &'static Statement, period: RwSignal<Period>) -> impl IntoView {
    view! {
        <HfPanel class="financials-tab__statement">
            <HfPanelHeader title=statement.title />
            {move || {
                let current = period.get();
                let labels = current.labels();
                view! {
                    <HfTable class="financials-tab__table">
                        <HfTableHeader>
                            <HfTableRow>
                                <HfTableHeaderCell>"Line item"</HfTableHeaderCell>
                                {labels
                                    .into_iter()
                                    .map(|label| {
                                        view! { <HfTableHeaderCell class="tnum">{label}</HfTableHeaderCell> }
                                    })
                                    .collect_view()}
                                <HfTableHeaderCell class="tnum">{current.growth_label()}</HfTableHeaderCell>
                            </HfTableRow>
                        </HfTableHeader>
                        <HfTableBody>
                            {statement
                                .rows
                                .iter()
                                .map(|row| {
                                    let values = match current {
                                        Period::Annual => row.annual,
                                        Period::Quarterly => row.quarterly,
                                    };
                                    let growth = growth_pct(values);
                                    view! {
                                        <HfTableRow>
                                            <HfTableCell>{row.label}</HfTableCell>
                                            {values
                                                .into_iter()
                                                .map(|v| {
                                                    view! { <HfTableCell class="tnum">{format_cr(v)}</HfTableCell> }
                                                })
                                                .collect_view()}
                                            <HfTableCell class="tnum">
                                                {match growth {
                                                    Some(g) => view! {
                                                        <HfPLNumber value=g format=HfPLNumberFormat::Percent />
                                                    }
                                                    .into_any(),
                                                    None => view! { <span>"—"</span> }.into_any(),
                                                }}
                                            </HfTableCell>
                                        </HfTableRow>
                                    }
                                })
                                .collect_view()}
                        </HfTableBody>
                    </HfTable>
                }
            }}
        </HfPanel>
    }
}
