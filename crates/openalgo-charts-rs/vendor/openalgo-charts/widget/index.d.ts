import { ShortcutListItem, ChartTheme, SymbolMatch, Chart, SeriesApi, IndicatorInputPresentation, ChartSettingsInput, ChartObjects, AlertController, DataVariant, ChartSettingsValues, TradingCapabilitySource, TradingCapabilityRequest, ContextMenuEvent, IndicatorApi, ChartSettingsTabId, AlertSource, SessionPhase, SessionCalendarSource, ChartEvent, ChartEventDetails, EventMarkerDetails, QuoteFeed, InstrumentKey, NewsFeed, DataLoadingController, RestoreReport, ChartOptions, DataFeed, DataLoadingOptions, ResolvedLinkOptions, LinkOptions, QuoteSnapshot, NewsItem, Bar, IndicatorInputCondition, IndicatorInput, IndicatorSettings } from 'openalgo-charts';
export { SymbolMatch } from 'openalgo-charts';
import { DrawingController, SettingsField, Drawing, MagnetMode, InstrumentDrawings, DrawingDocumentStore } from 'openalgo-charts/draw';
import { DrawingTemplateStore, DrawingTemplateCatalog, DrawingTemplate, DrawingTemplateValues, WorkspaceStore, WorkspaceCatalog, WorkspaceDocument, WorkspacePayload, WatchlistStore, IndicatorTemplateInput, IndicatorTemplateDocument } from 'openalgo-charts/workspace';
import { AccountStateSource } from 'openalgo-charts/trade';

/**
 * The chart grid's layouts: every arrangement `setPreset` and the grid bar's
 * picker offer, from one chart to sixteen.
 *
 * A layout is data, not code: its rows and columns, one slot per chart with
 * the spans the workspace schema already carries, and the track weights it
 * starts with. The grid builds from the slots, the picker draws its tiles
 * from the same slots, and a saved desk records the id, so a layout added
 * here needs no code anywhere else.
 *
 * Every layout stays inside what a workspace document accepts: at most four
 * tracks a side (the document takes eight; a picker tile reads to four) and
 * sixteen charts. Slots are listed in reading order, by row and then column,
 * which is the order `cells()` promises.
 */
/**
 * The uniform layouts, rows by columns: every chart one cell, no spans. The
 * first six are the ones 2.5.9 shipped.
 */
type ChartGridPreset = '1x1' | '1x2' | '1x3' | '2x1' | '3x1' | '2x2' | '1x4' | '4x1' | '2x3' | '3x2' | '2x4' | '4x2' | '3x3' | '3x4' | '4x3' | '4x4';
/**
 * The uneven layouts, named for where the large chart sits and how many
 * charts share the rest: `left-3` is a large chart on the left with three
 * stacked beside it, `corner-5` a large chart in the top left corner with
 * five around it on a three by three grid.
 */
type ChartGridUnevenLayout = 'left-2' | 'right-2' | 'top-2' | 'bottom-2' | 'left-3' | 'top-3' | 'left-4' | 'top-4' | 'corner-5' | 'corner-7';
/** Any layout the grid can take: a uniform preset or an uneven layout. */
type ChartGridLayoutId = ChartGridPreset | ChartGridUnevenLayout;
/** One chart's place in a layout, in the workspace schema's own terms. */
interface ChartGridLayoutSlot {
    readonly row: number;
    readonly column: number;
    readonly rowSpan: number;
    readonly columnSpan: number;
}
interface ChartGridLayoutSpec {
    readonly rows: number;
    readonly columns: number;
    /** One slot per chart, in reading order. */
    readonly slots: readonly ChartGridLayoutSlot[];
    /** The weights the tracks start with; a splitter changes them afterwards. */
    readonly rowWeights: readonly number[];
    readonly columnWeights: readonly number[];
}
/** Rows and columns of each uniform preset. The first six keep their 2.5.9 order. */
declare const CHART_GRID_PRESETS: Readonly<Record<ChartGridPreset, readonly [rows: number, columns: number]>>;
/**
 * Every layout, in the order the grid bar's picker lists them: by the number
 * of charts, and within one count the uniform ones first.
 */
declare const CHART_GRID_LAYOUTS: Readonly<Record<ChartGridLayoutId, ChartGridLayoutSpec>>;

/**
 * The chart grid's own words, as message keys. They join the widget's
 * catalog through `WidgetBuiltinMessage`, so a host translates them with the
 * same `translate` function it gives every chart, and a typed host catalog
 * names them like any other key.
 */

type ChartGridMessage = 'One chart' | 'Two columns' | 'Two rows' | 'Three columns' | 'Three rows' | 'Two by two' | 'Four columns' | 'Four rows' | 'Two rows of three' | 'Three rows of two' | 'Two rows of four' | 'Four rows of two' | 'Three by three' | 'Three rows of four' | 'Four rows of three' | 'Four by four' | 'Large left, two on the right' | 'Large right, two on the left' | 'Large top, two below' | 'Large bottom, two above' | 'Large left, three on the right' | 'Large top, three below' | 'Large left, four on the right' | 'Large top, four below' | 'Large corner, five around' | 'Large corner, seven around' | 'Chart grid' | 'Arrange charts' | 'Arrange charts: {name}' | '1 chart' | '{count} charts' | '{name}, {count} charts' | 'Link: {name}' | 'Maximize the chart' | 'Restore the grid' | 'There is only one chart' | 'The grid shows one chart at a time at this width' | 'Link' | 'Linking' | 'Link group' | 'Not linked' | 'Group {letter}' | 'New group' | 'Rename group' | 'Group name' | 'Save' | 'Cancel' | 'Links in {group}' | 'Crosshair' | 'Time range' | 'Symbol' | 'Interval' | 'Chart type' | 'Appearance' | 'Drawings' | 'Nearest bar' | 'where a chart has no bar at that time' | 'same instrument only' | 'Share this chart\'s drawings' | 'Shared {count} drawing' | 'Shared {count} drawings' | 'No drawing here can be shared' | 'Link this chart to a group first' | 'Switch Drawings on first' | 'Capture every chart' | 'Every chart' | 'Download PNG of every chart' | 'Copy image of every chart' | 'Saved a PNG of every chart' | 'Every chart copied' | 'The image could not be saved: {error}' | 'Show every chart to capture them together' | 'Chart {index}, linked in {name}' | 'Chart {index}, not linked' | 'Linked in {name}' | 'Maximize or restore the chart' | 'Make the chart to the left active' | 'Make the chart to the right active' | 'Make the chart above active' | 'Make the chart below active' | 'Swap with the chart to the left' | 'Swap with the chart to the right' | 'Swap with the chart above' | 'Swap with the chart below';
/** The name each layout shows in the picker and announces. */
declare const CHART_GRID_LAYOUT_NAMES: Readonly<Record<ChartGridLayoutId, ChartGridMessage>>;

/** English source keys keep fallback messages available without a locale bundle. */
type WidgetBuiltinMessage = ChartGridMessage | "Only loaded rows are exported. Blank bounds include all loaded times." | "Study alignment" | "Source rows" | "Displayed rows" | "Use captured visible range" | "All loaded rows" | "Download CSV" | "The From bound must be before or equal to the To bound" | "Enable panning" | "Enable zooming" | "Go to" | "Go" | "Date" | "Range" | "From" | "To" | "Time" | "From time" | "To time" | "Showing {date}" | "Showing {from} to {to}" | "The range is wider than the chart. Showing {from} to {to}" | "History starts at {date}" | "The history limit stops at {date}" | "No older bars were found before {date}" | "Older history cannot load now. Showing from {date}" | "No bars at or after {date}" | "No bars between {from} and {to}" | "Go to needs a time-based interval" | "Enter a valid date" | "Could not load history: {error}" | "The end must not be before the start" | "Loading history" | "Times are in {zone}. A blank time is the start of the day, or its end for To." | "Dates are in {zone}." | "Charts" | "Chart {index}" | "Resize columns {first} and {second}" | "Resize rows {first} and {second}" | "Search" | "Enter a finite number" | "Minimum: {value}" | "Maximum: {value}" | "Use HHMM-HHMM with optional :days (1 to 7)" | "Depends on {inputs}" | "Not used with the current settings" | "Shown: {inputs}" | "Hidden: {inputs}" | "Available: {inputs}" | "Unavailable: {inputs}" | "{label} (UTC seconds)" | "Symbol search is not configured; enter an instrument manually" | "Finish or cancel the active drawing before picking" | "This host cannot suspend the settings dialog" | "Choose an explicit pane and scale for this price input" | "Pick on chart" | "Pick {label} on the chart" | "Pick point on chart" | "Pick {time} and {price} on the chart" | "Cancel pick" | "The value could not be picked" | "Add a study to the chart" | "Add level" | "Added {name}" | "Account" | "Accounts" | "Account: {name}" | "Analyzer" | "Available" | "Could not switch account: {error}" | "Equity" | "Loading account" | "Margin used" | "Stale" | "Alerts" | "Alerts are unavailable in this host" | "Alerts..." | "Active" | "Auto-fit to the data" | "Fit primary prices only" | "Collapse indicator legends" | "Collapse pane" | "Expand pane" | "Move pane up" | "Move pane down" | "at the top" | "at the bottom" | "price pane stays on top" | "Price pane" | "Axis settings..." | "Bar close" | "Bar close evaluates confirmed values. Intrabar touch can fire on a wick that is absent from final history." | "Behind the series" | "Bring to front" | "Buy limit at {price}" | "Buy market" | "Buy stop at {price}" | "C" | "Cancel" | "Candle condition" | "Candle condition is unavailable" | "Capture" | "Capture chart" | "Chart alerts" | "Chart branding" | "Chart controls" | "Chart copied" | "Chart data download started" | "Chart header" | "Chart menu" | "Chart objects" | "Chart settings" | "Chart toolbar" | "Chart type" | "Claimed by {name}" | "Click to switch the magnet off" | "Close" | "Color" | "Compare with" | "Copy drawing" | "Copy failed: {error}" | "Copy image" | "Copy {count} drawings" | "Could not change lock for {name}" | "Could not change visibility for {name}" | "Could not focus {name}" | "Could not load" | "{name} could not load: {error}" | "{name}: {error}" | "Event details" | "Events" | "Loading details..." | "No additional details." | "Unable to load additional details." | "Could not load older history" | "Could not load {symbol} {interval}" | "Could not load {symbol} {interval}: {error}" | "Could not open settings for {name}" | "Could not remove {name}" | "Could not save this alert" | "Could not select {name}" | "Create alert" | "Create alert at {price}..." | "Create drawing alert..." | "Create study alert..." | "Cursor" | "Custom" | "Cut drawing" | "Cut {count} drawings" | "Data export failed: {error}" | "Default level" | "Defaults" | "Delete" | "Delete drawing" | "Delete {count} drawings" | "Disable" | "Disable {name}" | "Disabled" | "Dismiss" | "Done" | "Download PNG" | "Download SVG" | "Download chart data (CSV)" | "Draw" | "Drawing" | "Drawing actions" | "Drawing level is unavailable" | "Drawing text" | "Drawing tools" | "Duplicate" | "Edit" | "Edit alert" | "Edit levels..." | "Edit text" | "Edit the expiry before enabling this alert" | "Enable" | "Enable {name}" | "Enabled" | "Enter a valid expiry date and time in UTC" | "Dark" | "Light" | "Enter an expiry date and time in UTC" | "Expires {time} UTC" | "Last fired {time} UTC" | "Symbol search results" | "That side is already in use" | "other side taken" | "Unlock drawing" | "Show drawing" | "Dark theme" | "Light theme" | "Stay: {mode}" | "Edit on chart" | "History is stale for {symbol} {interval}. Reload to retry." | "Magnet weak: snaps when O/H/L/C is within a few pixels" | "Magnet strong: every anchor lands on the nearest O/H/L/C" | "Click for weak: snaps when O/H/L/C is within a few pixels" | "Click for strong: every anchor lands on the nearest O/H/L/C" | "Ratio" | "Remove level" | "From (UTC seconds)" | "To (UTC seconds)" | "Enter finite UTC seconds or leave the bound blank" | "Enter an expiry date and time" | "Every match" | "Expired" | "Expires {time}" | "Expires" | "Finish" | "Fit all bars" | "Focus" | "Focus {name}" | "H" | "Hidden" | "Hide" | "Hide drawing" | "Hide drawing tools" | "Hide {name}" | "History is stale for {symbol} {interval}" | "History is stale. Refreshing {symbol} {interval}" | "History retention limit reached" | "In front of the series" | "Indicators" | "Indicators..." | "Input plot" | "Inputs" | "Interval" | "Interval {code}" | "Intrabar touch" | "Invert" | "Keep tool active" | "Keyboard shortcuts" | "L" | "Label" | "Last fired {time}" | "Level" | "Levels" | "Loading" | "Loading older history" | "Loading {symbol} {interval}" | "Regular hours" | "Extended hours" | "Adjusted prices" | "Raw prices" | "Not available from this source: {variant}" | "Lock" | "Lock drawing" | "Lock {name}" | "Locked" | "Magnet off" | "Magnet: {mode}" | "More" | "Move the scale closer to the plot" | "Move the scale further from the plot" | "Move the scale to the left" | "Move the scale to the right" | "No alerts. Create an alert for this chart." | "No bars for {symbol} {interval}" | "No data" | "No drawings to remove" | "No indicator on the chart to configure" | "No indicator to configure" | "No indicators are registered. Import the indicators tier to fill this list." | "No levels. Add one, or reset to the defaults." | "No match" | "No objects match your search." | "No objects on this chart." | "Nothing measured on this scale yet" | "Nothing to configure on this chart yet." | "Nothing to paste" | "O" | "OI" | "OK" | "Objects" | "Objects panel requires an object model" | "Off: one drawing per pick" | "On: the tool stays active after each drawing" | "Once" | "One drawing per pick" | "Open interest is unavailable for this instrument." | "Order" | "Order entry is locked by the host" | "Order entry is locked during replay" | "Order entry is unavailable" | "Order entry is unavailable: {reason}" | "Other" | "PNG, SVG, CSV or the clipboard" | "Pane {number}" | "Paste" | "Pick an indicator from the legend first" | "Pin price per bar" | "Pin to rail" | "Pinned. Right-click to unpin" | "Plot" | "Price" | "Price chart" | "Price {price}" | "Price {price} to {upper}" | "Properties" | "Properties of the selection..." | "Properties..." | "Redo" | "That step could not be undone: {error}" | "That step could not be redone: {error}" | "Refreshing {symbol} {interval}" | "Remove" | "Remove all drawings ({count})" | "Remove {name}" | "Removed {count} drawing" | "Removed {count} drawings" | "Reset" | "Restore defaults" | "Restore this tab" | "Retry" | "Retry chart data" | "Retry older history" | "Retry {name}" | "Right-click to remove all" | "Save" | "Saved a PNG of the chart" | "Saved an SVG of the chart" | "Scale" | "Search indicators" | "Search name, type or pane" | "Search objects" | "Searching" | "Select a drawing first" | "Select a drawing with levels first" | "Select a text drawing first" | "Select all ({count})" | "Select an input plot on the drawing pane" | "Select one first. Right-click to remove all" | "Select {name}" | "Selected" | "Selected drawing" | "Sell limit at {price}" | "Sell market" | "Sell stop at {price}" | "Send to back" | "Settings" | "Settings for {name}" | "Settings..." | "Show" | "Show drawing tools" | "Show labels" | "Show {name}" | "Source" | "Stays selected, so the eye brings it back" | "Studies" | "Study" | "Study instance is unavailable" | "Study plot is unavailable" | "Style" | "Switch to the dark theme" | "Switch to the light theme" | "Symbol" | "Symbols" | "Text" | "That indicator is no longer on the chart" | "That text is off the chart" | "The anchor changes only while the drawing's pane is on screen" | "The chart takes the space. The tab at its bottom left brings them back" | "The canvas produced no image" | "The chart changed; reopen Capture for its current source" | "The chart changed; reopen the menu before placing an order" | "The chart layout could not be saved" | "The chart layout could not be saved: {error}" | "The drawings for {instrument} could not be saved" | "The current plot value is unavailable. The alert waits for observed values." | "The indicator picker is not in this build" | "The instrument context changed. Reopen the editor for the intended instrument." | "The saved layout could not be restored: {error}" | "Saved chart settings could not be read, so changes are kept for this session only: {error}" | "Saved chart settings could not be written: {error}" | "The scale could not be moved" | "The settings dialog is not in this build" | "The study settings could not be applied" | "Theme" | "These drawings share no settings." | "This alert was removed" | "This runtime cannot save files" | "Tools" | "Tools stay active after each drawing" | "Trade" | "Triggered" | "Unavailable candle condition" | "Unavailable drawing" | "Unavailable study" | "Unavailable study output: {instanceId} / {plotKey}" | "Undo" | "Unlock" | "Unlock {name}" | "Unlocked" | "Unpin from rail" | "Unsupported" | "Visible" | "Vol" | "Wait for this chart to finish loading its data" | "locked" | "needs https or localhost" | "nothing measured" | "nothing on this side" | "object" | "off" | "on" | "on chart" | "on chart x{count}" | "one instance on the chart" | "paste it anywhere" | "protected" | "read-only" | "text stays text" | "{count} alert" | "{count} alerts" | "{count} bar" | "{count} bars" | "{count} drawings" | "{count} instances on the chart" | "{count} object" | "{count} objects" | "{group}: chevron for the rest" | "{group}: chevron for the rest. Double-click keeps it active" | "{name} / {plot}: {value}" | "{name} / {plot}: {value} to {upper}" | "{name} has nothing to configure" | "{name} settings" | "{name} settings are protected" | "{name} settings..." | "{name} stays active until Escape" | "{seconds}s cooldown" | "Drawing templates could not be loaded" | "The drawing template could not be saved: {error}" | "New {tool} drawings start with this look" | "New {tool} drawings start with the standard look" | "Saved the template {name}" | "Removed the template" | "Templates" | "Save as default for this tool" | "one tool at a time" | "Forget the default for this tool" | "Save as template..." | "Apply {name}" | "Save as template" | "Template name" | "Enter a name" | "Drawing toolbar" | "More drawing actions" | "{value} px" | "Line width" | "Line width: {value}" | "Line style" | "Line style: {style}" | "Dashed" | "Dotted" | "Solid" | "Color: {value}" | "mixed" | "Mixed" | "Enter a date and a time on the chart clock" | "Enter a price as a number" | "A logarithmic scale needs a price above zero" | "Point {number}" | "{field} of point {number}" | "Pinned to the screen: its anchors are not a time and a price." | "A freehand stroke has {count} points. Move it on the chart." | "Coordinates" | "Times are on the chart clock, {zone}." | "{shown} of {count} object" | "{shown} of {count} objects" | "{count} chart shortcut struck through: another shortcut uses the same chord here and takes precedence." | "{count} chart shortcuts struck through: another shortcut uses the same chord here and takes precedence." | "Change" | "Change {name}" | "Cancel changing {name}" | "Reset {name} to {chord}" | "Reset all" | "Put every shortcut back to its default?" | "Every shortcut is back to its default." | "Press keys" | "Press the new shortcut for {name}. Esc cancels." | "Not set" | "Replace" | "{chord} is used by {names}." | "Taken from {names}." | "{name} is now {chord}." | "{name} is back to {chord}." | "{chord} belongs to {names}, which cannot change. Press another." | "{chord} belongs to the browser. Press another." | "{chord} types into the chart. Add {mod} or {alt}." | "{chord} presses the focused control. Add {mod} or {alt}." | "That key cannot be used here. Press another." | "Nothing changed." | "unknown error" | "{name}, hidden" | "Level {level} enabled" | "Level {level} ratio" | "Level {level} color" | "Level {level} label" | "Remove level {level}" | "Search {label}" | "{name} (pinned)" | "Cancel order" | "Close position" | "Keep tool active: {mode}" | "Delete {name}" | "Magnet weak: snaps when a bar or study value is within a few pixels" | "Magnet strong: every anchor lands on the nearest bar or study value" | "Click for weak: snaps when a bar or study value is within a few pixels" | "Click for strong: every anchor lands on the nearest bar or study value" | "Armed" | "Keep tool armed" | "On: the tool stays armed after each drawing" | "Tools stay armed after each drawing" | "{group}: chevron for the rest. Double-click keeps it armed" | "{name} stays armed until Escape" | "{count} chart shortcut struck through: the same chord arms a drawing tool here and takes precedence." | "{count} chart shortcuts struck through: the same chord arms a drawing tool here and takes precedence.";
/**
 * Every key a translator is asked for. Widget text is keyed three ways, and
 * no published key is ever renamed:
 *
 * - The English source in `WidgetBuiltinMessage`, typed, for new chrome text.
 * - `schema.*`: descriptor metadata (a settings field's label, a study input,
 *   a drawing tool's name, a chart type), and under `schema.ui.<area>.<key>`
 *   chrome keyed by where it sits (the bottom bar, the watchlist, the symbol
 *   picker), with its English passed as the fallback. A surface already keyed
 *   there keeps adding its words there, so its catalog entries stay together.
 * - A feature's own typed union joined into the English one
 *   (`ChartGridMessage`), for a feature that keeps its words beside it.
 *
 * None of them carries saved user values: symbols, names and user text stay literal.
 */
type WidgetMessageKey = WidgetBuiltinMessage | `schema.${string}`;
type WidgetMessageValues = Readonly<Record<string, string | number>>;
/** A synchronous host catalog. Missing entries return undefined or null. */
type WidgetTranslator = (key: WidgetMessageKey, fallback: string, values: WidgetMessageValues) => string | null | undefined;
interface WidgetTranslationOptions {
    /** Translates widget-owned text. Omit to preserve English rendering. */
    translate?: WidgetTranslator | undefined;
}
/** Required interpolation names are inferred from the English source message. */
type WidgetMessageParameters<Message extends string> = Message extends `${string}{${infer Name}}${infer Rest}` ? Readonly<Record<Name, string | number> & WidgetMessageParameters<Rest>> : Readonly<Record<never, never>>;
/** Resolve a message, then interpolate literal values once. Output is plain text. */
declare function widgetText<K extends WidgetBuiltinMessage>(context: WidgetTranslationOptions, key: K, ...args: [keyof WidgetMessageParameters<K>] extends [never] ? [values?: WidgetMessageValues] : [values: WidgetMessageParameters<K>]): string;
declare function widgetText(context: WidgetTranslationOptions, key: `schema.${string}`, values: WidgetMessageValues, fallback: string): string;

/**
 * Where a binding applies. `global` always; `widget` while the pointer or the
 * focus is inside the widget; `chart` while either is on the chart itself;
 * `rail` while the focus is in the tool rail; `overlay` while a dialog or a
 * menu is open, when nothing else fires. The shell decides which are active
 * (see `setScopes`); they are tried narrowest first.
 */
type KeyScope = 'global' | 'widget' | 'chart' | 'rail' | 'overlay' | (string & {});
/** The event fields the keymap reads. A DOM `KeyboardEvent` satisfies it. */
interface KeyEventLike {
    key: string;
    code?: string;
    ctrlKey?: boolean;
    metaKey?: boolean;
    altKey?: boolean;
    shiftKey?: boolean;
    target?: unknown;
    preventDefault?(): void;
    stopPropagation?(): void;
    /** Used while a chord is being captured, so no other listener on the document sees the press. */
    stopImmediatePropagation?(): void;
}
/**
 * What runs for a chord. Return `false` to decline: the next binding on the
 * same chord (or the engine, or the browser) gets the key. Anything else
 * claims it, and the event is prevented and stopped.
 */
type KeyAction = (e: KeyEventLike) => boolean | void;
interface KeyBindingOptions {
    /** Shown in the shortcuts panel. Defaults to the chord itself. */
    label?: string;
    /** Section of the shortcuts panel. Default `Widget`. */
    group?: string;
    /** Gate read at key time; a false skips the binding without declining for others. */
    when?: (() => boolean) | undefined;
    /** Fire even when the focus is in a text field. Default false. */
    inText?: boolean;
    /** Keep out of the shortcuts panel (a binding that only exists to layer under another). */
    hidden?: boolean | undefined;
    /**
     * Declares that this binding shares its chord on purpose: it declines when
     * it does not apply, so an earlier widget binding or the engine's own
     * command on the same chord still fires. Without it a second registration
     * on a chord, or one on a chord the engine binds, is recorded as a conflict.
     */
    layered?: boolean | undefined;
    /**
     * The stable name of what the binding does (`undo`, `tool:trend-line`),
     * unique among live bindings. `rebind`, `reset` and a saved override name
     * the binding by it. A binding without one cannot be rebound. The `chart:`
     * prefix is kept for the engine's commands.
     */
    command?: string;
    /**
     * Whether a user may move the binding to another chord. Default: true when
     * it has a `command`. False keeps a key that a convention or a neighbouring
     * control depends on where it is: Escape, Enter, the arrows.
     */
    rebindable?: boolean;
}
interface KeyBinding {
    /** Registration number, in the order bindings were made. */
    readonly id: number;
    /** Canonical chord in force (see `parseKeyCombo`): the default, or the user's. `''` while unbound. */
    readonly combo: string;
    /** The chord it was registered with, which `reset` returns it to. */
    readonly defaultCombo: string;
    /** Its stable name, or null for a binding registered without one. */
    readonly command: string | null;
    /** Whether `rebind` may move it. */
    readonly rebindable: boolean;
    readonly scope: KeyScope;
    readonly label: string;
    readonly group: string;
    readonly hidden: boolean;
    readonly inText: boolean;
    /** Shares its chord deliberately; see `KeyBindingOptions.layered`. */
    readonly layered: boolean;
    readonly action: KeyAction;
    readonly when?: (() => boolean) | undefined;
}
interface KeyConflict {
    readonly combo: string;
    readonly scope: KeyScope;
    /** Label of the binding that fires. */
    readonly kept: string;
    /** Label of the binding that never will while the kept one claims. */
    readonly shadowed: string;
    /** `widget` for two widget bindings, `chart` when the engine's own keymap loses the chord. */
    readonly source: 'widget' | 'chart';
}
/** A binding that holds a chord, as a rebind or `conflictsFor` reports it. */
interface KeyChordUse {
    /** Its command (`chart:<command>` for the engine's), or null for a binding registered without one. */
    readonly command: string | null;
    readonly label: string;
    readonly group: string;
    readonly source: 'widget' | 'chart';
    /** False for a fixed binding, which no rebind can take the chord from. */
    readonly rebindable: boolean;
}
/** What `rebind` or `reset` did. */
interface KeyRebindResult {
    /** True when the command now has the chord asked for. */
    readonly ok: boolean;
    /**
     * Why it did not: the command is `unknown` or `fixed`, the chord is
     * `invalid` or `reserved` by the browser, or it is `taken` by the bindings
     * in `conflicts`. Absent on success.
     */
    readonly reason?: 'unknown' | 'fixed' | 'invalid' | 'reserved' | 'taken';
    /** The bindings on the chord: what stopped the change, or, with `replace`, what gave the chord up. */
    readonly conflicts: readonly KeyChordUse[];
}
/**
 * The user's chords by command, JSON-safe, as `overrides()` returns them and
 * `applyOverrides` takes them. A widget command maps to one chord in the key
 * grammar (`Alt+y`); an engine command (`chart:zoomIn`) to one chord or
 * several in the engine's code grammar (`Alt+KeyZ`). Null unbinds.
 */
type KeymapOverrides = Readonly<Record<string, string | readonly string[] | null>>;
/** What `onChange` reports: the commands whose chord moved. */
interface KeymapChange {
    readonly commands: readonly string[];
}
/** Where the widget keeps the user's chords in its storage. */
declare const KEYMAP_KEY = "keymap";
/**
 * Canonical form of a chord (`'shift+alt+t'` -> `'Alt+Shift+t'`), or `''`
 * when the spec is not one. Letters are lower-cased and keep Shift; a symbol
 * or a digit drops it, because the character already says which key was
 * pressed with Shift held. Ctrl, Cmd and Meta all read as `Mod`.
 */
