//! Screener column-header contextual menu (screener-design-doc.md §5.4):
//! sort options, move, hide, criteria-bound inline filters and the blue
//! "Filter" badge for actively filtered columns.

use honba_kao_ui_core::primitive::base::button::{
    ButtonSize, ButtonVariant, HfButton, HfButtonEvent, HfButtonProps,
};
use honba_kao_ui_core::primitive::base::input::{HfInput, HfInputEvent, HfInputProps};
use honba_kao_ui_core::theme::tokens::{HonbaStyle, colors, radius};
use nsh_core::prelude::*;
use num_traits::ToPrimitive as _;
use rust_decimal::Decimal;

use crate::features::screener::types::{FilterCriteria, ScreenerColumn, SortDirection, SortState};

/// True when the criteria-bound field(s) of `col_id` carry an active filter.
///
/// Binding map (only these column ids are criteria-bound):
/// `price`, `change_percent`, `pe_ratio`, `rsi_14`, `volume`, `dividend_yield`,
/// `pb_ratio`, `roe`, `sector` — everything else is never badge-active.
pub fn column_filters_active(col_id: &str, criteria: &FilterCriteria) -> bool {
    match col_id {
        "price" => criteria.min_price.is_some() || criteria.max_price.is_some(),
        "change_percent" => criteria.min_change_pct.is_some() || criteria.max_change_pct.is_some(),
        "volume" => criteria.min_volume.is_some(),
        "pe_ratio" => criteria.min_pe.is_some() || criteria.max_pe.is_some(),
        "rsi_14" => criteria.min_rsi.is_some() || criteria.max_rsi.is_some(),
        "dividend_yield" => criteria.min_dividend.is_some(),
        "pb_ratio" => criteria.max_pb.is_some(),
        "roe" => criteria.min_roe.is_some(),
        "sector" => !criteria.sector.is_empty() && criteria.sector != "All",
        _ => false,
    }
}

