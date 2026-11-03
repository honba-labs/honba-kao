//! Landing page — Sovereign marketing & onboarding surface for Honba Workstation.
//! Ported directly from legacy `FlowLanding.tsx` / `flow_landing.rs` onto NSH primitives.
//!
//! Architectural highlights:
//! - Immersive full-page scroll container with graphite dark palette (`#11141b`).
//! - Desktop 2-column Hero: high-impact copy on left, interactive Frontier preview card on right.
//! - Interactive Execution Plan selector (Market, Limit, Ladder, TWAP) powered by NSH `signal()`.
//! - Visual trade corridor canvas detailing Target gain zone, Entry level, and Hard Stop.
//! - Supported markets ribbon: NSE, BSE, Zerodha, Dhan, Upstox, Angel One, Fyers, Groww.
//! - 3 core capability cards ("See the alternatives", "Know where you stand", "Make it your own").
//! - Sovereign local execution strip with CLI command snippet and hardware keyring custody.

use crate::router::chrome::app_shell;
use crate::router::paths::*;
use honba_kao_ui_core::icons::canvas_icons::{IconName, icon};
use honba_kao_ui_core::primitive::badge::{BadgeVariant, HfStatusChip, HfStatusChipProps};
use honba_kao_ui_core::primitive::base::button::{
    ButtonSize, ButtonVariant, HfButton, HfButtonEvent, HfButtonProps,
};
use honba_kao_ui_core::primitive::card::{HfCard, HfCardProps};
use honba_kao_ui_core::primitive::currency::{Currency, HfAssetBadge, HfAssetBadgeProps};
use honba_kao_ui_core::theme::tokens::{HonbaStyle, colors};
use nsh_core::prelude::*;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct LandingPageProps {
    #[prop(default)]
    pub _unused: String,
}

/// The four execution order plans available in the Frontier preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExecutionPlan {
    Market,
    Limit,
    Ladder,
    Twap,
}

impl ExecutionPlan {
    const ALL: [ExecutionPlan; 4] = [
        ExecutionPlan::Market,
        ExecutionPlan::Limit,
        ExecutionPlan::Ladder,
        ExecutionPlan::Twap,
    ];

    fn number(&self) -> &'static str {
        match self {
            ExecutionPlan::Market => "01",
            ExecutionPlan::Limit => "02",
            ExecutionPlan::Ladder => "03",
            ExecutionPlan::Twap => "04",
        }
    }

    fn title(&self) -> &'static str {
        match self {
            ExecutionPlan::Market => "Market",
            ExecutionPlan::Limit => "Limit",
            ExecutionPlan::Ladder => "Ladder",
            ExecutionPlan::Twap => "TWAP",
        }
    }

    fn subtitle(&self) -> &'static str {
        match self {
            ExecutionPlan::Market => "Enter now",
            ExecutionPlan::Limit => "Choose a price",
            ExecutionPlan::Ladder => "Split by price",
            ExecutionPlan::Twap => "Split by time",
        }
    }

    fn detail_text(&self) -> &'static str {
        match self {
            ExecutionPlan::Market => {
                "Immediate fill sweeping top of book with mandatory circuit band protection and sub-15 µs risk checks. Best for high-conviction momentum entries."
            }
            ExecutionPlan::Limit => {
                "Places resting liquidity directly onto NSE/BSE books at ₹2,450.00. Zero taker fee impact with automated order cancellation on risk trigger."
            }
            ExecutionPlan::Ladder => {
                "Splits lot into 3 tranches (30% / 40% / 30%) across a ₹12 corridor to systematically minimize market impact in volatile conditions."
            }
            ExecutionPlan::Twap => {
                "Time-weighted average price execution sliced evenly across 60 minutes with randomized intervals to eliminate predatory front-running."
            }
        }
    }
}

