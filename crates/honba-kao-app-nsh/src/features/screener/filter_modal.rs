//! Screener filter modal (screener-design-doc.md §4.2 granular filters).

use honba_kao_ui_core::primitive::base::button::{
    ButtonSize, ButtonVariant, HfButton, HfButtonEvent, HfButtonProps,
};
use honba_kao_ui_core::primitive::base::input::{HfInput, HfInputEvent, HfInputProps};
use honba_kao_ui_core::theme::tokens::{colors, radius, rgb_a};
use nsh_core::prelude::*;
use rust_decimal::Decimal;

use crate::features::screener::types::{FilterCriteria, MarketCapTier};

/// Seeds a draft tail from a stored `Option<Decimal>` bound
/// (`None` → empty tail, i.e. "no bound").
fn decimal_tail(value: Option<Decimal>) -> String {
    value.map(|d| d.to_string()).unwrap_or_default()
}

/// Parses a typed tail back into a bound; blank or junk tails are
/// treated as absent (`None`).
fn parse_decimal_tail(tail: &str) -> Option<Decimal> {
    tail.trim().parse::<Decimal>().ok()
}

/// The `min_volume` tail parses as an integer share count.
fn parse_volume_tail(tail: &str) -> Option<i64> {
    tail.trim().parse::<i64>().ok()
}

/// One label + one text entry for a numeric filter tail.
fn tail_field(label: &'static str, draft: Signal<String>, placeholder: &'static str) -> AnyElement {
    view()
        .flex_col()
        .gap(px(4.0))
        .w_full()
        .child(text(label).text_size(px(11.0)).text_color(colors::MUTED))
        .child(
            view().w_full().child(
                HfInput::element(HfInputProps {
                    value: Prop::bind(move || draft.get()),
                    placeholder: Prop::from(Some(placeholder.to_string())),
                    disabled: Prop::from(false),
                })
                .on_event(move |event: &HfInputEvent| {
                    let HfInputEvent::Change(v) = event;
                    draft.set(v.clone());
                }),
            ),
        )
        .into_element()
}

/// A muted section label above each chip row / tail block.
fn section_label(label: &'static str) -> AnyElement {
    text(label)
        .text_size(px(11.0))
        .text_color(colors::MUTED)
        .font_weight(FontWeight::MEDIUM)
        .into_element()
}