/// The flyout body for one column header.
///
/// Built once per mount inside the grid's `dynamic` region; the closures run
/// reactively against the supplied signals, so `.get()` reads here are
/// build-time snapshots (correct — the caller remounts on change).
pub fn column_menu(
    col: &'static ScreenerColumn,
    sector_options: Vec<String>,
    criteria: Signal<FilterCriteria>,
    visible_columns: Signal<Vec<String>>,
    sort_signal: Signal<Option<SortState>>,
) -> View {
    let mut card = view()
        .flex_col()
        .w_full()
        .gap(px(4.0))
        .px(px(8.0))
        .py(px(8.0))
        .rounded(px(radius::CONTROL))
        .bg(colors::SURFACE)
        .border_1()
        .border_color(colors::BORDER);

    // 9. Header line + (8) the blue "Filter" badge when this column is filtered.
    let header = view().flex_row().w_full().gap(px(6.0)).child(
        text(format!("{} — controls", col.label))
            .text_size(px(9.0))
            .text_muted(),
    );
    let header = if column_filters_active(col.id, &criteria.get()) {
        header.child(filter_badge())
    } else {
        header
    };
    card = card.child(header);

    // 1./2. Sort direction.
    card = card.child(menu_row("Sort Ascending", None, move || {
        sort_signal.set(Some(SortState {
            column_id: col.id.to_string(),
            direction: SortDirection::Asc,
        }));
    }));
    card = card.child(menu_row("Sort Descending", None, move || {
        sort_signal.set(Some(SortState {
            column_id: col.id.to_string(),
            direction: SortDirection::Desc,
        }));
    }));

    // 3./4. Move within the visible column order.
    card = card.child(menu_row("◀ Move Left", None, move || {
        visible_columns.update(|ids| {
            if let Some(pos) = ids.iter().position(|id| id == col.id)
                && pos > 0
            {
                ids.swap(pos, pos - 1);
            }
        });
    }));
    card = card.child(menu_row("▶ Move Right", None, move || {
        visible_columns.update(|ids| {
            if let Some(pos) = ids.iter().position(|id| id == col.id)
                && pos + 1 < ids.len()
            {
                ids.swap(pos, pos + 1);
            }
        });
    }));

    // 5. Hide.
    card = card.child(menu_row("✕ Hide Column", Some(colors::DOWN), move || {
        visible_columns.update(|ids| ids.retain(|id| id != col.id));
    }));

    // 6. Criteria-bound numeric inline filter (min/max drafts + Apply).
    if is_numeric_filter(col.id) {
        let current = criteria.get();
        let (min_current, max_current) = numeric_bounds(col.id, &current);
        let min_draft: Signal<String> =
            signal(min_current.map(|d| d.to_string()).unwrap_or_default());
        let max_draft: Signal<String> =
            signal(max_current.map(|d| d.to_string()).unwrap_or_default());

        let show_min = matches!(
            col.id,
            "price"
                | "change_percent"
                | "pe_ratio"
                | "rsi_14"
                | "volume"
                | "dividend_yield"
                | "roe"
        );
        let show_max = matches!(
            col.id,
            "price" | "change_percent" | "pe_ratio" | "rsi_14" | "pb_ratio"
        );

        let mut filter_row = view().flex_row().w_full().gap(px(4.0));
        if show_min {
            filter_row = filter_row
                .child(text("Min").text_size(px(9.0)).text_muted())
                .child(
                    HfInput::element(HfInputProps {
                        value: Prop::bind(move || min_draft.get()),
                        placeholder: Prop::from(Some("min".to_string())),
                        disabled: Prop::from(false),
                    })
                    .on_event({
                        move |event: &HfInputEvent| {
                            let HfInputEvent::Change(v) = event;
                            min_draft.set(v.clone());
                        }
                    }),
                );
        }
        if show_max {
            filter_row = filter_row
                .child(text("Max").text_size(px(9.0)).text_muted())
                .child(
                    HfInput::element(HfInputProps {
                        value: Prop::bind(move || max_draft.get()),
                        placeholder: Prop::from(Some("max".to_string())),
                        disabled: Prop::from(false),
                    })
                    .on_event({
                        move |event: &HfInputEvent| {
                            let HfInputEvent::Change(v) = event;
                            max_draft.set(v.clone());
                        }
                    }),
                );
        }
        filter_row = filter_row.child(
            HfButton::element(ghost_props())
                .child(text("Apply"))
                .on_event({
                    move |event: &HfButtonEvent| {
                        if *event == HfButtonEvent::Press {
                            apply_numeric(col.id, min_draft, max_draft, criteria);
                        }
                    }
                }),
        );
        card = card.child(filter_row);
    }

    // 7. Sector single-select.
    if col.id == "sector" {
        let mut options = sector_options;
        options.sort();
        let active_sector = criteria.get().sector;
        for value in options {
            let is_active = active_sector == value;
            let pressed = value.clone();
            card = card.child(sector_row(&value, is_active, move || {
                criteria.update(|c| {
                    if c.sector == pressed {
                        c.sector = String::new();
                    } else {
                        c.sector = pressed.clone();
                    }
                });
            }));
        }
        let all_active = active_sector.is_empty() || active_sector == "All";
        card = card.child(sector_row("All Sectors", all_active, move || {
            criteria.update(|c| c.sector = String::new());
        }));
    }

    card
}

/// True for the columns that expose the numeric inline filter row.
fn is_numeric_filter(col_id: &str) -> bool {
    matches!(
        col_id,
        "price"
            | "change_percent"
            | "pe_ratio"
            | "rsi_14"
            | "volume"
            | "dividend_yield"
            | "pb_ratio"
            | "roe"
    )
}

/// Current `(min, max)` criteria bounds for a column (binding map mirror).
fn numeric_bounds(col_id: &str, criteria: &FilterCriteria) -> (Option<Decimal>, Option<Decimal>) {
    match col_id {
        "price" => (criteria.min_price, criteria.max_price),
        "change_percent" => (criteria.min_change_pct, criteria.max_change_pct),
        "pe_ratio" => (criteria.min_pe, criteria.max_pe),
        "rsi_14" => (criteria.min_rsi, criteria.max_rsi),
        "volume" => (criteria.min_volume.map(Decimal::from), None),
        "dividend_yield" => (criteria.min_dividend, None),
        "pb_ratio" => (None, criteria.max_pb),
        "roe" => (criteria.min_roe, None),
        _ => (None, None),
    }
}