/// Landing — Sovereign Workstation onboarding page ported to NSH.
#[component]
pub fn LandingPage(_props: LandingPageProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let selected_plan = signal(ExecutionPlan::Limit);

    view().flex_1().child(
        scroll()
            .flex_1()
            .flex_col()
            .w_full()
            .bg(colors::BACKGROUND)
            .child(
                view()
                    .flex_col()
                    .w_full()
                    .max_w(px(1440.0))
                    .px(px(6.0))
                    .py(px(6.0))
                    .gap(px(10.0))
                    // 1. Two-Column Hero: Copy + Product Preview Card
                    .child(render_hero_section(selected_plan))
                    // 2. Markets & Exchanges Strip ("Your markets. Connected.")
                    .child(render_exchanges_strip())
                    // 3. Feature Capabilities ("One connected workspace")
                    .child(render_features_section())
                    // 4. Sovereign Local Execution ("Built to run locally")
                    .child(render_local_section()),
            ),
    )
}

/// 1. Hero Section: 2-column layout (Copy on left, Frontier Preview Card on right).
fn render_hero_section(selected_plan: Signal<ExecutionPlan>) -> impl IntoElement {
    view()
        .flex_row()
        .items_center()
        .justify_between()
        .gap(px(8.0))
        .w_full()
        .pt(px(4.0))
        .pb(px(4.0))
        .child(
            // Left Column: Headline, Subtitle, Actions, Trust points
            view()
                .flex_1()
                .flex_col()
                .items_start()
                .gap(px(4.0))
                .child(
                    // Eyebrow badge
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(2.0))
                        .px(px(3.0))
                        .py(px(1.0))
                        .rounded(px(999.0))
                        .surface_muted()
                        .border_soft()
                        .child(
                            view()
                                .w(px(2.0))
                                .h(px(2.0))
                                .rounded(px(999.0))
                                .bg(AdaptiveColor::rgb(0x10b981, 0x10b981)),
                        )
                        .child(
                            text("OPEN SOURCE • SELF HOSTED • INDIAN BROKERS")
                                .text_positive()
                                .text_size(px(11.0))
                                .font_weight(FontWeight::SEMIBOLD),
                        ),
                )
                .child(
                    // Headline
                    view()
                        .flex_col()
                        .items_start()
                        .gap(px(1.0))
                        .child(
                            text("Intelligence")
                                .text_high()
                                .text_size(px(38.0))
                                .font_weight(FontWeight::BOLD)
                                .line_height(px(44.0)),
                        )
                        .child(
                            text("that trades NSE & BSE.")
                                .text_positive()
                                .text_size(px(38.0))
                                .font_weight(FontWeight::BOLD)
                                .line_height(px(44.0)),
                        ),
                )
                .child(
                    // Lead paragraph
                    text("An open, sovereign trading system you run yourself — built for Indian brokers, sub-15 µs risk gating, and double-entry FIFO accounting.")
                        .text_muted()
                        .text_size(px(15.0))
                        .line_height(px(22.0))
                        .max_w(px(520.0)),
                )
                .child(
                    // Dual Action CTAs
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(3.0))
                        .pt(px(2.0))
                        .child(
                            HfButton::element(HfButtonProps {
                                href: Prop::default(),
                                size: Prop::from(Some(ButtonSize::Md)),
                                variant: Prop::from(Some(ButtonVariant::Primary)),
                                full_width: Prop::from(false),
                                disabled: Prop::from(false),
                                aria_label: Prop::from(Some("Open Trading Cockpit".to_string())),
                                class: Prop::default(),
                                children: Children::default(),
                            })
                            .child(
                                view()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(2.0))
                                    .child(icon().name(IconName::Stocks).size(16.0))
                                    .child(text("Open Trading Cockpit")),
                            )
                            .on_event(|event: &HfButtonEvent| {
                                if *event == HfButtonEvent::Press {
                                    app_shell::open_path(ROUTE_TRADING_COCKPIT);
                                }
                            }),
                        )
                        .child(
                            HfButton::element(HfButtonProps {
                                href: Prop::default(),
                                size: Prop::from(Some(ButtonSize::Md)),
                                variant: Prop::from(Some(ButtonVariant::Outline)),
                                full_width: Prop::from(false),
                                disabled: Prop::from(false),
                                aria_label: Prop::from(Some("Open Portfolio".to_string())),
                                class: Prop::default(),
                                children: Children::default(),
                            })
                            .child(
                                view()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(2.0))
                                    .child(icon().name(IconName::Wealth).size(16.0))
                                    .child(text("Open your portfolio")),
                            )
                            .on_event(|event: &HfButtonEvent| {
                                if *event == HfButtonEvent::Press {
                                    app_shell::open_path(ROUTE_PORTFOLIO);
                                }
                            }),
                        ),
                )
                .child(
                    // Trust Badges
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(4.0))
                        .pt(px(2.0))
                        .child(render_trust_item("Runs on your machine"))
                        .child(render_trust_item("Your broker accounts"))
                        .child(render_trust_item("SEBI Algo ID compliant")),
                ),
        )
        .child(
            // Right Column: Frontier Product Preview Card
            view()
                .flex_1()
                .w_full()
                .child(render_product_preview(selected_plan)),
        )
}

