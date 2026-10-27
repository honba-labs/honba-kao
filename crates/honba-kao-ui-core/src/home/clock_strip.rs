//! LiveClockStrip — Multi-market live clocks.
//!
//! Models OpenTerminalUI's `LiveClockStrip.tsx`. Displays timezone clocks (defaulting to
//! IST, EST, UTC) with live ticks, accessible `<time>` markup and ISO-8601 timestamps.

use chrono::{DateTime, Datelike, NaiveDateTime, Timelike, Utc};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveClockZone {
    pub id: String,
    pub label: String,
    pub time_zone: String,
}

impl LiveClockZone {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        time_zone: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            time_zone: time_zone.into(),
        }
    }
}

pub fn default_clock_zones() -> Vec<LiveClockZone> {
    vec![
        LiveClockZone::new("ist", "IST", "Asia/Kolkata"),
        LiveClockZone::new("est", "EST", "America/New_York"),
        LiveClockZone::new("utc", "UTC", "UTC"),
    ]
}

/// Determines whether a given UTC naive datetime falls within US Daylight Saving Time.
/// (Starts 2nd Sunday in March at 02:00, ends 1st Sunday in November at 02:00).
fn is_us_dst(utc: NaiveDateTime) -> bool {
    let month = utc.month();
    if month < 3 || month > 11 {
        return false;
    }
    if month > 3 && month < 11 {
        return true;
    }
    let day = utc.day();
    let weekday = utc.weekday().num_days_from_sunday(); // 0 = Sun
    if month == 3 {
        // Second Sunday in March: earliest 8th, latest 14th
        let second_sunday = 14 - (weekday + 14 - day) % 7;
        if day < second_sunday {
            false
        } else if day > second_sunday {
            true
        } else {
            utc.hour() >= 7 // 02:00 EST is 07:00 UTC
        }
    } else {
        // First Sunday in November: earliest 1st, latest 7th
        let first_sunday = 7 - (weekday + 7 - day) % 7;
        if day < first_sunday {
            true
        } else if day > first_sunday {
            false
        } else {
            utc.hour() < 6 // 02:00 EDT is 06:00 UTC
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn format_clock_wasm(millis: f64, time_zone: &str) -> Option<String> {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(millis));
    let opts = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&opts, &"hour".into(), &"2-digit".into());
    let _ = js_sys::Reflect::set(&opts, &"minute".into(), &"2-digit".into());
    let _ = js_sys::Reflect::set(&opts, &"second".into(), &"2-digit".into());
    let _ = js_sys::Reflect::set(&opts, &"hour12".into(), &false.into());
    let _ = js_sys::Reflect::set(&opts, &"timeZone".into(), &time_zone.into());

    let locales = js_sys::Array::new();
    locales.push(&"en-US".into());

    let formatter = js_sys::Intl::DateTimeFormat::new(&locales, &opts);
    let format_fn = formatter.format();
    let formatted = format_fn.call1(&wasm_bindgen::JsValue::NULL, &date).ok()?;
    formatted.as_string()
}

/// Formats a UTC timestamp into HH:MM:SS for the requested IANA timezone string.
pub fn format_clock_for_zone(dt: &DateTime<Utc>, time_zone: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(s) = format_clock_wasm(dt.timestamp_millis() as f64, time_zone) {
            return s;
        }
    }

    let tz_lower = time_zone.to_ascii_lowercase();
    let offset_seconds = if tz_lower.contains("kolkata") || tz_lower == "ist" {
        19800 // +5h 30m
    } else if tz_lower.contains("new_york") || tz_lower == "est" || tz_lower == "edt" {
        let is_dst = is_us_dst(dt.naive_utc());
        if is_dst { -14400 } else { -18000 }
    } else {
        0 // UTC
    };

    let shifted = *dt + chrono::Duration::seconds(offset_seconds);
    shifted.format("%H:%M:%S").to_string()
}

#[component]
pub fn LiveClockStrip(
    #[prop(optional)] zones: Option<Vec<LiveClockZone>>,
    #[prop(optional)] now: Option<DateTime<Utc>>,
    #[prop(optional, default = 1000)] update_interval_ms: u32,
    #[prop(optional, into)] class: String,
    #[prop(optional, into, default = "Live timezone clocks".to_string())] aria_label: String,
) -> impl IntoView {
    let clock_zones = zones.unwrap_or_else(default_clock_zones);
    let is_fixed = now.is_some();
    let (current_time, set_current_time) = signal(now.unwrap_or_else(Utc::now));

    if !is_fixed {
        if let Ok(handle) = set_interval_with_handle(
            move || {
                set_current_time.set(Utc::now());
            },
            core::time::Duration::from_millis(update_interval_ms as u64),
        ) {
            on_cleanup(move || handle.clear());
        }
    }

    let class_name = format!("ot-home-widget-clock-strip {class}")
        .trim()
        .to_string();

    view! {
        <div class=class_name role="list" aria-label=aria_label>
            {clock_zones.into_iter().map(|zone| {
                let zone_tz = zone.time_zone.clone();
                let zone_label = zone.label.clone();
                let time_val = move || {
                    let dt = current_time.get();
                    format_clock_for_zone(&dt, &zone_tz)
                };
                let iso_val = move || {
                    current_time.get().to_rfc3339()
                };

                view! {
                    <div class="ot-home-widget-clock-chip" role="listitem">
                        <span class="ot-home-widget-clock-label">{zone_label}</span>
                        <time class="ot-home-widget-clock-value" date-time=iso_val>
                            {time_val}
                        </time>
                    </div>
                }
            }).collect_view()}
        </div>
    }
}
