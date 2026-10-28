use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AlertSeverity {
    #[default]
    Info,
    Success,
    Warning,
    Critical,
}

impl AlertSeverity {
    pub fn badge_class(self) -> &'static str {
        match self {
            AlertSeverity::Info => "bg-blue-500/10 text-blue-400 border-blue-500/20",
            AlertSeverity::Success => "bg-emerald-500/10 text-emerald-400 border-emerald-500/20",
            AlertSeverity::Warning => "bg-amber-500/10 text-amber-400 border-amber-500/20",
            AlertSeverity::Critical => "bg-rose-500/10 text-rose-400 border-rose-500/20",
        }
    }
}

/// Alert banner stack rendered at the top of an institutional page.
#[component]
pub fn HfAlertBannerStack(#[prop(into, optional)] children: Option<Children>) -> impl IntoView {
    view! {
        <div class="hf-alert-banner-stack flex flex-col gap-2 w-full">
            {children.map(|c| c())}
        </div>
    }
}

/// `HfPage` layout template — standard container enforcing margins, responsive max-widths,
/// optional alert banner slot, and standardized page header.
#[component]
pub fn HfPage(
    #[prop(into, optional)] title: Option<String>,
    #[prop(into, optional)] subtitle: Option<String>,
    #[prop(into, optional)] actions: Option<AnyView>,
    #[prop(into, optional)] alert_banner: Option<AnyView>,
    #[prop(default = "desktop:max-w-[1920px]")] max_width_class: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!("page-container mx-auto px-4 py-3 tablet:px-6 laptop:py-4 {max_width_class}")>
            {alert_banner.map(|banner| view! { <div class="page-alert-stack mb-3">{banner}</div> })}
            {title.map(|t| {
                let s = subtitle.clone();
                let a = actions;
                view! {
                    <div class="page-header flex items-center justify-between border-b border-border-soft pb-3 mb-4">
                        <div>
                            <h1 class="text-lg font-semibold text-text-high tracking-tight">{t}</h1>
                            {s.map(|sub| view! { <p class="text-xs text-text-muted mt-0.5">{sub}</p> })}
                        </div>
                        {a.map(|act| view! { <div class="page-header-actions flex items-center gap-2">{act}</div> })}
                    </div>
                }
            })}
            <div class="page-body flex flex-col gap-4">
                {children()}
            </div>
        </div>
    }
}