fn render_trust_item(label: &'static str) -> impl IntoElement {
    view()
        .flex_row()
        .items_center()
        .gap(px(1.5))
        .child(
            icon()
                .name(IconName::Check)
                .size(12.0)
                .tint(AdaptiveColor::rgb(0x10b981, 0x10b981)),
        )
        .child(text(label).text_muted().text_size(px(11.0)))
}

/// Product Preview Card: Replicates Frontier planning workspace with interactive tabs.
fn render_product_preview(selected_plan: Signal<ExecutionPlan>) -> impl IntoElement {
    HfCard::element(HfCardProps {
        outlined: Prop::from(true),
        children: Children::default(),
    })
    .child(
        view()
            .flex_col()
            .w_full()
            .p(px(5.0))
            .gap(px(3.5))
            .surface_base()
            .child(
                // Preview Header
                view()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .pb(px(2.5))
                    .border_soft()
                    .child(
                        view()
                            .flex_row()
                            .items_center()
                            .gap(px(2.5))
                            .child(
                                HfAssetBadge::element(HfAssetBadgeProps {
                                    currency: Prop::from(Currency::Inr),
                                    symbol: Prop::from(Some("FRONTIER".to_string())),
                                    children: Children::default(),
                                })
                                .child(text("NSE:RELIANCE")),
                            )
                            .child(
                                text("Frontier Workspace Preview")
                                    .text_high()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_size(px(13.0)),
                            ),
                    )
                    .child(HfStatusChip::element(HfStatusChipProps {
                        status: Prop::from("Paper Wallet Connected".to_string()),
                        variant: Prop::from(Some(BadgeVariant::Success)),
                    })),
            )
            .child(
                // Scenario Subhead
                view()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        view()
                            .flex_col()
                            .gap(px(0.5))
                            .child(
                                text("One idea. A clear plan.")
                                    .text_muted()
                                    .text_size(px(11.0)),
                            )
                            .child(
                                text("Compare your entry.")
                                    .text_high()
                                    .font_weight(FontWeight::BOLD)
                                    .text_size(px(16.0)),
                            ),
                    )
                    .child(
                        text("Ref: ₹2,450.00 | Lot: 250")
                            .text_muted()
                            .text_size(px(11.0)),
                    ),
            )
            .child(
                // Visual Corridor Diagram (Target / Entry / Stop)
                render_trade_corridor(selected_plan),
            )
            .child(
                // 4 Interactive Plan Buttons (01 Market, 02 Limit, 03 Ladder, 04 TWAP)
                view().flex_row().gap(px(2.0)).w_full().child({
                    let mut plan_bar = view().flex_row().gap(px(2.0)).w_full();
                    for plan in ExecutionPlan::ALL {
                        let is_active = selected_plan;
                        plan_bar = plan_bar.child(
                            view()
                                .flex_1()
                                .flex_col()
                                .p(px(2.5))
                                .gap(px(0.5))
                                .rounded(px(6.0))
                                .border_soft()
                                .surface_muted()
                                .child(
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .child(text(plan.number()).text_muted().text_size(px(10.0)))
                                        .child(
                                            text(move || {
                                                if is_active.get() == plan {
                                                    "● ACTIVE"
                                                } else {
                                                    ""
                                                }
                                            })
                                            .text_positive()
                                            .text_size(px(9.0)),
                                        ),
                                )
                                .child(
                                    text(plan.title())
                                        .text_high()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_size(px(12.0)),
                                )
                                .child(text(plan.subtitle()).text_muted().text_size(px(10.0)))
                                .on_press(move |_: &()| {
                                    selected_plan.set(plan);
                                }),
                        );
                    }
                    plan_bar
                }),
            )
            .child(
                // Preview Bottom with Review & Action
                view()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .pt(px(2.0))
                    .border_soft()
                    .child(
                        view()
                            .flex_row()
                            .items_center()
                            .gap(px(1.5))
                            .child(
                                icon()
                                    .name(IconName::Check)
                                    .size(13.0)
                                    .tint(AdaptiveColor::rgb(0x10b981, 0x10b981)),
                            )
                            .child(
                                text("Review before you trade")
                                    .text_muted()
                                    .text_size(px(11.0)),
                            ),
                    )
                    .child(
                        view().flex_row().items_center().gap(px(2.0)).child(
                            HfButton::element(HfButtonProps {
                                href: Prop::default(),
                                size: Prop::from(Some(ButtonSize::Xs)),
                                variant: Prop::from(Some(ButtonVariant::Secondary)),
                                full_width: Prop::from(false),
                                disabled: Prop::from(false),
                                aria_label: Prop::from(Some("Launch Plan".to_string())),
                                class: Prop::default(),
                                children: Children::default(),
                            })
                            .child(text("Make your next move →"))
                            .on_event(|event: &HfButtonEvent| {
                                if *event == HfButtonEvent::Press {
                                    app_shell::open_path(ROUTE_TRADING_COCKPIT);
                                }
                            }),
                        ),
                    ),
            ),
    )
}