/// Write the draft strings back into `criteria` (exact decimal parsing only).
fn apply_numeric(
    col_id: &str,
    min_draft: Signal<String>,
    max_draft: Signal<String>,
    criteria: Signal<FilterCriteria>,
) {
    criteria.update(|c| match col_id {
        "price" => {
            c.min_price = parse_decimal(min_draft.get());
            c.max_price = parse_decimal(max_draft.get());
        }
        "change_percent" => {
            c.min_change_pct = parse_decimal(min_draft.get());
            c.max_change_pct = parse_decimal(max_draft.get());
        }
        "pe_ratio" => {
            c.min_pe = parse_decimal(min_draft.get());
            c.max_pe = parse_decimal(max_draft.get());
        }
        "rsi_14" => {
            c.min_rsi = parse_decimal(min_draft.get());
            c.max_rsi = parse_decimal(max_draft.get());
        }
        "volume" => {
            c.min_volume = parse_decimal(min_draft.get()).and_then(|d| d.round_dp(0).to_i64());
        }
        "dividend_yield" => c.min_dividend = parse_decimal(min_draft.get()),
        "pb_ratio" => c.max_pb = parse_decimal(max_draft.get()),
        "roe" => c.min_roe = parse_decimal(min_draft.get()),
        _ => {}
    });
}

/// Exact-decimal draft parsing — blank clears the bound, no floats anywhere.
fn parse_decimal(text: String) -> Option<Decimal> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        trimmed.parse::<Decimal>().ok()
    }
}

/// Standard ghost button props (verbatim shape from `filter_bar.rs`).
fn ghost_props() -> HfButtonProps {
    HfButtonProps {
        href: Prop::default(),
        size: Prop::from(Some(ButtonSize::Sm)),
        variant: Prop::from(Some(ButtonVariant::Ghost)),
        full_width: Prop::from(false),
        disabled: Prop::from(false),
        aria_label: Prop::from(None),
        class: Prop::default(),
        children: Children::default(),
    }
}

/// Blue "Filter" badge shown next to the header of an actively filtered column.
fn filter_badge() -> View {
    view()
        .px(px(4.0))
        .py(px(1.0))
        .rounded(px(radius::CONTROL))
        .bg(colors::ACCENT)
        .child(
            text("Filter")
                .text_size(px(8.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors::ON_ACCENT),
        )
}

/// One pressable menu row; `ink` overrides the label colour when set.
fn menu_row<F>(label: &str, ink: Option<AdaptiveColor>, on_press: F) -> View
where
    F: Fn() + 'static,
{
    let base = view()
        .w_full()
        .px(px(10.0))
        .py(px(5.0))
        .rounded(px(radius::CONTROL))
        .bg(colors::INSET);
    let row = match ink {
        Some(color) => base.child(
            text(label.to_string())
                .text_size(px(10.0))
                .text_color(color),
        ),
        None => base.child(text(label.to_string()).text_size(px(10.0))),
    };
    row.on_press(on_press)
}

/// One pressable sector row; the active option gets the ✓ accent treatment.
fn sector_row<F>(label: &str, active: bool, on_press: F) -> View
where
    F: Fn() + 'static,
{
    let base = view()
        .w_full()
        .px(px(10.0))
        .py(px(5.0))
        .rounded(px(radius::CONTROL));
    if active {
        base.bg(colors::ACCENT)
            .child(
                text(format!("✓ {label}"))
                    .text_size(px(10.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors::ON_ACCENT),
            )
            .on_press(on_press)
    } else {
        base.bg(colors::INSET)
            .child(text(label.to_string()).text_size(px(10.0)))
            .on_press(on_press)
    }
}
