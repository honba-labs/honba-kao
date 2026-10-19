//! Routing layer. Mirrors `web/src/router/`.
//!
//! * [`paths`] — single source of truth for route strings (like `paths.ts`).
//! * [`app_routes`] — the route table (like `AppRoutes.tsx`).
//! * [`app_chrome`] — the shared layout wrapper (like `AppChrome` in `AppRoutes.tsx`).

pub mod app_chrome;
pub mod app_routes;
pub mod paths;

pub use app_chrome::AppChrome;
pub use app_routes::AppRoutes;
pub use paths::{Page, ROUTES, get_current_page_for_path};
