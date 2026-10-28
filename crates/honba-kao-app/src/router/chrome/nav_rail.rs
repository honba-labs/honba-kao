use leptos::prelude::*;

#[derive(Clone, Debug)]
struct NavItem {
    label: &'static str,
    path: &'static str,
    icon: &'static str,
}

#[component]
pub fn NavRail() -> impl IntoView {
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
                view! {
                    <a
                        href=item.path
                        class="flex flex-col items-center justify-center w-11 h-11 rounded hover:bg-surface-elevated text-text-muted hover:text-text-high transition-colors"
                        title=item.label
                    >
                        <span class="text-base">{item.icon}</span>
                        <span class="text-[9px] font-sans tracking-tight">{item.label}</span>
                    </a>
                }
            }).collect::<Vec<_>>()}
        </nav>
    }
}
