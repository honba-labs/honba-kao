use leptos::prelude::*;

#[component]
pub fn HfTapeRow(
    price: f64,
    size: f64,
    time: String,
    is_buyer_maker: bool,
    #[prop(into, optional)] on_click: Option<Callback<()>>,
) -> impl IntoView {
    let side_class = if is_buyer_maker {
        "text-rose-400"
    } else {
        "text-emerald-400"
    };
    view! {
        <div
            class="hf-tape-row flex items-center justify-between px-2 py-0.5 hover:bg-surface-elevated font-mono text-xs cursor-pointer"
            on:click=move |_| {
                if let Some(cb) = on_click {
                    cb.run(());
                }
            }
        >
            <span class=side_class>{format!("{:.2}", price)}</span>
            <span class="text-text-muted">{format!("{:.4}", size)}</span>
            <span class="text-[10px] text-text-muted">{time}</span>
        </div>
    }
}
