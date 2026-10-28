use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

#[derive(Clone, Debug)]
struct NavItem {
    label: &'static str,
    path: &'static str,
    icon: &'static str,
}

#[component]
pub fn NavRail() -> impl IntoView {
    let location = use_location();
    let items = vec![
        NavItem {
            label: "Trading",
            path: "/trading",
            icon: "📈",
        },
        NavItem {
            label: "F&O",
            path: "/derivatives/option-chain",
            icon: "⚡",
        },
        NavItem {
            label: "Portfolio",
            path: "/portfolio",
            icon: "💼",
        },
        NavItem {
            label: "Wealth",
            path: "/wealth/mutual-funds",
            icon: "🌱",
        },
        NavItem {
            label: "Quant",
            path: "/quant/simulator",
            icon: "🔬",
        },
        NavItem {
            label: "Screener",
            path: "/screener",
            icon: "🔍",
        },
        NavItem {
            label: "Research",
            path: "/research/security",
            icon: "📊",
        },
        NavItem {
            label: "Risk & Ops",
            path: "/risk",
            icon: "🛡️",
        },
    ];

    view! {
        <nav class="nav-rail flex flex-col gap-1 w-14 py-3 bg-surface-base border-r border-border-soft items-center select-none">
            {items.into_iter().map(|item| {
                let active = Signal::derive(move || {
                    let p = location.pathname.get();
                    p.as_str() == item.path || p.as_str().starts_with(&format!("{}/", item.path))
                });
                let class = Signal::derive(move || {
                    format!(
                        "flex flex-col items-center justify-center w-11 h-11 rounded transition-colors {}",
                        if active.get() {
                            "bg-surface-elevated text-text-high border-l-2 border-emerald-600"
                        } else {
                            "hover:bg-surface-elevated text-text-muted hover:text-text-high"
                        }
                    )
                });
                view! {
                    <A
                        href=item.path
                        attr:class=class
                        attr:title=item.label
                    >
                        <span class="text-base">{item.icon}</span>
                        <span class="text-[9px] font-sans tracking-tight">{item.label}</span>
                    </A>
                }
            }).collect::<Vec<_>>()}
        </nav>
    }
}