/// Trade Corridor Diagram: Visual Target, Entry, Stop levels + dynamic plan explanation.
fn render_trade_corridor(selected_plan: Signal<ExecutionPlan>) -> impl IntoElement {
    view()
        .flex_col()
        .gap(px(2.0))
        .p(px(3.0))
        .rounded(px(8.0))
        .surface_muted()
        .border_soft()
        .child(
            // Target Corridor (Gain Zone)
            view()
                .flex_row()
                .items_center()
                .justify_between()
                .p(px(2.0))
                .rounded(px(6.0))
                .bg(AdaptiveColor::rgb(0x10b981, 0x10b981).opacity(0.12))
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(1.5))
                        .child(
                            view()
                                .w(px(2.0))
                                .h(px(2.0))
                                .rounded(px(999.0))
                                .bg(AdaptiveColor::rgb(0x10b981, 0x10b981)),
                        )
                        .child(
                            text("TARGET (+2.86%)")
                                .text_positive()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_size(px(11.0)),
                        )
                        .child(
                            text("VWAP upper band corridor")
                                .text_muted()
                                .text_size(px(10.0)),
                        ),
                )
                .child(
                    text("₹2,520.00")
                        .text_positive()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(12.0)),
                ),
        )
        .child(
            // Entry Level
            view()
                .flex_row()
                .items_center()
                .justify_between()
                .p(px(2.0))
                .rounded(px(6.0))
                .border_soft()
                .surface_elevated()
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(1.5))
                        .child(
                            view()
                                .w(px(2.0))
                                .h(px(2.0))
                                .rounded(px(999.0))
                                .bg(AdaptiveColor::rgb(0xfbbf24, 0xfbbf24)),
                        )
                        .child(
                            text("ENTRY BENCHMARK")
                                .text_high()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_size(px(11.0)),
                        )
                        .child(
                            text("Planned execution price")
                                .text_muted()
                                .text_size(px(10.0)),
                        ),
                )
                .child(
                    text("₹2,450.00")
                        .text_high()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(12.0)),
                ),
        )
        .child(
            // Hard Stop Level
            view()
                .flex_row()
                .items_center()
                .justify_between()
                .p(px(2.0))
                .rounded(px(6.0))
                .bg(AdaptiveColor::rgb(0xef4444, 0xef4444).opacity(0.12))
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(1.5))
                        .child(
                            view()
                                .w(px(2.0))
                                .h(px(2.0))
                                .rounded(px(999.0))
                                .bg(AdaptiveColor::rgb(0xef4444, 0xef4444)),
                        )
                        .child(
                            text("STOP LEVEL (-1.43%)")
                                .text_negative()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_size(px(11.0)),
                        )
                        .child(
                            text("Circuit band stop limit")
                                .text_muted()
                                .text_size(px(10.0)),
                        ),
                )
                .child(
                    text("₹2,415.00")
                        .text_negative()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(12.0)),
                ),
        )
        .child(
            // Reactive Plan Details Note
            view().p(px(2.0)).child(
                text(move || selected_plan.get().detail_text())
                    .text_muted()
                    .text_size(px(11.0))
                    .line_height(px(16.0)),
            ),
        )
}