declare function parseKeyCombo(spec: string): string;
/** The canonical chord an event stands for, or `''` for a bare modifier press. */
declare function eventKeyCombo(e: KeyEventLike): string;
/** A chord as a user reads it: `Ctrl+Shift+Z`, or `Cmd+Shift+Z` on a Mac. */
declare function formatKeyCombo(combo: string, isMac?: boolean): string;
/**
 * The engine's code-based combo (`Alt+KeyV`, `Mod+Shift+KeyS`, `Equal`) in
 * this keymap's key-based form, so the two can be compared. Layout-specific
 * symbols are read as a US layout would produce them, which is what the
 * engine's own display does too.
 */
declare function fromChartCombo(combo: string): string;
/** The slice of the engine's shortcut manager the keymap reads, and drives when the user rebinds. */
interface ChartShortcutSource {
    list(): ShortcutListItem[];
    /** With `disable` and `resetBinding`, what makes the chart's commands rebindable from the keymap. */
    setBinding?(command: string, combo: string | string[]): boolean;
    disable?(command: string): void;
    resetBinding?(command: string): void;
}
interface KeymapOptions {
    isMac?: boolean;
    /** The chart's shortcut manager, for conflict detection, the panel's Chart section and rebinding its commands. */
    chart?: ChartShortcutSource | null;
    /** Which scopes are active right now, narrowest first. Default: `['global']`. */
    scopes?: () => readonly KeyScope[];
}
/** One row of the shortcuts panel. */
interface KeymapRow {
    label: string;
    /** The chord in force, key form; `''` while unbound. */
    combo: string;
    display: string;
    /** Label of the widget binding that takes this chart chord first. */
    shadowedBy?: string;
    /** The command a rebind names; absent for a binding registered without one. */
    command?: string;
    /** Whether a user may move it. Set with `command`. */
    rebindable?: boolean;
    /** The chord it resets to, key form. Set with `command`. */
    defaultCombo?: string;
    /** True when it differs from its default, so there is something to reset. */
    changed?: boolean;
}
interface KeymapGroup {
    readonly group: string;
    readonly rows: ReadonlyArray<KeymapRow>;
}
declare class Keymap {
    private readonly _isMac;
    private readonly _chart;
    private _scopes;
    private readonly _bindings;
    private readonly _byCombo;
    private readonly _conflictListeners;
    private readonly _changeListeners;
    /** The user's widget chords by command, kept for a command that registers later too. */
    private readonly _overrides;
    /** The user's engine chords by engine command, in the engine's grammar. */
    private readonly _chartOverrides;
    /** What each engine command had before the user touched it: a host's own bindings count as its default. */
    private _chartBaseline;
    private _capture;
    private _detach;
    /** The node `attach` listens on, which is what makes another keymap a neighbour. */
    private _target;
    private _nextId;
    constructor(opts?: KeymapOptions);
    get isMac(): boolean;
    /** Replace the scope resolver. */
    setScopes(fn: () => readonly KeyScope[]): void;
    activeScopes(): readonly KeyScope[];
    /**
     * Bind a chord. Returns a disposer. Throws on a chord the grammar cannot
     * read, or a command another live binding has, because a binding that can
     * never fire or never be told apart is a defect at the call site, not at
     * key time. A saved override for the command applies at once.
     */
    register(binding: string, action: KeyAction, scope?: KeyScope, opts?: KeyBindingOptions): () => void;
    private _index;
    private _unindex;
    private _unregister;
    /** Widget against widget: a non-layered binding with an earlier one in its scope on its chord. */
    private _widgetConflicts;
    /** The collisions in force, as keys; null when nobody listens for new ones. */
    private _conflictKeys;
    /** Tell the conflict listeners about every collision that was not there before. */
    private _announce;
    /** Be told when a registration collides with an earlier one. */
    onConflict(fn: (c: KeyConflict) => void): () => void;
    /** Be told when the user's chords change: a rebind, a reset, or overrides applied. */
    onChange(fn: (e: KeymapChange) => void): () => void;
    private _emit;
    /**
     * Hand the next key presses to `fn` ahead of every binding, for a control
     * that records a chord. Each press is prevented and stopped outright, so no
     * binding, no open dialog's Escape and no chart shortcut sees it, and
     * another keymap attached to the same node claims nothing meanwhile; `fn`
     * returns false to let one through (Tab moving the focus). Returns the
     * release. A second capture replaces the first.
     */
    capture(fn: KeyAction): () => void;
    /** Whether a control is recording a chord right now. */
    get capturing(): boolean;
    /**
     * Resolve and run the binding for an event. True when a binding claimed it,
     * in which case the event has been prevented and stopped.
     */
    handle(e: KeyEventLike): boolean;
    /**
     * Listen on `target` in the capture phase, so a claimed chord never reaches
     * the engine's own listener on the document. Returns the detacher; a second
     * call replaces the first.
     */
    attach(target: {
        addEventListener(t: string, fn: EventListener, opts?: boolean): void;
        removeEventListener(t: string, fn: EventListener, opts?: boolean): void;
    }): () => void;
    list(): readonly KeyBinding[];
    /** Every collision in force: widget against widget, and widget against the engine's keymap. */
    conflicts(): readonly KeyConflict[];
    format(combo: string): string;
    private _use;
    private _chartEditable;
    private _chartUse;
    /** Who holds a key-form chord where a binding in `scope` would compete for it. */
    private _holders;
    /** Who holds any of these engine chords: another engine command, or a widget binding that claims first. */
    private _chartHolders;
    /**
     * The bindings that would compete with `combo` (key form) for a binding in
     * `scope`: widget bindings on it in that scope or one live at the same time
     * (`global`, `widget` and `chart` nest), and the engine's commands on it
     * unless the scope is `rail` or `overlay`. An unreadable chord has none.
     */
    conflictsFor(combo: string, scope?: KeyScope): KeyChordUse[];
    /**
     * The chord a command answers to now, in key form: `''` while it is
     * unbound, null for a command nothing registered. An engine command
     * (`chart:<command>`) reports its first chord.
     */
    chord(command: string): string | null;
    private _setWidget;
    private _chartNow;
    /**
     * What an engine command had before the keymap first changed it, recorded
     * at that moment: a host's own binding is its default, not the engine's.
     */
    private _base;
    private _setChart;
    /** An engine command that gave chords up to a replace keeps the rest, or is unbound. */
    private _dropChart;
    /** A widget binding that gave its chord up to a replace is left unbound. */
    private _dropWidget;
    /**
     * Move a command to another chord, or unbind it with null. A widget command
     * takes one chord in the key grammar; an engine command (`chart:<command>`)
     * one or several in the engine's code grammar, as `eventToCombo` gives
     * them. A chord the browser keeps is refused, and so is one another binding
     * holds, unless `replace` is set and every holder is rebindable: each then
     * gives the chord up and is left without it.
     */
    rebind(command: string, combo: string | readonly string[] | null, opts?: {
        replace?: boolean;
    }): KeyRebindResult;
    /**
     * Put a command back on its default chord. The default may have been taken
     * meanwhile by a binding of the same kind (widget or chart); that is refused
     * as `taken` unless `replace` is set, as with `rebind`. A command nothing
     * registered yet just forgets its saved chord.
     */
    reset(command: string, opts?: {
        replace?: boolean;
    }): KeyRebindResult;
    /** Every command back on its default chord, including those not registered yet. */
    resetAll(): void;
    private _clearAll;
    private _rebind;
    private _rebindChart;
    /** The user's chords, JSON-safe: what a host saves and hands back to `applyOverrides`. */
    overrides(): Record<string, string | string[] | null>;
    /**
     * Make `overrides` the user's chords: every command it leaves out goes back
     * to its default, and each one it names moves (null unbinds). Written for a
     * saved record, so it reads anything: an entry that does not parse, is
     * reserved, or names a fixed binding is dropped, and one for a command
     * nothing registered yet is kept for when it does. Collisions a saved chord
     * makes are reported through `onConflict`, as a registration's are.
     */
    applyOverrides(overrides: unknown): void;
    /** The chord an engine command resets to, key form. */
    private _chartDefault;
    /**
     * The bindings as the shortcuts panel shows them: the widget's own groups,
     * then the engine's chart commands, with any chord the widget claims first
     * marked as shadowed. An engine command the user unbound stays listed, so
     * it can be reset.
     */
    describe(): KeymapGroup[];
    destroy(): void;
}
interface ShortcutsPanelOptions {
    /**
     * Rows a user may change carry Change and Reset, and the panel a Reset all.
     * Default true, unless the widget was built with `shortcutsEditor: false`;
     * false lists the chords only.
     */
    edit?: boolean;
}
/**
 * The shortcuts panel: every group from `keymap.describe()`, two columns,
 * closed by Escape or its button, with the editing controls unless `edit` is
 * false. Returns the closer. The panel loads on first use (since 2.5.10), so
 * the first one opens once it has arrived, unless the closer ran or the user
 * moved on before then (lazy.ts); a panel that cannot load says so in a toast.
 */
declare function openShortcutsPanel(ctx: WidgetContext, opts?: ShortcutsPanelOptions): () => void;

type ToastKind = 'info' | 'success' | 'error';
/** How long each kind stays, in ms. 0 keeps it until it is dismissed. */
declare const TOAST_MS: Readonly<Record<ToastKind, number>>;
/** Newest at the bottom; beyond this many the oldest goes. */
declare const TOAST_MAX = 5;
/** How long the leave transition runs before the node is removed. */
declare const TOAST_LEAVE_MS = 160;
interface ToastOptions {
    /** Overrides the kind's stay. 0 keeps it up. */
    ms?: number;
}
interface ToastHandle {
    readonly node: HTMLElement;
    dismiss(): void;
}
interface Toaster {
    toast(message: string, kind?: ToastKind, opts?: ToastOptions): ToastHandle;
    /** Take every toast down now. */
    clear(): void;
    /** Number of toasts currently showing. */
    count(): number;
    destroy(): void;
}
/**
 * Mount the toast stack into `host` (an empty element the shell positions
 * over the chart). `doc` is the host's document unless given, for a fake DOM.
 */
declare function mountToasts(host: HTMLElement, doc?: Document, translation?: WidgetTranslationOptions): Toaster;

/**
 * Design tokens for the widget chrome, as CSS custom properties derived from
 * the chart theme in force.
 *
 * The engine's `ChartTheme` names a dozen colours for the canvas (background,
 * grid, axis text, the candle pair, the line colour). The rail, the top bar
 * and every dialog need perhaps thirty: panel and raised-panel greys, a soft
 * and a firm border, three text weights, an accent and its pressed tint. None
 * of those is worth a second palette a host has to keep in step with the
 * first, so every one is computed here from the theme, and switching the
 * chart's theme restyles the chrome in the same call.
 *
 * Pure: strings in, strings out. `applyTokens` is the one function that
 * touches an element, and it only writes `style.setProperty`.
 */

/** Every custom property carries this prefix, so a host stylesheet cannot collide with one. */
declare const TOKEN_PREFIX = "--oac-";
type WidgetThemeName = 'dark' | 'light';
/** Property name (with the prefix) to value. */
type WidgetTokens = Readonly<Record<string, string>>;
interface Rgba {
    r: number;
    g: number;
    b: number;
    a: number;
}
/**
 * Parse a CSS colour in the forms a theme uses: `#rgb`, `#rgba`, `#rrggbb`,
 * `#rrggbbaa`, `rgb()` and `rgba()`. Anything else (a named colour, `hsl()`)
 * is returned as null, and the caller falls back to a sensible neutral rather
 * than throwing: a theme with one exotic colour should still get chrome.
 */
declare function parseColor(input: string): Rgba | null;
/** Serialise: opaque colours as `#rrggbb`, translucent ones as `rgba()`. */
declare function formatColor(c: Rgba): string;
/**
 * Relative luminance, 0 (black) to 1 (white), on the sRGB curve. The canvas
 * helpers' arithmetic read through this module's parser, so a colour only the
 * widget can parse (space-separated `rgb()`) still gets a real reading.
 */
declare function luminance(color: string): number;
/**
 * The WCAG contrast ratio of two colours, 1 (none) to 21 (black on white).
 * Alpha is ignored: a token is read as the colour it is painted in.
 */
declare function contrastRatio(a: string, b: string): number;
/** The WCAG AA minimum for body-size text, which every chrome label is. */
declare const TEXT_CONTRAST = 4.5;
/**
 * `color` as text on every one of `surfaces`: unchanged when it already reads
 * at `min` against each, else stepped toward `pole` by the least amount that
 * does. The least, so a dim role stays as dim as the rule allows and keeps
 * its place below the brighter ones. A pole that cannot reach `min` gives
 * the pole itself, the most any colour could do on those surfaces.
 */
declare function readableOn(color: string, surfaces: readonly string[], pole: string, min?: number): string;
/** Which of the two modes a theme is, judged from its background. */
declare function themeMode(theme: Pick<ChartTheme, 'background'>): WidgetThemeName;
/** Linear blend of `a` toward `b` by `t` (0 keeps `a`, 1 gives `b`). Alpha blends too. */
declare function mix(a: string, b: string, t: number): string;
/**
 * The colour with its alpha replaced, written as a token value: `#rrggbb` when
 * opaque, otherwise `rgba()` with the alpha clamped and rounded. Deliberately
 * not the base package's `withAlpha`, which feeds canvas and always writes
 * `rgba()` with the alpha as given; callers of each rely on its output.
 */
declare function withAlpha(color: string, alpha: number): string;
/** The system UI stack every piece of chrome sets in text. */
declare const WIDGET_FONT = "ui-sans-serif, system-ui, -apple-system, \"Segoe UI\", Roboto, sans-serif";
declare const WIDGET_MONO = "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace";
/** Layout measurements, in CSS pixels, shared by the stylesheet and the placement maths. */
declare const RAIL_WIDTH = 42;
declare const TOPBAR_HEIGHT = 40;
declare const STATUSLINE_HEIGHT = 24;
/**
 * Compute the token set for a theme.
 *
 * Every grey is the background stepped toward the opposite pole (white on a
 * dark theme, near-black on a light one), so a host theme with a blue-black
 * background gets blue-black panels rather than a neutral grey pasted onto
 * it. The accent is the theme's line colour, the up and down pair are the
 * candle colours, and text comes from the axis text colour lifted for
 * legibility: the axis is deliberately muted on the canvas, and a panel label
 * at that weight reads as disabled.
 */
declare function widgetTokens(theme: ChartTheme, mode?: WidgetThemeName): WidgetTokens;
/** Write a token set onto an element as inline custom properties. */
declare function applyTokens(el: HTMLElement, tokens: WidgetTokens): void;
/** `var(--oac-name)`, for code that builds inline style values from a token. */
declare const token: (name: string) => string;

/**
 * A host's lookup. The original query-only callback remains valid; `signal`
 * aborts once a newer query or a closed picker makes the answer stale.
 */
type SymbolSearch = (query: string, context?: {
    signal: AbortSignal;
}) => Promise<readonly SymbolMatch[]> | readonly SymbolMatch[];
declare const SEARCH_DEBOUNCE_MS = 150;
interface SymbolPickerOptions {
    search: SymbolSearch;
    onSelect(symbol: string, exchange?: string): void;
    /** Changes to this key invalidate answers for the previous chart context. */
    context?: () => unknown;
    variant?: 'desktop' | 'mobile';
}
interface SymbolPickerHandle {
    open(query?: string): void;
    /** Raw entry may be committed only after a search has finished without matches. */
    canCommitRaw(): boolean;
    close(): void;
    destroy(): void;
}
/** Accept secure absolute URLs and root-relative host assets only. */
declare function safeSymbolIconUrl(value: string | undefined): string | null;
/** Attach the same result controller to either widget input or a host input. */
declare function mountSymbolPicker(ctx: WidgetContext, input: HTMLInputElement, options: SymbolPickerOptions): SymbolPickerHandle;
/** Rules to add to the widget's shared stylesheet or a custom host stylesheet. */
declare const SYMBOL_PICKER_CSS = "\n.oac-widget .oac-symbol-picker{min-width:250px;max-width:min(420px,calc(100vw - 24px))}\n.oac-widget .oac-symbol-picker__categories{display:flex;gap:4px;overflow-x:auto;padding:5px 7px;scrollbar-color:var(--oac-sb-thumb) var(--oac-panel);scrollbar-width:thin}\n.oac-widget .oac-symbol-picker__categories::-webkit-scrollbar{height:6px}\n.oac-widget .oac-symbol-picker__categories::-webkit-scrollbar-track{background:var(--oac-panel)}\n.oac-widget .oac-symbol-picker__categories::-webkit-scrollbar-thumb{background:var(--oac-sb-thumb);border-radius:3px}\n.oac-widget .oac-symbol-picker__categories::-webkit-scrollbar-thumb:hover{background:var(--oac-sb-thumb-hover)}\n.oac-widget .oac-symbol-picker__categories:empty{display:none}\n.oac-widget .oac-symbol-picker__category,.oac-widget .oac-symbol-picker__back{background:transparent;border:0;color:inherit;cursor:pointer;padding:4px 7px;font:inherit}\n.oac-widget .oac-symbol-picker__category[aria-pressed=true]{background:var(--oac-elev);border-radius:4px}\n.oac-widget .oac-symbol-picker__row{display:flex;align-items:center;gap:7px;width:100%;text-align:left}\n.oac-widget .oac-symbol-picker__icon{width:18px;height:18px;object-fit:contain}\n.oac-widget .oac-symbol-picker__group{margin-left:auto;opacity:.7}\n.oac-widget .oac-symbol-picker__empty,.oac-widget .oac-symbol-picker__status{padding:9px;color:var(--oac-mut)}\n";

/**
 * The chart history's captures and what differs between two of them
 * (history.ts): the snapshot a step holds of the chart, the field-by-field
 * diff that scopes what a press writes, and the entries the timeline keeps.
 * Pure and stateless, like the value helpers in history-values.ts, so they
 * live apart from the class that records and walks the steps.
 *
 * The two public types the model names, `ChartHistoryChange` and
 * `ChartHistoryCommand`, are declared here so this module reads nothing back
 * from history.ts; history.ts re-exports them, and the tier entry takes them
 * from there as before.
 */

/** What a step changes, for a label or a test. */
type ChartHistoryChange = 'study-add' | 'study-remove' | 'study-settings' | 'study-visibility' | 'study-scale' | 'study-pane' | 'study-order' | 'chart-type' | 'series-scale' | 'pane-add' | 'pane-remove' | 'pane-order' | 'pane-weight' | 'pane-collapse' | 'axis' | 'settings' | 'drawing' | 'command';
/**
 * A host's own reversible step, for a change the history cannot observe or
 * make itself: a host that rebuilds its chart to switch the chart type. A
 * return of exactly `false`, or a throw, is a failure.
 */
interface ChartHistoryCommand {
    label?: string | undefined;
    undo(): unknown;
    redo(): unknown;
}

/** A description of the step an undo or redo would take. */
interface ChartHistoryStep {
    /** The label a host gave `transact`, `group` or `push`. */
    label?: string;
    changes: readonly ChartHistoryChange[];
}
/** A step that could not be applied. The history has already put the chart back. */
interface ChartHistoryError {
    direction: 'undo' | 'redo';
    step: ChartHistoryStep;
    error: unknown;
}
interface ChartHistoryOptions {
    /** The drawing controller whose steps join the timeline. */
    draw?: DrawingController | null;
    /** Most steps kept; the oldest go first. Default 100. */
    limit?: number;
    /** The series whose type and scale are history. Default: the chart's primary series. */
    series?: () => SeriesApi | null;
    /** How the chart type is set, so a host's own bookkeeping runs; given a transform's id for a transformed series. Default: on the series. */
    setChartType?: (type: string) => unknown;
    /** Called when an undo or redo fails. */
    onError?: (error: ChartHistoryError) => void;
}
/**
 * The chart-wide undo and redo coordinator. Build one per chart, give it the
 * drawing controller, and send every undo and redo press through it.
 */
declare class ChartHistory {
    private _chart;
    private _draw;
    private readonly _opts;
    private readonly _limit;
    private _undo;
    private _redo;
    /** What the chart looked like after the last recorded, applied or ignored change. */
    private _base;
    /** Bumped whenever the baseline moves for a change that is not recorded. */
    private _epoch;
    private _pending;
    private _applying;
    private _ignoring;
    private _restoring;
    private _tx;
    /** The transaction in progress announced an appearance change. */
    private _styled;
    /** The drawings as of the last change the controller reported: the `before` of the next step. */
    private _drawings;
    private _group;
    private _dragging;
    /** A study or pane change seen during a drawing drag, recorded when the drag ends. */
    private _deferred;
    /** Drawings the controller dropped without a step, waiting to learn whether a pane went with them. */
    private _dropped;
    private _orphans;
    private readonly _keys;
    private _nextKey;
    /** Every pane key a capture has held, and the panes among them the host made. */
    private readonly _seen;
    private readonly _hostPanes;
    /**
     * The name each study object goes by in the steps, and the object each name
     * reaches: the one on the chart, or the last one it had, whose policy is the
     * host's latest word on it. Objects, not ids, tell studies apart. A name is
     * kept only while its study is on the chart or something a press can reach
     * names it (`_prune`).
     */
    private readonly _names;
    private readonly _known;
    /** The name of the study that last held each id, for a setting reading an id no study holds now. */
    private readonly _lastNames;
    /** Studies that left the chart by the host's hand: no press brings them back. */
    private readonly _hostRemoved;
    private readonly _listeners;
    private _off;
    /** Gives the drawing controller its study anchor steps back. */
    private _release;
    /** Numbers the names a study holding an id another study held first goes by. */
    private _nextHolder;
    /** `_ready()` as of the last notice. */
    private _heard;
    private _destroyed;
    constructor(chart: Chart, options?: ChartHistoryOptions);
    get isDestroyed(): boolean;
    /**
     * Follow a chart a host rebuilt (and its drawing controller), keeping the
     * timeline. Panes are matched by slot and studies by id, which is what a
     * chart restored from the old one's `getState` keeps. Drawing steps the old
     * controller held are taken back from the drawings either side of them,
     * since the new controller holds none of them. A study the old chart held
     * and the new one does not was left out by the host, and no press brings it back.
     */
    attach(chart: Chart, draw?: DrawingController | null): void;
    canUndo(): boolean;
    canRedo(): boolean;
    /** The step an undo would take, or null. */
    peekUndo(): ChartHistoryStep | null;
    /** The step a redo would take, or null. */
    peekRedo(): ChartHistoryStep | null;
    /** Take back the latest step. False when there was none, or it could not be applied. */
    undo(): boolean;
    /** Apply again the step an undo took back. False when there was none, or it could not be applied. */
    redo(): boolean;
    /**
     * Run `fn` as one step: every change it makes to the chart and its drawings
     * is taken back by one undo. A chart setting (the grid, the status line)
     * and a scale's auto-fit or pinned ratio are read only in a transaction's
     * full capture, so they are recorded only this way; a pane weight or a
     * scale option set outside one is a step of its own, which the chart
     * announces with `layout:change`. Nested calls join the outermost one. A
     * throw still records what `fn` changed before it, then rethrows. An
     * `ignore` inside it is left out of the step, which is taken back on
     * either side of it.
     */
    transact<T>(fn: () => T, label?: string): T;
    /** Record what the transaction changed since it began, or since the last ignore inside it. */
    private _cut;
    /** Measure the chart afresh after a change that is not a step, so no later step takes it in. */
    private _rebase;
    /**
     * Merge everything recorded until the returned function runs into one step:
     * a dialog that previews its edits live, a colour dragged through a picker.
     * A group whose changes cancel out (a dialog's Cancel) leaves no step.
     * Groups nest; the outermost decides.
     */
    group(label?: string): () => void;
    /**
     * Run `fn` as the host's own change rather than a user step: nothing it
     * does to the chart or its drawings is recorded, and no undo or redo takes
     * it back, so a study it removes is never brought back by a press, and a
     * command it pushes is not recorded. The redo branch is kept. Inside a
     * transaction or a group, the step is recorded on either side of it and
     * leaves it alone.
     */
    ignore<T>(fn: () => T): T;
    /** Run `fn` with the drawing controller recording nothing, as the host's own act. */
    private _untracked;
    /**
     * Record a host's own reversible step, for a change the history cannot
     * observe. Inside `ignore`, or while a press is applied, it records nothing:
     * the change is the host's own there.
     */
    push(command: ChartHistoryCommand): void;
    /** Forget every step. The chart is left as it is. In place: a press whose command restores the chart still files its step. */
    clear(): void;
    /** Called after every change to what `canUndo`, `canRedo` or the peeks report. */
    subscribe(listener: () => void): () => void;
    destroy(): void;
    /**
     * Take the study anchor steps of the drawing controller in hand, giving the
     * previous one's back. The controller would otherwise hold each move as a
     * step of its own beside the settings patch recorded here, so one move
     * would need two presses, and a press on the drawing history alone would
     * find a step this timeline had already taken back.
     */
    private _claim;
    /**
     * An anchor move has written its patch: record it now, as the step it is,
     * rather than at the end of the turn, so a control asking `canUndo` right
     * after the release already sees it. Inside a transaction or a group it
     * joins that step as any observed change does.
     */
    private _anchorMoved;
    private _listen;
    private _observe;
    /** Record the change observed since the last capture, now rather than at the end of the turn. */
    private _flush;
    private _drawingChange;
    private _document;
    /**
     * The controller loaded a whole other document (another instrument's
     * drawings, a layout), and cleared its own history. No drawing step or
     * orphan recorded before describes it: taking one back would put a drawing
     * of the other document on this one, a line of one symbol on another. The
     * studies, panes and settings of those steps still describe this chart.
     */
    private _forgetDrawings;
    /** Whether a pane the last capture held has gone from the chart since. */
    private _paneGone;
    /** The controller holding these steps is gone: from now on they are taken back from their documents. */
    private _detachSteps;
    private _takeOrphans;
    private _record;
    private _merge;
    private _closer;
    private _keyOf;
    /**
     * The history's name for a study: its id, unless a study the history knew
     * before held that id and is not this one (the host placed another under
     * an id that came free), which then goes by a name of its own. Told apart
     * by the chart's object for each study, so another study of the same kind
     * under the same id is still another study.
     */
    private _nameOf;
    private _bind;
    /** The id the study a name reaches answers to, or had last. */
    private _liveId;
    /** The policy the host holds now for a study: the one it has on the chart, or had when it left. */
    private _policyOf;
    /** Studies `before` held and `after` does not, gone by the host's hand: no press brings them back. */
    private _took;
    /**
     * Forget every study and pane that is off the chart and that nothing a press
     * can reach names: a step on either branch, what an open group holds aside
     * to give back, the open transaction's start, and the baseline the next step
     * is measured from. The names otherwise outlive the step limit, one for
     * every study the chart ever had, and each study object held keeps what it
     * computed alive. Run whenever steps are recorded, trimmed or dropped, and
     * when an `ignore` ends.
     */
    private _prune;
    /** The chart as it is now, as the rules read it. */
    private _view;
    private _series;
    private _shot;
    private _peek;
    /**
     * The step as it was made, in what a press would still change: the
     * policies the studies hold now leave out what they keep.
     */
    private _describe;
    /**
     * The stretches a press would walk, in the order it walks them (newest
     * first for an undo), each from where it left the chart back to where it
     * found it, with what the chart lets it change now. Each is read against
     * the chart the stretches before it in the press leave, so a later one
     * whose stack the chart holds now still puts back the stack an earlier one
     * changes. One the policies leave nothing to do is left out.
     */
    private _moves;
    /** `view` as `_apply` leaves the chart once it has made it look like `to` in what `d` reaches. */
    private _project;
    /**
     * One pane's stack put into the order `to` records, over the studies both
     * hold, the rest keeping their rows; null when that is the order it has.
     * `blocked` when two studies that may not move (`movable: false`) would
     * trade places: another study may pass one, as the chart lets it, but
     * nothing moves one past another.
     */
    private _plan;
    /** Whether a press would find anything to do in this step. */
    private _reachable;
    private _move;
    private _run;
    /** Put back what a failed press had changed, as far as it can be. */
    private _rollback;
    /** The chart settings a linked chart follows, as the chart reads now. */
    private _look;
    /** Ask the drawing controller to take each step back (or again), newest first for an undo. */
    private _drawSteps;
    /** Someone moved along the drawing branches directly: follow, so the next press is not out of step. */
    private _follow;
    private _restoreOrphans;
    private _slot;
    /**
     * The study on the chart a step names: the object the name reaches, while
     * the chart holds it. A study the host placed under an id that came free is
     * another object, of whatever kind, and is left alone rather than taken for
     * the one the step means.
     */
    private _find;
    /** What the chart `view` describes lets a step do (see `diff`). */
    private _rulesFor;
    /** What the chart as it is now lets a step do. */
    private get _rules();
    /** Make the chart look like `to` in everything `d` reaches, and nothing else. */
    private _apply;
    /**
     * Bring a study back under the instance id it had, so the studies reading
     * it and the alerts naming it find it again, with the policy its host holds
     * now: the one the study last had on the chart, never an older one the step
     * captured. A study the host placed under that id since, of whatever kind,
     * holds it and is another study, so this one comes back under a fresh id
     * with the settings the step gives it: every step that names it and the
     * studies `to` has reading it follow, and the holder is left as it is.
     */
    private _add;
    private _liveSettings;
    /** The settings keys, visibility and scale assignment a step changed on one study. */
    private _fit;
    /**
     * Put the panes `to` knows into its order, one adjacent move at a time,
     * leaving any pane it does not know (a host's own, added since) in its slot.
     */
    private _arrange;
    /**
     * Put each pane's studies back into `to`'s stacking order, where no two
     * studies that may not move would have to trade places. One that may not
     * move is never moved by a call of its own: another study passes it, as the
     * chart lets it.
     */
    private _stack;
    /** One pane's axes as `p` has them, in the fields `only` names per scale, or every field for a pane brought back. */
    private _axes;
    /** The structure a step promises: each study it reaches present or gone, on its pane, and the panes in order. */
    private _check;
    private _notify;
    /** What an undo and a redo would do, as `subscribe` reports it: the two peeks, null when a press would do nothing. */
    private _ready;
    private _notifyIfMoved;
}

