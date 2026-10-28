use leptos::prelude::*;

/// `HfTwoColumnSplitLayout` layout template — 70/30 asymmetric split layout.
/// Used for portfolio, collateral manager, wealth, and risk telemetry dashboards.
#[component]
pub fn HfTwoColumnSplitLayout(
    primary: AnyView,
    rail: AnyView,
    #[prop(default = false)] reverse_on_mobile: bool,
) -> impl IntoView {
    let order_class = if reverse_on_mobile {
        "flex-col-reverse"
    } else {
        "flex-col"
    };
    view! {
        <div class=format!("split-layout grid grid-cols-12 gap-4 w-full {order_class}")>
            <div class="col-span-12 laptop:col-span-8 desktop:col-span-8 flex flex-col gap-4">
                {primary}
            </div>
            <div class="col-span-12 laptop:col-span-4 desktop:col-span-4 flex flex-col gap-4">
                {rail}
            </div>
        </div>
    }
}