/// Shared chrome for the footer's ghost buttons.
fn ghost_button_props() -> HfButtonProps {
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

#[allow(non_snake_case)]
pub fn ScreenerFilterModal(
    criteria: Signal<FilterCriteria>,
    sectors: Signal<Vec<String>>,
    open: Signal<bool>,
) -> View {
    // Draft-per-open: the page re-constructs this modal on every open,
    // so the seeds below are snapshotted once per presentation.
    let seed = criteria.get_untracked();

    let min_price = signal(decimal_tail(seed.min_price));
    let max_price = signal(decimal_tail(seed.max_price));
    let min_change_pct = signal(decimal_tail(seed.min_change_pct));
    let max_change_pct = signal(decimal_tail(seed.max_change_pct));
    let min_volume = signal(seed.min_volume.map(|v| v.to_string()).unwrap_or_default());
    let min_pe = signal(decimal_tail(seed.min_pe));
    let max_pe = signal(decimal_tail(seed.max_pe));
    let max_pb = signal(decimal_tail(seed.max_pb));
    let min_dividend = signal(decimal_tail(seed.min_dividend));
    let min_rsi = signal(decimal_tail(seed.min_rsi));
    let max_rsi = signal(decimal_tail(seed.max_rsi));
    let min_roe = signal(decimal_tail(seed.min_roe));
    let min_fcf_yield = signal(decimal_tail(seed.min_fcf_yield));
    let sector_draft = signal(if seed.sector.is_empty() || seed.sector == "All" {
        String::new()
    } else {
        seed.sector.clone()
    });
    let tier_draft = signal(seed.market_cap_tier);

    // Sector chips: the page-built roster ("All" first, then sorted
    // unique labels); an empty draft means no sector filter.
    let sector_chip_row =
        scroll()
            .flex_row()
            .gap(px(6.0))
            .w_full()
            .children(sectors.get().into_iter().map(|sector| {
                let target = if sector.is_empty() || sector == "All" {
                    String::new()
                } else {
                    sector.clone()
                };
                let label = if target.is_empty() {
                    "All".to_string()
                } else {
                    target.clone()
                };
                let bg_target = target.clone();
                let text_target = target.clone();
                let press_target = target.clone();
                view()
                    .px(px(10.0))
                    .py(px(4.0))
                    .rounded(px(radius::CONTROL))
                    .bg(move || {
                        if sector_draft.get() == bg_target {
                            colors::ACCENT
                        } else {
                            colors::INSET
                        }
                    })
                    .child(
                        text(label)
                            .text_size(px(10.5))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(move || {
                                if sector_draft.get() == text_target {
                                    colors::ON_ACCENT
                                } else {
                                    colors::TEXT
                                }
                            }),
                    )
                    .on_press(move || sector_draft.set(press_target.clone()))
            }));

    // Market-cap tier chips from the shared tier table.
    let tier_chip_row =
        view()
            .flex_row()
            .gap(px(6.0))
            .w_full()
            .children(MarketCapTier::ALL.iter().copied().map(|tier| {
                view()
                    .px(px(10.0))
                    .py(px(4.0))
                    .rounded(px(radius::CONTROL))
                    .bg(move || {
                        if tier_draft.get() == tier {
                            colors::ACCENT
                        } else {
                            colors::INSET
                        }
                    })
                    .child(
                        text(tier.label())
                            .text_size(px(10.5))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(move || {
                                if tier_draft.get() == tier {
                                    colors::ON_ACCENT
                                } else {
                                    colors::TEXT
                                }
                            }),
                    )
                    .on_press(move || tier_draft.set(tier))
            }));

    let body = view()
        .flex_col()
        .gap(px(10.0))
        .w_full()
        .child(section_label("Sector"))
        .child(sector_chip_row)
        .child(section_label("Market cap"))
        .child(tier_chip_row)
        .child(tail_field("Min price (₹)", min_price, "e.g. 500"))
        .child(tail_field("Max price (₹)", max_price, "e.g. 50000"))
        .child(tail_field("Min day-change (%)", min_change_pct, "e.g. -5"))
        .child(tail_field("Max day-change (%)", max_change_pct, "e.g. 5"))
        .child(tail_field("Min volume (shares)", min_volume, "e.g. 100000"))
        .child(tail_field("Min P/E", min_pe, "e.g. 0"))
        .child(tail_field("Max P/E", max_pe, "e.g. 25"))
        .child(tail_field("Max P/B", max_pb, "e.g. 3"))
        .child(tail_field("Min ROE (%)", min_roe, "e.g. 20"))
        .child(tail_field("Min FCF yield (%)", min_fcf_yield, "e.g. 3"))
        .child(tail_field(
            "Min dividend yield (%)",
            min_dividend,
            "e.g. 1.5",
        ))
        .child(tail_field("Min RSI(14)", min_rsi, "e.g. 30"))
        .child(tail_field("Max RSI(14)", max_rsi, "e.g. 70"));

    let footer = view()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .child(
            HfButton::element(ghost_button_props())
                .child(text("Reset all"))
                .on_event(move |event: &HfButtonEvent| {
                    if *event == HfButtonEvent::Press {
                        sector_draft.set(String::new());
                        tier_draft.set(MarketCapTier::All);
                        min_price.set(String::new());
                        max_price.set(String::new());
                        min_change_pct.set(String::new());
                        max_change_pct.set(String::new());
                        min_volume.set(String::new());
                        min_pe.set(String::new());
                        max_pe.set(String::new());
                        max_pb.set(String::new());
                        min_dividend.set(String::new());
                        min_rsi.set(String::new());
                        max_rsi.set(String::new());
                        min_roe.set(String::new());
                        min_fcf_yield.set(String::new());
                    }
                }),
        )
        .child(
            HfButton::element(ghost_button_props())
                .child(text("Cancel"))
                .on_event(move |event: &HfButtonEvent| {
                    if *event == HfButtonEvent::Press {
                        open.set(false);
                    }
                }),
        )
        .child(
            view()
                .px(px(14.0))
                .py(px(6.0))
                .rounded(px(radius::CONTROL))
                .bg(colors::ACCENT)
                .child(
                    text("Apply")
                        .text_size(px(11.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors::ON_ACCENT),
                )
                .on_press(move || {
                    batch(move || {
                        let sector = sector_draft.get();
                        let next = FilterCriteria {
                            sector: if sector == "All" {
                                String::new()
                            } else {
                                sector
                            },
                            market_cap_tier: tier_draft.get(),
                            min_price: parse_decimal_tail(&min_price.get()),
                            max_price: parse_decimal_tail(&max_price.get()),
                            min_change_pct: parse_decimal_tail(&min_change_pct.get()),
                            max_change_pct: parse_decimal_tail(&max_change_pct.get()),
                            min_volume: parse_volume_tail(&min_volume.get()),
                            min_pe: parse_decimal_tail(&min_pe.get()),
                            max_pe: parse_decimal_tail(&max_pe.get()),
                            max_pb: parse_decimal_tail(&max_pb.get()),
                            min_dividend: parse_decimal_tail(&min_dividend.get()),
                            min_rsi: parse_decimal_tail(&min_rsi.get()),
                            max_rsi: parse_decimal_tail(&max_rsi.get()),
                            min_roe: parse_decimal_tail(&min_roe.get()),
                            min_fcf_yield: parse_decimal_tail(&min_fcf_yield.get()),
                        };
                        criteria.set(next);
                        open.set(false);
                    });
                }),
        );

    let panel = view()
        .flex_col()
        .w(px(560.0))
        .h(px(620.0))
        .gap(px(radius::INSET))
        .px(px(radius::CARD))
        .py(px(radius::CARD))
        .rounded(px(radius::CARD))
        .bg(colors::SURFACE)
        .border_1()
        .border_color(colors::BORDER)
        .child(
            view()
                .flex_col()
                .gap(px(2.0))
                .w_full()
                .child(
                    text("Filters")
                        .text_size(px(14.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors::TEXT),
                )
                .child(
                    text("Refine the screener universe")
                        .text_size(px(11.0))
                        .text_color(colors::MUTED),
                ),
        )
        .child(scroll().flex_1().flex_col().gap(px(10.0)).child(body))
        .child(footer);

    // The press handler lives only on the backdrop; the panel sits in
    // a press-swallowing wrapper so panel presses never close the modal.
    view()
        .w_full()
        .h_full()
        .flex_row()
        .items_center()
        .justify_center()
        .bg(rgb_a(0x0b0d12, 0.72))
        .on_press(move || open.set(false))
        .child(view().on_press(move || {}).child(panel))
}
