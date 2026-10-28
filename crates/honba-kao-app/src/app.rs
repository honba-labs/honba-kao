use crate::router::AppRouter;
use crate::shared::contexts::HonbaClientContext;
use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    // Initialize the headless client context (binds SSE streams + REST
    // snapshots) and provide it to the whole component tree so feature pages
    // consume live data from honba-kao-client instead of dead mock values.
    let client = HonbaClientContext::new();
    provide_context(client);

    view! {
        <AppRouter />
    }
}
