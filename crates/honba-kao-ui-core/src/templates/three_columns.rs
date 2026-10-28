use leptos::prelude::*;

/// `HfThreeColumnTradingLayout` layout template — 3-column pro trading cockpit.
/// TopBar + Left Charts + Center Depth/Tape + Right OrderTicket + Spanning Bottom Dock.
#[component]
pub fn HfThreeColumnTradingLayout(
    #[prop(into, optional)] market_tape: Option<AnyView>,
    top_bar: AnyView,
    visualizer: AnyView,
    market_depth: AnyView,
    order_ticket: AnyView,
    dock: AnyView,
) -> impl IntoView {
    view! {
        <div class="trading-cockpit flex flex-col gap-2 h-full w-full">
            {market_tape.map(|tape| view! { <div class="trading-cockpit__tape w-full">{tape}</div> })}
            <div class="trading-cockpit__top-bar w-full">{top_bar}</div>

            <div class="trading-cockpit__grid grid grid-cols-12 gap-2 flex-grow min-h-0">
                // Visualizer: Chart / Payoff (7 cols on laptop, 8 on desktop)
                <div class="col-span-12 laptop:col-span-6 desktop:col-span-7 flex flex-col min-h-[320px] desktop:min-h-[460px] h-full min-w-0">
                    <div class="visualizer-wrapper flex-grow min-h-0 rounded border border-border-soft bg-surface-base">
                        {visualizer}
                    </div>
                </div>

                // Depth Ladder: L2 Order Book (3 cols on laptop, 2.5 on desktop)
                <div class="col-span-12 laptop:col-span-3 desktop:col-span-2 flex flex-col min-h-[320px] desktop:min-h-[460px] h-full min-w-0">
                    <div class="depth-wrapper flex-grow min-h-0 rounded border border-border-soft bg-surface-base overflow-hidden">
                        {market_depth}
                    </div>
                </div>

                // Order Ticket Panel (3 cols)
                <div class="col-span-12 laptop:col-span-3 desktop:col-span-3 flex flex-col min-h-[320px] desktop:min-h-[460px] h-full min-w-0">
                    <div class="ticket-wrapper flex-grow min-h-0 rounded border border-border-soft bg-surface-base overflow-y-auto">
                        {order_ticket}
                    </div>
                </div>

                // Bottom Dock: Positions, Orders, History (Spans Columns 1 & 2)
                <div class="col-span-12 laptop:col-span-9 desktop:col-span-9 min-h-[200px] desktop:min-h-[260px] rounded border border-border-soft bg-surface-base overflow-hidden min-w-0">
                    {dock}
                </div>
            </div>
        </div>
    }
}