interface ColorPickerOptions extends WidgetTranslationOptions {
    id: string;
    label: string;
    value: unknown;
    disabledReason?: string | null;
    live?: boolean | undefined;
    /** Use the host's overlay stack to escape scrolling ancestors and share focus/Escape ownership. */
    openOverlay?: WidgetContext['openOverlay'] | undefined;
    onChange(value: string): void;
}
interface ColorPickerHandle {
    el: HTMLElement;
    input: HTMLInputElement;
    trigger: HTMLButtonElement;
    read(): unknown;
    write(value: unknown): void;
    destroy(): void;
}
/** A compact colour swatch with an optional alpha channel retained in its original format. */
declare function createColorPicker(doc: Document, options: ColorPickerOptions): ColorPickerHandle;
/** Include this string beside the widget stylesheet to support strict style CSP. */
declare const COLOR_PICKER_CSS = "\n.oac-widget .oac-color { position: relative; display: inline-flex; width: 27px; height: 27px; flex: 0 0 27px; }\n.oac-widget .oac-color > .oac-swatch-input { position: absolute; inset: 0; width: 27px; height: 27px; padding: 0; opacity: 0; pointer-events: none; }\n.oac-widget .oac-color__trigger { position: absolute; inset: 0; width: 27px; height: 27px; border: 1px solid var(--oac-bd); border-radius: 5px; background: transparent; cursor: pointer; }\n.oac-widget .oac-color__trigger:focus-visible { outline: 2px solid var(--oac-ring); outline-offset: 2px; }\n.oac-widget .oac-color__trigger:disabled { cursor: not-allowed; opacity: .5; }\n.oac-widget .oac-color__popover { position: absolute; z-index: 80; top: 30px; right: 0; width: 188px; padding: 9px; border: 1px solid var(--oac-bd); border-radius: 7px; background: var(--oac-elev); color: var(--oac-tx); box-shadow: var(--oac-shadow); }\n.oac-widget .oac-color__popover--above { top: auto; bottom: 30px; }\n.oac-widget .oac-color__popover.oac-pop { right: auto; bottom: auto; min-width: 0; max-width: calc(100% - 16px); max-height: calc(100% - 16px); overflow: auto; }\n.oac-widget .oac-color__popover[hidden], .oac-widget .oac-color__recent[hidden] { display: none; }\n.oac-widget .oac-color__palette, .oac-widget .oac-color__recent { display: grid; grid-template-columns: repeat(5, 1fr); gap: 5px; margin-bottom: 7px; }\n.oac-widget .oac-color__choice { width: 28px; height: 28px; padding: 0; border: 1px solid var(--oac-bd); border-radius: 5px; background: transparent; overflow: hidden; cursor: pointer; }\n.oac-widget .oac-color__custom { width: 100%; padding: 5px; border: 1px solid var(--oac-bd); border-radius: 4px; background: transparent; color: inherit; cursor: pointer; }\n.oac-widget .oac-color__opacity { display: flex; align-items: center; gap: 5px; margin-top: 7px; font-size: 11px; }\n.oac-widget .oac-color__range { min-width: 0; flex: 1; }\n.oac-widget .oac-color__percent { min-width: 30px; text-align: right; }\n";

type FormKind = 'boolean' | 'number' | 'color' | 'text' | 'multiline' | 'select' | 'opacity' | 'colorPair' | 'custom' | 'symbol' | 'session' | 'price' | 'timestamp';
/**
 * One row of a generated form, whatever schema it came from. The presentation
 * fields come from the input: `activeWhen` and `visibleWhen` are re-read on
 * every change, and consecutive controls sharing `inline` share one row. A
 * `colorPair`, a `custom` body and a `multiline` box always take a row of
 * their own, whatever `inline` they carry.
 */
interface FormControl extends IndicatorInputPresentation {
    /** The flat key `onChange` reports. A `colorPair` reports its halves' keys instead. */
    key: string;
    kind: FormKind;
    label: string;
    /** Sub-heading the row sits under; consecutive rows with one group share a header. */
    group?: string | undefined;
    /** Help text; rendered as a hover affordance beside the label. */
    tooltip?: string | undefined;
    min?: number | undefined;
    max?: number | undefined;
    step?: number | undefined;
    /** `select` only. Absent means free-form: the control becomes a text box. */
    options?: readonly {
        label: string;
        value: string;
    }[] | undefined;
    /** `colorPair` only: the switch (optional) and the two swatches. */
    pair?: {
        enabled?: {
            key: string;
        } | undefined;
        up: {
            key: string;
            label: string;
        };
        down: {
            key: string;
            label: string;
        };
    } | undefined;
    /** `custom` only: what the dialog renders in the control column. */
    custom?: string | undefined;
}
type FormValues = Readonly<Record<string, unknown>>;
interface FormOptions extends WidgetTranslationOptions {
    values: FormValues;
    /** Shared overlay owner for nested colour pickers. */
    openOverlay?: ColorPickerOptions['openOverlay'];
    /** Every edit, with the value in the control's declared type. */
    onChange(key: string, value: unknown): void;
    /**
     * Why a control, or one option of a select, cannot act right now, or null
     * when it can. Drawn disabled with the reason as its title, never hidden: a
     * greyed row is information, an absent one is a mystery. Asked again after
     * every edit and every `sync`, beside the control's `activeWhen`, and its
     * reason wins where both apply because the host knows the context.
     */
    unavailable?(key: string, option?: string): string | null;
    /** Prefix for element ids, so two forms in one document never share one. */
    idPrefix: string;
    /**
     * Emit colour and slider edits on `input` as well as `change`, for a dialog
     * that previews live. Off, a colour is one edit when the picker closes and a
     * slider one edit on release, which is what a per-edit undo history wants.
     */
    live?: boolean;
    /** Keep an invalid numeric draft visible for validation when a form is saved. */
    preserveInvalidNumbers?: boolean;
    /** Renders the control column of a `custom` row. Null skips the row. */
    custom?(control: FormControl, row: HTMLElement): HTMLElement | null;
}
interface FormHandle {
    el: HTMLElement;
    /**
     * Re-read `values` into every control the user is not currently in, hidden
     * ones included, then re-read which controls are shown and editable.
     */
    sync(values: FormValues): void;
    /** Every control's current value, keyed the way `onChange` reports it; a hidden draft included. */
    values(): Record<string, unknown>;
    /**
     * Validate typed drafts without replacing them or committing settings. A
     * hidden or disabled control is skipped: it cannot be corrected from where
     * the user is, so its draft never blocks a save, and it is never written.
     */
    validate(): boolean;
    /** Attach a rejected native write to its field; null clears that error. */
    setError(key: string, message: string | null): void;
    /** Focus the first enabled control. */
    focusFirst(): boolean;
    /** Dispose nested controls and any open overlays before removing the form. */
    destroy(): void;
}
/**
 * Chart-settings and indicator inputs. `source` becomes a select over the
 * canonical price sources; a `colorPair` keeps its two or three keys.
 * `intervals`, the codes the host serves, fills an `interval` select in their
 * order; without it the select offers the built-in tokens and every
 * registered code.
 */
declare function controlsFromInputs(inputs: readonly ChartSettingsInput[], translation?: FormTranslationOptions, intervals?: readonly string[]): FormControl[];
interface FormTranslationOptions extends WidgetTranslationOptions {
    /** Stable descriptor scope, for example settings or indicator.ema. */
    scope: string;
}
/**
 * Draw-tier fields. `lineStyle` is a select with a fixed option list, an
 * `opacity` is a slider read in percent, `levels` is left to the dialog (the
 * level editor is its own surface), and the text tool's content is the one
 * multi-line box.
 */
declare function controlsFromFields(fields: readonly SettingsField[], translation?: FormTranslationOptions): FormControl[];
/** What every mount function returns: the shell's `DialogHandle` plus the node and a liveness probe. */
interface PanelHandle {
    el: HTMLElement;
    close(): void;
    isOpen(): boolean;
}
/**
 * Render `controls` into `host` (emptied first). Rows sit under small
 * uppercase group headers; a boolean sits in the switch column in front of its
 * label; everything else sits in the control column on the right.
 */
declare function renderForm(host: HTMLElement, controls: readonly FormControl[], opts: FormOptions): FormHandle;

/** What the widget holds for a template store; see `createDrawingTemplates`. */
interface DrawingTemplates {
    readonly store: DrawingTemplateStore;
    /** The catalog as last loaded or committed; null until the first load lands, or when it failed. */
    catalog(): DrawingTemplateCatalog | null;
    /** The saved templates of one tool, in the order they were saved. */
    templatesFor(tool: string): DrawingTemplate[];
    /** The look new drawings of `tool` start with, if one was saved. */
    defaultFor(tool: string): DrawingTemplateValues | undefined;
    /** The look of a drawing: every setting its schema declares, except the words it says and where it is. */
    capture(drawing: Drawing): DrawingTemplateValues;
    /** Apply a look to the editable drawings among `ids` as one undo step. Returns how many changed. */
    apply(ids: readonly string[], values: DrawingTemplateValues): number;
    /** Save the look of drawing `id` as its tool's default. */
    saveDefault(id: string): Promise<void>;
    /** Forget a tool's default. */
    clearDefault(tool: string): Promise<void>;
    /** Save the look of drawing `id` under `name`. */
    saveTemplate(id: string, name: string): Promise<DrawingTemplate>;
    removeTemplate(templateId: string): Promise<void>;
    /** Called after the held catalog changes. Returns the unsubscribe. */
    subscribe(listener: () => void): () => void;
    destroy(): void;
}
/**
 * Hold `store`'s catalog for the widget in `ctx` and give each drawing the
 * user places the default saved for its tool. `destroy` stops both.
 */
declare function createDrawingTemplates(ctx: WidgetContext, store: DrawingTemplateStore): DrawingTemplates;
declare const DRAWING_TEMPLATES_CSS = "\n.oac-widget .oac-template-name { width: 300px; }\n.oac-widget .oac-template-name .oac-dialog__body { display: flex; flex-direction: column; gap: 6px; padding-top: 8px; }\n.oac-widget .oac-template-name__label { color: var(--oac-mut); font-size: 11px; }\n.oac-widget .oac-template-name__input { width: 100%; }\n";

/**
 * The widget's small DOM kit: element and glyph factories, the focus rules
 * the overlay stack and the keymap share, and the placement maths that puts
 * a panel beside, below or over its anchor inside the widget. No state, and
 * nothing of the widget's own: the leaf the rest of the chrome builds on.
 */
/** HTML-escape the four characters that matter in text and attribute values. */
declare const esc: (s: unknown) => string;
/** Create an element with a class and optional attributes. */
declare function h<K extends keyof HTMLElementTagNameMap>(doc: Document, tag: K, className?: string, attrs?: Record<string, string>): HTMLElementTagNameMap[K];
/**
 * A glyph span. The markup is a string from the draw tier's icon builders,
 * so this is the one place `innerHTML` is written, and it only ever wraps a
 * trusted `<svg>` from the registry.
 */
declare function glyph(doc: Document, svg: string, kind: 'tool' | 'chrome'): HTMLSpanElement;
/** Whether a key event came from a text control, where chords stay out of the way. */
declare function inTextField(target: unknown): boolean;
/** Whether `n` can take focus: a control, enabled, and not under a `hidden`. */
declare function focusable(n: Element | null): n is HTMLElement;
declare const focusables: (root: Element) => HTMLElement[];
interface Box {
    left: number;
    top: number;
    right: number;
    bottom: number;
    width: number;
    height: number;
}
interface Size {
    width: number;
    height: number;
}
/**
 * Where a panel of `size` goes beside `anchor` without leaving `bounds`: to
 * the right of it by `gap`, its top on the anchor's top, then pulled up (never
 * past `pad`) when it would run off the bottom, and flipped to the left only
 * when the right has no room at all. Coordinates are in whatever space the
 * anchor and the bounds share.
 */
declare function placeBeside(anchor: Box, size: Size, bounds: Size, gap?: number, pad?: number): {
    left: number;
    top: number;
    side: 'left' | 'right';
};
/** A panel under `anchor`, left-aligned with it, flipped above when there is no room below. */
declare function placeBelow(anchor: Box, size: Size, bounds: Size, gap?: number, pad?: number): {
    left: number;
    top: number;
};
type TipSide = 'right' | 'left' | 'top' | 'bottom';
/** A tip centred on the anchor's edge, flipped across it when its side has no room. */
declare function placeTip(anchor: Box, size: Size, bounds: Size, side?: TipSide, gap?: number, pad?: number): {
    left: number;
    top: number;
};
/** An element's box in the coordinate space of `root` (the widget), not the viewport. */
declare function boxIn(root: HTMLElement, el: Element): Box;

/**
 * What sits over the widget: the overlay stack, which knows what is open and
 * so owns Escape, the focus trap, the outside press and where focus goes
 * back, and the tooltip every control's hover label shares.
 */

interface OverlayOptions {
    /** The control the panel opens from; positions the panel and keeps a click on it from counting as outside. */
    anchor?: HTMLElement | undefined;
    /** Where the panel goes relative to the anchor. Default `below`; `center` ignores the anchor. */
    placement?: 'below' | 'beside' | 'center';
    /** A second element the panel should clear when placed beside (the rail, not just the button in it). */
    edge?: HTMLElement | undefined;
    /** Draw a scrim and refuse outside dismissal. Default false; `center` placement implies it. */
    modal?: boolean;
    /** Where focus lands: an element, `null` to leave focus where it is, or the first control (default). */
    initialFocus?: HTMLElement | null | undefined;
    /** Close on a press outside the panel and its anchor. Default: not modal. */
    dismissOnOutside?: boolean;
    /** Close on Escape. Default true. */
    dismissOnEscape?: boolean;
    /** Give focus back to the opener on close. Default true. */
    restoreFocus?: boolean;
    onClose?: () => void;
}
interface OverlayStack {
    open(el: HTMLElement, opts?: OverlayOptions): () => void;
    /** Close the newest overlay. False when nothing is open. */
    closeTop(): boolean;
    closeAll(): void;
    top(): HTMLElement | null;
    size(): number;
    /** Hide one overlay and its scrim without closing its session. Returns an idempotent resume. */
    suspend?(el: HTMLElement): () => void;
    /** The layer element overlays are appended to. */
    readonly layer: HTMLElement;
    destroy(): void;
}
/**
 * One stack per widget. Each module opens and closes its own panel; what none
 * of them can know is what else is open. The stack knows: Escape closes only
 * the newest thing, Tab stays inside it, a press outside closes a popover but
 * not a dialog, and focus goes back where it came from.
 */
declare function createOverlayStack(root: HTMLElement, doc: Document): OverlayStack;
interface TipSpec {
    title: string;
    /** The accessible name, when it must say more than the title. Default: the title. Since 2.5.10. */
    label?: string | undefined;
    /** The chord, shown in monospace after the title. */
    chord?: string | undefined;
    /** A second, muted line. */
    sub?: string | undefined;
    side?: TipSide;
}
/** A spec, or a function read at show time for a control whose label follows its state. */
type TipSource = TipSpec | (() => TipSpec | null);
interface TipController {
    /** Give `target` a hover label. `dwellMs` delays it; 0 shows on entry. */
    attach(target: HTMLElement, spec: TipSource, dwellMs?: number): void;
    /** Re-read a control's spec into its accessible name. */
    refreshLabel(target: HTMLElement): void;
    show(target: HTMLElement): void;
    hide(): void;
    /** The control the tip is up for, or null. */
    target(): HTMLElement | null;
    destroy(): void;
}
/** How long the pointer rests on a rail control before its label appears. */
declare const TIP_DWELL_MS = 600;
/**
 * One tip node per widget. A `title` waits about a second, cannot be styled
 * and cannot carry a second line, which is no use on a rail of near-identical
 * glyphs or on a bar of bare icons. Specs are read at show time, because a
 * group button stands for whichever tool was last picked from it.
 */
declare function createTipController(root: HTMLElement, layer: HTMLElement, doc: Document): TipController;

/** The three calls the widget makes on a synchronous store. `localStorage` satisfies it. */
interface StorageLike {
    getItem(key: string): string | null;
    setItem(key: string, value: string): void;
    removeItem(key: string): void;
}
/**
 * A store whose calls settle later, as IndexedDB's do. The widget reads a
 * namespace from it once (`entries`) and answers every later read from that
 * copy, so its own reads stay synchronous; writes follow behind. Having an
 * `entries` method is what marks a store as asynchronous.
 */
interface AsyncStorageLike {
    /** Every stored key that starts with `prefix`, with its value. */
    entries(prefix: string): Promise<ReadonlyArray<readonly [key: string, value: string]>>;
    setItem(key: string, value: string): Promise<void>;
    removeItem(key: string): Promise<void>;
    /**
     * A synchronous store the writes still pending are copied to when the page
     * goes away, and read back by the next `load`, because an asynchronous
     * write started as a page closes may never land. Without one, those writes
     * are lost with the page.
     */
    readonly journal?: StorageLike | null | undefined;
    /**
     * Hear each change other users of the store make once it has landed:
     * another tab on the same database, or another widget on the page. The
     * widget keeps its copy of the namespace current with it, as a synchronous
     * store's reads are, so an instrument whose drawings another tab changed
     * opens with them rather than overwriting them with an older copy. A null
     * value is a removal. Returns the call that stops listening. Without it,
     * the copy knows only what it read at load and what it wrote itself.
     */
    subscribe?(listener: (key: string, value: string | null) => void): () => void;
}
/** A write or a read an asynchronous store refused, as `WidgetStorageOptions.onError` receives it. */
interface WidgetStorageError {
    /**
     * `load` when the namespace could not be read, after which the session
     * keeps its changes in memory only, since writing could overwrite what was
     * never read; `write` or `remove` for one key, which is tried again with
     * the next change or `flush`.
     */
    operation: 'load' | 'write' | 'remove';
    /** The full key, or the namespace prefix for `load`. */
    key: string;
    error: unknown;
}
interface WidgetStorageOptions {
    /** Called for each failure of an asynchronous store. The widget shows it on its status line. */
    onError?: (error: WidgetStorageError) => void;
}
/** Every key the widget writes sits under this prefix, so a host on the same origin cannot collide. */
declare const STORAGE_PREFIX = "oac-widget:";
/**
 * Namespaced JSON storage that never throws: a private window, a full quota
 * or a blocked store all read as "nothing saved" and write as "not kept".
 *
 * Over a synchronous store every call goes straight through. Over an
 * asynchronous one, `get`, `set` and `remove` work on a copy of the namespace
 * in memory: `load` fills it once, and each change is sent behind it, the
 * changes to one key made within one task coalesced into one write. A write
 * made before `load` settles waits for it and wins over the stored value,
 * because it is the newer of the two. A store that can say so (`subscribe`)
 * keeps the copy current with the writes other tabs land, as a synchronous
 * store's reads always were, until `close`.
 */
declare class WidgetStorage {
    private readonly _store;
    private readonly _async;
    private readonly _ns;
    private readonly _journalKey;
    private readonly _onError;
    /** The asynchronous namespace as last read or written, by full key, as JSON text. */
    private readonly _mirror;
    /** Changes not sent yet, by full key: the text, or null for a remove. */
    private readonly _pending;
    /** Changes sent and not settled yet. */
    private readonly _sending;
    /** Changes the store refused, sent again with the next change or flush. */
    private readonly _refused;
    private _loading;
    private _loaded;
    private _memoryOnly;
    private _drain;
    private _queued;
    private _journaled;
    /** Changes other users made, heard while the namespace was being read, applied over what it returned. */
    private readonly _heard;
    private _unsubscribe;
    private _closed;
    constructor(namespace: string, store: StorageLike | AsyncStorageLike | null, options?: WidgetStorageOptions);
    key(name: string): string;
    get(name: string): unknown;
    /** True when the write landed; over an asynchronous store, when it was taken to be sent. */
    set(name: string, value: unknown): boolean;
    remove(name: string): void;
    /** Whether writes can land at all. */
    get enabled(): boolean;
    /** Whether reads answer from what is stored: always over a synchronous store, once `load` has settled over an asynchronous one. */
    get loaded(): boolean;
    /**
     * Read the namespace from an asynchronous store, once; later calls return
     * the same promise. Settles at once over a synchronous store. Never
     * rejects: a store that fails or does not answer within a few seconds is
     * reported through `onError`, and the session then runs on memory.
     */
    load(): Promise<void>;
    /**
     * Stop following the changes other users of an asynchronous store make.
     * Changes already taken are still sent. The widget calls it when it is
     * destroyed; a copy nobody reads again need not follow the store.
     */
    close(): void;
    /**
     * Send every change made so far now, and settle once each has landed or
     * failed. Before the sends it copies them to the store's journal, which is
     * what makes it the call for a page going away. Never rejects.
     */
    flush(): Promise<void>;
    private _queue;
    private _requeue;
    private _read;
    /**
     * One sender at a time, in batches: a key is not sent again while its last
     * write is out, so a newer value always lands after an older one. The
     * journal goes once everything it held has landed.
     */
    private _send;
    /**
     * A change another user of the store made, once it landed. A key this copy
     * has changed and not seen land keeps its own value: that write goes out
     * after, so it is the one the store keeps.
     */
    private _hear;
    private _writeJournal;
    private _report;
}
/** The page's `localStorage` when it exists and works, else null. */
declare function defaultStorage(): StorageLike | null;

/**
 * The contract between the widget shell and everything mounted inside it.
 *
 * `WidgetContext` is what the shell hands to the rail, the top bar, the
 * status line and every dialog: the chart and the drawing controller, the
 * root element, the theme in force, the keymap, a toast, an overlay opener
 * that positions, focus-traps and Escape-closes whatever it is given, and a
 * small event bus. A dialog module never reaches for the document on its own;
 * everything it needs to place itself or to hand focus back comes from here.
 *
 * The rest of this file is what every part of that contract shares: the
 * drawing and history helpers the controls act through, the bus, and the
 * dialog registry through which the dialog tier makes its mount functions
 * known. The furniture behind the contract has modules of its own, and this
 * one exports their names too: the DOM kit (dom.ts), the storage wrapper
 * (widget-storage.ts), and the overlay stack and the tooltip (overlays.ts).
 */

/**
 * One undo or redo press from any control: the chart-wide timeline when the
 * context carries one, the drawing controller's own otherwise, so every
 * button, chord and sheet walks the same steps.
 */
declare function historyPress(ctx: Pick<WidgetContext, 'draw' | 'history'>, direction: 'undo' | 'redo'): boolean;
/** Whether that press would do anything, for the control's enabled state. */
declare function historyReady(ctx: Pick<WidgetContext, 'draw' | 'history'>, direction: 'undo' | 'redo'): boolean;
type BusHandler<T = unknown> = (payload: T) => void;
/**
 * A tiny typed event bus. Handlers run in registration order; one that throws
 * does not stop the others, because a host's listener failing must not take
 * the shell's own bookkeeping with it.
 */
declare class WidgetBus<Events extends Record<string, unknown> = Record<string, unknown>> {
    private readonly _handlers;
    on<K extends keyof Events & string>(event: K, handler: BusHandler<Events[K]>): () => void;
    off<K extends keyof Events & string>(event: K, handler?: BusHandler<Events[K]>): void;
    emit<K extends keyof Events & string>(event: K, payload: Events[K]): void;
    clear(): void;
}
/** What a dialog module's mount function returns. */
interface DialogHandle {
    close(): void;
}
/** The mount signature every dialog exports: `(ctx, anchor?) => { close }`. */
type DialogMount = (ctx: WidgetContext, anchor?: HTMLElement) => DialogHandle;
/**
 * The dialogs the shell knows how to ask for. Each is mounted by the dialog
 * tier and registered here; a shell built without one renders the control
 * that would open it disabled, with its state visible, rather than dead.
 */
