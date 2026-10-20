The Intended ui-core
The repository defines a clear primitive layer in crates/honba-kao-app/src/components/ui/:

HfButton — modeled after bfx-hf-ui-core .hfui-button, with variants and sizes.

HfPanel, HfPanelHeader, HfIconButton — modeled after .hfui-panel.

HfModal, HfCard, HfBadge, HfKpi, HfPageHeader, HfPlNumber, HfStatusCircle, etc..

The module comment explicitly states it "mirrors web/src/components/ui/ … and the bfx-hf-ui-core design language". So the architectural intent is unambiguous: all UI should flow through this layer.

🧐 Where the Inconsistency Shows Up
The problem is not that the primitives are missing — it's that they are not used uniformly across the domain components.

✅ Components that do use ui-core
strategy_studio.rs imports HfButton, HfPanel, HfPanelHeader and renders a "New strategy" action through HfButton.

grid_risk_panel.rs uses HfPanel and HfPanelHeader for its container.

palette_studio.rs uses HfModal for the modal shell.

These are good examples of the intended pattern.

❌ Components that bypass ui-core
filter_bar.rs renders a toolbar full of raw <button> elements with custom class names like "sc-filter-bar__market" — no HfButton anywhere. This means the filter bar's buttons will not inherit the HfButton variants, sizes, or future updates.

screener_table.rs contains no HfButton or ui:: imports at all; it builds its interactive elements directly from HTML primitives. The table's header menus, pagination controls, and cell actions are therefore outside the design system.

header_bar.rs renders raw buttons for theme and account controls, using ad‑hoc classes rather than HfButton or HfIconButton.

The result is a split design system: some screens follow the bfx-hf-ui-core language through the Hf* primitives, while others invent their own button, panel, and toolbar styling inline.

🔍 Root Causes
Missing primitives for complex controls. The screener filter bar needs segmented pills, toggle groups, and badge‑augmented buttons. The ui module currently offers only a generic HfButton and HfIconButton — not a segmented control or a toolbar primitive. Rather than extend ui-core, developers reached for raw HTML.

No enforcement mechanism. There is no lint rule, code‑review checklist, or architectural test that forbids raw <button> tags in domain components. Because the ui module lives inside the app crate (honba-kao-app), it is easy to ignore.

Implicit design‑token coupling. The custom classes in filter_bar.rs reference sc-filter-bar__* names, not the 256‑color token palette. When a theme changes, those elements may drift from the rest of the UI.

🛠️ Recommendations: Make ui-core the Only Layer
Your instinct is correct: every component in honba-kao should be composed from a shared ui-core, and new UI needs should be added to ui-core rather than implemented ad‑hoc. Here is how to make that a reality.

1. Extract ui-core into its own crate
Move components/ui into a dedicated workspace crate, e.g. honba-kao-ui-core. Then add a workspace dependency:

toml
[workspace.dependencies]
honba-kao-ui-core = { path = "crates/honba-kao-ui-core" }
Every component crate depends on honba-kao-ui-core, never on a local components/ui folder. This makes the primitive layer explicit, versionable, and reusable across other Honba frontends (if they exist).

2. Audit and consolidate every UI pattern
Walk through auth, charts, common, landing, modals, screener, and strategy, and list every distinct UI pattern: buttons, panels, modals, toolbars, badges, inputs, table cells, tabs, etc.

For each pattern, decide:

Does a primitive already exist in ui-core? → Replace the ad‑hoc implementation with the ui-core primitive.

Does it not exist yet? → Add a new primitive to ui-core, designed generically enough to cover the domain use case.

Examples of primitives that are currently missing and should be added:

HfSegmentedControl — for the asset‑class pills and view toggle in filter_bar.rs.

HfToolbar — a layout primitive for filter bars and action rows.

HfTableAction — a button primitive specifically for data‑table cells, if HfButton is too heavy.

HfToggleGroup — for the market tabs (IN/US) and table/heatmap switch.

3. Enforce usage with linting and review
Clippy lint: Add a custom lint (or a simple grep in CI) that fails if a file under components/ (except ui-core) contains <button or <input tags directly. Allow exceptions only with an explicit // ui-core-exempt: reason comment.

Code review checklist: Require reviewers to reject any PR that introduces new UI styling outside ui-core.

Design‑token audit: Ensure all class names used in ui-core map to the 256‑color token palette. Domain components should never reference raw colors or ad‑hoc class names.

4. Provide a migration path
Refactoring every component at once is risky. Start with high‑traffic areas:

Screener filter bar — replace raw buttons with HfButton and new HfSegmentedControl / HfToolbar primitives.

Header bar — swap the theme and account buttons for HfIconButton.

Screener table — replace cell actions and pagination controls with HfButton variants.

Once these are done, the remaining components will have clear examples to follow.

💎 The Payoff
A strict ui-core layer gives you:

Visual consistency — every button, panel, and modal behaves the same across all screens.

Theme safety — all styling flows from the 256‑color token palette, so theme switching works uniformly.

Easier maintenance — a single change to HfButton propagates everywhere.

Faster onboarding — new contributors compose from a known kit instead of inventing CSS.

Reusability — ui-core can be published as a crate for other Honba frontends or even the nofx reference implementation.

The architecture is already pointing in the right direction; the missing piece is discipline and enforcement. By extracting ui-core into its own crate, adding the missing primitives, and making raw HTML a code‑review violation, honba-kao can move from a “mostly consistent” UI to a truly systematic one.
