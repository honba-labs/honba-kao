//! The trade journal table — the Leptos port of honba-fe
//! `features/simulator/components/trade-journal-table.tsx` (port phase P5).
//!
//! One row per derived round trip, filterable and sortable, with a CSV
//! export built from the very rows on screen so the file and the table
//! agree by construction (`sim::analytics::journal_rows` + `to_csv`, design
//! wart W5: one journal source). **Slippage has no column value**: the
//! journal is fills with no signal price, so it is left blank rather than
//! guessed (risk R5) — the column exists to document the absence.

use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::sim::analytics::{
    format_timestamp, to_csv, CsvColumn, JournalRow,
};
use crate::sim::pairing::{Direction, RoundTripStatus};

/// fe's `SortKey` — the eight sortable columns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortKey {
    EntryTs,
    ExitTs,
    Quantity,
    AvgEntry,
    AvgExit,
    RealizedPnl,
    ReturnPct,
    Fees,
}

/// A column of the table: its header, text alignment and (for the eight
/// covered keys) its sortable field.
struct Column {
    /// Sort key; `None` for the columns that are never reordered.
    key: Option<SortKey>,
    /// Header text.
    header: &'static str,
    /// Right-aligned numeric column.
    numeric: bool,
}

const COLUMNS: [Column; 12] = [
    Column { key: None, header: "Symbol", numeric: false },
    Column { key: None, header: "Side", numeric: false },
    Column { key: None, header: "Status", numeric: false },
    Column { key: Some(SortKey::EntryTs), header: "Entry (UTC)", numeric: false },
    Column { key: Some(SortKey::ExitTs), header: "Exit (UTC)", numeric: false },
    Column { key: Some(SortKey::Quantity), header: "Qty", numeric: true },
    Column { key: Some(SortKey::AvgEntry), header: "Avg Entry", numeric: true },
    Column { key: Some(SortKey::AvgExit), header: "Avg Exit", numeric: true },
    Column { key: Some(SortKey::RealizedPnl), header: "Net PnL", numeric: true },
    Column { key: Some(SortKey::ReturnPct), header: "Return %", numeric: true },
    Column { key: Some(SortKey::Fees), header: "Fees", numeric: true },
    Column { key: None, header: "Slippage", numeric: true },
];

/// fe's `num` helper — fixed decimals, `—` for absent/non-finite.
fn num(value: Option<f64>, digits: usize) -> String {
    match value {
        Some(value) if value.is_finite() => format!("{value:.digits$}"),
        _ => "—".to_string(),
    }
}

/// fe's `csvColumns` — the same fields the table shows, plus the raw ids.
/// Headers and values mirror fe's spellings exactly; [`to_csv`] applies
/// RFC-4180 quoting (`csv_field`), so the closures return plain strings.
fn csv_columns() -> Vec<CsvColumn<JournalRow>> {
    vec![
        CsvColumn { header: "id", value: |row| row.id.clone() },
        CsvColumn { header: "symbol", value: |row| row.instrument.clone() },
        CsvColumn { header: "side", value: |row| row.direction.as_str().to_string() },
        CsvColumn { header: "status", value: |row| row.status.as_str().to_string() },
        CsvColumn { header: "entry_ts", value: |row| format_timestamp(Some(row.entry_ts)) },
        CsvColumn { header: "exit_ts", value: |row| format_timestamp(row.exit_ts) },
        CsvColumn { header: "qty", value: |row| row.quantity.to_string() },
        CsvColumn { header: "open_qty", value: |row| row.open_quantity.to_string() },
        CsvColumn { header: "avg_entry", value: |row| row.avg_entry.to_string() },
        CsvColumn {
            header: "avg_exit",
            value: |row| row.avg_exit.map_or_else(String::new, |value| value.to_string()),
        },
        CsvColumn {
            header: "net_pnl",
            value: |row| row.realized_pnl.map_or_else(String::new, |value| value.to_string()),
        },
        CsvColumn {
            header: "return_pct",
            value: |row| {
                row.return_pct.map_or_else(String::new, |value| format!("{:.4}", value * 100.0))
            },
        },
        CsvColumn { header: "fees", value: |row| row.fees.to_string() },
        // The journal carries no signal price, so slippage is deliberately
        // blank (R5) — the column documents the absence instead of a guess.
        CsvColumn { header: "slippage", value: |_row| String::new() },
    ]
}