type WidgetDialogName = 'settings' | 'indicatorPicker' | 'indicatorSettings' | 'drawingProperties' | 'contextMenu' | 'levelEditor' | 'textEditor' | 'alertEditor' | 'alerts';
/** Make a dialog's mount function known to every shell. Returns a disposer. */
declare function registerWidgetDialog(name: WidgetDialogName, mount: DialogMount): () => void;
/** Register several at once, from a module's exports. */
declare function registerWidgetDialogs(mounts: Partial<Record<WidgetDialogName, DialogMount>>): () => void;
/** Take a dialog out of the registry. False when nothing was registered under the name. */
declare function unregisterWidgetDialog(name: WidgetDialogName): boolean;
/** The registered mount for a dialog, or null. */
declare function widgetDialog(name: WidgetDialogName): DialogMount | null;
declare function registeredWidgetDialogs(): WidgetDialogName[];
/** The events the shell publishes on `ctx.bus`. */
interface WidgetBusEvents {
    symbol: {
        symbol: string;
        exchange: string;
    };
    interval: {
        interval: string;
    };
    /** The data variant changed; undefined is the feed's default series. */
    variant: {
        variant?: Readonly<DataVariant> | undefined;
    };
    theme: {
        theme: WidgetThemeName;
        chartTheme: ChartTheme;
    };
    /** Something about the workspace changed: the chart type, a restored layout, a pane. */
    layout: {
        reason: string;
        chartType?: string;
    };
    /** The status line's transient message changed. */
    status: {
        text: string;
        kind: 'info' | 'error';
    };
    /** A keymap registration collided with another binding. */
    'keymap:conflict': {
        combo: string;
        kept: string;
        shadowed: string;
    };
    /** Bars finished loading, or failed to. */
    data: {
        symbol: string;
        interval: string;
        bars: number;
        error?: string | undefined;
    };
    [key: string]: unknown;
}
interface WidgetContext {
    readonly chart: Chart;
    readonly draw: DrawingController;
    /** Live inventory owned by the widget, optional for custom contexts. */
    readonly objects?: ChartObjects | undefined;
    /** Trader alerts owned by the widget, optional for custom contexts. */
    readonly alerts?: AlertController | undefined;
    /**
     * The chart-wide undo timeline. Every undo and redo control goes through it
     * when it is there; a custom context without one falls back to the drawing
     * controller's own history.
     */
    readonly history?: ChartHistory | undefined;
    /**
     * Saved drawing looks, when the host gave the widget a template store:
     * the properties dialog and the floating toolbar offer them.
     */
    readonly drawingTemplates?: DrawingTemplates | undefined;
    /** The `.oac-widget` element every piece of chrome lives in. */
    readonly root: HTMLElement;
    readonly document: Document;
    /** `dark` or `light`, following the chart theme in force. */
    readonly theme: WidgetThemeName;
    /** The engine palette the chart is drawing with. */
    readonly chartTheme: ChartTheme;
    readonly keymap: Keymap;
    readonly bus: WidgetBus<WidgetBusEvents>;
    /** Per-widget persisted preferences; `enabled` is false when `persist` is off. */
    readonly storage: WidgetStorage;
    readonly locale: string | undefined;
    readonly translate?: WidgetTranslator | undefined;
    /** Configured instrument lookup, shared by the header and indicator inputs. */
    readonly symbolSearch?: SymbolSearch | undefined;
    toast(message: string, kind?: ToastKind): ToastHandle;
    /**
     * Show `el` over the widget: positioned from `opts.anchor` (or centred as a
     * dialog), focus-trapped, closed by Escape and by a press outside. Returns
     * the closer.
     */
    openOverlay(el: HTMLElement, opts?: OverlayOptions): () => void;
    /** Put a transient message on the status line. */
    status(text: string, kind?: 'info' | 'error'): void;
    /** Hover labels for controls. */
    readonly tips: TipController;
    /** The overlay stack behind `openOverlay`, for a module that needs to ask what is open. */
    readonly overlays: OverlayStack;
    /** The current symbol and interval, for dialogs that name them. */
    symbol(): {
        symbol: string;
        exchange: string;
    };
    interval(): string;
    /**
     * The intervals the host serves, when it named them (`WidgetOptions.intervals`):
     * a study's timeframe select offers these.
     */
    readonly intervals?: readonly string[] | undefined;
    /**
     * Settings the widget starts at a value of its own rather than the
     * engine's, keyed as the settings schema keys them (since 2.6.3). The
     * dialog's Restore returns to these, so it lands where a fresh widget
     * starts: the drag that moves time alone, not the engine's time and price.
     */
    readonly settingsDefaults?: Readonly<ChartSettingsValues> | undefined;
}

declare const MAGNET_MODES: readonly MagnetMode[];
interface RailGroupItem {
    head?: string;
    tool?: string;
}
interface RailGroup {
    id?: string;
    title?: string;
    items?: RailGroupItem[];
    sep?: boolean;
}
/**
 * Rail groups. Only registered tools are shown, so a build that predates a
 * tool leaves a gap rather than a dead row. Names come from the descriptors,
 * not from here: the rail cannot spell a tool differently from the registry.
 */
declare const RAIL_GROUPS: readonly RailGroup[];
/** A tool's display name from the registry, or the id itself for one the registry lacks. */
declare const toolName: (id: string | null) => string;
interface RailPrefs {
    favorites: string[];
    magnet: MagnetMode;
    stay: boolean;
    /** Group id to the tool its button stands for. */
    last: Record<string, string>;
    /** The rail is folded away, leaving its tab. Absent means shown. */
    collapsed?: boolean;
}
interface RailOptions {
    /** Restrict the rail to these tool ids. Default: every registered tool the groups know. */
    tools?: readonly string[];
    /** Pins to start with when nothing is stored. */
    favorites?: readonly string[];
    /**
     * The element the armed tool's cursor goes on: the chart container. The
     * cursor rides a custom property the stylesheet reads, not the inline
     * cursor, which the engine owns for its own hover hints.
     */
    cursorTarget?: HTMLElement;
}
interface RailHandle {
    readonly el: HTMLElement;
    /** Reflect the armed tool on the buttons. The controller's `draw:tool` event drives it; a host may call it too. */
    sync(tool: string | null): void;
    /** Re-read the controller for the controls block (selection, history). */
    refresh(): void;
    prefs(): RailPrefs;
    restorePrefs(prefs: unknown): void;
    magnetMode(): MagnetMode;
    setMagnetMode(mode: MagnetMode): void;
    /** off -> weak -> strong -> off. Returns the new mode. */
    cycleMagnet(): MagnetMode;
    stayMode(): boolean;
    setStayMode(on: boolean): void;
    favorites(): string[];
    isFavorite(id: string): boolean;
    toggleFavorite(id: string, on?: boolean): void;
    /** The double-click hold: one tool kept armed until something leaves it. */
    setDrawLock(on: boolean): void;
    drawLocked(): boolean;
    /**
     * Whether the rail is folded away behind its tab. Optional because a host
     * may hand `mountMobile` a handle of its own, written before the fold.
     */
    collapsed?(): boolean;
    /** Fold the rail away, leaving its tab, or bring it back. Optional, as `collapsed`. */
    setCollapsed?(on: boolean): void;
    /** Open a group's flyout beside its button. */
    openGroup(groupId: string, viaKeyboard?: boolean): void;
    closeFlyout(): void;
    flyoutOpen(): boolean;
    destroy(): void;
}
/** Storage key the preferences live under. */
declare const RAIL_PREFS_KEY = "rail";
/**
 * A tool's glyph as a `<use>` into the sprite. A tool the tier has no glyph
 * for (a host-registered one) gets its initial instead of a throw, so one
 * custom tool cannot blank the rail.
 */
declare function toolGlyph(doc: Document, id: string): HTMLElement;
/** Validate a stored preference object field by field, dropping what this build cannot honour. */
declare function sanitizeRailPrefs(raw: unknown, groups: readonly RailGroup[], toolsOf: (g: RailGroup) => string[]): RailPrefs;
declare function mountRail(ctx: WidgetContext, host: HTMLElement, opts?: RailOptions): RailHandle;

/**
 * The layouts controller: which saved layout a widget or a grid holds, the
 * catalog revision its writes are checked against, whether the chart has
 * moved away from what was saved, and the autosave that writes it back. It
 * builds no DOM: a menu drives it and renders its state.
 *
 * It takes a `WorkspaceStore` (the workspace tier's `WorkspaceRepository`, or
 * a host's own) and imports that tier as types only, so a widget host loads
 * the workspace bundle only when it hands a store over.
 */

/** What showing a layout did: `reason` says why nothing changed. */
interface LayoutApplyReport {
    applied: boolean;
    reason?: string;
}
/**
 * The widget or grid a controller saves and restores. A grid is
 * `{ capture: () => grid.getWorkspace(), apply: p => grid.applyWorkspace(p) }`.
 */
interface LayoutTarget {
    /** The target as it stands, as a portable workspace payload. */
    capture(): WorkspacePayload;
    /**
     * Show a saved layout, or change nothing and say why. The controller also
     * puts the previous layout back through it when the store refuses to
     * record the new one, so a refused apply must leave the target untouched.
     */
    apply(payload: WorkspacePayload): LayoutApplyReport;
    /** Called on each change the user makes. Without it the host calls `changed()`. Returns the unsubscribe. */
    subscribe?(listener: () => void): () => void;
    /**
     * True while the target shows something other than the layout the user is
     * building, as a chart does during a replay: its bars stop at a moment in
     * the past and a replay may change the view on every step. Autosave waits
     * and `open` is refused until it turns false, which the target announces
     * through its `subscribe` listener (or the host through `changed()`). A
     * save the user asks for still goes through. Since 2.5.10.
     */
    suspended?(): boolean;
}
/**
 * `off` while the catalog's autosave preference is off or no layout is held;
 * otherwise `pending` (a change waits for the quiet period), `saving`,
 * `saved` (the stored layout matches the target) or `failed` (the last write
 * of this layout failed, or it conflicts; autosave waits for a save, a
 * reload or, for a conflict, the user's choice).
 */
type LayoutAutosaveStatus = 'off' | 'pending' | 'saving' | 'saved' | 'failed';
interface LayoutsState {
    /** The newest catalog the controller has seen: its own copy, to read and not change. Null until the first load. */
    catalog: WorkspaceCatalog | null;
    /** The saved layout the target was opened from or last saved to; null for none. */
    layoutId: string | null;
    /**
     * The revision the next write into the held layout is checked against;
     * null until the first load. It trails `catalog.revision` while the held
     * layout conflicts, so that write is refused rather than laid over a
     * version nobody here has seen.
     */
    revision: number | null;
    /** The target differs from its layout. Checked when changes settle and by `flush()`; false with no layout. */
    dirty: boolean;
    /** A layout operation (anything but an autosave) is queued or running. */
    busy: boolean;
    /** The target is suspended (`LayoutTarget.suspended`): autosave waits and `open` is refused. Since 2.5.10. */
    suspended: boolean;
    /**
     * The held layout was changed or deleted in another session, or by another
     * control on the page, since this controller last opened or saved it, and
     * a write into it was refused or a reload found it so. `save()` and
     * autosave are refused until `overwrite()`, `saveAs()` or `open()` settles
     * it; a reload clears it only when it finds the layout as this controller
     * left it.
     */
    conflict: boolean;
    autosave: LayoutAutosaveStatus;
    /** Why the last operation or autosave failed; null after a success. */
    error: unknown;
}
interface LayoutsControllerOptions {
    /**
     * Quiet time in ms after the last change before the target is compared
     * with its layout and autosaved. Default 1000; a value that is not a
     * finite number takes the default.
     */
    autosaveDelay?: number;
}
/**
 * Operations run one at a time, in call order, each checked against the
 * revision the previous one left, so a burst of clicks never interleaves.
 *
 * A write the store refuses because the catalog moved (another tab, another
 * control on the page) is tried again on the newer catalog when what it acts
 * on is unchanged there: the held layout for a save, the named layout for
 * rename, duplicate, remove and open. Only a change to that is a conflict.
 */
interface LayoutsController {
    readonly store: WorkspaceStore;
    state(): LayoutsState;
    /** Called after the state changes. Returns the unsubscribe. */
    subscribe(listener: (state: LayoutsState) => void): () => void;
    /**
     * Read the catalog again, and resolve with a copy of it. The target is not
     * touched. A paused autosave resumes, unless the held layout conflicts.
     * After a page load, `open(catalog.activeWorkspaceId)` continues on the
     * layout that was active.
     */
    reload(): Promise<WorkspaceCatalog>;
    /**
     * Show a saved layout and record it as active and recent, unless it already
     * is. A change still waiting for the quiet period is autosaved into the
     * layout being left first; when that write fails this rejects and changes
     * nothing, and opening again goes ahead without it. A refused apply
     * resolves with its report and leaves the target and the held layout as
     * they were; when the store refuses the record, the previous layout goes
     * back on the target and this rejects. It rejects too while the target is
     * suspended.
     */
    open(id: string): Promise<LayoutApplyReport>;
    /** Write the target into the held layout. Rejects when no layout is held, and while it conflicts. */
    save(): Promise<WorkspaceDocument>;
    /** Save the target as a new layout, which becomes the held, active one. */
    saveAs(name: string): Promise<WorkspaceDocument>;
    /** Reload, then write the target over the held layout, whatever another session saved there. */
    overwrite(): Promise<WorkspaceDocument>;
    rename(id: string, name: string): Promise<void>;
    duplicate(id: string, name: string): Promise<WorkspaceDocument>;
    /** Delete a saved layout. Deleting the held one leaves the target as it is, holding none. */
    remove(id: string): Promise<void>;
    setAutosave(enabled: boolean): Promise<void>;
    /** Tell the controller the target may have changed, for a target without `subscribe`. */
    changed(): void;
    /** Compare the target with its layout now, autosave if due, and wait for every queued operation. */
    flush(): Promise<void>;
    /** Stop listening. A queued operation rejects; a pending autosave is dropped, so `flush()` first to keep it. */
    destroy(): void;
}
/** Drive `target`'s saved layouts through `store`. Call `reload()` or any operation to load the catalog. */
declare function createLayoutsController(store: WorkspaceStore, target: LayoutTarget, options?: LayoutsControllerOptions): LayoutsController;

interface MenuRow {
    label: string;
    /**
     * A chrome icon id for a glyph before the label. Once one row has a glyph,
     * every row keeps the column, so the labels share a left edge; an id the
     * registry does not carry leaves its slot empty. Since 2.5.10.
     */
    icon?: string | undefined;
    sub?: string | undefined;
    /** Shown at the right edge, for a chord. */
    key?: string | undefined;
    /** Makes the row one of a set of choices, true for the current one; a row without it is an action. */
    on?: boolean;
    disabled?: boolean;
    danger?: boolean;
    onSelect: () => void;
}
interface MenuOptions {
    /** A search box at the top with this placeholder; rows filter as the user types. */
    find?: string;
    ariaLabel?: string;
    /** Where the menu opens: under the anchor (default), or beside it, as a column of buttons wants. Since 2.6.0. */
    placement?: 'below' | 'beside';
    /** Beside only: a wider element the menu clears as well as the anchor (the rail, not one button in it). Since 2.6.0. */
    edge?: HTMLElement | undefined;
}
/**
 * A popup menu under `anchor`, or beside it. Rows are buttons; a `{ head }` string starts a
 * group. Returns the closer. Exported for the dialog tier, whose context menu
 * and pickers want the same shape. It reads only the document, the overlay
 * opener and the translations, so any widget context serves, and so does a
 * bottom bar context.
 */
declare function openMenu(ctx: WidgetContext | Pick<WidgetContext, 'document' | 'openOverlay' | 'translate'>, anchor: HTMLElement, rows: ReadonlyArray<MenuRow | string>, opts?: MenuOptions): () => void;
/** Hand `text` to the browser as a file. False when the runtime has no way to (no `Blob`, no object URLs). */
declare function downloadText(doc: Document, filename: string, text: string, mime: string): boolean;
/** `SYMBOL-5m-2026-01-31-09-15` with the characters a filename cannot carry removed. */
declare function captureName(symbol: string, interval: string, now?: Date): string;

/** Labels for the built-in chart types; a transform's is its own name, and anything else is read from its id. */
declare const CHART_TYPE_LABELS: Readonly<Record<string, string>>;
declare function chartTypeLabel(id: string): string;
/**
 * The chart types a primary series can be: every registered renderer that
 * declares itself a price series, then every transform the chart can apply
 * (Heikin Ashi to Kagi, once the transform tier is imported), which the chart
 * derives from the bars rather than drawing them as they come. Point and
 * figure and Kagi are both a renderer and a transform, and are listed once,
 * with the transforms. A volume histogram is registered too and would draw,
 * but it is not a chart type anyone picks for the instrument.
 */
declare function chartTypeChoices(): string[];
/**
 * A pill label for an interval code: minutes and hours keep their lower-case
 * unit (`5m`, `1h`), days and weeks read as a capital (`D`, `W`, `2W`), and
 * anything else (a registered calendar code) is upper-cased as written.
 */
declare function intervalLabel(code: string): string;
interface TopbarState {
    symbol: string;
    exchange: string;
    interval: string;
    chartType: string;
    theme: WidgetThemeName;
}
interface TopbarOptions {
    intervals: readonly string[];
    /** Show the indicators button. Default true. */
    indicators?: boolean | undefined;
    search?: SymbolSearch | undefined;
    /** The current facts, read on every refresh. */
    state: () => TopbarState;
    onSymbol(symbol: string, exchange?: string): void;
    onInterval(code: string): void;
    onChartType(id: string): void;
    onTheme(next: WidgetThemeName): void;
    /** Open the settings dialog from `anchor`. Return false when no dialog is registered. */
    onSettings(anchor: HTMLElement): boolean;
    onIndicators(anchor: HTMLElement): boolean;
    /** A text control for the host's object inventory, omitted without a handler. */
    onObjects?(anchor: HTMLElement): boolean;
    /** Open the docked data window, omitted without a handler. */
    onDataWindow?(anchor: HTMLElement): void | boolean;
    onAlerts?(anchor: HTMLElement): boolean;
    /** Open the docked watchlist, omitted without a handler (the host supplied no lists). */
    onWatchlist?(anchor: HTMLElement): void | boolean;
    /** Open the docked news reader, omitted without a handler (the host supplied no news source). */
    onNews?(anchor: HTMLElement): void | boolean;
    /** Open the date and range navigation panel, omitted without a handler. */
    onGoTo?(anchor: HTMLElement): void | boolean;
    /**
     * The saved layouts the Layouts button names: it shows the held layout and
     * marks one with unsaved changes. Omitted, with `onLayouts`, without a store.
     */
    layouts?: LayoutsController | undefined;
    /** Open the Layouts menu from `anchor`. */
    onLayouts?(anchor: HTMLElement): void | boolean;
    settingsAvailable(): boolean;
    indicatorsAvailable(): boolean;
    /** Refuse CSV export while the host is replacing or recovering its data. */
    dataAvailable?(): boolean;
    /** More capture menu rows, read on every open (the chart grid adds its whole-grid capture); a string starts a group. */
    captureRows?: (() => ReadonlyArray<MenuRow | string>) | undefined;
}
interface TopbarHandle {
    readonly el: HTMLElement;
    /** Open the shared capture menu from desktop or mobile chrome. */
    openCapture(anchor: HTMLElement): void;
    /** Repaint every control from `state()`. */
    refresh(): void;
    /** Put the caret in the symbol box, text selected. */
    focusSymbol(): void;
    destroy(): void;
}
declare function mountTopbar(ctx: WidgetContext, host: HTMLElement, opts: TopbarOptions): TopbarHandle;

/** What the menu asks a host to do when an order row is picked. */
interface OrderRequest {
    side: 'BUY' | 'SELL';
    type: 'MARKET' | 'LIMIT' | 'SL';
    /**
     * The price under the pointer, not snapped to the instrument's tick or
     * tick schedule, which the widget does not know; round it (`validatePrice`
     * does) before sending. Null for a market order raised off the plot.
     */
    price: number | null;
    paneIndex: number;
}
interface MenuItem {
    kind?: 'item';
    label: string;
    /** Chrome icon id, or inline SVG markup starting with `<svg`. */
    icon?: string;
    /** Chord hint, shown at the right. */
    chord?: string | undefined;
    /** A switch (tick) or one option of a choice (dot). */
    mark?: 'check' | 'radio';
    on?: boolean;
    disabled?: boolean;
    /** Why a disabled row is disabled; an empty greyed row reads as a bug. */
    note?: string | undefined;
    danger?: boolean;
    /** Keep the menu up after running (a switch the user may flip twice). */
    keepOpen?: boolean;
    /** Stable id, for tests and for a host that wants to find a row. */
    id?: string;
    run?(): void;
}
type MenuEntry = MenuItem | {
    kind: 'separator';
} | {
    kind: 'header';
    label: string;
};
interface ContextMenuHooks {
    /** Order entry. Without it no trade rows are drawn: the engine places no orders itself. */
    onOrder?(order: OrderRequest): void;
    /** Omitted capabilities preserve the host's existing supported order routes. */
    tradingCapabilities?: TradingCapabilitySource | undefined;
    /** Required when the capability declaration limits live or analyzer mode. */
    tradingMode?: TradingCapabilityRequest['mode'];
    /** Host replay selection or workspace transitions that also prevent order entry. */
    tradingLocked?(): boolean;
    /** Extra rows a host appends, built per event. */
    items?(e: ContextMenuEvent): MenuEntry[];
}
interface ContextMenuOptions {
    /** The chart's event. Without one the menu is the chart-level menu at the anchor. */
    event?: ContextMenuEvent;
    hooks?: ContextMenuHooks;
}
/**
 * Every row the menu shows for `e`, in order. Pure apart from the closures:
 * nothing is rendered, so a test (or a host building its own menu) can read
 * the list.
 */
declare function contextMenuEntries(ctx: WidgetContext, e: ContextMenuEvent, hooks?: ContextMenuHooks): MenuEntry[];
/**
 * Show the menu for `opts.event` at the pointer, or the chart-level menu below
 * `anchor` when no event is given.
 */
declare function mountContextMenu(ctx: WidgetContext, anchor?: HTMLElement, opts?: ContextMenuOptions): PanelHandle;
/**
 * Wire the chart's `contextmenu` event to the menu. Returns the unsubscriber.
 * The browser's own menu is suppressed only once ours is up, so a host that
 * detaches this gets the native one back.
 */
declare function attachContextMenu(ctx: WidgetContext, hooks?: ContextMenuHooks): () => void;

interface DrawingPropertiesOptions {
    /** The drawings to edit; they become the selection. Default: the current selection. */
    ids?: readonly string[];
    /** The tab to open on: the settings (default) or the anchors. */
    tab?: 'style' | 'coordinates';
    /** Runs once when the dialog is gone. */
    onClose?(): void;
}
/**
 * Open the properties dialog for the selection, below `anchor` when there is
 * one (a toolbar button), else beside the drawing.
 */
declare function mountDrawingProperties(ctx: WidgetContext, anchor?: HTMLElement, opts?: DrawingPropertiesOptions): PanelHandle;

interface IndicatorPickerOptions {
    /** Runs after each instance is added, with its handle. */
    onAdd?(inst: IndicatorApi): void;
    /** Close the picker after the first add. Default false. */
    closeOnAdd?: boolean;
    /**
     * Where saved indicator templates live. Default: the store the widget was
     * given (`WidgetOptions.workspaces`); null offers none. A store without
     * `planIndicatorTemplateState` offers none either. Since 2.5.10.
     */
    templates?: WorkspaceStore | null;
}
/**
 * Open the picker below `anchor` (the toolbar's Indicators button), or centred
 * when there is none.
 */
declare function mountIndicatorPicker(ctx: WidgetContext, anchor?: HTMLElement, opts?: IndicatorPickerOptions): PanelHandle;

type IndicatorSettingsTab = 'inputs' | 'style';
interface IndicatorSettingsOptions {
    /**
     * The instance to edit. Without it the dialog reads `data-instance-id` off
     * the anchor (a legend row's gear), and failing that edits the chart's only
     * indicator; with several and no id it declines with a toast.
     */
    instanceId?: string;
    tab?: IndicatorSettingsTab;
    /** Runs after every write, including a revert. */
    onChange?(inst: IndicatorApi): void;
    /** Runs once when the dialog is gone; `committed` is false after Cancel or Escape. */
    onClose?(committed: boolean): void;
}
/** A handle for a dialog that never opened, so a caller can `close()` it regardless. */
declare function mountIndicatorSettings(ctx: WidgetContext, anchor?: HTMLElement, opts?: IndicatorSettingsOptions): PanelHandle;

interface LevelEditorOptions {
    /** The drawings to edit. Default: the controller's selection. */
    ids?: readonly string[];
}
/**
 * Open the editor for the selected ladder drawings, below `anchor` when there
 * is one (the properties dialog's button), else beside the selection.
 */
declare function mountLevelEditor(ctx: WidgetContext, anchor?: HTMLElement, opts?: LevelEditorOptions): PanelHandle;

interface SettingsDialogOptions {
    /** The tab to open on. Default: the first tab with a control in it. */
    tab?: ChartSettingsTabId;
    /**
     * Why a control, or one option of a select, cannot act in this host right
     * now, or null. A host that draws its own trade chrome knows whether there
     * is a position to recolour; the engine does not.
     */
    unavailable?(key: string, option?: string): string | null;
    /** Runs after every write, including a revert, for state a host keeps outside the schema. */
    onApply?(patch: Readonly<Partial<ChartSettingsValues>>): void;
    /** Runs once when the dialog is gone; `committed` is false after Cancel or Escape. */
    onClose?(committed: boolean): void;
}
/**
 * Open the chart settings dialog, centred over the widget. `anchor` is
 * accepted for the registry's uniform signature and not used: a dialog this
 * size is not a popover.
 */
declare function mountSettingsDialog(ctx: WidgetContext, _anchor?: HTMLElement, opts?: SettingsDialogOptions): PanelHandle;

interface TextEditorOptions {
    /** The drawing to edit. Default: the one selected drawing. */
    id?: string;
    /** Runs once after the box is gone; `committed` is true when the text changed. */
    onDone?(committed: boolean): void;
}
interface TextEditorHandle extends PanelHandle {
    commit(): void;
    cancel(): void;
}
/**
 * Open the editor over the drawing. `anchor` is accepted for the registry's
 * uniform signature and not used: the box sits where the text is painted.
 */
declare function mountTextEditor(ctx: WidgetContext, _anchor?: HTMLElement, opts?: TextEditorOptions): TextEditorHandle;

interface AlertEditorOptions {
    /** Edit this record; omit to create a new alert. */
    alertId?: string;
    /** Seed a draft from a plot, clicked price or drawing anchor. */
    source?: AlertSource;
    onClose?(): void;
}
interface AlertsPanelOptions {
    onClose?(): void;
}
/** Draft edits never arm an alert until Save. Closing always discards the draft. */
declare function mountAlertEditor(ctx: WidgetContext, anchor?: HTMLElement, opts?: AlertEditorOptions): PanelHandle;
/** All lifecycle states remain visible until explicitly deleted. */
declare function mountAlertsPanel(ctx: WidgetContext, anchor?: HTMLElement, opts?: AlertsPanelOptions): PanelHandle;

interface DrawingCoordinatesHandle {
    readonly el: HTMLElement;
    /** Read the drawings again; a row being edited keeps what is typed in it. */
    refresh(): void;
    destroy(): void;
}
/**
 * Render the anchors of `ids()` into `host`. `why` is the reason the
 * selection cannot be edited, or null; with one, every field is read-only.
 */
declare function mountDrawingCoordinates(ctx: WidgetContext, host: HTMLElement, ids: () => readonly string[], why: () => string | null): DrawingCoordinatesHandle;
declare const DRAWING_COORDINATES_CSS = "\n.oac-widget .oac-coords { display: flex; flex-direction: column; gap: 4px; min-width: 0; }\n.oac-widget .oac-coords__head, .oac-widget .oac-coords__fields { display: grid; grid-template-columns: minmax(0, 1.3fr) minmax(0, 1fr) minmax(0, 1fr); align-items: center; gap: 6px; }\n.oac-widget .oac-coords__head { order: -1; color: var(--oac-faint); font-size: 10px; font-weight: 600; text-transform: uppercase; letter-spacing: .5px; padding: 2px 0 0; }\n.oac-widget .oac-coords__zone { margin: 0 0 4px; color: var(--oac-faint); font-size: 11px; }\n.oac-widget .oac-coords__row { display: flex; flex-direction: column; gap: 3px; }\n.oac-widget .oac-coords__label { color: var(--oac-mut); font-size: 11px; }\n.oac-widget .oac-coords input { min-width: 0; width: 100%; height: var(--oac-ctl-h); padding: 0 6px; border: 1px solid var(--oac-bd);\n  border-radius: var(--oac-radius); background: var(--oac-elev); color: var(--oac-tx); color-scheme: inherit; font-variant-numeric: tabular-nums; }\n.oac-widget .oac-coords input:disabled { opacity: .6; cursor: not-allowed; }\n.oac-widget .oac-coords .oac-input-error { margin: 0; }\n.oac-widget .oac-coords__note { margin: 4px 0; color: var(--oac-mut); font-size: 12px; overflow-wrap: anywhere; }\n";

