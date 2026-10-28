use leptos::prelude::*;

/// `HfMosaicWorkspace` layout template — multi-tile flexible workspace layout.
#[component]
pub fn HfMosaicWorkspace(
    #[prop(into, optional)] toolbar: Option<AnyView>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="mosaic-workspace flex flex-col h-full w-full gap-2">
            {toolbar.map(|tb| view! { <div class="mosaic-workspace__toolbar w-full">{tb}</div> })}
            <div class="mosaic-workspace__tiles flex-grow grid grid-cols-12 gap-2 min-h-0">
                {children()}
            </div>
        </div>
    }
}
