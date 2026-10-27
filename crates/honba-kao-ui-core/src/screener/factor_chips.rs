//! FactorChips — factor driver pills for the screener "Signals" column.

use leptos::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FactorDriver {
    Value,
    Momentum,
    Quality,
    LowVol,
}

impl FactorDriver {
    pub fn as_label(&self) -> &'static str {
        match self {
            FactorDriver::Value => "VALUE",
            FactorDriver::Momentum => "MOM",
            FactorDriver::Quality => "QUALITY",
            FactorDriver::LowVol => "LOW-VOL",
        }
    }

    pub fn as_class(&self) -> &'static str {
        match self {
            FactorDriver::Value => "factor-value",
            FactorDriver::Momentum => "factor-momentum",
            FactorDriver::Quality => "factor-quality",
            FactorDriver::LowVol => "factor-lowvol",
        }
    }
}

#[component]
pub fn FactorChips(
    drivers: Vec<FactorDriver>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    view! {
        <div class=format!("sc-factor-chips {class}").trim().to_string()>
            {drivers
                .into_iter()
                .map(|driver| {
                    view! {
                        <span class=format!("sc-factor-chip {}", driver.as_class())>
                            {driver.as_label()}
                        </span>
                    }
                })
                .collect_view()}
        </div>
    }
}