/** The mounts under the names the shell's registry knows them by. */
declare const WIDGET_DIALOGS: {
    settings: typeof mountSettingsDialog;
    indicatorPicker: typeof mountIndicatorPicker;
    indicatorSettings: typeof mountIndicatorSettings;
    drawingProperties: typeof mountDrawingProperties;
    contextMenu: typeof mountContextMenu;
    levelEditor: typeof mountLevelEditor;
    textEditor: typeof mountTextEditor;
    alertEditor: typeof mountAlertEditor;
    alerts: typeof mountAlertsPanel;
};
/**
 * The rules the dialogs add to the widget stylesheet. Scoped under
 * `.oac-widget` and coloured only through tokens, like the shell's own; the
 * generic furniture (`.oac-dialog__*`, `.oac-btn`, the form controls) is the
 * shell's and is not restated here.
 */
declare const DIALOG_CSS: string;

/**
 * Date and range navigation for a host: load the history a request needs
 * through the host's own loader, then place it with the chart's public
 * viewport and time conversion. DOM-free, so the packaged widget and a custom
 * host apply one set of rules.
 *
 * Three decisions carry it:
 *
 * - **A date names a bar by its calendar day, not by its stamp.** Daily bars
 *   arrive stamped at local midnight, at the session open or at UTC midnight,
 *   so a typed date is before its own bar in two of those three. Bars a day or
 *   longer are therefore counted from the local midnight of their first day;
 *   shorter bars keep their stamp, and an instant in an overnight or weekend
 *   gap moves to the next session.
 * - **History stays contiguous.** Missing history is older bars prepended to
 *   what is loaded, never a detached window, because the gapless axis would
 *   draw a detached window as if it continued straight into the live tail.
 * - **The placement is the last thing that happens.** It runs after the
 *   accepted load and only for the request, instrument and chart it was made
 *   for, so an obsolete answer cannot move the view and a later refresh keeps
 *   the anchor it finds.
 */

/** A date to show, or an explicit range. UTC seconds throughout. */
interface DateNavigationTarget {
    /** The date to show, or the start of the range. */
    from: number;
    /**
     * Inclusive end of a range: bars that open at or before it, a bar a day or
     * longer counting from the local midnight of its first day. Omit to centre
     * `from` at the current zoom.
     */
    to?: number;
}
/**
 * What one call to a host history loader achieved. `loaded` means older bars
 * reached the chart; `empty` means the windows it inspected held none, though
 * older history may still exist; `exhausted` means the source has nothing
 * older; `limited` means a retention limit stopped it; `unavailable` means
 * history cannot load now (no loader, replay, a paused source).
 */
type HistoryReach = 'loaded' | 'empty' | 'exhausted' | 'limited' | 'unavailable';
/**
 * `placed`: the request is in view. `partial`: something is in view, but
 * history stops after the requested start or the range is wider than the
 * chart. `no-data`: nothing to show, the view is unchanged. `unsupported`:
 * the interval has no time buckets (tick or volume bars), so it has no dates
 * to go to. `invalid`: `from` or `to` is not a finite time, `to` is before
 * `from`, or (from `setRange`) the range id is not one the widget has.
 * `cancelled`: a newer request, a context change or destruction took over.
 * `error`: the history loader failed; the result's `error` says how.
 */
type DateNavigationStatus = 'placed' | 'partial' | 'no-data' | 'unsupported' | 'invalid' | 'cancelled' | 'error';
interface DateNavigationResult {
    status: DateNavigationStatus;
    /** Open time of the first bar the placement shows. */
    from?: number | undefined;
    /** Open time of the last bar the placement shows. */
    to?: number | undefined;
    /** Why loaded history stops after the requested start. */
    history?: Exclude<HistoryReach, 'loaded'>;
    /** The range held more bars than the chart can show at its narrowest spacing; its start is in view. */
    clipped?: boolean;
    error?: Error;
}
interface DateNavigatorOptions {
    /** The chart to place. A function is read again after every load, for a host that rebuilds its chart. */
    chart: Chart | (() => Chart | null);
    /**
     * Prepend older primary bars reaching back to `time`, resolving once they
     * are on the chart. Called again while it adds bars and `time` is not yet
     * reached. Omit to navigate loaded history only.
     */
    loadHistory?(time: number, signal: AbortSignal): Promise<HistoryReach>;
    /** Interval code of the primary bars. Default: the chart's data context. */
    interval?(): string | undefined;
}
/**
 * Loads missing history, then places a date or an explicit UTC range. One
 * request at a time: a new `goTo` supersedes the one in flight, whose promise
 * settles `cancelled` without touching the view.
 */
declare class DateNavigator {
    private readonly _options;
    private _abort;
    private _destroyed;
    constructor(options: DateNavigatorOptions);
    goTo(target: DateNavigationTarget): Promise<DateNavigationResult>;
    /** Abandon the request in flight. Bars it already loaded stay. */
    cancel(): void;
    destroy(): void;
    private _chart;
    private _run;
}

/**
 * The market status and the clock as words: what the bottom bar, the status
 * line and the phone layout's sheet print from a session calendar.
 *
 * The status is read from the chart's own calendar
 * (`chart.dataLayer.sessionCalendar`), never from a clock rule of the
 * widget's: the calendar knows the venue's zone, its weekly hours, its closed
 * dates and its pre and post hours, and a guess from the wall clock knows none
 * of them. A calendar that cannot answer, such as one whose window a clock
 * change removes, reads as no status at all rather than stopping the chrome.
 */

/** The status as the chrome prints it. */
interface MarketStatusReading {
    readonly phase: SessionPhase;
    /** The state: "Market open", "Pre-open", "Holiday". */
    readonly label: string;
    /** What comes next, on the chart's clock: "closes 15:30", "opens Mon 09:15". Empty when nothing is due within 14 days. */
    readonly detail: string;
}

/**
 * Preset ranges: a named span of history (a day, five days, a month, the year
 * to date) with the interval it reads best at. DOM-free, so the packaged
 * widget and a custom host size and place a range by one set of rules.
 *
 * Two decisions carry it:
 *
 * - **A day is a trading session, not 24 hours.** One NSE day at one minute is
 *   375 bars, and asking a feed for 1,440 minutes back from 10:00 lands in the
 *   previous afternoon. With a session calendar a range walks back through the
 *   calendar's sessions, so a weekend or a closed date is skipped rather than
 *   counted. Without one it counts weekdays, and once bars are loaded it counts
 *   the dates the bars fall on.
 * - **A range ends at the latest bar it has, not at the wall clock.** Before
 *   the open, over a weekend, or with a feed that lags, "one day" is the last
 *   session that traded. The clock sizes the fetch; the bars place the view.
 */

/**
 * How a range measures its span: trading sessions, calendar months or years
 * back, the year to date, or all the history the source has.
 */
type WidgetRangeUnit = 'session' | 'month' | 'year' | 'ytd' | 'all';
/** A preset range for the bottom bar and `Widget.setRange`. */
interface WidgetRange {
    /** Stable id: what `setRange` takes and `range()` returns. */
    readonly id: string;
    /** The button's text. */
    readonly label: string;
    /**
     * The interval the range shows. A host that does not offer it gets the
     * nearest interval it does offer (`rangeInterval`).
     */
    readonly interval: string;
    readonly unit: WidgetRangeUnit;
    /** How many units back, for `session`, `month` and `year`. Default 1. */
    readonly count?: number;
}
/** The window a range covers: UTC seconds, both ends inclusive. */
interface WidgetRangeWindow {
    from: number;
    to: number;
}
interface RangeWindowOptions {
    /**
     * The instant the range ends at, in UTC seconds: the clock when sizing a
     * fetch, the latest bar when placing a view.
     */
    readonly end: number;
    /** The zone a month, a year and a date without a calendar are counted in. Default `DEFAULT_TIMEZONE`. */
    readonly zone?: string;
    /** Trading hours: a `session` range counts this calendar's sessions. */
    readonly calendar?: SessionCalendarSource | null;
    /**
     * Bars already loaded, oldest first. Without a calendar a `session` range
     * counts the dates they fall on, and `all` starts at the first of them.
     */
    readonly bars?: readonly {
        readonly time: number;
    }[] | undefined;
}
/**
 * The presets the bottom bar offers when the host names none. Each pairs a
 * span with an interval that draws it in a few hundred bars, not tens of
 * thousands: a day at one minute, a year at one day.
 */
declare const DEFAULT_RANGES: readonly WidgetRange[];
/**
 * The interval a range is shown at, from those `offered`: its own when it is
 * offered, else the nearest time-based one by ratio (a 30-minute range picks
 * 15 minutes or an hour, not 5 minutes), the longer on a tie so the range
 * loads fewer bars. Its own interval when nothing offered is time-based.
 */
declare function rangeInterval(range: WidgetRange, offered: readonly string[]): string;
/**
 * The window a range covers, ending at `options.end`. A `session` range walks
 * back through the calendar's sessions when there is one; without one it
 * counts the dates of the loaded bars, or weekdays when none are loaded. A
 * month or a year runs from midnight on the same date that far back, the year
 * to date from the first of January, all history from the first loaded bar
 * (or 30 years back before any are loaded). Months, years and dates without a
 * calendar are counted in `options.zone`.
 */
declare function rangeWindow(range: WidgetRange, options: RangeWindowOptions): WidgetRangeWindow;

/**
 * The bottom bar: a thin strip under the chart. On the left, preset ranges
 * and Go to, which move the chart through time; on the right, the market
 * status, the clock and the price scale toggles, which say where the chart
 * stands.
 *
 * It acts on a target read at every use rather than on a chart it was given,
 * so one bar serves a grid by pointing at the focused chart, and a host that
 * rebuilds its chart keeps its bar. Everything it shows is re-read from that
 * target: the range from `range()`, the scale from `priceAxisState`, the zone
 * from `timezone()`, the status from the chart's session calendar. It keeps
 * no copy that could drift from the chart, and no control here does anything
 * the chart's public API does not.
 *
 * Three rules it follows:
 *
 * - **One clock.** The time, the market status and the scale toggles are
 *   re-read on one timer, once a second, stopped while the page is hidden.
 *   The status is worked out again only when it can have changed.
 * - **A control it cannot back is shown greyed, not dropped.** A target with
 *   no ranges greys the range buttons, an interval without time buckets greys
 *   Go to, a chart with no price scale to act on greys the scale toggles.
 * - **Its menus open in the host's overlay stack**, so Escape, focus return
 *   and outside presses behave as they do for every other popup, and a bar at
 *   the bottom of the page opens its menu upward.
 */

/** The bar's height in CSS px. */
declare const BOTTOMBAR_HEIGHT = 28;
/**
 * What the bar needs from its host: a document, an overlay stack and tips for
 * its menus and labels, a status line for what a range did, and translations.
 * A `WidgetContext` is one; a custom host builds one from `createOverlayStack`
 * and `createTipController` over an `.oac-widget` root.
 */
interface BottombarContext extends WidgetTranslationOptions {
    readonly document: Document;
    /** BCP 47 tag for weekday names. Default: the runtime's. */
    readonly locale?: string | undefined;
    readonly tips: TipController;
    openOverlay(el: HTMLElement, opts?: OverlayOptions): () => void;
    /** Report what a control did, and why a range fell short. */
    status(text: string, kind?: 'info' | 'error'): void;
    /** The chart the bar acts on when `BottombarOptions.target` is not given. */
    readonly chart?: Chart;
}
/**
 * The chart the bar acts on, and the widget's range methods when it has them.
 * A `Widget` is one.
 */
interface BottombarTarget {
    readonly chart: Chart;
    /** The interval in force, for Go to. Default: the chart's data context. */
    interval?(): string;
    /** The range in force, pressed in the bar; null when none is. */
    range?(): string | null;
    /** Apply a range. The bar reports what it returns. Without it the range buttons are greyed. */
    setRange?(id: string): Promise<DateNavigationResult> | DateNavigationResult | void;
}
interface BottombarOptions {
    /**
     * What the bar acts on, read at every use: a grid passes its focused
     * chart. Default: the context's chart, with no ranges.
     */
    target?: () => BottombarTarget | null;
    /** The range buttons. `[]` leaves them out. Default `DEFAULT_RANGES`. */
    ranges?: readonly WidgetRange[] | undefined;
    /** Open the go-to panel from the bar's button. Omit to leave the button out. */
    onGoTo?(anchor: HTMLElement): void | boolean;
    /** Clock for the time and the market status, in milliseconds. Default `Date.now`. */
    now?: (() => number) | undefined;
    /** Zones the timezone menu lists first. Default: those the settings dialog offers. */
    timezones?: readonly string[];
    /** Called after the bar moves the chart to another zone, for a host that keeps its own copy. */
    onTimezone?(zone: string): void;
}
/** The three price scale toggles. */
type BottombarScaleToggle = 'auto' | 'log' | 'percent';
/** Which toggles are on for the price scale the bar acts on. */
interface BottombarScaleState {
    auto: boolean;
    log: boolean;
    percent: boolean;
}
/**
 * The bar's controls without its markup: what the phone layout's sheets show
 * in its place, since the bar itself is hidden there.
 */
interface BottombarControls {
    /** The range buttons, or none when the target cannot take a range. */
    ranges(): readonly WidgetRange[];
    range(): string | null;
    setRange(id: string): void;
    /** The market status, or null without a calendar or with "Session state" off. */
    marketStatus(): MarketStatusReading | null;
    /** The time and the zone's offset, `14:32 UTC+5:30`. */
    clock(): string;
    timezone(): string;
    /** The zones the timezone choice lists, the current one included. */
    timezones(): readonly string[];
    setTimezone(zone: string): void;
    /** Null when the chart has no price scale to act on yet. */
    scale(): BottombarScaleState | null;
    toggleScale(which: BottombarScaleToggle): void;
    /** Called when a range, the zone, the scale or the status changes; returns the unsubscribe. */
    subscribe(listener: () => void): () => void;
}
interface BottombarHandle {
    readonly el: HTMLElement;
    readonly controls: BottombarControls;
    /** Read the target again and repaint: after the host points it elsewhere. */
    refresh(): void;
    destroy(): void;
}
/** Mount the bottom bar into `host`. */
declare function mountBottombar(ctx: BottombarContext, host: HTMLElement, opts?: BottombarOptions): BottombarHandle;
/**
 * The bar's rules, scoped under `.oac-widget` like the rest of the chrome.
 * A widget root carrying the bar takes a fourth row for it, between the chart
 * and the status line; the phone layout hides it, its controls having moved
 * into the sheets. A bar marked `is-kept` has no sheet to move into (a widget
 * without a top bar has no More sheet), so it stays under the chart there and
 * the phone footer takes the last row.
 *
 * On a narrow bar the ranges give way and scroll; the status, the clock and
 * the scale toggles are never cut. The narrowest bar shows the status by its
 * glyph, its reading kept as the status region's name.
 */
declare const BOTTOMBAR_CSS: string;

type MobileMode = 'auto' | 'always' | 'never';
interface MobileOptions {
    mode?: MobileMode | undefined;
    container: HTMLElement;
    intervals: readonly string[];
    topbar: boolean;
    rail: RailHandle | null;
    tools?: readonly string[] | undefined;
    indicators: boolean;
    search?: SymbolSearch | undefined;
    state(): TopbarState;
    onSymbol(symbol: string, exchange?: string): void;
    onInterval(code: string): void;
    onChartType(id: string): void;
    onTheme(): void;
    onSettings(anchor: HTMLElement): boolean;
    onIndicators(anchor: HTMLElement): boolean;
    onObjects(anchor: HTMLElement): boolean;
    onDataWindow?(anchor: HTMLElement): void | boolean;
    onAlerts?(anchor: HTMLElement): boolean;
    onWatchlist?(anchor: HTMLElement): void | boolean;
    onNews?(anchor: HTMLElement): void | boolean;
    onCapture?(anchor: HTMLElement): void;
    onGoTo?(anchor: HTMLElement): void | boolean;
    /** Open the Layouts menu, centred. Omitted without a store. Since 2.5.10. */
    onLayouts?(anchor: HTMLElement): void | boolean;
    onProperties(anchor: HTMLElement): boolean;
    settingsAvailable(): boolean;
    indicatorsAvailable(): boolean;
    /**
     * The bottom bar's controls. The phone layout hides the bar, so the More
     * sheet lists them instead: the market status and the clock, the preset
     * ranges, the price scale toggles and the timezone.
     */
    bottombar?: BottombarControls | undefined;
}
interface MobileHandle {
    readonly el: HTMLElement;
    active(): boolean;
    refresh(): void;
    destroy(): void;
}
/** Mount the narrow widget controls against the same chart and controllers as the desktop chrome. */
declare function mountMobile(ctx: WidgetContext, opts: MobileOptions): MobileHandle;

type EventDetailsLoader = (event: ChartEvent, context: {
    signal: AbortSignal;
}) => Promise<ChartEventDetails | string | null>;
/** A button under an event's details. */
interface EventDetailAction {
    /** Shown as text. */
    label: string;
    /** Called with a copy of the shown event, once the popup has closed. */
    run(event: ChartEvent): void;
}
interface EventDetailsLabels {
    title: string;
    close: string;
    events: string;
    loading: string;
    empty: string;
    error: string;
}
interface EventDetailsPopupOptions {
    /** Host-owned lookup. No request is made unless a loader is supplied. */
    loadDetails?: EventDetailsLoader;
    /** Receives the original UTC timestamp in seconds. Defaults to Asia/Kolkata. */
    formatTime?: (time: number) => string;
    labels?: Partial<EventDetailsLabels>;
    /** The buttons for an event, asked each time one is shown. None by default. */
    actions?: (event: ChartEvent) => readonly EventDetailAction[];
    /** Pass the widget context's stack to share focus, Escape and shortcut handling. */
    overlays?: OverlayStack;
    styleNonce?: string | undefined;
    /** Set false when the host already includes EVENT_DETAILS_CSS. */
    injectStyles?: boolean;
}
declare const EVENT_DETAILS_CSS = "\n.oac-event-layer{position:absolute;inset:0;pointer-events:none;z-index:50}\n.oac-event-details{position:absolute;pointer-events:auto;box-sizing:border-box;width:340px;max-width:calc(100% - 16px);max-height:calc(100% - 16px);overflow:auto;background:var(--oac-panel,#171b23);color:var(--oac-tx,#d9dfe8);border:1px solid var(--oac-bd,#343b48);border-radius:8px;box-shadow:var(--oac-shadow,0 12px 32px #0005);font:12px/1.5 var(--oac-font,system-ui,sans-serif);scrollbar-width:thin;scrollbar-color:var(--oac-sb-thumb,#414958) var(--oac-panel,#171b23)}\n.oac-event-details::-webkit-scrollbar{width:6px;height:6px}\n.oac-event-details::-webkit-scrollbar-track{background:var(--oac-panel,#171b23)}\n.oac-event-details::-webkit-scrollbar-thumb{background:var(--oac-sb-thumb,#414958);border-radius:3px}\n.oac-event-details::-webkit-scrollbar-thumb:hover{background:var(--oac-sb-thumb-hover,#586276)}\n.oac-event-details__header{display:flex;align-items:center;justify-content:space-between;gap:12px;padding:10px 12px;border-bottom:1px solid var(--oac-bd,#343b48)}\n.oac-event-details__heading{margin:0;font:600 12px/1.5 var(--oac-font,system-ui,sans-serif)}\n.oac-event-details button{font:inherit;cursor:pointer;border:1px solid var(--oac-bd,#343b48);border-radius:4px;padding:4px 8px;background:var(--oac-elev,#252c37);color:inherit;text-align:left}\n.oac-event-details button:hover{background:var(--oac-elev-2,#333d4b)}\n.oac-event-details button:focus-visible{outline:2px solid var(--oac-ring,#74a7fa);outline-offset:2px}\n.oac-event-details__members{display:flex;flex-wrap:wrap;gap:6px;padding:10px 12px 0}\n.oac-event-details__members button{max-width:100%;overflow-wrap:anywhere}\n.oac-event-details__members button[aria-pressed=\"true\"]{border-color:var(--oac-acc,#74a7fa);background:var(--oac-on-bg,#274263)}\n.oac-event-details__content{padding:12px;overflow-wrap:anywhere}\n.oac-event-details__title{margin:0 0 4px;font-size:14px;font-weight:600}\n.oac-event-details__time,.oac-event-details__status{color:var(--oac-mut,#a0aabc)}\n.oac-event-details__summary{white-space:pre-wrap;margin:10px 0 0}\n.oac-event-details__fields{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,1fr);gap:5px 12px;margin:10px 0 0}\n.oac-event-details__fields dt{color:var(--oac-mut,#a0aabc)}\n.oac-event-details__fields dd{margin:0;text-align:right;white-space:pre-wrap}\n.oac-event-details__status{padding:0 12px 10px}\n.oac-event-details__status:empty{display:none}\n.oac-event-details__subhead{margin:10px 0 0;font-size:12px;font-weight:600}\n.oac-event-details__text,.oac-event-details__list{margin:6px 0 0}\n.oac-event-details__list{padding-left:18px}\n.oac-event-details a{color:var(--oac-acc,#74a7fa)}\n.oac-event-details__actions{display:flex;flex-wrap:wrap;justify-content:flex-end;gap:6px;padding:0 12px 12px}\n.oac-event-details__actions:empty{display:none}\n";
/**
 * A selectable event popup. Feed content is untrusted: every string becomes
 * text, and a link opens only an http or https URL. Destroy it when its host
 * is disposed.
 */
declare class EventDetailsPopup {
    private readonly _container;
    private readonly _options;
    readonly element: HTMLElement;
    private readonly _doc;
    private readonly _overlays;
    private readonly _ownOverlays;
    private readonly _labels;
    private readonly _formatTime;
    private _close;
    private _abort;
    private _revision;
    private _disposed;
    private _events;
    private _members;
    private _content;
    private _status;
    private _actions;
    private _anchor;
    constructor(_container: HTMLElement, _options?: EventDetailsPopupOptions);
    /** Anchor coordinates are CSS pixels relative to the supplied container. */
    open(details: EventMarkerDetails, anchor?: {
        x: number;
        y: number;
    }): void;
    close(): void;
    destroy(): void;
    private _cancel;
    private _select;
    private _render;
    /** Text only: a feed's string never reaches the page as markup, and a link only as a vetted URL. */
    private _inline;
    private _position;
}

type PanelDockId = 'data' | 'objects' | 'watchlist' | 'news';
interface PanelDockState {
    panel: PanelDockId | null;
    width: number;
}
interface PanelDockContent {
    initialFocus?: HTMLElement;
    destroy(): void;
}
interface PanelDockOptions {
    data(host: HTMLElement): PanelDockContent;
    objects(host: HTMLElement): PanelDockContent;
    /** Named symbol lists. Its tab appears only when this is supplied. */
    watchlist?(host: HTMLElement): PanelDockContent;
    /** The chart instrument's news. Its tab appears only when this is supplied. */
    news?(host: HTMLElement): PanelDockContent;
    state?: unknown;
    onChange?(state: PanelDockState): void;
}
interface PanelDockHandle {
    el: HTMLElement;
    open(panel: PanelDockId, focus?: boolean): void;
    close(): void;
    toggle(panel: PanelDockId): void;
    state(): PanelDockState;
    restore(state: unknown): void;
    destroy(): void;
}
/** Older documents have no dock; malformed preferences cannot hide the plot. */
declare function sanitizePanelDockState(raw: unknown): PanelDockState;
/** A host-owned information dock. It mounts just one content observer at a time. */
declare function mountPanelDock(ctx: WidgetContext, stage: HTMLElement, opts: PanelDockOptions): PanelDockHandle;
declare const PANEL_DOCK_CSS = "\n.oac-widget .oac-panel-dock { position: relative; flex: none; min-width: 0; min-height: 0; display: flex; flex-direction: column; background: var(--oac-panel); border-left: 1px solid var(--oac-bd); }\n.oac-widget .oac-panel-dock__header { display: flex; align-items: center; justify-content: space-between; gap: 8px; padding: 9px 10px; border-bottom: 1px solid var(--oac-bd-soft); flex: none; }\n.oac-widget .oac-panel-dock__tabs { display: flex; flex-wrap: wrap; gap: 3px; min-width: 0; }\n.oac-widget .oac-panel-dock__tabs .oac-btn, .oac-widget .oac-panel-dock__close { padding: 0 6px; }\n.oac-widget .oac-panel-dock__tabs .oac-btn { border-color: transparent; background: transparent; }\n.oac-widget .oac-panel-dock__tabs .oac-btn[aria-pressed=\"true\"] { background: var(--oac-elev); color: var(--oac-tx); border-color: var(--oac-bd); }\n.oac-widget .oac-panel-dock__close { color: var(--oac-mut); }\n.oac-widget .oac-panel-dock__body { overflow: auto; min-height: 0; flex: 1 1 auto; overscroll-behavior: contain; }\n.oac-widget .oac-panel-dock__resize { position: absolute; z-index: 2; top: 0; bottom: 0; left: -4px; width: 8px; cursor: col-resize; touch-action: none; }\n.oac-widget .oac-panel-dock__resize:hover, .oac-widget .oac-panel-dock__resize:focus-visible { background: var(--oac-ring-soft); }\n.oac-widget .oac-panel-dock[data-sheet=\"true\"] { position: absolute !important; inset: 0 !important; width: 100% !important; max-width: none !important; height: 100%; max-height: none; transform: none !important; border: 0; border-radius: 0; }\n.oac-widget .oac-panel-dock[data-sheet=\"true\"] .oac-panel-dock__header { padding: 12px; }\n.oac-widget .oac-panel-dock[data-sheet=\"true\"] .oac-btn { min-height: 40px; }\n.oac-widget .oac-panel-dock__body > .oac-objects { width: auto; max-width: none; border: 0; box-shadow: none; }\n";

/**
 * Named watchlists in a dock panel: one table of the active list, whose rows
 * take their prices only from a quote source.
 *
 * Three decisions worth recording:
 *
 * - **A row without a quote shows no price.** The chart's bars are never read
 *   here, because a last candle presented as a quote is a wrong number with
 *   nothing to say so.
 * - **Streams follow the rows on screen.** An intersection observer reports
 *   which rows are visible, and only those hold a subscription; a list switch,
 *   a hidden page or closing the panel releases them.
 * - **Sorting is stable and holds still under the hand.** Ties keep list
 *   order and unknown values sink in either direction. While the pointer is
 *   over the rows, or focus is in them, prices update in place but rows do not
 *   move, so a click cannot land on a row that jumped there a moment before.
 */