/// Sort accessor over one row field. `None` sorts as negative infinity —
/// fe's `?? Number.NEGATIVE_INFINITY`, so absent values sink on ascending
/// runs and lift on descending ones.
fn field_of(row: &JournalRow, key: SortKey) -> Option<f64> {
    match key {
        SortKey::EntryTs => Some(row.entry_ts as f64),
        SortKey::ExitTs => row.exit_ts.map(|value| value as f64),
        SortKey::Quantity => Some(row.quantity),
        SortKey::AvgEntry => Some(row.avg_entry),
        SortKey::AvgExit => row.avg_exit,
        SortKey::RealizedPnl => row.realized_pnl,
        SortKey::ReturnPct => row.return_pct,
        SortKey::Fees => Some(row.fees),
    }
}

/// `data:`-URL percent-encoding: RFC-3986 unreserved bytes pass through,
/// everything else escapes as `%XX`. Journal cells may hold quotes,
/// semicolons and spaces, so unlike the screener's looser encoder this one
/// escapes every unsafe byte and the CSV stays byte-exact.
fn percent_encode_strict(raw: &str) -> String {
    let mut encoded = String::with_capacity(raw.len());
    for byte in raw.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(*byte as char)
            }
            other => encoded.push_str(&format!("%{other:02X}")),
        }
    }
    encoded
}

/// Downloads `csv` as a file via fe's `Blob` + `URL.createObjectURL` idiom so
/// the exact bytes land on disk (RFC-4180 quoting, CRLF, UTF-8) with no
/// data-URL size ceiling for a long journal.
fn download_csv(file_name: &str, csv: &str) -> Option<()> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let body = document.body()?;

    let text = js_sys::Uint8Array::from(csv.as_bytes());
    let parts = js_sys::Array::new();
    parts.push(&text);
    let bag = web_sys::BlobPropertyBag::new();
    bag.set_type("text/csv;charset=utf-8");
    let blob =
        web_sys::Blob::new_with_u8_slice_sequence_and_options(&parts.into(), &bag).ok()?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).ok()?;

    let anchor = document
        .create_element("a")
        .ok()?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .ok()?;
    anchor.set_href(&url);
    anchor.set_download(file_name);
    let _ = body.append_child(&anchor);
    anchor.click();
    let _ = body.remove_child(&anchor);
    let _ = web_sys::Url::revoke_object_url(&url);
    Some(())
}

