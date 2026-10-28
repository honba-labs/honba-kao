//! Routing layer and persistent app shell. Mirrors `web/src/router/`.
//!
//! * [`paths`] — single source of truth for route strings (like `paths.ts`).
//! * [`routes`] — the route table (like `AppRoutes.tsx`).
//! * [`app_chrome`] — the shared layout wrapper (like `AppChrome` in `AppRoutes.tsx`).
//! * [`chrome`] — the shell's pieces: header bar, nav rail, footer, status bar.

pub mod app_chrome;
pub mod chrome;
pub mod paths;
pub mod routes;

pub use app_chrome::AppChrome;
pub use paths::{Page, ROUTES, get_current_page_for_path};
pub use routes::AppRoutes;