type WatchlistSortKey = 'list' | 'symbol' | 'last' | 'change' | 'percent';
interface WatchlistSort {
    key: WatchlistSortKey;
    direction: 'ascending' | 'descending';
}
interface WatchlistPanelOptions {
    /** Named lists: a `WatchlistRepository` from `openalgo-charts/workspace`, or any implementation of its store contract. */
    store: WatchlistStore;
    /** Quotes for the rows on screen. Without one, rows show symbols only. */
    quotes?: QuoteFeed;
    /** A snapshot older than this is shown as stale. Default 60000 ms. */
    staleAfterMs?: number;
    /** Refresh interval for a snapshot-only source. 0 disables. Default 15000 ms. */
    pollMs?: number;
    /** A row was chosen, by click or Enter. */
    onSelect?(instrument: InstrumentKey): void;
    /**
     * The instrument the host charts for an entry. The panel saves what it adds
     * in this form, and compares entries with the chart through it, so a row is
     * marked as, and refused as a repeat of, what choosing it would chart.
     * Default: unchanged. The widget upper-cases the symbol, as its symbol box does.
     */
    normalize?(instrument: InstrumentKey): InstrumentKey;
    /** Price text for a row. Default: the locale's grouping with two to six decimals. */
    formatPrice?(value: number, instrument: InstrumentKey): string;
}
interface WatchlistPanelHandle extends PanelDockContent {
    readonly el: HTMLElement;
    /** Read the lists from the store again, replacing what is shown. */
    reload(): Promise<void>;
}
/** Mount the watchlist into a host element, typically the widget's panel dock. */
declare function mountWatchlistPanel(ctx: WidgetContext, host: HTMLElement, options: WatchlistPanelOptions): WatchlistPanelHandle;
/** Rules to add to the widget's shared stylesheet or a custom host stylesheet. */
declare const WATCHLIST_PANEL_CSS = "\n.oac-widget .oac-watchlist { display: flex; flex-direction: column; gap: 6px; height: 100%; min-height: 0; padding: 8px 10px 0; box-sizing: border-box; font-size: 12px; }\n.oac-widget .oac-watchlist__bar, .oac-widget .oac-watchlist__add, .oac-widget .oac-watchlist__name, .oac-widget .oac-watchlist__confirm { display: flex; align-items: center; gap: 4px; flex: none; min-width: 0; }\n.oac-widget .oac-watchlist__bar .oac-select { flex: 1 1 auto; min-width: 0; }\n.oac-widget .oac-watchlist__bar .oac-select select { width: 100%; min-width: 0; }\n.oac-widget .oac-watchlist .oac-btn { height: 26px; padding: 0 7px; font-size: 11px; flex: none; }\n.oac-widget .oac-watchlist__name[hidden], .oac-widget .oac-watchlist__confirm[hidden], .oac-widget .oac-watchlist__scroll[hidden] { display: none; }\n.oac-widget .oac-watchlist__name input, .oac-widget .oac-watchlist__add input { flex: 1 1 auto; min-width: 0; height: 26px; }\n.oac-widget .oac-watchlist__confirm { flex-wrap: wrap; }\n.oac-widget .oac-watchlist__confirm-text { flex: 1 1 100%; color: var(--oac-tx); overflow-wrap: anywhere; }\n.oac-widget .oac-watchlist__add-current { max-width: 45%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }\n.oac-widget .oac-watchlist__status { flex: none; font-size: 11px; color: var(--oac-mut); }\n.oac-widget .oac-watchlist__status[data-state=\"live\"] { color: var(--oac-tx); }\n.oac-widget .oac-watchlist__status[data-state=\"reconnecting\"], .oac-widget .oac-watchlist__status[data-state=\"disconnected\"], .oac-widget .oac-watchlist__status[data-state=\"error\"] { color: var(--oac-danger); }\n.oac-widget .oac-watchlist__message { flex: none; font-size: 11px; color: var(--oac-danger); overflow-wrap: anywhere; }\n.oac-widget .oac-watchlist__empty { margin: 6px 0; font-size: 11px; color: var(--oac-mut); }\n.oac-widget .oac-watchlist__scroll { flex: 1 1 auto; min-height: 0; overflow: auto; overscroll-behavior: contain; margin: 0 -10px; }\n/* Numbers take the width they need and the symbol the rest, so a price is never the part cut short. */\n.oac-widget .oac-watchlist__table { width: 100%; border-collapse: collapse; font-variant-numeric: tabular-nums; }\n.oac-widget .oac-watchlist__table thead th { position: sticky; top: 0; z-index: 1; background: var(--oac-panel); border-bottom: 1px solid var(--oac-bd-soft); padding: 0; }\n.oac-widget .oac-watchlist__sort { width: 100%; height: 24px; padding: 0 6px; border: 0; background: none; color: var(--oac-mut); font: 600 10px/1 var(--oac-font); text-transform: uppercase; letter-spacing: .04em; text-align: right; white-space: nowrap; cursor: pointer; }\n.oac-widget .oac-watchlist__col--symbol .oac-watchlist__sort { text-align: left; padding-left: 10px; }\n.oac-widget .oac-watchlist__sort:hover, .oac-widget th[aria-sort=\"ascending\"] .oac-watchlist__sort, .oac-widget th[aria-sort=\"descending\"] .oac-watchlist__sort { color: var(--oac-tx); }\n.oac-widget th[aria-sort=\"ascending\"] .oac-watchlist__sort::after { content: \" \\2191\"; }\n.oac-widget th[aria-sort=\"descending\"] .oac-watchlist__sort::after { content: \" \\2193\"; }\n.oac-widget .oac-watchlist__col--symbol { width: 100%; }\n.oac-widget .oac-watchlist__col--actions { width: 26px; min-width: 26px; }\n.oac-widget .oac-watchlist__row { height: 28px; border-bottom: 1px solid var(--oac-bd-soft); }\n.oac-widget .oac-watchlist__row:hover, .oac-widget .oac-watchlist__row:focus-within { background: var(--oac-elev); }\n/* On the cell: some engines paint no shadow on a table row. */\n.oac-widget .oac-watchlist__row[aria-current=\"true\"] > th { box-shadow: inset 2px 0 0 var(--oac-acc); }\n.oac-widget .oac-watchlist__row td, .oac-widget .oac-watchlist__row th { padding: 0 6px; text-align: right; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-weight: 400; }\n.oac-widget .oac-watchlist__row .oac-watchlist__instrument { padding: 0; max-width: 0; text-align: left; }\n.oac-widget .oac-watchlist__open { display: block; width: 100%; height: 28px; line-height: 28px; padding: 0 6px 0 10px; border: 0; background: none; color: var(--oac-tx); font: inherit; text-align: left; cursor: pointer; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }\n.oac-widget .oac-watchlist__symbol { font-weight: 600; }\n.oac-widget .oac-watchlist__exchange { margin-left: 5px; font-size: 10px; color: var(--oac-mut); }\n.oac-widget .oac-watchlist__last { color: var(--oac-tx); }\n/* The theme's own pair, lifted to read as text. A fallback colour here would pin one palette. */\n.oac-widget .oac-watchlist .is-up { color: var(--oac-up); }\n.oac-widget .oac-watchlist .is-down { color: var(--oac-down); }\n.oac-widget .oac-watchlist__row[data-state=\"stale\"] td, .oac-widget .oac-watchlist__row[data-state=\"unavailable\"] td, .oac-widget .oac-watchlist__row[data-state=\"error\"] td, .oac-widget .oac-watchlist__row[data-state=\"loading\"] td { color: var(--oac-mut); }\n.oac-widget .oac-watchlist__row[data-state=\"stale\"] .oac-watchlist__last { text-decoration: underline dotted; text-underline-offset: 3px; }\n.oac-widget .oac-watchlist__actions { padding: 0 4px 0 0 !important; }\n.oac-widget .oac-watchlist__actions .oac-btn { width: 22px; height: 22px; padding: 0; border-color: transparent; background: transparent; color: var(--oac-mut); opacity: 0; }\n.oac-widget .oac-watchlist__row:hover .oac-watchlist__actions .oac-btn, .oac-widget .oac-watchlist__row:focus-within .oac-watchlist__actions .oac-btn { opacity: 1; }\n.oac-widget .oac-panel-dock[data-sheet=\"true\"] .oac-watchlist__actions .oac-btn { opacity: 1; width: 32px; height: 32px; }\n.oac-widget .oac-panel-dock[data-sheet=\"true\"] .oac-watchlist__row, .oac-widget .oac-panel-dock[data-sheet=\"true\"] .oac-watchlist__open { height: 40px; line-height: 40px; }\n@media (hover: none) { .oac-widget .oac-watchlist__actions .oac-btn { opacity: 1; } }\n";

/**
 * The chart instrument's news in a dock panel: a list of headlines with their
 * source and time, a detail view, and older pages on request.
 *
 * Provider text is only ever assigned as text, so a headline carrying markup
 * shows the markup rather than running it, and an article opens only through
 * an http or https link that cannot reach back into this page.
 */

interface NewsPanelOptions {
    feed: NewsFeed;
    /** Items asked for per page. Default 20. */
    pageSize?: number;
    /** News older than this since it loaded is flagged stale. Default 300000 ms. */
    staleAfterMs?: number;
    /** Items held at most. Default 500. */
    maxItems?: number;
}
interface NewsPanelHandle extends PanelDockContent {
    readonly el: HTMLElement;
    /** Reload the newest page for the chart's instrument. */
    refresh(): Promise<void>;
}
/** An absolute http or https URL without credentials, normalised, or null. */
declare function safeNewsUrl(value: string | undefined): string | null;
/** Mount the news reader into a host element, typically the widget's panel dock. */
declare function mountNewsPanel(ctx: WidgetContext, host: HTMLElement, options: NewsPanelOptions): NewsPanelHandle;
/** Rules to add to the widget's shared stylesheet or a custom host stylesheet. */
declare const NEWS_PANEL_CSS = "\n.oac-widget .oac-news { display: flex; flex-direction: column; gap: 6px; padding: 8px 10px 10px; font-size: 12px; min-height: 0; }\n.oac-widget .oac-news__bar { display: flex; align-items: center; justify-content: space-between; gap: 8px; }\n.oac-widget .oac-news__heading { display: flex; align-items: baseline; gap: 5px; min-width: 0; }\n.oac-widget .oac-news__instrument { font-weight: 600; color: var(--oac-tx); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }\n.oac-widget .oac-news__exchange { font-size: 10px; color: var(--oac-mut); }\n.oac-widget .oac-news .oac-btn { height: 26px; padding: 0 8px; font-size: 11px; }\n.oac-widget .oac-news__status { font-size: 11px; color: var(--oac-mut); overflow-wrap: anywhere; }\n.oac-widget .oac-news__status[data-state=\"error\"], .oac-widget .oac-news__more-error { color: var(--oac-danger); }\n.oac-widget .oac-news__status[data-state=\"stale\"] { color: var(--oac-warn, var(--oac-mut)); }\n.oac-widget .oac-news__list { list-style: none; margin: 0; padding: 0; }\n.oac-widget .oac-news__list[hidden], .oac-widget .oac-news__more[hidden], .oac-widget .oac-news__detail[hidden], .oac-widget .oac-news__more > [hidden] { display: none; }\n.oac-widget .oac-news__item { border-bottom: 1px solid var(--oac-bd-soft); }\n.oac-widget .oac-news__open { display: flex; flex-direction: column; gap: 3px; width: 100%; padding: 7px 2px; border: 0; background: none; color: var(--oac-tx); font: inherit; text-align: left; cursor: pointer; }\n.oac-widget .oac-news__open:hover, .oac-widget .oac-news__open:focus-visible { background: var(--oac-elev); }\n.oac-widget .oac-news__headline { font-weight: 600; line-height: 1.35; overflow-wrap: anywhere; display: -webkit-box; -webkit-line-clamp: 3; -webkit-box-orient: vertical; overflow: hidden; }\n.oac-widget .oac-news__meta, .oac-widget .oac-news__detail-meta { display: flex; flex-wrap: wrap; gap: 8px; font-size: 11px; color: var(--oac-mut); }\n.oac-widget .oac-news__source { overflow-wrap: anywhere; }\n.oac-widget .oac-news__more { display: flex; flex-direction: column; align-items: stretch; gap: 4px; padding-top: 4px; font-size: 11px; color: var(--oac-mut); text-align: center; }\n.oac-widget .oac-news__more-error { margin: 0; text-align: left; overflow-wrap: anywhere; }\n.oac-widget .oac-news__detail { display: flex; flex-direction: column; align-items: flex-start; gap: 8px; }\n.oac-widget .oac-news__detail-headline { margin: 0; font: 600 14px/1.35 var(--oac-font); color: var(--oac-tx); overflow-wrap: anywhere; }\n.oac-widget .oac-news__detail-meta { margin: 0; }\n.oac-widget .oac-news__detail-summary { margin: 0; white-space: pre-line; line-height: 1.5; color: var(--oac-tx); overflow-wrap: anywhere; }\n.oac-widget .oac-news__article { display: inline-flex; align-items: center; text-decoration: none; }\n.oac-widget .oac-news__no-link { margin: 0; font-size: 11px; color: var(--oac-mut); }\n.oac-widget .oac-panel-dock[data-sheet=\"true\"] .oac-news__open { padding: 10px 2px; }\n";

/**
 * The widget's side of the bottom bar: the options it reads, the preset
 * ranges it applies, the session calendar it hands the chart and the shading
 * it attaches from that calendar.
 *
 * Its own module so widget.ts keeps only the hooks: a mount call, the load
 * window a range widens, and the two public methods. The shell calls
 * `attachBottombar` with itself as `this`, typed `BottombarHost`, for the
 * reasons widget-keys.ts gives: a member the shell renames or retypes fails
 * to compile here.
 *
 * A range is the widget's, not the bar's: `setRange` works with the bar
 * turned off, and a grid cell (which has none) is driven by the grid's bar
 * through it. The range holds while its interval does. A symbol change keeps
 * it and places it again on the new instrument; an interval changed by any
 * other means leaves it, and loads go back to the ordinary lookback.
 */

/**
 * Trading hours for the widget's instruments: one calendar for every symbol,
 * or a function asked on each symbol change. A `SessionCalendar` or an
 * `Instrument` is one; null means none.
 */
type WidgetSessionCalendar = SessionCalendarSource | null | ((instrument: {
    readonly symbol: string;
    readonly exchange: string;
}) => SessionCalendarSource | null | undefined);
/** The widget options the bottom bar and the session calendar read. */
interface WidgetBottombarOptions {
    /**
     * The strip under the chart: preset ranges and Go to on the left, the
     * market status, a clock that opens the timezone choice and the price
     * scale toggles on the right. The phone layout hides it and lists its
     * controls in the More sheet. Default true.
     */
    bottombar?: boolean;
    /** Preset ranges for the bar and `setRange`. `[]` leaves the range buttons out. Default `DEFAULT_RANGES`. */
    ranges?: readonly WidgetRange[];
    /**
     * Trading hours, applied with `chart.setSessionCalendar` for the first
     * symbol and on every symbol change. They size a range's fetch in
     * sessions, give the bar its market status and the shading its hours.
     * Default: the widget leaves the chart's calendar to the host.
     */
    sessionCalendar?: WidgetSessionCalendar;
    /**
     * Shade the pre-open, post-close and extended-hours bars from the chart's
     * calendar (`attachSessionShading`). A calendar without those hours, or no
     * calendar, shades nothing. Default true.
     */
    sessionShading?: boolean;
}

/**
 * The widget shell: a chart with its chrome, built in one call.
 *
 * `createWidget(container, options)` puts a `.oac-widget` root into the
 * container with a top bar, a stage (the tool rail beside the chart) and a
 * status line, creates the chart and the drawing controller inside it, wires
 * the keymap, the overlay stack and the toasts, and hands every mounted piece
 * one `WidgetContext`. A host that wants the chart alone still has the engine;
 * this is for the host that wants the terminal.
 *
 * Three decisions worth recording:
 *
 * - **The shell owns the theme, the symbol and the interval; the chart owns
 *   everything else.** The engine has no instrument concept, so symbol and
 *   interval live here, drive the feed from here, and are published on the bus
 *   as `symbol` and `interval` for a host (or a link group) to follow.
 * - **Persisted state is validated field by field and applied to the dataset
 *   it was captured on.** A viewport is a range of bar indices and means
 *   nothing on different bars, so a saved layout landing on another symbol
 *   keeps its indicators and panes and drops its view. Its drawings belong
 *   to the symbol it was saved on (`drawingScope`): each instrument keeps its
 *   own, and a layout from before that attaches its drawings to its symbol.
 * - **Every chord goes through one keymap in the capture phase.** The rail,
 *   the editing keys and the tool chords register there with a scope, so a
 *   dialog being open or the focus being in the rail is decided once, not in
 *   every listener.
 */

/** The intervals offered when the host names none: the registry's codes are appended. */
declare const DEFAULT_INTERVALS: readonly string[];
/** Bars asked of the feed per load when the host names no lookback. */
declare const DEFAULT_LOOKBACK_BARS = 500;
/** Debounce on writing the persisted layout, because drags fire per frame. */
declare const SAVE_DEBOUNCE_MS = 250;
/** The storage entry the layout lives under. */
declare const STATE_KEY = "state";
/**
 * Where each instrument's drawings live beside the layout: this prefix, then
 * the instrument's key (`instrumentDrawingsKey`), as in `drawings:NSE:INFY`.
 */
declare const DRAWINGS_KEY_PREFIX = "drawings:";
declare const WIDGET_STATE_VERSION = 1;
/** Named lists and their quotes for the docked watchlist. A chosen row charts that instrument. */
type WidgetWatchlistOptions = Omit<WatchlistPanelOptions, 'onSelect' | 'normalize'>;
/** The news source for the docked reader, which follows the chart's instrument. */
type WidgetNewsOptions = NewsPanelOptions;
interface WidgetOptions extends Omit<ChartOptions, 'theme'>, WidgetBottombarOptions {
    /** Docked Data and Objects panels. False retains the original Objects dialog. Default true. */
    panels?: boolean;
    /** Unclaimed letters and digits open symbol and interval entry on the focused chart. Default true. */
    typingNavigation?: boolean;
    /**
     * The `?` panel lets the user change the widget's and the chart's chords,
     * kept in `storage` when `persist` is on. False lists them only and applies
     * no saved chords. Default true.
     */
    shortcutsEditor?: boolean;
    /**
     * A docked watchlist: named lists from a store (a `WatchlistRepository` from
     * `openalgo-charts/workspace`), with prices only from `quotes`. Needs `panels`.
     */
    watchlist?: WidgetWatchlistOptions;
    /** A docked reader for the chart instrument's news. Needs `panels`. */
    news?: WidgetNewsOptions;
    /** Event marker clicks open details. Set false to provide a host-owned view. */
    eventDetails?: false | EventDetailsPopupOptions;
    /** Where bars come from. Without one the chart shows what the host sets on `widget.series` itself. */
    feed?: DataFeed | undefined;
    /** Shared history, paging and recovery options. `now` here uses UTC seconds. */
    loading?: DataLoadingOptions;
    symbol?: string | undefined;
    /** Exchange passed to the feed with the symbol. Default `''`. */
    exchange?: string | undefined;
    /** Interval code the registry knows (a built-in token or one passed to `registerInterval`). Default `1d`. */
    interval?: string | undefined;
    /**
     * Which of the feed's series to show: a session, an adjustment, a currency
     * or a unit. Default: the feed's own default series. The feed must declare
     * it through `dataVariants`, or the chart reports it unsupported.
     */
    variant?: DataVariant | undefined;
    /** The interval pills, each a known code. Default: `DEFAULT_INTERVALS` plus every registered code. */
    intervals?: readonly string[];
    /**
     * Primary chart type. Default `candlestick`. A registered renderer, or a
     * transform the chart applies (`registeredSeriesTransforms`, once the
     * transform tier is imported).
     */
    chartType?: string | undefined;
    /** `dark` (default), `light`, or a full `ChartTheme`; the chrome derives its palette from it. */
    theme?: WidgetThemeName | ChartTheme;
    /** The drawing rail. `false` hides it; an object restricts its tools or seeds its pins. Default on. */
    rail?: boolean | RailOptions;
    topbar?: boolean;
    statusline?: boolean;
    /** Narrow controls. Auto activates when the container is at most 640 CSS px wide, or, with a coarse primary pointer, at most 960 px wide and under 600 px tall (a phone on its side); a tablet or a touch laptop keeps the desktop chrome. Default auto. */
    mobile?: MobileMode;
    /**
     * Keep the layout, the rail preferences, the symbol, the interval and the
     * theme between visits. `true` uses one shared namespace; a string names one,
     * so two widgets on a page keep separate layouts. Default off.
     */
    persist?: boolean | string;
    /**
     * The store behind `persist`. Default: IndexedDB where the page has it
     * (since 2.5.10), else the page's `localStorage`. Over an asynchronous
     * store (one with `entries`, such as `createIndexedDbWidgetStorage`) the
     * saved layout lands when `ready` settles; a synchronous one, such as
     * `localStorage` passed here, applies it before `createWidget` returns.
     */
    storage?: StorageLike | AsyncStorageLike | null;
    /**
     * Whose drawings the chart shows. `'instrument'` (default): each symbol and
     * exchange keeps its own, saved when the chart moves to another instrument
     * and brought back when it returns, which is what a trader drawing levels
     * expects. `'chart'`: one drawing set that stays on screen whatever
     * instrument is loaded, as earlier releases did, for a host that wants
     * drawings to follow the chart.
     */
    drawingScope?: 'instrument' | 'chart';
    /**
     * Where the drawings of each instrument are kept in `'instrument'` scope.
     * Default: beside the layout in `storage` when `persist` is on
     * (`DRAWINGS_KEY_PREFIX`), else in memory for the life of the widget.
     */
    drawingStore?: DrawingDocumentStore;
    /** Saved drawing looks, a tool's default and named templates: a `DrawingTemplateRepository` from `openalgo-charts/workspace`. */
    drawingTemplates?: DrawingTemplateStore;
    /**
     * Saved layouts and indicator templates: a `WorkspaceRepository` from
     * `openalgo-charts/workspace`, or a host's own `WorkspaceStore`. It adds the
     * Layouts menu (top bar, and the More sheet on a phone) and templates in the
     * indicator picker, and reopens the layout that was active when the page
     * last closed. Taken as a type only: the workspace tier loads with the store.
     */
    workspaces?: WorkspaceStore;
    /** What the Layouts menu drives: default a controller over this widget; false for no menu. A chart grid decides what its charts get (`ChartGridOptions.layouts`). */
    layouts?: LayoutsController | false;
    /** The floating toolbar over the selected drawings on a desktop layout. Default: shown with the rail. */
    drawingToolbar?: boolean;
    /**
     * More rows at the end of the capture menu, read each time it opens; a
     * string starts a group. The chart grid adds its whole-grid capture here.
     */
    captureRows?: () => ReadonlyArray<MenuRow | string>;
    /** BCP 47 tag for the numbers on the status line. Default: the runtime's. */
    locale?: string;
    /** Host translations for widget chrome and dialogs, with English fallback. */
    translate?: WidgetTranslator;
    /** Show the Indicators button. Default true. */
    indicators?: boolean;
    /** Symbol lookup for the top bar's box, called as the user types. Default: the feed's `searchSymbols`, when it has one. */
    symbolSearch?: SymbolSearch;
    /** How many bars a load asks the feed for. Default `DEFAULT_LOOKBACK_BARS`. */
    lookbackBars?: number;
    /**
     * The widget's wall clock, in epoch milliseconds: the history load window,
     * the loading controller's clock (unless `loading.now` gives it one, in UTC
     * seconds), the status line and the bottom bar's clock and ranges. Default
     * `Date.now`. It shadows `ChartOptions.now`, the chart's animation clock
     * (monotonic, `performance.now` by default), which the widget does not pass
     * to its chart: its kinetic animation runs on the real clock.
     */
    now?: () => number;
    /** Order entry from the right-click menu. Without it the menu draws no trade rows. */
    onOrder?: (order: OrderRequest) => void;
    /** Supported host order routes, optionally resolved again for each request. */
    tradingCapabilities?: TradingCapabilitySource;
    /** The requested execution mode when the host capabilities constrain it. */
    tradingMode?: TradingCapabilityRequest['mode'];
    /** Locks order entry during host replay selection or workspace transitions. */
    tradingLocked?: () => boolean;
    /**
     * Account state for the status line, usually a trade-tier `AccountManager`.
     * Omitted shows no account; a source whose provider declares no accounts is
     * shown disabled with the reason. It only reads and switches accounts.
     */
    account?: AccountStateSource;
    /** Host CSP nonce for the widget and dialog stylesheet, assigned before insertion. */
    styleNonce?: string;
    /**
     * Which widget answers the keyboard when a host shows several. True sends
     * chords here as if the chart had focus, false silences this widget and its
     * chart's shortcuts, and undefined leaves the pointer and the focus to
     * decide, as they do for a lone widget. The chart grid supplies it per cell.
     */
    keyboardRoute?: () => boolean | undefined;
}
type WidgetChartState = ReturnType<Chart['getState']>;
/** What `getState` returns and `restoreState` takes. JSON-safe. */
interface WidgetState {
    version: typeof WIDGET_STATE_VERSION;
    symbol: string;
    exchange: string;
    interval: string;
    chartType: string;
    theme: WidgetThemeName;
    /** The data variant, absent for the feed's default. Optional in older records. */
    variant?: DataVariant;
    chart: WidgetChartState;
    rail: RailPrefs | null;
    /** Optional in older records. Width is bounded when restored. */
    panels?: PanelDockState | undefined;
    /**
     * The library version that wrote the state (since 2.6.3). Absent in older
     * records, which wrote every navigation option whether chosen or not.
     */
    savedBy?: string;
}
interface WidgetRestoreReport {
    applied: boolean;
    reason?: string | undefined;
    /** The engine's own report for the chart half, when it was reached. */
    chart?: RestoreReport;
}
type WidgetEventName = 'symbol' | 'interval' | 'variant' | 'theme' | 'layout' | 'data' | 'status';
interface Widget {
    /** Managed data owner, or null when the host supplies series data directly. */
    readonly dataController: DataLoadingController | null;
    readonly chart: Chart;
    readonly draw: DrawingController;
    /**
     * What keeps the drawings per instrument, or null in `'chart'` scope. A
     * host reads another instrument's drawings through it (`document`), or
     * writes them now (`save`).
     */
    readonly instrumentDrawings: InstrumentDrawings | null;
    /** The template store's catalog as the widget holds it, or null without a `drawingTemplates` store. */
    readonly drawingTemplates: DrawingTemplates | null;
    /** The controller behind the Layouts menu, or null without one (`workspaces`, `layouts`). */
    readonly layouts: LayoutsController | null;
    readonly alerts: AlertController;
    /**
     * Settles once the persisted layout has been applied and the first load
     * has started (since 2.5.10). At once without `persist` or over a
     * synchronous store; over an asynchronous one (IndexedDB, the default)
     * when it has been read, and until then the widget stays out of sight and
     * shows the defaults. Never rejects: a store that fails is reported on the
     * status line and the widget opens on its defaults. A host that reads or
     * edits the restored layout (`getState`, the indicators, the view) awaits it.
     */
    readonly ready: Promise<void>;
    /** Shared inventory and supported actions for drawings, indicators and registered profiles. */
    readonly objects: ChartObjects;
    /**
     * One undo timeline for the chart: studies, their settings, the chart type,
     * price scales, panes and drawings. Ctrl+Z, Ctrl+Y, the rail and the mobile
     * controls all walk it. A host's own change that should not be a step goes
     * through `history.ignore`; loading a layout with `restoreState` clears it.
     */
    readonly history: ChartHistory;
    /** The `.oac-widget` element. */
    readonly root: HTMLElement;
    /** What every mounted piece was handed; a host mounting its own panel wants the same. */
    readonly context: WidgetContext;
    /** The primary series; its handle stays stable across chart-type changes. */
    readonly series: SeriesApi;
    symbol(): string;
    exchange(): string;
    interval(): string;
    /** The data variant in use, undefined for the feed's default series. */
    variant(): Readonly<DataVariant> | undefined;
    chartType(): string;
    theme(): WidgetThemeName;
    setSymbol(symbol: string, exchange?: string): void;
    setInterval(code: string): void;
    /**
     * Show another of the feed's series for the same instrument. A change is a
     * new source: the load in flight is cancelled, the bars are cleared and the
     * variant is loaded, or reported unsupported when the feed does not declare
     * it. Undefined returns to the default. Throws a TypeError for a malformed one.
     */
    setDataVariant(variant: DataVariant | undefined): void;
    /**
     * Select a chart type while retaining series state: a renderer, or a
     * transform the chart applies to the bars the widget loads (Heikin Ashi,
     * Renko, range bars, line break, point and figure, Kagi). A widget whose host
     * feeds `series` itself keeps point and figure and Kagi as renderers over the
     * elements that host prepares, as before.
     */
    setChartType(id: string): void;
    setTheme(theme: WidgetThemeName | ChartTheme): void;
    /** Open the settings dialog. False when the dialog tier has not registered one. */
    openSettings(): boolean;
    openIndicatorPicker(): boolean;
    /** Open the searchable object inventory. False after destruction. */
    openObjects(): boolean;
    /** Show candle and study readings. False when panels are disabled or after destruction. */
    openDataWindow(): boolean;
    /** Open trader alerts and their lifecycle states. False after destruction. */
    openAlerts(): boolean;
    /** Open the docked watchlist. False without a `watchlist` source, with panels off, or after destruction. */
    openWatchlist(): boolean;
    /** Open the docked news reader. False without a `news` source, with panels off, or after destruction. */
    openNews(): boolean;
    /** Open the Layouts menu. False without a `workspaces` store (or `layouts` controller), or after destruction. */
    openLayouts(): boolean;
    getState(): WidgetState;
    restoreState(state: unknown): WidgetRestoreReport;
    /** Load (or reload) bars from the feed for the current symbol and interval. */
    reload(): Promise<void>;
    /**
     * Show a date, or an explicit UTC range, after loading the older history it
     * needs through the feed. Waits for a load in flight; a newer request, a
     * symbol or interval change, a pan or zoom while history loads, or
     * destruction settles it `cancelled`.
     */
    goTo(target: DateNavigationTarget): Promise<DateNavigationResult>;
    /** Open the go-to panel. False after destruction or on an interval without time buckets. */
    openDateNavigation(): boolean;
    /** Show a preset range (`WidgetOptions.ranges`): its interval, its span in view, and the history that needs. One wider than the plot keeps its latest bars in view (`clipped`). */
    setRange(id: string): Promise<DateNavigationResult>;
    /** The preset range in force, or null: none was set, or the interval has changed since. */
    range(): string | null;
    on<K extends WidgetEventName>(event: K, cb: (payload: WidgetBusEvents[K]) => void): () => void;
    off<K extends WidgetEventName>(event: K, cb?: (payload: WidgetBusEvents[K]) => void): void;
    destroy(): void;
    readonly isDestroyed: boolean;
}
/**
 * The saved chart state without what describes the old view: the viewport
 * and every pinned price range go, the indicators, drawings and pane weights
 * stay. For a layout about to land on a different dataset.
 */
