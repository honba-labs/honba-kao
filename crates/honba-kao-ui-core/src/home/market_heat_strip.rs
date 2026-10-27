//! MarketHeatStrip — Real-time index and commodity price strip.
//!
//! Models OpenTerminalUI's `MarketHeatStrip.tsx`. Displays index/commodity chips with
//! flash-up/down animations, tone colors, selection state and interactive click callbacks.

use leptos::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketTone {
    Up,
    Down,
    Neutral,
}

impl MarketTone {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::Neutral => "neutral",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketFlash {
    Up,
    Down,
}

impl MarketFlash {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MarketHeatStripItem {
    pub id: String,
    pub label: String,
    pub value: Option<f64>,
    pub value_str: Option<String>,
    pub change_pct: Option<f64>,
    pub change_label: Option<String>,
    pub flash: Option<MarketFlash>,
    pub on_select: Option<Callback<()>>,
}

impl PartialEq for MarketHeatStripItem {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.label == other.label
            && self.value == other.value
            && self.value_str == other.value_str
            && self.change_pct == other.change_pct
            && self.change_label == other.change_label
            && self.flash == other.flash
    }
}

impl MarketHeatStripItem {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        value: Option<f64>,
        change_pct: Option<f64>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value,
            value_str: None,
            change_pct,
            change_label: None,
            flash: None,
            on_select: None,
        }
    }

    pub fn with_flash(mut self, flash: MarketFlash) -> Self {
        self.flash = Some(flash);
        self
    }

    pub fn with_change_label(mut self, label: impl Into<String>) -> Self {
        self.change_label = Some(label.into());
        self
    }

    pub fn with_value_str(mut self, value_str: impl Into<String>) -> Self {
        self.value_str = Some(value_str.into());
        self
    }

    pub fn with_on_select(mut self, cb: Callback<()>) -> Self {
        self.on_select = Some(cb);
        self
    }
}

pub fn tone_for_change(change_pct: Option<f64>) -> MarketTone {
    match change_pct {
        Some(pct) if pct.is_finite() && pct > 0.0 => MarketTone::Up,
        Some(pct) if pct.is_finite() && pct < 0.0 => MarketTone::Down,
        _ => MarketTone::Neutral,
    }
}

/// Formats a number with US comma thousands separators and up to `max_decimals` decimal places.
pub fn format_number_with_commas(val: f64, max_decimals: usize) -> String {
    if !val.is_finite() {
        return "--".to_string();
    }
    let is_neg = val < 0.0;
    let abs_val = val.abs();
    let int_part = abs_val.trunc() as u64;
    let frac_part = abs_val - (int_part as f64);

    let int_str = int_part.to_string();
    let mut formatted_int = String::with_capacity(int_str.len() + int_str.len() / 3);
    let bytes = int_str.as_bytes();
    let len = bytes.len();
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            formatted_int.push(',');
        }
        formatted_int.push(b as char);
    }

    if max_decimals == 0 {
        return if is_neg {
            format!("-{formatted_int}")
        } else {
            formatted_int
        };
    }

    let frac_multiplier = 10f64.powi(max_decimals as i32);
    let rounded_frac = (frac_part * frac_multiplier).round() as u64;
    if rounded_frac == 0 {
        if is_neg {
            format!("-{formatted_int}")
        } else {
            formatted_int
        }
    } else {
        let mut frac_str = format!("{:0width$}", rounded_frac, width = max_decimals);
        while frac_str.ends_with('0') {
            frac_str.pop();
        }
        let sign = if is_neg { "-" } else { "" };
        format!("{sign}{formatted_int}.{frac_str}")
    }
}

pub fn format_default_market_value(value: Option<f64>, value_str: Option<&str>) -> String {
    if let Some(s) = value_str {
        return s.to_string();
    }
    if let Some(v) = value {
        format_number_with_commas(v, 2)
    } else {
        "--".to_string()
    }
}

#[component]
pub fn MarketHeatStrip(
    items: Vec<MarketHeatStripItem>,
    #[prop(optional, into)] selected_item_id: Option<String>,
    #[prop(optional)] on_select: Option<Callback<MarketHeatStripItem>>,
    #[prop(optional, into)] class: String,
    #[prop(optional, into, default = "Market heat strip".to_string())] aria_label: String,
) -> impl IntoView {
    let class_name = format!("ot-home-widget-strip {class}").trim().to_string();

    view! {
        <div class=class_name role="list" aria-label=aria_label>
            {items.into_iter().map(|item| {
                let is_selected = selected_item_id.as_ref() == Some(&item.id);
                let tone = tone_for_change(item.change_pct);
                let tone_str = tone.as_str();

                let val_str = format_default_market_value(item.value, item.value_str.as_deref());
                let change_display = if let Some(ref label) = item.change_label {
                    label.clone()
                } else if let Some(pct) = item.change_pct {
                    if pct.is_finite() {
                        let sign = if pct >= 0.0 { "+" } else { "" };
                        format!("{sign}{pct:.2}%")
                    } else {
                        "--".to_string()
                    }
                } else {
                    "--".to_string()
                };

                let flash_class = item.flash.map(|f| format!("ot-flash-{}", f.as_str())).unwrap_or_default();
                let chip_class = format!("ot-home-widget-strip-chip {flash_class}").trim().to_string();

                let aria_pct_desc = item.change_pct.map(|p| format!("{:.2} percent", p)).unwrap_or_default();
                let item_aria_label = format!("{} {} {}", item.label, val_str, aria_pct_desc).trim().to_string();

                let has_action = item.on_select.is_some() || on_select.is_some();
                let item_clone = item.clone();
                let on_select_cb = on_select;

                let handle_click = move |_| {
                    if let Some(ref item_cb) = item_clone.on_select {
                        item_cb.run(());
                    }
                    if let Some(ref strip_cb) = on_select_cb {
                        strip_cb.run(item_clone.clone());
                    }
                };

                let content = view! {
                    <span class="ot-home-widget-strip-label">{item.label.clone()}</span>
                    <span class="ot-home-widget-strip-value">{val_str}</span>
                    <span class="ot-home-widget-strip-change" data-tone=tone_str>
                        {change_display}
                    </span>
                };

                view! {
                    <div role="listitem">
                        {if has_action {
                            view! {
                                <button
                                    type="button"
                                    class=chip_class.clone()
                                    data-tone=tone_str
                                    data-selected=if is_selected { "true" } else { "false" }
                                    aria-pressed=if is_selected { "true" } else { "false" }
                                    aria-label=item_aria_label.clone()
                                    on:click=handle_click
                                >
                                    {content}
                                </button>
                            }.into_any()
                        } else {
                            view! {
                                <div
                                    class=chip_class.clone()
                                    data-tone=tone_str
                                    data-selected=if is_selected { "true" } else { "false" }
                                    aria-label=item_aria_label.clone()
                                >
                                    {content}
                                </div>
                            }.into_any()
                        }}
                    </div>
                }
            }).collect_view()}
        </div>
    }
}
