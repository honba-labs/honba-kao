use crate::router::AppRouter;
use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <AppRouter />
    }
}
