use leptos::prelude::*;

/// `HfTwoColumnSplitLayout` layout template — 70/30 asymmetric split layout.
/// Used for portfolio, collateral manager, wealth, and risk telemetry dashboards.
///
/// Desktop (laptop+): an 8/4 grid (~70/30) of `primary` and `rail`.
/// Mobile: the two columns stack vertically. `reverse_on_mobile` flips the
/// stacking order on small viewports (issue #39 reverse bug, #74 breakpoints).
#[component]
pub fn HfTwoColumnSplitLayout(
    primary: AnyView,
    rail: AnyView,
    #[prop(default = false)] reverse_on_mobile: bool,
) -> impl IntoView {
    let stack_dir = if reverse_on_mobile {
        "flex-col-reverse"
    } else {
        "flex-col"
    };
    view! {
        <div class=format!("split-layout w-full gap-4 flex {} laptop:grid", stack_dir)>
            <div class="col-span-12 laptop:col-span-8 flex flex-col gap-4 min-w-0">
                {primary}
            </div>
            <div class="col-span-12 laptop:col-span-4 flex flex-col gap-4 min-w-0">
                {rail}
            </div>
        </div>
    }
}
