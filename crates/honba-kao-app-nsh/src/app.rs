//! Root `App` component — NSH port per nsh-port.md Phase 5 (Milestone 5.1).
//!
//! Builds the headless client context (binds SSE streams + REST snapshots)
//! and provides it to the whole component tree, then renders the persistent
//! chrome shell. `HonbaClientContext::new()` must be called from here: it
//! calls `signal`/`spawn` internally, both of which need the current
//! `Owner` reactive scope that only exists inside a component body.

use crate::router::chrome::{AppChrome, AppChromeProps};
use crate::shared::contexts::HonbaClientContext;
use nsh_core::prelude::*;

/// Props for [`App`].
#[derive(Debug, Clone, PartialEq, Props)]
pub struct AppProps {}

/// The application root.
#[component]
pub fn App(_props: AppProps, _cx: Scope<NoEvent>) -> impl IntoElement {
    let client = HonbaClientContext::new();
    provide_context(client);

    AppChrome::element(AppChromeProps::default())
}