/// The trade journal table (`TradeJournalTable`).
#[component]
pub fn JournalTable(
    /// The journal rows, derived per `store.rows` (W5 single source).
    #[prop(into)]
    rows: Signal<Vec<JournalRow>>,
    /// Base file name for the CSV export (strategy/run).
    #[prop(into)]
    export_name: Signal<String>,
) -> impl IntoView {
    // fe kept filter/sort local to the component instance; local signals
    // keep that parity (a pager-owned store would share resets across tabs).
    let filter = RwSignal::new(String::new());
    let sort_key = RwSignal::new(SortKey::EntryTs);
    let sort_dir = RwSignal::new(true); // true = asc

    let toggle_sort = move |key: SortKey| {
        if sort_key.get_untracked() == key {
            sort_dir.update(|asc| *asc = !*asc);
        } else {
            sort_key.set(key);
            sort_dir.set(true);
        }
    };

    let visible = Memo::new(move |_| {
        let needle = filter.get().trim().to_lowercase();
        let rows = rows.get();
        let mut rows: Vec<JournalRow> = if needle.is_empty() {
            rows
        } else {
            rows.into_iter()
                .filter(|row| {
                    [row.instrument.as_str(), row.direction.as_str(), row.status.as_str(), row.exit_reason.as_str()]
                        .iter()
                        .any(|field| field.to_lowercase().contains(&needle))
                })
                .collect()
        };
        let key = sort_key.get();
        let ascending = sort_dir.get();
        // fe's comparator: the value comparison flips on descending runs;
        // exact ties fall back to entry time in the same direction.
        rows.sort_by(|a, b| {
            let av = field_of(a, key).unwrap_or(f64::NEG_INFINITY);
            let bv = field_of(b, key).unwrap_or(f64::NEG_INFINITY);
            if av == bv {
                a.entry_ts.cmp(&b.entry_ts)
            } else {
                let cmp = av.partial_cmp(&bv).unwrap_or(std::cmp::Ordering::Equal);
                if ascending {
                    cmp
                } else {
                    cmp.reverse()
                }
            }
        });
        rows
    });

    let total = Memo::new(move |_| {
        visible
            .get()
            .iter()
            .map(|row| row.realized_pnl.unwrap_or(0.0))
            .sum::<f64>()
    });

    view! {
        <div class="sim-journal" data-testid="trade-journal">
            <div class="sim-journal__bar">
                <input
                    type="text"
                    class="sim-journal__filter"
                    placeholder="Filter by symbol, side, status or exit reason"
                    aria-label="Filter the trade journal"
                    prop:value=filter
                    on:input=move |event| filter.set(event_target_value(&event))
                />
                <span class="sim-journal__count">
                    {move || {
                        let total = total.get();
                        let sign = if total >= 0.0 { "+" } else { "" };
                        format!(
                            "{} of {} · net {sign}{:.2}",
                            visible.get().len(),
                            rows.get().len(),
                            total
                        )
                    }}
                </span>
                <button
                    type="button"
                    class="sim-journal__export"
                    disabled=move || visible.get().is_empty()
                    title="Export the visible rows as CSV (opens in Excel)"
                    on:click=move |_| {
                        let csv = to_csv(visible.get_untracked().as_slice(), &csv_columns());
                        // fe: `exportName.replace(/[^\w.-]+/g, '_')`.
                        let base: String = export_name
                            .get_untracked()
                            .chars()
                            .map(|ch| if ch.is_alphanumeric() || matches!(ch, '.' | '-' | '_') { ch } else { '_' })
                            .collect();
                        let file_name = format!("{base}-journal.csv");
                        if download_csv(&file_name, &csv).is_none() {
                            // No browser surface (test embed, blocked popup):
                            // fall back to the data:URL anchor only when the
                            // blob path is unavailable.
                            let window = web_sys::window().expect("window for CSV export");
                            let document = window.document().expect("document for CSV export");
                            let url = format!(
                                "data:text/csv;charset=utf-8,{}",
                                percent_encode_strict(&csv)
                            );
                            let link = document
                                .create_element("a")
                                .expect("anchor for CSV export")
                                .dyn_into::<web_sys::HtmlAnchorElement>()
                                .expect("anchor element");
                            link.set_href(&url);
                            link.set_download(&file_name);
                            link.click();
                        }
                    }
                >
                    CSV
                </button>
            </div>
            <table class="sim-journal__table">
                <thead>
                    <tr>
                        {COLUMNS
                            .iter()
                            .map(|column| {
                                let header = column.header;
                                let numeric = column.numeric;
                                let key = column.key;
                                view! {
                                    <th
                                        class=("sim-journal__num", numeric)
                                        scope="col"
                                    >
                                        {move || {
                                            if let Some(key) = key {
                                                let active = sort_key.get() == key;
                                                let arrow = if sort_dir.get() { "↑" } else { "↓" };
                                                let class = if active {
                                                    "sim-journal__head-btn sim-journal__head-btn--active"
                                                } else {
                                                    "sim-journal__head-btn"
                                                };
                                                view! {
                                                    <button
                                                        type="button"
                                                        class=class
                                                        title="Sort"
                                                        on:click=move |_| toggle_sort(key)
                                                    >
                                                        {format!("{header} {arrow}")}
                                                    </button>
                                                }
                                                    .into_any()
                                            } else {
                                                view! { <span>{header}</span> }.into_any()
                                            }
                                        }}
                                    </th>
                                }
                            })
                            .collect_view()}
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let rows = visible.get();
                        if rows.is_empty() {
                            view! {
                                <tr>
                                    <td class="sim-journal__empty" colspan=COLUMNS.len()>
                                        "No round trips to show."
                                    </td>
                                </tr>
                            }
                                .into_any()
                        } else {
                            rows
                                .into_iter()
                                .map(|row| {
                                    let side_class = if row.direction == Direction::Long {
                                        "sim-journal__side--long"
                                    } else {
                                        "sim-journal__side--short"
                                    };
                                    let (status_text, status_class) =
                                        if row.status == RoundTripStatus::Open {
                                            (
                                                format!("open ({})", row.exit_reason.as_str()),
                                                "sim-journal__status--open",
                                            )
                                        } else {
                                            (row.exit_reason.as_str().to_string(), "")
                                        };
                                    let pnl_class = match row.realized_pnl {
                                        None => "sim-journal__pnl--muted",
                                        Some(value) if value >= 0.0 => "sim-journal__pnl--gain",
                                        Some(_) => "sim-journal__pnl--loss",
                                    };
                                    let pnl_text = row.realized_pnl.map_or_else(
                                        || "—".to_string(),
                                        |value| format!("{}{:.2}", if value >= 0.0 { "+" } else { "" }, value),
                                    );
                                    let pct_text = row.return_pct.map_or_else(
                                        || "—".to_string(),
                                        |value| format!("{}{:.2}%", if value >= 0.0 { "+" } else { "" }, value * 100.0),
                                    );
                                    view! {
                                        <tr>
                                            <td class="sim-journal__mono">{row.instrument}</td>
                                            <td class=side_class>{row.direction.as_str()}</td>
                                            <td class=("sim-journal__status--open", !status_class.is_empty())>{status_text}</td>
                                            <td class="sim-journal__mono">{format_timestamp(Some(row.entry_ts))}</td>
                                            <td class="sim-journal__mono">{format_timestamp(row.exit_ts)}</td>
                                            <td class="sim-journal__num">{num(Some(row.quantity), 0)}</td>
                                            <td class="sim-journal__num">{num(Some(row.avg_entry), 2)}</td>
                                            <td class="sim-journal__num">{num(row.avg_exit, 2)}</td>
                                            <td class=format!("sim-journal__num {pnl_class}")>{pnl_text}</td>
                                            <td class="sim-journal__num">{pct_text}</td>
                                            <td class="sim-journal__num">{num(Some(row.fees), 2)}</td>
                                            <td
                                                class="sim-journal__num sim-journal__na"
                                                title="The journal is fills with no signal price, so slippage cannot be derived (R5)."
                                            >
                                                "—"
                                            </td>
                                        </tr>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }
                    }}
                </tbody>
            </table>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::pairing::RoundTripExitReason;

    fn sample_row() -> JournalRow {
        JournalRow {
            id: "t1".to_string(),
            instrument: "TCS.NSE".to_string(),
            direction: Direction::Long,
            status: RoundTripStatus::Closed,
            entry_ts: 1_700_002_205,
            exit_ts: Some(1_700_067_051),
            quantity: 10.0,
            open_quantity: 0.0,
            avg_entry: 22_010.0,
            avg_exit: Some(22_060.0),
            realized_pnl: Some(500.0),
            return_pct: Some(0.002_27),
            fees: 1.5,
            exit_reason: RoundTripExitReason::PositionClosed,
        }
    }

    #[test]
    fn num_formats_absent_and_infinite_values_as_a_dash() {
        assert_eq!(num(Some(10.5), 2), "10.50");
        assert_eq!(num(None, 2), "—");
        assert_eq!(num(Some(f64::NAN), 0), "—");
        assert_eq!(num(Some(7.0), 0), "7");
    }

    #[test]
    fn percent_encode_strict_escapes_every_unsafe_byte() {
        assert_eq!(percent_encode_strict("plain-._~"), "plain-._~");
        // A comma, CRLF, quote and space all survive the data URL as %XX.
        assert_eq!(
            percent_encode_strict("a,b\rc\nd \"e\" f"),
            "a%2Cb%0Dc%0Ad%20%22e%22%20f"
        );
    }

    #[test]
    fn csv_columns_cover_the_table_and_leave_slippage_blank() {
        let columns = csv_columns();
        assert_eq!(
            columns.iter().map(|column| column.header).collect::<Vec<_>>(),
            [
                "id", "symbol", "side", "status", "entry_ts", "exit_ts", "qty", "open_qty",
                "avg_entry", "avg_exit", "net_pnl", "return_pct", "fees", "slippage",
            ]
        );
        let row = sample_row();
        let render: Vec<String> = columns.iter().map(|column| (column.value)(&row)).collect();
        // The slippage cell is deliberately blank — the journal is fills with
        // no signal price (R5); every other column carries its value.
        assert_eq!(render[13], "");
        assert!(!render[4].is_empty());
    }

    #[test]
    fn field_of_reads_the_eight_sortable_fields() {
        let row = sample_row();
        assert_eq!(field_of(&row, SortKey::Quantity), Some(10.0));
        assert_eq!(field_of(&row, SortKey::AvgExit), Some(22_060.0));
        assert_eq!(field_of(&row, SortKey::RealizedPnl), Some(500.0));
        assert_eq!(field_of(&row, SortKey::Fees), Some(1.5));
        // An open trip's absent exit and realised PnL read as `None` — fe's
        // `?? Number.NEGATIVE_INFINITY` fall-through.
        let mut open = row.clone();
        open.exit_ts = None;
        open.realized_pnl = None;
        assert_eq!(field_of(&open, SortKey::ExitTs), None);
        assert_eq!(field_of(&open, SortKey::RealizedPnl), None);
    }
}