/// 2. Exchanges Strip: "Your markets. Connected." (NSE, BSE, Zerodha, Dhan, Upstox, Angel One, Fyers).
fn render_exchanges_strip() -> impl IntoElement {
    view()
        .flex_col()
        .gap(px(2.5))
        .py(px(4.0))
        .border_soft()
        .child(
            view()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    text("YOUR MARKETS. CONNECTED.")
                        .text_muted()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::SEMIBOLD),
                )
                .child(
                    text("Level 2 Depth Feeds • Sub-15 µs Gating • Direct REST/WS APIs")
                        .text_muted()
                        .text_size(px(11.0)),
                ),
        )
        .child(
            view()
                .flex_row()
                .items_center()
                .justify_between()
                .gap(px(3.0))
                .pt(px(1.0))
                .child(render_market_chip("NSE", "National Stock Exchange"))
                .child(render_market_chip("BSE", "Bombay Stock Exchange"))
                .child(render_market_chip("Zerodha", "Kite Connect"))
                .child(render_market_chip("Dhan", "HQ WebSocket"))
                .child(render_market_chip("Upstox", "Pro API v2"))
                .child(render_market_chip("Angel One", "SmartAPI"))
                .child(render_market_chip("Fyers", "Data 3.0")),
        )
}

fn render_market_chip(title: &'static str, subtitle: &'static str) -> impl IntoElement {
    view()
        .flex_1()
        .flex_col()
        .items_center()
        .p(px(2.5))
        .rounded(px(6.0))
        .surface_muted()
        .child(
            text(title)
                .text_high()
                .font_weight(FontWeight::BOLD)
                .text_size(px(13.0)),
        )
        .child(text(subtitle).text_muted().text_size(px(9.5)))
}

/// 3. Features Section: "One connected workspace" with 3 capability cards.
fn render_features_section() -> impl IntoElement {
    view()
        .flex_col()
        .gap(px(5.0))
        .pt(px(4.0))
        .child(
            // Section heading
            view()
                .flex_col()
                .gap(px(1.0))
                .child(
                    text("ONE CONNECTED WORKSPACE")
                        .text_positive()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_size(px(11.0)),
                )
                .child(
                    text("Less friction. More clarity.")
                        .text_high()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(24.0)),
                )
                .child(
                    text("AI helps you explore. Clear controls help you act.")
                        .text_muted()
                        .text_size(px(13.0)),
                ),
        )
        .child(
            // 3 Capability Cards
            view()
                .flex_row()
                .gap(px(4.0))
                .w_full()
                .child(render_feature_article(
                    IconName::Lists,
                    "See the alternatives.",
                    "Compare market, limit, ladder and TWAP plans. See entries, size and exits before committing.",
                    "One portfolio for everything →",
                    ROUTE_PORTFOLIO,
                ))
                .child(render_feature_article(
                    IconName::Wealth,
                    "Know where you stand.",
                    "Every broker in one book. Holdings, funds and margin stay in view — Zerodha, Dhan, Upstox, Angel One, Fyers and Groww.",
                    "Connect your accounts →",
                    ROUTE_ACCOUNT,
                ))
                .child(render_feature_article(
                    IconName::Quant,
                    "Make it your own.",
                    "Choose your AI model and broker accounts. Set risk gates once, then let agents trade within them.",
                    "Build a strategy →",
                    ROUTE_DERIVATIVES_STRATEGY,
                )),
        )
}