declare function stripView(state: WidgetChartState): WidgetChartState;
/** Resolve a theme option to the engine palette and the chrome mode. */
declare function resolveTheme(t: WidgetThemeName | ChartTheme | undefined): {
    theme: ChartTheme;
    name: WidgetThemeName;
};
/** The window the feed is asked for: `lookback` bars back from now, or five years for a non-time bucketing. */
declare function loadWindow(interval: string, lookback: number, nowSec: number): {
    from: number;
    to: number;
};
/**
 * Build a widget inside `container`: an element, or a selector (or id)
 * resolved against `options.document` or the page.
 */
declare function createWidget(container: HTMLElement | string, options?: WidgetOptions): Widget;

/**
 * The chart grid's link groups: which charts link with which, on which
 * channels, and how a group reads and writes itself in a workspace.
 *
 * Each group is one engine `LinkGroup` (crosshair, time range, symbol,
 * interval, chart type, appearance) and one draw tier `DrawingLinkGroup`
 * that the link group's drawings switch lets each member join. A chart is in
 * one group or in none. The grid starts with one group holding every chart,
 * which is the whole desk linking as it did before groups existed, and it
 * writes a workspace exactly as it did then until the user makes a second
 * group, names one or takes a chart out.
 *
 * Three decisions carry it:
 *
 * - **A group lasts as long as its charts.** The grid drops a group the
 *   moment its last chart leaves, so no group keeps an instrument, interval or
 *   chart type nobody is showing for the next chart to adopt: a new group
 *   starts from the chart that makes it.
 * - **A group's letter is its own.** Letters are handed out as groups are
 *   made and kept, so group C stays C when group B empties; colour follows the
 *   letter and never carries the meaning alone.
 * - **Older readers get only what every chart can follow together.** With
 *   groups written, the flat channels a reader from before groups applies to
 *   the whole desk are on only when one group holds every chart; otherwise
 *   they are all off, which can neither refuse a desk nor pull one chart onto
 *   another's instrument.
 */

/** One link group as the grid reports it. */
interface ChartGridLinkGroup {
    /** What each chart's `linkGroup` and a saved pane's `linkGroup` name. */
    readonly id: string;
    /** The group's name: the one given it, or `Group A` and on, in the host's language. */
    readonly name: string;
    /** The letter its charts are marked with, A to P. */
    readonly letter: string;
    readonly links: ResolvedLinkOptions;
    /** The pane ids of its charts, in reading order. */
    readonly cells: readonly string[];
}

interface ChartGridOptions extends Omit<WidgetOptions, 'persist' | 'storage' | 'keyboardRoute' | 'feed' | 'drawingStore' | 'captureRows'> {
    /**
     * Where the charts load bars: one feed for every chart, or a function that
     * builds each chart's feed from its pane id and the `historyPeriod` its
     * workspace pane carries, for a host whose source answers by period.
     */
    feed?: DataFeed | ((chart: {
        readonly id: string;
        readonly historyPeriod?: string | undefined;
    }) => DataFeed);
    /** The layout to start with when nothing is restored: a preset or any `CHART_GRID_LAYOUTS` id. Default `1x1`. */
    preset?: ChartGridLayoutId;
    /** Link channels the first group starts with, and every new group. Default: crosshair and viewport on, the rest off. */
    links?: LinkOptions;
    /** At or below this width in CSS px only the active chart shows, with tabs to switch. 0 turns it off. Default 640. */
    compactWidth?: number;
    /** Keep the workspace between visits: `true` for one shared namespace, a string to name one. Default off. */
    persist?: boolean | string;
    /**
     * The store behind `persist`. Default: IndexedDB where the page has it
     * (since 2.5.10), else the page's `localStorage`. Over an asynchronous
     * store the saved desk lands when `ready` settles; a synchronous one, such
     * as `localStorage` passed here, restores it before `createChartGrid` returns.
     */
    storage?: StorageLike | AsyncStorageLike | null;
    /**
     * Show the grid bar over the charts: the layout picker, maximize and
     * restore, the link menu, a capture of every chart and, with `workspaces`,
     * the desk's saved layouts. Default false, so a host with its own controls
     * keeps its page as it was.
     */
    toolbar?: boolean;
    /**
     * One bottom bar under the grid, acting on the active chart: preset ranges,
     * Go to, the market status, the clock and the price scale toggles (since
     * 2.5.10). The charts then leave Go to out of their own bars and the market
     * status off their status lines. Default false, as for `toolbar`: each
     * chart keeps both.
     */
    bottombar?: boolean;
    /**
     * Saved layouts of the whole desk, and indicator templates in every
     * chart's picker: a `WorkspaceRepository` from `openalgo-charts/workspace`,
     * or a host's own `WorkspaceStore`. With the grid bar (`toolbar`) the grid
     * keeps a layouts controller over every chart: a Layouts control in the bar
     * saves and opens the desk, autosave follows it, and the layout that was
     * active when the page last closed opens once `ready` settles, unless the
     * host has applied a workspace by then. Without the bar the charts get their
     * templates only. Taken as a type only.
     */
    workspaces?: WorkspaceStore;
    /**
     * What the grid bar's Layouts control drives instead of the grid's own
     * controller: one the host built over this grid; false for no control.
     * Without the bar (`toolbar`), a controller given here drives each chart's
     * own Layouts button instead. A chart never saves a layout of its own inside
     * a grid.
     */
    layouts?: LayoutsController | false;
    /**
     * The layouts the bar's picker offers, in its order, as `intervals` lists
     * the intervals beside `interval`. Default: every `CHART_GRID_LAYOUTS`
     * entry. Empty: the bar has no Layout control. (`layouts` keeps its widget
     * meaning here: the saved-layouts controller every chart's menu drives.)
     */
    presets?: readonly ChartGridLayoutId[];
}
interface ChartGridCell {
    /** The workspace pane id. */
    readonly id: string;
    readonly widget: Widget;
    /** The `.oac-grid__cell` element the widget lives in. */
    readonly element: HTMLElement;
    readonly row: number;
    readonly column: number;
    readonly rowSpan: number;
    readonly columnSpan: number;
    /**
     * The host's name for how much history the chart loads, from the applied
     * workspace pane or the active chart a preset copied. The grid only keeps it
     * and writes it back; a `feed` function is what honours it.
     */
    readonly historyPeriod?: string;
    /** The id of the link group the chart is in, or null when it links with none. */
    readonly linkGroup: string | null;
}
interface ChartGridLayout {
    rows: number;
    columns: number;
    /** The layout id or saved preset name the layout came from, or null. */
    preset: string | null;
    rowWeights: number[];
    columnWeights: number[];
}
interface ChartGridApplyReport {
    applied: boolean;
    reason?: string;
}
interface ChartGridEvents {
    active: {
        id: string;
    };
    /** `preset`, `weights`, `workspace`, `compact`, `maximize` or `swap`. */
    layout: {
        reason: string;
    };
    /** The active chart's link channels, after any link, group or active chart change. */
    links: ResolvedLinkOptions;
    /** Every chart changed theme, through `setTheme` or one chart's own control. */
    theme: {
        theme: WidgetThemeName;
    };
    [key: string]: unknown;
}
type ChartGridEventName = 'active' | 'layout' | 'links' | 'theme';

interface ChartGrid {
    /** The `.oac-grid` element. */
    readonly root: HTMLElement;
    readonly isDestroyed: boolean;
    /** The cells in reading order: by row, then by column. */
    cells(): readonly ChartGridCell[];
    active(): ChartGridCell;
    /** Make a cell the one the keyboard and shared controls act on. False for an unknown id. */
    setActive(id: string, options?: {
        focus?: boolean;
    }): boolean;
    layout(): ChartGridLayout;
    /**
     * What restoring the persisted workspace did when the grid was built: null
     * when nothing was stored, and until `ready` settles over an asynchronous
     * store. A refused desk stays stored, untouched, until the user changes the grid.
     */
    restored(): ChartGridApplyReport | null;
    /**
     * Settles once the persisted workspace has been read and applied (since
     * 2.5.10): at once without `persist` or over a synchronous store. Over an
     * asynchronous one (IndexedDB, the default) the grid is built from the
     * preset, out of sight and loading nothing, until then; a workspace applied
     * meanwhile wins over the stored one. Never rejects.
     */
    readonly ready: Promise<void>;
    /**
     * Reflow into a layout. The charts that fit keep their state in reading
     * order and new ones copy the active chart's instrument; in an uneven layout
     * the active chart takes the large slot, and stays even when it sat past the
     * charts that fit. Weights reset to the layout's.
     * Throws for an id that is not in `CHART_GRID_LAYOUTS`.
     */
    setPreset(preset: ChartGridLayoutId): void;
    /** Show one chart alone, the active one by default; the others stay alive. False for an unknown id or a single chart. */
    maximize(id?: string): boolean;
    /** Show every chart again after `maximize`. */
    restore(): void;
    /** The maximized chart's id, or null while the grid shows every chart. */
    maximized(): string | null;
    /** Trade the places of two charts, spans included. False for an unknown or repeated id. */
    swap(first: string, second: string): boolean;
    /** The active chart's link channels (the whole desk's until groups are made); every channel off when it links with none. */
    linkOptions(): ResolvedLinkOptions;
    /**
     * Switch channels of the active chart's group, or of every group when it is
     * in none. Switching symbol, interval or chart type linking on adopts the
     * active chart's choice.
     */
    setLinks(patch: LinkOptions): void;
    /** Every link group, in the order they were made, with the charts in each. */
    linkGroups(): readonly ChartGridLinkGroup[];
    /** Move a chart into a group, or out of every group with null. It adopts what the group links on. */
    setLinkGroup(cell: string, group: string | null): boolean;
    /** Start a group with this chart in it; the id, or null for an unknown chart or when sixteen groups exist. */
    addLinkGroup(cell: string, options?: {
        name?: string;
        links?: LinkOptions;
    }): string | null;
    /** Switch channels of one group. False for an unknown group. */
    setGroupLinks(group: string, patch: LinkOptions): boolean;
    /** Name a group, 1 to 120 characters after trimming. */
    renameLinkGroup(group: string, name: string): boolean;
    /**
     * Send a chart's drawings to the charts its group shares drawings with, on
     * the same instrument. Drawings made before the switch went on stay private
     * until shared. The number shared, 0 when there is nowhere to share them.
     */
    shareDrawings(cell: string): number;
    theme(): WidgetThemeName;
    setTheme(theme: WidgetThemeName | ChartTheme): void;
    /** Whether the grid is narrow enough to show one chart at a time. */
    compact(): boolean;
    /**
     * One image of every chart at its place on the grid, at the device pixel
     * ratio. Null while the grid shows one chart at a time (compact or
     * maximized), when the hidden charts would come out blank.
     */
    takeScreenshot(): HTMLCanvasElement | null;
    /** Save `takeScreenshot()` as a PNG. False when there is no image or the runtime cannot save it. */
    downloadScreenshot(filename?: string): boolean;
    getWorkspace(): WorkspacePayload;
    /** Replace every chart with a validated payload, or change nothing and say why. */
    applyWorkspace(payload: WorkspacePayload): ChartGridApplyReport;
    on<K extends ChartGridEventName>(event: K, cb: (payload: ChartGridEvents[K]) => void): () => void;
    destroy(): void;
}
/**
 * Build a chart grid inside `container`: an element, or a selector resolved
 * against `options.document` or the page.
 */
declare function createChartGrid(container: HTMLElement | string, options?: ChartGridOptions): ChartGrid;

/**
 * The chart grid's rules. Kept apart from grid.ts so the shared component
 * sheet can carry them without importing the grid, which imports the shell.
 * Every colour is a widget token, written on the grid root by the grid, apart
 * from the link group hues, which have one value per theme here. The bar's
 * menus bring their own rules when they load (grid-menus.ts).
 */
declare const CHART_GRID_CSS: string;

/**
 * Open the Layouts menu for `controller`: under `anchor`, or centred as a
 * dialog without one and on a phone layout. The menu loads on first use
 * (since 2.5.10), so this resolves with its handle once it is open, and
 * rejects when it could not load or the widget was destroyed by then.
 */
declare function openLayoutsMenu(ctx: WidgetContext, controller: LayoutsController, anchor?: HTMLElement): Promise<PanelHandle>;

/**
 * A `LayoutTarget` for one widget: pass it to `createLayoutsController`. The
 * widget's own `workspaces` option builds one; a host driving its own menu
 * builds it here.
 */
declare function widgetLayoutTarget(widget: Widget): LayoutTarget;

type IndicatorTemplateApplyMode = 'replace' | 'append';
/**
 * Apply a template to the chart in `ctx`: `replace` puts its studies in place
 * of the user's (a study the host keeps stays), `append` adds them in new
 * panes. False when appending found nothing to add. A failure puts the chart
 * back as it was and throws.
 */
declare function applyIndicatorTemplate(ctx: Pick<WidgetContext, 'chart' | 'history'>, store: WorkspaceStore, template: IndicatorTemplateInput, mode: IndicatorTemplateApplyMode, label?: string): boolean;

/**
 * Save the user's studies on the chart as a template: with their panes and
 * scales when the store can capture them, else as the plain study list.
 * Rejects when the chart has none of the user's own, rather than store an
 * empty template.
 */
declare function saveIndicatorTemplate(ctx: Pick<WidgetContext, 'chart'>, store: WorkspaceStore, name: string): Promise<IndicatorTemplateDocument>;

interface ObjectsPanelOptions {
    /** Overrides the widget's inventory for a custom host. The caller owns it. */
    objects?: ChartObjects;
    /** Runs once for either programmatic or overlay dismissal. */
    onClose?: () => void;
}
interface ObjectsPanelContent {
    element: HTMLElement;
    initialFocus: HTMLElement;
    destroy(): void;
}
/** Searchable object tree for a host-owned dock or sheet. */
declare function createObjectsPanelContent(ctx: WidgetContext, opts?: ObjectsPanelOptions): ObjectsPanelContent;
/** Open the object tree in the existing popup interface. */
declare function mountObjectsPanel(ctx: WidgetContext, anchor?: HTMLElement, opts?: ObjectsPanelOptions): PanelHandle;
/** Append beside the shared widget and dialog styles when mounting this panel. */
declare const OBJECTS_PANEL_CSS = "\n.oac-widget .oac-objects { width: 440px; min-width: 0; }\n.oac-widget .oac-objects .oac-dialog__body { display: flex; flex-direction: column; gap: 8px; overflow: hidden; }\n.oac-widget .oac-objects-content { display: flex; flex-direction: column; min-height: 0; gap: 8px; }\n.oac-widget .oac-objects__pane-title { margin: 6px 2px; color: var(--oac-mut); font-size: 11px; text-transform: uppercase; }\n.oac-widget .oac-objects__grouping { display: flex; gap: 4px; }\n.oac-widget .oac-objects__group-name { min-width: 0; width: 100%; }\n.oac-widget .oac-objects__move { min-width: 90px; max-width: 130px; height: 26px; font-size: 11px; }\n.oac-widget .oac-objects__row.is-group-member { margin-left: 10px; border: 0; border-left: 1px solid var(--oac-bd-soft); border-radius: 0; }\n.oac-widget .oac-objects__members { min-width: 0; }\n.oac-widget .oac-objects__list { scrollbar-width: thin; scrollbar-color: var(--oac-bd-soft) var(--oac-bg); }\n.oac-widget .oac-objects__list::-webkit-scrollbar { width: 6px; }\n.oac-widget .oac-objects__list::-webkit-scrollbar-track { background: var(--oac-bg); }\n.oac-widget .oac-objects__list::-webkit-scrollbar-thumb { background: var(--oac-bd-soft); border-radius: 3px; }\n.oac-widget .oac-objects__list::-webkit-scrollbar-thumb:hover { background: var(--oac-mut); }\n.oac-widget .oac-objects__find { width: 100%; min-width: 0; flex: none; }\n.oac-widget .oac-objects__list { min-height: 0; overflow: auto; overscroll-behavior: contain; padding: 2px; }\n.oac-widget .oac-objects__row { position: relative; display: flex; flex-direction: column; gap: 4px; padding: 6px; margin-bottom: 4px;\n  border: 1px solid var(--oac-bd-soft); border-radius: 6px; min-width: 0; }\n.oac-widget .oac-objects__row[draggable=\"true\"] { cursor: grab; }\n.oac-widget .oac-objects__grip { display: none; position: absolute; top: 7px; right: 6px; color: var(--oac-faint); line-height: 0; pointer-events: none; }\n.oac-widget .oac-objects__row[draggable=\"true\"] > .oac-objects__grip { display: block; }\n.oac-widget .oac-objects__grip > svg { width: 14px; height: 14px; fill: none; stroke: currentColor; stroke-width: 2; stroke-linecap: round; }\n.oac-widget .oac-objects__row[draggable=\"true\"] > .oac-objects__summary { padding-right: 18px; }\n.oac-widget .oac-objects__hint { margin: 0 2px; color: var(--oac-faint); font-size: 11px; line-height: 1.4; }\n.oac-widget .oac-objects__row.is-selected { border-color: var(--oac-acc); background: var(--oac-elev); }\n.oac-widget .oac-objects__row.is-drop-before { box-shadow: inset 0 2px 0 var(--oac-acc); }\n.oac-widget .oac-objects__row.is-drop-after { box-shadow: inset 0 -2px 0 var(--oac-acc); }\n.oac-widget .oac-objects__summary { display: flex; flex-direction: column; align-items: flex-start; justify-content: center;\n  gap: 2px; width: 100%; min-width: 0; height: auto; padding: 3px 4px; text-align: left; white-space: normal; }\n.oac-widget .oac-objects__name { font-weight: 600; overflow-wrap: anywhere; }\n.oac-widget .oac-objects__meta, .oac-widget .oac-objects__status { font-size: 11px; color: var(--oac-mut); overflow-wrap: anywhere; }\n.oac-widget .oac-objects__status[data-state=\"error\"] { color: var(--oac-danger); }\n.oac-widget .oac-objects__actions { display: flex; flex-wrap: wrap; gap: 3px; }\n.oac-widget .oac-objects__actions .oac-btn { height: 26px; padding: 0 6px; font-size: 11px; }\n.oac-widget .oac-objects__count { color: var(--oac-mut); font-size: 11px; }\n";

interface DataWindowRow {
    key: string;
    label: string;
    value: string;
    /** Null means absent, never an inferred zero. */
    raw: number | null;
}
interface DataWindowSection {
    id: string;
    title: string;
    rows: DataWindowRow[];
}
interface DataWindowSnapshot {
    symbol: string;
    exchange: string;
    time: number | null;
    timestamp: string;
    sections: DataWindowSection[];
}
interface DataWindowOptions extends WidgetTranslationOptions {
    locale?: string | undefined;
}
interface DataWindowHandle {
    el: HTMLElement;
    refresh(): void;
    destroy(): void;
}
/** Read an exact candle, or the latest when time is omitted. Does not change the crosshair. */
declare function readDataWindow(chart: Chart, time?: number, options?: DataWindowOptions): DataWindowSnapshot;
/** Mount selectable readings in a host-owned panel. No polling or single-callback subscription. */
declare function mountDataWindow(ctx: WidgetContext, host: HTMLElement): DataWindowHandle;
declare const DATA_WINDOW_CSS = "\n.oac-widget .oac-data-window { padding: 12px; font-size: 12px; user-select: text; -webkit-user-select: text; }\n.oac-widget .oac-data-window__instrument { font-weight: 600; color: var(--oac-tx); overflow-wrap: anywhere; }\n.oac-widget .oac-data-window__time { font-size: 11px; color: var(--oac-mut); margin: 4px 0 14px; line-height: 1.5; }\n.oac-widget .oac-data-window__section + .oac-data-window__section { margin-top: 16px; }\n.oac-widget .oac-data-window__title { margin: 0 0 6px; padding: 0; font: 600 11px/1.4 var(--oac-font); color: var(--oac-mut); text-transform: uppercase; letter-spacing: .04em; }\n.oac-widget .oac-data-window__row { display: flex; align-items: baseline; justify-content: space-between; gap: 16px; min-height: 26px; padding: 4px 0; }\n.oac-widget .oac-data-window__label { min-width: 0; overflow-wrap: anywhere; }\n.oac-widget .oac-data-window__value { font-variant-numeric: tabular-nums; text-align: right; overflow-wrap: anywhere; }\n.oac-widget .oac-data-window__value.is-unavailable { font-size: 11px; color: var(--oac-mut); }\n";

/**
 * Quote state for a list of instruments, without DOM.
 *
 * The board holds one stream per visible instrument and nothing for the rest,
 * so a watchlist of hundreds costs only what is on screen. It reports every
 * quote with a status that says how far to trust it: a quote held while its
 * stream reconnects, or one that outlived its row's visibility, is stale until
 * a fresh answer replaces it. It never derives a quote from bars.
 */

/** How far a row's quote can be trusted right now. */
type QuoteRowStatus = 'loading' | 'live' | 'delayed' | 'snapshot' | 'stale' | 'unavailable' | 'error';
/** The whole board: the source kind and the worst stream state among visible rows. */
type QuoteBoardStatus = 'unavailable' | 'idle' | 'snapshot' | 'connecting' | 'live' | 'reconnecting' | 'disconnected' | 'error';
interface QuoteRow {
    instrument: InstrumentKey;
    quote: QuoteSnapshot | null;
    status: QuoteRowStatus;
    /** Local clock in milliseconds when the quote shown arrived. */
    receivedAt: number | null;
}
interface QuoteBoardOptions {
    /** Without one, every row is `unavailable`. */
    feed?: QuoteFeed | null | undefined;
    /** A snapshot older than this is stale. Default 60000 ms. */
    staleAfterMs?: number | undefined;
    /** Refresh interval for visible rows when the source cannot stream. 0 disables. Default 15000 ms. */
    pollMs?: number | undefined;
    now?: () => number;
    /** Called after any change a row or the status shows. */
    onChange?(): void;
}
/** Change and percent change against the provider's reference close, or null when it gave none. */
declare function quoteChange(quote: QuoteSnapshot): {
    change: number;
    percent: number;
} | null;
declare class QuoteBoard {
    private readonly _feed;
    private readonly _staleAfter;
    private readonly _pollMs;
    private readonly _now;
    private readonly _onChange;
    private readonly _entries;
    private _visible;
    private readonly _requests;
    private _seq;
    private _error;
    private _poll;
    private _stale;
    private _resync;
    private _destroyed;
    constructor(options: QuoteBoardOptions);
    /** The instruments on screen. Streams and snapshot requests follow this set exactly. */
    setVisible(instruments: readonly InstrumentKey[]): void;
    row(instrument: InstrumentKey): QuoteRow;
    status(): QuoteBoardStatus;
    /** The last snapshot failure, cleared by the next success. */
    error(): string | null;
    /** Instruments with an open stream, in screen order. */
    subscribed(): InstrumentKey[];
    destroy(): void;
    private _show;
    private _hide;
    private _trim;
    private _accept;
    private _queueResync;
    private _snapshot;
    private _schedulePoll;
    /** When a visible row turns stale by age alone, or null when age cannot change what it shows. */
    private _expires;
    /**
     * One timer, for the next moment a row on screen ages into stale. A deadline
     * already passed is a row already shown stale, so an idle board waits for
     * nothing and repaints nothing.
     */
    private _scheduleStale;
}

/**
 * An instrument's news, without DOM: the newest page, older pages by the
 * provider's cursor, and one lifecycle per instrument. Switching instrument or
 * destroying aborts what is in flight, and an answer that arrives anyway is
 * dropped by generation, because a provider may ignore the signal. Text is
 * kept as plain text; deciding which links are safe to open is the view's.
 */

type NewsStatus = 'idle' | 'loading' | 'ready' | 'empty' | 'error';
interface NewsSnapshot {
    instrument: InstrumentKey | null;
    /** `loading` covers a refresh too; the items already shown stay in `items`. */
    status: NewsStatus;
    /** Newest first, one per provider ID. */
    items: readonly NewsItem[];
    /** The provider offered an older page. */
    hasMore: boolean;
    loadingMore: boolean;
    /** The items shown could not be refreshed, or have outlived the stale window. */
    stale: boolean;
    /** The newest page failed to reload and the items shown are the earlier answer. */
    refreshFailed: boolean;
    /** The latest failure, cleared by the next success. */
    error: string | null;
    /** Local clock in milliseconds of the last newest-page answer. */
    loadedAt: number | null;
}
interface NewsReaderOptions {
    feed: NewsFeed;
    /** Items asked for per page. Default 20. */
    pageSize?: number | undefined;
    /** News older than this since its load is flagged stale. Default 300000 ms. */
    staleAfterMs?: number | undefined;
    /** Items held at most; paging stops there. Default 500. */
    maxItems?: number | undefined;
    now?: () => number;
    onChange?(snapshot: NewsSnapshot): void;
}
declare class NewsReader {
    private readonly _feed;
    private readonly _pageSize;
    private readonly _staleAfter;
    private readonly _maxItems;
    private readonly _now;
    private readonly _onChange;
    private _instrument;
    private _status;
    private _items;
    private _cursor;
    private _loadingMore;
    private _refreshFailed;
    private _error;
    private _loadedAt;
    private _generation;
    private readonly _controllers;
    private _staleTimer;
    private _destroyed;
    constructor(options: NewsReaderOptions);
    /** Follow another instrument. The same identity is a no-op; null clears the list. */
    setInstrument(instrument: InstrumentKey | null): void;
    /** Reload the newest page. The items shown stay until it answers. */
    refresh(): Promise<void>;
    /** Append the next older page, if the provider offered one and none is loading. */
    loadMore(): Promise<void>;
    snapshot(): NewsSnapshot;
    destroy(): void;
    /** One page for the current instrument; undefined when superseded, an Error when it failed. */
    private _request;
    /** Merge a page, newest first, one per ID. Returns how many were new, or 0 once the cap stops paging. */
    private _take;
    /** The next cursor, or null when there is none or it would ask for the same page again. */
    private _nextCursor;
    private _cancel;
    private _scheduleStale;
    private _emit;
}