fn render_feature_article(
    icon_name: IconName,
    title: &'static str,
    body: &'static str,
    action_label: &'static str,
    route: &'static str,
) -> impl IntoElement {
    HfCard::element(HfCardProps {
        outlined: Prop::from(true),
        children: Children::default(),
    })
    .child(
        view()
            .flex_1()
            .flex_col()
            .p(px(4.5))
            .gap(px(3.0))
            .surface_base()
            .child(
                view()
                    .w(px(8.0))
                    .h(px(8.0))
                    .rounded(px(8.0))
                    .surface_muted()
                    .flex_row()
                    .items_center()
                    .justify_center()
                    .child(icon().name(icon_name).size(20.0).tint(colors::TEXT)),
            )
            .child(
                text(title)
                    .text_high()
                    .font_weight(FontWeight::BOLD)
                    .text_size(px(16.0)),
            )
            .child(
                text(body)
                    .text_muted()
                    .text_size(px(12.5))
                    .line_height(px(19.0)),
            )
            .child(
                HfButton::element(HfButtonProps {
                    href: Prop::default(),
                    size: Prop::from(Some(ButtonSize::Xs)),
                    variant: Prop::from(Some(ButtonVariant::Ghost)),
                    full_width: Prop::from(false),
                    disabled: Prop::from(false),
                    aria_label: Prop::from(Some(format!("Navigate to {title}"))),
                    class: Prop::default(),
                    children: Children::default(),
                })
                .child(text(action_label).text_positive())
                .on_event(move |event: &HfButtonEvent| {
                    if *event == HfButtonEvent::Press {
                        app_shell::open_path(route);
                    }
                }),
            ),
    )
}

/// 4. Local Execution Section: "Built to run locally" (Sovereign hardware runtime).
fn render_local_section() -> impl IntoElement {
    HfCard::element(HfCardProps {
        outlined: Prop::from(true),
        children: Children::default(),
    })
    .child(
        view()
            .flex_row()
            .items_center()
            .justify_between()
            .p(px(6.0))
            .surface_muted()
            .child(
                view()
                    .flex_col()
                    .gap(px(1.5))
                    .max_w(px(580.0))
                    .child(
                        view()
                            .flex_row()
                            .items_center()
                            .gap(px(2.0))
                            .child(icon().name(IconName::Storage).size(16.0))
                            .child(
                                text("BUILT TO RUN LOCALLY")
                                    .text_positive()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_size(px(11.0)),
                            ),
                    )
                    .child(
                        text("Your workspace. On your terms.")
                            .text_high()
                            .font_weight(FontWeight::BOLD)
                            .text_size(px(20.0)),
                    )
                    .child(
                        text("Install honba-kao on your machine, connect your broker accounts and choose how you want to trade. Zero cloud telemetry. Private keys stay in your hardware keyring.")
                            .text_muted()
                            .text_size(px(13.0))
                            .line_height(px(19.0)),
                    ),
            )
            .child(
                view()
                    .flex_col()
                    .items_end()
                    .gap(px(2.5))
                    .child(
                        view()
                            .p(px(3.0))
                            .rounded(px(6.0))
                            .surface_base()
                            .border_soft()
                            .child(
                                text("cargo run -p honba-kao-app --release")
                                    .text_high()
                                    .font_family("monospace")
                                    .text_size(px(12.0)),
                            ),
                    )
                    .child(
                        HfButton::element(HfButtonProps {
                            href: Prop::default(),
                            size: Prop::from(Some(ButtonSize::Sm)),
                            variant: Prop::from(Some(ButtonVariant::Primary)),
                            full_width: Prop::from(false),
                            disabled: Prop::from(false),
                            aria_label: Prop::from(Some("Launch Workstation".to_string())),
                            class: Prop::default(),
                            children: Children::default(),
                        })
                        .child(text("Launch Workstation"))
                        .on_event(|event: &HfButtonEvent| {
                            if *event == HfButtonEvent::Press {
                                app_shell::open_path(ROUTE_TRADING_COCKPIT);
                            }
                        }),
                    ),
            ),
    )
}