interface QuickEntryOptions {
    /** The host decides whether this chart currently owns direct typing. */
    enabled(): boolean;
    onSymbol(symbol: string, exchange?: string): void;
    onInterval(code: string): void;
    search?: SymbolSearch | undefined;
}
interface QuickEntryHandle {
    close(): void;
    destroy(): void;
}
/** Start a symbol or interval box from an unclaimed plain key on the focused chart. */
declare function mountQuickEntry(ctx: WidgetContext, options: QuickEntryOptions): QuickEntryHandle;
declare const QUICK_ENTRY_CSS = "\n.oac-widget .oac-quick-entry{width:min(330px,calc(100vw - 32px));padding:12px}\n.oac-widget .oac-quick-entry__input{width:100%;box-sizing:border-box;background:var(--oac-panel);color:var(--oac-tx);border:1px solid var(--oac-bd);border-radius:4px;padding:9px;font:inherit}\n.oac-widget .oac-quick-entry__message{min-height:20px;padding-top:5px;color:var(--oac-danger)}\n";

/**
 * The go-to panel: a date, or a range of two, typed on the chart's own clock
 * and handed to a `DateNavigator` (through `navigate`) as UTC seconds.
 *
 * The fields read the chart's timezone rather than the browser's, for the
 * reason the alert expiry does: the axis the user reads is labelled in that
 * zone, so a date typed in any other would land hours away from the candle
 * they meant. A result that is not a clean placement keeps the panel open
 * with the reason, since the next thing the user does is change the date.
 * Bars a day or longer are named by their date alone, so their panel has no
 * time fields: a time could not change which bar a date names. Both choices
 * hold for one interval and one zone, so a change to either closes the panel.
 */

interface DateNavigationDialogOptions {
    /** Run one request. The panel reports the result and closes once it is placed. */
    navigate(target: DateNavigationTarget): Promise<DateNavigationResult>;
    /**
     * Abandon the panel's request. Called when the panel closes while that
     * request is still loading (dismissed, or closed by an interval or timezone
     * change), so the view cannot jump after the user has moved on; not called
     * when the panel closed because its chart was destroyed.
     */
    cancel?(): void;
    /**
     * A request already under way that this panel shows and reports as its own,
     * for a host whose history load rebuilds the chart and so closes the panel
     * that started it. A closed panel reports nothing.
     */
    pending?: {
        target: DateNavigationTarget;
        result: Promise<DateNavigationResult>;
    };
    onClose?(): void;
}
/** Open the go-to panel below `anchor`, prefilled with the visible window or the pending request. */
declare function openDateNavigation(ctx: WidgetContext, anchor: HTMLElement | undefined, options: DateNavigationDialogOptions): PanelHandle;
/** Styles for the go-to panel; part of the widget component sheet. */
declare const DATE_NAVIGATION_CSS: string;

interface IndexedDbWidgetStorageOptions {
    /**
     * Where the writes still pending when the page goes away are kept until the
     * next load. Default: the page's `localStorage`. Null keeps no journal, and
     * those writes may be lost with the page.
     */
    journal?: StorageLike | null;
    /**
     * Where earlier releases kept the widget's keys, copied once into a
     * namespace this database has never held. Default: the page's
     * `localStorage`. Null copies nothing.
     */
    migrateFrom?: Storage | null;
}
interface IndexedDbWidgetStorage extends AsyncStorageLike {
    /**
     * Hear each write that lands: this store's own, and those of any other
     * store over the same database in another tab. Returns the call that stops
     * listening.
     */
    subscribe(listener: (key: string, value: string | null) => void): () => void;
    /** Release the connection and stop listening. Later calls reject. */
    close(): void;
}
/**
 * A key-value store for the widget over IndexedDB: one object store of JSON
 * texts under the widget's keys, one transaction per call. Pass the page's
 * `indexedDB`; importing this module needs no browser. The store's code loads
 * as the first store is created (since 2.5.10): calls made before it arrives
 * wait for it.
 *
 * ```ts
 * createWidget(el, { persist: 'desk', storage: createIndexedDbWidgetStorage(indexedDB, 'my-app-charts') });
 * ```
 */
declare function createIndexedDbWidgetStorage(factory: IDBFactory, name?: string, options?: IndexedDbWidgetStorageOptions): IndexedDbWidgetStorage;

interface StatuslineOptions {
    /** BCP 47 tag for number formatting. Default: the runtime's. */
    locale?: string | undefined;
    /**
     * Show the market status from the chart's session calendar, while the
     * chart's "Session state" switch is on. Nothing shows without a calendar
     * that lays out phases. Default false.
     */
    marketStatus?: boolean;
    /** Clock for the market status, in milliseconds. Default `Date.now`. */
    now?: (() => number) | undefined;
}
interface StatuslineHandle {
    readonly el: HTMLElement;
    /** Put a transient message at the right. `error` tints it. */
    setMessage(text: string, kind?: 'info' | 'error'): void;
    /**
     * The title: symbol, optional exchange, and the interval code. Changing a
     * title already set clears the readings until the chart's primary data is
     * next replaced, since the bars still on the chart are the previous title's.
     */
    setSymbol(symbol: string, exchange: string, interval: string): void;
    /**
     * Show a bar's readings and hold them. Null lets the row follow the latest
     * bar again, as it does while the pointer is away.
     */
    setBar(bar: Bar | null, time: number | null): void;
    /** Re-read the chart's switches and repaint. */
    refresh(): void;
    destroy(): void;
}
/**
 * Decimals a price is printed with: the price scale's own, so the row agrees
 * with the axis, floored at two so a reading a trader compares against a level
 * survives the comparison (the same floor the engine puts under a study pane).
 */
declare const MIN_PRICE_DIGITS = 2;
declare function priceDigits(chart: Chart): number;
declare function mountStatusline(ctx: WidgetContext, host: HTMLElement, opts?: StatuslineOptions): StatuslineHandle;

/**
 * The account summary: the selected account, its equity and margin, and
 * whether those figures are current. Apart from switching account it is
 * read-only, with no order control beside it, so choosing an account is never
 * one misplaced click away from sending something.
 *
 * It shows what the source reports and nothing it does not. A source whose
 * provider declares no account data renders disabled with that provider's
 * reason, the way a control with no data in the current context is shown.
 * Figures from before a dropped connection stay visible, marked stale, rather
 * than vanishing or passing for current ones.
 */

interface AccountSummaryOptions {
    /** Usually an `AccountManager` from the trade tier. */
    source: AccountStateSource;
    /** BCP 47 tag for the figures. Default: the widget's. */
    locale?: string | undefined;
}
interface AccountSummaryHandle {
    readonly el: HTMLElement;
    /** Re-read the source and repaint. */
    refresh(): void;
    destroy(): void;
}
/** Mount the summary into `host`, before its timezone readout when it has one. */
declare function mountAccountSummary(ctx: WidgetContext, host: HTMLElement, options: AccountSummaryOptions): AccountSummaryHandle;
declare const ACCOUNT_SUMMARY_CSS = "\n.oac-widget .oac-statusline.oac-has-account{container:oac-status/inline-size}\n.oac-widget .oac-statusline.oac-has-account .oac-statusline__time{flex:0 1 auto;min-width:0;overflow:hidden;text-overflow:ellipsis}\n.oac-widget .oac-statusline .oac-account{flex:none}\n.oac-widget .oac-account>.oac-account__pick,.oac-widget .oac-account>.oac-account__tag,.oac-widget .oac-account>.oac-account__field{flex:none}\n.oac-widget .oac-account{display:inline-flex;align-items:center;gap:10px;min-width:0;white-space:nowrap;font-variant-numeric:tabular-nums}\n.oac-widget .oac-account__pick{display:inline-flex;align-items:center;gap:4px;max-width:180px;height:20px;padding:0 6px;border:1px solid var(--oac-bd);border-radius:5px;background:var(--oac-elev);color:var(--oac-tx-strong);font-size:11.5px;font-weight:600}\n.oac-widget .oac-account__pick:hover:not(:disabled){border-color:var(--oac-bd-hover)}\n.oac-widget .oac-account__pick:disabled{cursor:not-allowed;color:var(--oac-faint)}\n.oac-widget .oac-account__name{min-width:0;overflow:hidden;text-overflow:ellipsis}\n.oac-widget .oac-account__pick .oac-chev{display:inline-grid;width:10px;height:10px;color:var(--oac-mut)}\n.oac-widget .oac-account__tag{padding:0 5px;border:1px solid var(--oac-bd-soft);border-radius:4px;color:var(--oac-amber);font-size:10px;font-weight:600;letter-spacing:.5px;text-transform:uppercase}\n.oac-widget .oac-account__field{display:inline-flex;align-items:center;gap:4px}\n.oac-widget .oac-account__field>i{font-style:normal;color:var(--oac-faint)}\n.oac-widget .oac-account__field>b{font-weight:500;color:var(--oac-tx)}\n.oac-widget .oac-account__state{min-width:0;max-width:320px;overflow:hidden;text-overflow:ellipsis;color:var(--oac-mut)}\n.oac-widget .oac-account__state.is-stale{color:var(--oac-amber)}\n.oac-widget .oac-account__state.is-error{color:var(--oac-danger)}\n.oac-widget .oac-account.is-disabled .oac-account__state{color:var(--oac-faint)}\n.oac-widget .oac-account[data-status=\"stale\"] .oac-account__field>b{color:var(--oac-mut)}\n@container oac-status (max-width:1180px){.oac-widget .oac-account__used{display:none}}\n@container oac-status (max-width:1000px){.oac-widget .oac-account__equity{display:none}}\n@container oac-status (max-width:860px){.oac-widget .oac-account__available{display:none}\n.oac-widget .oac-statusline .oac-statusline__title{order:-2}.oac-widget .oac-statusline .oac-account{order:-1}}\n";

/**
 * The widget stylesheet: one `<style>` per document, every rule scoped under
 * `.oac-widget`, every colour a token from `tokens.ts`.
 *
 * Nothing here names a colour twice or reaches outside the widget root. A
 * host page keeps its own cascade, and two widgets on one page share the one
 * sheet while carrying their own token values inline, so one can be dark and
 * the other light.
 *
 * The rules follow the craft standard in CLAUDE.md: styled scrollbars on
 * every scrolling surface, no browser-default checkbox, select or colour
 * input, small square swatches, dense rows, muted uppercase section heads.
 * The dialog modules mount inside `.oac-widget`, so they inherit all of it.
 */
/** Id of the injected `<style>`; one per document. */
declare const WIDGET_STYLE_ID = "oac-widget-css";
declare const WIDGET_CSS: string;
/**
 * Put the stylesheet into `doc` once. Safe to call per widget: the second
 * call leaves a populated sheet untouched. An empty server-rendered sheet
 * is filled in place, preserving its nonce. `extra` is appended to the
 * shell's rules when filling the sheet; the dialog modules hand theirs in
 * here, so the page still carries one sheet. `nonce` authorizes the style
 * element under CSP; the host controls its style-attribute policy separately.
 */
declare function injectWidgetStyles(doc: Document, extra?: string, nonce?: string): HTMLStyleElement;

/** Shared first-mount styles keep embedded hosts and full widgets identical under CSP. */
declare const WIDGET_COMPONENT_CSS: string;

interface DrawingToolbarOptions {
    /** The chart's container; the toolbar keeps inside it. Default: the chart's own. */
    chart?: HTMLElement;
    /** Saved looks, for the template rows of the more menu. */
    templates?: DrawingTemplates | null;
}
interface DrawingToolbarHandle {
    readonly el: HTMLElement;
    /** Read the selection again: values, enabled states and position. */
    refresh(): void;
    destroy(): void;
}
/** Widths offered in the width menu: the steps a stroke visibly changes at. */
declare const TOOLBAR_LINE_WIDTHS: readonly number[];
/**
 * Mount the toolbar in `host` (the widget's stage), right after the chart in
 * the tab order, so Tab from the focused chart reaches it.
 */
declare function mountDrawingToolbar(ctx: WidgetContext, host: HTMLElement, opts?: DrawingToolbarOptions): DrawingToolbarHandle;
/** Append beside the shared widget styles when mounting the toolbar yourself. */
declare const DRAWING_TOOLBAR_CSS = "\n.oac-widget .oac-drawbar { position: absolute; z-index: 25; display: flex; align-items: center; gap: 2px; padding: 3px;\n  background: var(--oac-panel); border: 1px solid var(--oac-bd); border-radius: 9px; box-shadow: var(--oac-shadow); }\n.oac-widget .oac-drawbar .oac-btn { height: 28px; }\n.oac-widget .oac-drawbar .oac-sep { height: 18px; margin: 0 3px; }\n.oac-widget .oac-drawbar__color { margin: 0 2px; }\n.oac-widget .oac-drawbar__width { min-width: 44px; padding: 0 7px; font-size: 11.5px; font-variant-numeric: tabular-nums; color: var(--oac-tx); }\n.oac-widget .oac-drawbar .is-mixed { color: var(--oac-mut); font-style: italic; }\n.oac-widget .oac-drawbar .oac-btn[aria-pressed=\"mixed\"] { border-color: var(--oac-on-bd); border-style: dashed; }\n.oac-widget.is-mobile .oac-drawbar { display: none; }\n/* The properties dialog edits the same selection in full; two surfaces for it would sit on each other. */\n.oac-widget:has(.oac-layer > .oac-props) .oac-drawbar { visibility: hidden; }\n";

/**
 * The reader for a descriptor's `activeWhen` and `visibleWhen`.
 *
 * It lives in the widget tier rather than beside the types in the base: the
 * calculation never consults a condition, so a host that only draws studies
 * should not carry the reader. A host with its own form imports it from here,
 * which is what the reference host does, so both forms answer alike.
 */

/** Whether one input is shown and can be edited, given the current settings. */
interface InputState {
    /** False while `visibleWhen` fails, or an input it reads is hidden. */
    visible: boolean;
    /** False while `activeWhen` fails, or an input it reads is hidden or inactive. */
    active: boolean;
    /** The keys `activeWhen` reads, in the order it names them, for a reason naming them. */
    dependsOn: readonly string[];
}
type Values = Readonly<Record<string, unknown>>;
/**
 * Whether `condition` holds for `values`. Only own settings are read, so an
 * inherited name such as `constructor` never satisfies one. A shape it cannot
 * read holds, so a malformed descriptor never locks a user out of an input.
 */
declare function inputConditionMet(condition: IndicatorInputCondition | undefined, values: Values): boolean;
/**
 * The state of every input in `inputs`, keyed by input key.
 *
 * An input a condition reads counts for its own state as well as its value: a
 * control hidden behind a switch cannot also be what shows a third, and one
 * nobody can edit cannot be what enables another. A colour pair writes its
 * switch and its two colours under keys of their own, given on the input
 * (`enabled`, `up`, `down`) or under `pair` as a form control carries them,
 * and a condition reading any of those reads the pair. A key outside `inputs`
 * (a setting on another tab) is decided by its value alone, and so is a key
 * met again while it is being decided, so a cycle in a descriptor ends.
 */
declare function inputStates(inputs: readonly (IndicatorInputPresentation & {
    key: string;
    enabled?: {
        key: string;
    };
    up?: {
        key: string;
    };
    down?: {
        key: string;
    };
    pair?: {
        enabled?: {
            key: string;
        } | undefined;
        up?: {
            key: string;
        };
        down?: {
            key: string;
        };
    } | undefined;
})[], values: Values): Map<string, InputState>;

interface IndicatorInputControlsOptions {
    instance: IndicatorApi;
    inputs: readonly IndicatorInput[];
    panel: HTMLElement;
    field(key: string): HTMLInputElement | HTMLTextAreaElement | null;
    /** Commit one atomic patch. False keeps the existing settings and draft. */
    onPatch(patch: IndicatorSettings): boolean;
    /** A reference host can suspend its own modal instead of the widget stack. */
    suspend?(): () => void;
    current?(): boolean;
}
interface IndicatorInputControlsHandle {
    cancelPick(): void;
    refresh(): void;
    destroy(): void;
}
/**
 * Host actions for typed fields. Values remain ordinary settings scalars.
 * Call `refresh` after the form re-reads its conditions, so an action follows
 * its field in and out of play.
 */
declare function mountIndicatorInputControls(ctx: WidgetContext, options: IndicatorInputControlsOptions): IndicatorInputControlsHandle;

interface AlertUiOptions extends WidgetTranslationOptions {
    chart: Chart;
    draw: DrawingController;
    alerts: AlertController;
    theme?: WidgetThemeName;
    chartTheme?: ChartTheme;
    locale?: string;
    styleNonce?: string;
    /** Suspend host keyboard shortcuts while any alert dialog is open. */
    onOpenChange?(open: boolean): void;
}
interface AlertUi {
    readonly root: HTMLElement;
    /** Shared theme, overlays and controls for a custom host. Owned by this UI's lifetime. */
    readonly context: WidgetContext;
    openList(anchor?: HTMLElement): boolean;
    openEditor(options?: AlertEditorOptions): boolean;
    isOpen(): boolean;
    close(): void;
    setTheme(name: WidgetThemeName, theme?: ChartTheme): void;
    /** Releases UI only. The host still owns its chart and controllers. */
    destroy(): void;
}
/** Mount shared alert dialogs over a custom host's positioned chart container. */
declare function createAlertUi(container: HTMLElement, options: AlertUiOptions): AlertUi;

/**
 * Widget tier (opt-in: "openalgo-charts/widget").
 *
 * The chart with its chrome: a top bar (symbol, intervals, chart type,
 * indicators, capture, settings, theme), the drawing rail, a status line, a
 * keymap and the dialogs, in one call. It is the one tier that ships DOM,
 * because a toolbar is DOM; the engine underneath still ships none.
 * `createChartGrid` lays several widgets out as one linked workspace.
 *
 * ```ts
 * import { createWidget } from 'openalgo-charts/widget';
 *
 * const widget = createWidget('#chart', {
 *   feed, symbol: 'RELIANCE', exchange: 'NSE', interval: '5m',
 *   theme: 'dark', persist: true,
 * });
 * widget.on('symbol', ({ symbol }) => document.title = symbol);
 * ```
 *
 * Importing this module imports the draw tier, so every built-in drawing tool
 * is registered. The dialog modules register their mount functions through
 * `registerWidgetDialogs`; a shell built without them renders the buttons that
 * would open them disabled, with their state visible.
 */
declare const WIDGET_TIER: "widget";

export { ACCOUNT_SUMMARY_CSS, type AccountSummaryHandle, type AccountSummaryOptions, type AlertEditorOptions, type AlertUi, type AlertUiOptions, type AlertsPanelOptions, type AsyncStorageLike, BOTTOMBAR_CSS, BOTTOMBAR_HEIGHT, type BottombarContext, type BottombarControls, type BottombarHandle, type BottombarOptions, type BottombarScaleState, type BottombarScaleToggle, type BottombarTarget, type Box, type BusHandler, CHART_GRID_CSS, CHART_GRID_LAYOUTS, CHART_GRID_LAYOUT_NAMES, CHART_GRID_PRESETS, CHART_TYPE_LABELS, COLOR_PICKER_CSS, type ChartGrid, type ChartGridApplyReport, type ChartGridCell, type ChartGridEventName, type ChartGridEvents, type ChartGridLayout, type ChartGridLayoutId, type ChartGridLayoutSlot, type ChartGridLayoutSpec, type ChartGridLinkGroup, type ChartGridMessage, type ChartGridOptions, type ChartGridPreset, type ChartGridUnevenLayout, ChartHistory, type ChartHistoryChange, type ChartHistoryCommand, type ChartHistoryError, type ChartHistoryOptions, type ChartHistoryStep, type ChartShortcutSource, type ColorPickerHandle, type ColorPickerOptions, type ContextMenuHooks, type ContextMenuOptions, DATA_WINDOW_CSS, DATE_NAVIGATION_CSS, DEFAULT_INTERVALS, DEFAULT_LOOKBACK_BARS, DEFAULT_RANGES, DIALOG_CSS, DRAWINGS_KEY_PREFIX, DRAWING_COORDINATES_CSS, DRAWING_TEMPLATES_CSS, DRAWING_TOOLBAR_CSS, type DataWindowHandle, type DataWindowOptions, type DataWindowRow, type DataWindowSection, type DataWindowSnapshot, type DateNavigationDialogOptions, type DateNavigationResult, type DateNavigationStatus, type DateNavigationTarget, DateNavigator, type DateNavigatorOptions, type DialogHandle, type DialogMount, type DrawingCoordinatesHandle, type DrawingPropertiesOptions, type DrawingTemplates, type DrawingToolbarHandle, type DrawingToolbarOptions, EVENT_DETAILS_CSS, type EventDetailAction, type EventDetailsLabels, type EventDetailsLoader, EventDetailsPopup, type EventDetailsPopupOptions, type FormControl, type FormHandle, type FormKind, type FormOptions, type FormTranslationOptions, type HistoryReach, type IndexedDbWidgetStorage, type IndexedDbWidgetStorageOptions, type IndicatorInputControlsHandle, type IndicatorInputControlsOptions, type IndicatorPickerOptions, type IndicatorSettingsOptions, type IndicatorSettingsTab, type IndicatorTemplateApplyMode, type InputState, KEYMAP_KEY, type KeyAction, type KeyBinding, type KeyBindingOptions, type KeyChordUse, type KeyConflict, type KeyEventLike, type KeyRebindResult, type KeyScope, Keymap, type KeymapChange, type KeymapGroup, type KeymapOptions, type KeymapOverrides, type KeymapRow, type LayoutApplyReport, type LayoutAutosaveStatus, type LayoutTarget, type LayoutsController, type LayoutsControllerOptions, type LayoutsState, type LevelEditorOptions, MAGNET_MODES, MIN_PRICE_DIGITS, type MarketStatusReading, type MenuEntry, type MenuItem, type MenuOptions, type MenuRow, type MobileHandle, type MobileMode, type MobileOptions, NEWS_PANEL_CSS, type NewsPanelHandle, type NewsPanelOptions, NewsReader, type NewsReaderOptions, type NewsSnapshot, type NewsStatus, OBJECTS_PANEL_CSS, type ObjectsPanelContent, type ObjectsPanelOptions, type OrderRequest, type OverlayOptions, type OverlayStack, PANEL_DOCK_CSS, type PanelDockContent, type PanelDockHandle, type PanelDockId, type PanelDockOptions, type PanelDockState, type PanelHandle, QUICK_ENTRY_CSS, type QuickEntryHandle, type QuickEntryOptions, QuoteBoard, type QuoteBoardOptions, type QuoteBoardStatus, type QuoteRow, type QuoteRowStatus, RAIL_GROUPS, RAIL_PREFS_KEY, RAIL_WIDTH, type RailGroup, type RailGroupItem, type RailHandle, type RailOptions, type RailPrefs, type RangeWindowOptions, type Rgba, SAVE_DEBOUNCE_MS, SEARCH_DEBOUNCE_MS, STATE_KEY, STATUSLINE_HEIGHT, STORAGE_PREFIX, SYMBOL_PICKER_CSS, type SettingsDialogOptions, type ShortcutsPanelOptions, type Size, type StatuslineHandle, type StatuslineOptions, type StorageLike, type SymbolPickerHandle, type SymbolPickerOptions, type SymbolSearch, TEXT_CONTRAST, TIP_DWELL_MS, TOAST_LEAVE_MS, TOAST_MAX, TOAST_MS, TOKEN_PREFIX, TOOLBAR_LINE_WIDTHS, TOPBAR_HEIGHT, type TextEditorHandle, type TextEditorOptions, type TipController, type TipSide, type TipSource, type TipSpec, type ToastHandle, type ToastKind, type ToastOptions, type Toaster, type TopbarHandle, type TopbarOptions, type TopbarState, WATCHLIST_PANEL_CSS, WIDGET_COMPONENT_CSS, WIDGET_CSS, WIDGET_DIALOGS, WIDGET_FONT, WIDGET_MONO, WIDGET_STATE_VERSION, WIDGET_STYLE_ID, WIDGET_TIER, type WatchlistPanelHandle, type WatchlistPanelOptions, type WatchlistSort, type WatchlistSortKey, type Widget, type WidgetBottombarOptions, type WidgetBuiltinMessage, WidgetBus, type WidgetBusEvents, type WidgetChartState, type WidgetContext, type WidgetDialogName, type WidgetEventName, type WidgetMessageKey, type WidgetMessageParameters, type WidgetMessageValues, type WidgetNewsOptions, type WidgetOptions, type WidgetRange, type WidgetRangeUnit, type WidgetRangeWindow, type WidgetRestoreReport, type WidgetSessionCalendar, type WidgetState, WidgetStorage, type WidgetStorageError, type WidgetStorageOptions, type WidgetThemeName, type WidgetTokens, type WidgetTranslationOptions, type WidgetTranslator, type WidgetWatchlistOptions, applyIndicatorTemplate, applyTokens, attachContextMenu, boxIn, captureName, chartTypeChoices, chartTypeLabel, contextMenuEntries, contrastRatio, controlsFromFields, controlsFromInputs, createAlertUi, createChartGrid, createColorPicker, createDrawingTemplates, createIndexedDbWidgetStorage, createLayoutsController, createObjectsPanelContent, createOverlayStack, createTipController, createWidget, defaultStorage, downloadText, esc, eventKeyCombo, focusable, focusables, formatColor, formatKeyCombo, fromChartCombo, glyph, h, historyPress, historyReady, inTextField, injectWidgetStyles, inputConditionMet, inputStates, intervalLabel, loadWindow, luminance, mix, mountAccountSummary, mountAlertEditor, mountAlertsPanel, mountBottombar, mountContextMenu, mountDataWindow, mountDrawingCoordinates, mountDrawingProperties, mountDrawingToolbar, mountIndicatorInputControls, mountIndicatorPicker, mountIndicatorSettings, mountLevelEditor, mountMobile, mountNewsPanel, mountObjectsPanel, mountPanelDock, mountQuickEntry, mountRail, mountSettingsDialog, mountStatusline, mountSymbolPicker, mountTextEditor, mountToasts, mountTopbar, mountWatchlistPanel, openDateNavigation, openLayoutsMenu, openMenu, openShortcutsPanel, parseColor, parseKeyCombo, placeBelow, placeBeside, placeTip, priceDigits, quoteChange, rangeInterval, rangeWindow, readDataWindow, readableOn, registerWidgetDialog, registerWidgetDialogs, registeredWidgetDialogs, renderForm, resolveTheme, safeNewsUrl, safeSymbolIconUrl, sanitizePanelDockState, sanitizeRailPrefs, saveIndicatorTemplate, stripView, themeMode, token, toolGlyph, toolName, unregisterWidgetDialog, widgetDialog, widgetLayoutTarget, widgetText, widgetTokens, withAlpha };
