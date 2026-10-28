import { AlertDrawingValue, AlertDrawingLevel, PrimitiveRenderContext, Bar, IPrimitive, DataLayer, PlotRect, IndicatorApi, SeriesApi, PriceScaleId, AlertDrawingInfo, PrimitiveHost, ZOrder, PrimitiveHit, EmptyEvent } from 'openalgo-charts';

/**
 * Drawing model (ARCHITECTURE.md section 8). A drawing is **plain data**: anchors
 * in `{ time, price }`, a style bag, a tool id. It serialises straight into
 * `ChartState.drawings` with no custom encoder, and a tool is a registry entry
 * exactly like a chart type or an indicator.
 *
 * Anchors are stored in *data* space, never pixels: the chart's time axis is
 * gapless (section 5.3), so a pixel anchor would slide the moment a weekend
 * collapsed or the user zoomed. `DataLayer.timeToIndexFloat` maps them back,
 * which also gives anchors between bars and to the right of the last one,
 * where trend projections and forecasts live.
 *
 * Version 2 of the model split the text fields out of the style bag into
 * {@link DrawingText}, made `levels` a list of {@link FibLevel} rather than bare
 * ratios, and gave every drawing a `zIndex`. A 1.9.x document is upgraded by
 * `migrateDrawings` on the way in; nothing downstream sees the old shape.
 *
 * The one exception to data space is a drawing whose `space` is `'viewport'`:
 * a note or a box that belongs to the screen rather than to a bar. Its anchors
 * are {@link ViewportPoint} fractions of its pane's plot, kept in a field of
 * their own so a time or a price never has to mean a pixel. The field is
 * optional and additive, which is why the document stays version 2: a save
 * with no viewport drawing is byte for byte what it was.
 *
 * Version 3 added one field, {@link Drawing.intervals}: the range of chart
 * intervals a drawing is shown on. A version 2 document is a valid version 3
 * one and loads unchanged, and a document is written as version 3 only when a
 * drawing in it carries a range, so a save without one is still version 2.
 * The version is raised for a range because an earlier build would not keep
 * it: its migration drops the field, and the drawing would show on every
 * interval there and lose its range on the next save.
 */

/** One anchor, in data space. */
interface DrawingPoint {
    /** UTC seconds. May fall between bars, or past the last one. */
    time: number;
    price: number;
    /**
     * Pen pressure at this sample, 0..1, on a freehand stroke only. Absent means
     * the pointer events stand-in of 0.5, which is what a mouse reports and what
     * a stroke's width is calibrated to, so a mouse stroke stores nothing here.
     */
    pressure?: number;
}
/**
 * Which coordinates a drawing's anchors are in. `'data'`, the default, is time
 * and price: the drawing moves with the bars when the chart pans or zooms.
 * `'viewport'` pins it to the screen: its anchors are fractions of its pane's
 * plot, so pan and zoom leave it where it is and a resize keeps it in
 * proportion.
 */
type DrawingSpace = 'data' | 'viewport';
/**
 * One anchor of a viewport drawing, as a fraction of the plot area of the pane
 * that holds the drawing. Pane-relative, not whole-chart-relative: the pane's
 * layer paints, clips and hit-tests the drawing, so these are the coordinates
 * that follow the pane when it moves, resizes or folds.
 *
 * `x` is 0 at the plot's left edge and 1 at its right edge; the price axes are
 * outside that span. `y` is 0 at the top of the pane and 1 at the bottom of its
 * plot, which on the lowest pane is the top of the time axis. Sizes that a tool
 * sets in pixels (a font size, a wrap width) stay in pixels: a resize moves a
 * note in proportion without scaling its type.
 */
interface ViewportPoint {
    x: number;
    y: number;
}
/**
 * How new anchors snap to the bar under the cursor. `weak` snaps only when an
 * O/H/L/C sits within a few pixels of the pointer, so a click on empty space
 * lands where it was made; `strong` always takes the nearest of the four.
 */
type MagnetMode = 'off' | 'weak' | 'strong';
/**
 * The text a drawing carries, and how it is set. The text tool *is* this box;
 * a shape (rectangle, channel, callout) carries one as its label.
 *
 * Kept apart from {@link DrawingStyle} because the two answer different
 * questions: style is how the outline is stroked, text is what the label says
 * and how it is typeset. A purple rectangle with white bold text is one shape
 * with two colours, and a settings panel wants them on different tabs.
 */
interface DrawingText {
    /** Body text. `\n` starts a new line; see `wrap` for soft wrapping. */
    value: string;
    /** Falls back to `style.color`. */
    color?: string;
    /** Media px. Default 12. */
    fontSize?: number;
    /** CSS font stack. Defaults to the UI sans-serif stack. */
    fontFamily?: string;
    bold?: boolean;
    italic?: boolean;
    /** Horizontal alignment within the box. Default 'left'. */
    align?: 'left' | 'center' | 'right';
    /** Vertical placement within the box. Default 'top'. */
    valign?: 'top' | 'middle' | 'bottom';
    /** Soft-wrap at `wrapWidth` instead of running off the pane. */
    wrap?: boolean;
    /** Wrap width in media px. Default 220. */
    wrapWidth?: number;
    /** Draw a filled plate behind the text. */
    background?: boolean;
    backgroundColor?: string;
    /** Plate opacity, 0..1. Default 1: a text background is usually opaque. */
    backgroundOpacity?: number;
    /** Stroke a border around the plate. */
    border?: boolean;
    borderColor?: string;
    /**
     * Where a shape's label sits relative to the shape: `inside` the outline, or
     * `outside` just above it. Shapes only; the text tool is its own box.
     */
    position?: 'inside' | 'outside';
}
/** One level of a Fibonacci-family tool. */
interface FibLevel {
    /** The level as a fraction of the anchor span (0.618, 1.618, ...). */
    ratio: number;
    /** Stroke colour. Falls back to the conventional colour for the ratio. */
    color?: string;
    /** `false` hides the level without forgetting it. Default true. */
    enabled?: boolean;
    /** Printed instead of the ratio when set. */
    label?: string;
}
interface DrawingStyle {
    color?: string;
    lineWidth?: number;
    lineStyle?: 'solid' | 'dashed' | 'dotted';
    /** Fill the shape's interior (rectangle, ellipse, channel, position boxes). */
    fill?: boolean;
    fillColor?: string;
    /** 0..1. Defaults to 0.12. */
    fillOpacity?: number;
    /** Extend the line past its anchors. */
    extendLeft?: boolean;
    extendRight?: boolean;
    /** Draw level / price / ratio labels where the tool has them. */
    showLabels?: boolean;
    /** Fibonacci-family levels. Defaults per tool. */
    levels?: FibLevel[];
    /** Position tools: capital base and risk per trade, for the size readout. */
    accountSize?: number;
    risk?: number;
    /** Line tools: print the bar count, price change and angle at the midpoint. */
    showStats?: boolean;
    /** Brush: let pen pressure drive the stroke width. */
    pressure?: boolean;
}
/**
 * What a host allows the user to do with one drawing. The flags are
 * independent of one another and of `locked`, and each defaults to true, so a
 * drawing without a policy behaves exactly as one did before policies
 * existed. A copy (paste, duplicate) never inherits a policy: it is the
 * user's own drawing.
 *
 * `selectable`, `persistent` and `listed` bind every caller. `editable`
 * restricts the user: the controller cannot tell a click from a call, so its
 * methods treat every call as the user's unless it passes `{ force: true }`
 * (see `DrawingEditOptions`), which is how the host that placed the drawing
 * moves or retires it. Stacking order (front, back, either side of the
 * series) is outside the policy.
 */
interface DrawingPolicy {
    /**
     * `false`: never joins the selection, whoever asks. A click passes through
     * to what lies under it, and nothing hovers, drags or selects it.
     */
    selectable?: boolean;
    /**
     * `false`: read-only to the user. It still selects, copies and duplicates,
     * but nothing moves, reshapes, restyles, hides, locks, cuts, deletes or
     * regroups it, and undo and redo never touch it or the group it is in.
     */
    editable?: boolean;
    /**
     * `false`: transient. Left out of `toJSON`, and so out of the chart state
     * and every saved layout, while it renders and undoes as usual in the
     * session. A restore replaces it with what was saved.
     */
    persistent?: boolean;
    /** `false`: left out of the object inventory (`ChartObjects`) and every panel built on it. */
    listed?: boolean;
}
/**
 * The chart intervals a drawing is shown on, both ends included, as interval
 * codes the chart's data context names (`'1m'`, `'1h'`, `'D'`, or a code the
 * host registered). An absent end is no limit on that side, so `{ to: '1h' }`
 * shows the drawing up to hourly bars. Intervals compare by bar length, so the
 * two ends may be written either way round. See `drawingShownOnInterval`.
 */
interface DrawingIntervalRange {
    /** The shortest interval the drawing is shown on. */
    from?: string;
    /** The longest interval the drawing is shown on. */
    to?: string;
}
interface Drawing {
    id: string;
    /** Registered tool id. */
    tool: string;
    /** Anchors in time and price. Empty on a viewport drawing, whose anchors are `viewportPoints`. */
    points: DrawingPoint[];
    /**
     * Which coordinates the anchors are in. Absent means `'data'`. Only a tool
     * that declares {@link DrawingTool.viewport} can be anchored to the viewport.
     */
    space?: DrawingSpace;
    /** The anchors of a viewport drawing, one per handle. Present only when `space` is `'viewport'`. */
    viewportPoints?: ViewportPoint[];
    style: DrawingStyle;
    /** The label (or, for the text tool, the whole content). */
    text?: DrawingText;
    /**
     * Per-tool extras that do not belong on every drawing (a table's cells, a
     * callout's tail side). Keeps the base shape closed while a tool stays free
     * to carry what it needs. Must be JSON-safe: it is persisted verbatim.
     */
    props?: Record<string, unknown>;
    paneIndex: number;
    /**
     * Locked drawings render but cannot be selected or dragged on the chart.
     * The user toggles it; a host restriction the user cannot lift is a
     * {@link DrawingPolicy}.
     */
    locked?: boolean;
    /** Host restrictions on this drawing. Absent means none. */
    policy?: DrawingPolicy;
    /** Default true. */
    visible?: boolean;
    /**
     * The chart intervals the drawing is shown on. Absent means every interval.
     * On an interval outside the range the drawing is kept and saved but not
     * on the chart: it is not painted, hit-tested, boxed or erased there, and
     * the object inventory marks it. `visible` stays the user's own switch.
     */
    intervals?: DrawingIntervalRange;
    /**
     * Paint order. Below zero paints under the series, at or above zero paints
     * over it. Ties break by array order, so two drawings at 0 paint in the
     * order they sit in the list. With `stackAbove` it orders the drawings
     * placed above the same entry.
     */
    zIndex: number;
    /**
     * Placed inside the series band, directly above one entry of its pane's
     * `chart.seriesStack()`: `'source:primary'` or `'indicator:<id>'`. It paints
     * after that entry's series and before the next entry's. While the entry
     * plots no series on this drawing's pane (a study removed or moved away) it
     * paints by `zIndex` as if absent, and returns when the entry does. Absent
     * means the drawing is not in the series band. See
     * `DrawingController.placeInStack`.
     */
    stackAbove?: string;
    /** Epoch ms, set by the controller when the drawing is added. */
    createdAt?: number;
}
/**
 * What `DrawingController.add` accepts: a drawing minus the fields the
 * controller fills in. `zIndex` defaults to 0, `createdAt` to now, and an id is
 * minted unless one is supplied.
 */
type DrawingInput = Omit<Drawing, 'id' | 'zIndex' | 'createdAt'> & {
    id?: string;
    zIndex?: number;
    createdAt?: number;
};
/**
 * The fields `DrawingController.update` and `updateMany` can change. `policy`
 * merges flag by flag, the way `style` does.
 *
 * `space` on its own converts the anchors at the view on screen, so the
 * drawing stays exactly where it is and only stops (or starts) following the
 * bars. With the new space's anchors in the same patch (`viewportPoints` for
 * the viewport, `points` for data) they are taken as given. A conversion the
 * controller cannot make (a tool without viewport support, a pane with no
 * place on screen) is left out of the patch and the rest of it applies, and
 * `DrawingController.update` returns false.
 */
type DrawingPatch = Partial<Pick<Drawing, 'points' | 'style' | 'text' | 'props' | 'locked' | 'visible' | 'zIndex' | 'policy' | 'space' | 'viewportPoints'>> & {
    /** A series-band entry to paint directly above, or null to leave the series band. */
    stackAbove?: string | null;
    /**
     * The intervals to show the drawing on, replacing any range it had, or
     * null (or a range with neither end) to show it on every interval.
     */
    intervals?: DrawingIntervalRange | null;
};
/**
 * What `DrawingController.placeInStack` puts a drawing next to: another
 * drawing of its pane, or an entry of its pane's series band.
 */
type DrawingStackTarget = {
    drawing: string;
} | {
    entry: string;
};
/**
 * The newest document version this build reads and writes; bumped when
 * {@link Drawing} changes in a way an earlier build would lose. A document is
 * written as the lowest version that holds it (see {@link DrawingsDocument}).
 */
declare const DRAWING_STATE_VERSION = 3;
/** A named set of drawing ids. A drawing belongs to at most one group. */
interface DrawingGroup {
    id: string;
    name: string;
    members: string[];
}
/**
 * What `toJSON` returns and `ChartState.drawings` carries. Versioned so a 1.9.x
 * save (a bare `Drawing[]`) is recognisable and upgraded rather than misread.
 */
interface DrawingsDocument {
    /**
     * 3 when a drawing carries an interval range, else 2: the lowest version
     * that holds the content, so an earlier build reads a document without a
     * range exactly as it wrote one.
     */
    version: 2 | 3;
    drawings: Drawing[];
    groups?: DrawingGroup[];
}
/** A point mapped to the pane, in media px. */
interface ScreenPoint {
    x: number;
    y: number;
}
interface DrawContext {
    ctx: CanvasRenderingContext2D;
    rc: PrimitiveRenderContext;
    /** Anchors in **device** px, ready to stroke. */
    pts: ScreenPoint[];
    drawing: Drawing;
    style: Required<Pick<DrawingStyle, 'color' | 'lineWidth'>> & DrawingStyle;
    selected: boolean;
    /** Format a price the way the pane's axis does. */
    formatPrice(price: number): string;
}
interface HitContext {
    /** Anchors in **media** px, the same space as the incoming x/y. */
    pts: ScreenPoint[];
    drawing: Drawing;
    rc: PrimitiveRenderContext;
}
/** What {@link DrawingTool.expand} needs to size a default in chart units. */
interface ExpandContext {
    /** Seconds between adjacent bars, so a default can span a sane bar count. */
    barSeconds: number;
    /**
     * Bars currently on screen. A default sized in fixed bars is a hairline when
     * zoomed out and fills the pane when zoomed in, so size against this instead.
     */
    visibleBars: number;
    /**
     * Anchor to media px on the placing pane, and back. Present when the host
     * can map coordinates, absent otherwise. A default sized in pixels (a
     * position box 64 px of risk tall, 150 px wide) reads the same at every
     * zoom and on every instrument, which a price percentage cannot: 1% of a
     * 2.87 stock and 1% of an index are the same fraction and very different
     * boxes. Without the mapping a tool falls back to chart units.
     */
    toPixel?(p: DrawingPoint): ScreenPoint | null;
    fromPixel?(at: ScreenPoint): DrawingPoint | null;
}
/** Coordinate maps use the same pane and logical time axis as the rendered drawing. */
interface DrawingValueContext {
    drawing: Drawing;
    pts: readonly ScreenPoint[];
    time: number;
    x: number;
    fromY(y: number): number | null;
}
/**
 * The control a field wants. `interval` is a chart interval code, which the
 * host offers from its own interval list (the engine has none), with an
 * empty choice for no limit.
 */
type FieldKind = 'color' | 'number' | 'select' | 'lineStyle' | 'boolean' | 'text' | 'opacity' | 'levels' | 'interval';
/** The section a host groups a field under. */
type FieldGroup = 'line' | 'fill' | 'text' | 'levels' | 'behavior' | 'visibility';
interface SettingsField {
    /**
     * Dot path into the drawing. Two segments under `style`, `text`, `props`
     * or `intervals` (`style.lineWidth`, `intervals.to`), or one of the
     * top-level fields `locked`, `visible`, `zIndex`, `space`.
     */
    path: string;
    label: string;
    kind: FieldKind;
    min?: number;
    max?: number;
    step?: number;
    /** For `select` and `lineStyle`: the values a host may offer. */
    options?: ReadonlyArray<{
        value: string;
        label: string;
    }>;
    group?: FieldGroup;
    /** Value displayed when the drawing has not overridden this setting. */
    defaultValue?: string | number | boolean;
}
interface SettingsSchema {
    fields: SettingsField[];
    /**
     * The text *is* the drawing (a note, a callout, the text tool) rather than a
     * label on a shape, so a host should ask for it the moment the tool is
     * placed instead of waiting for a settings dialog.
     */
    textIsContent?: boolean;
}
interface DrawingTool {
    id: string;
    name: string;
    /** Opt in only when the geometry defines a numeric level or band at one time. */
    alertValue?(context: DrawingValueContext, level?: string): Omit<AlertDrawingValue, 'paneIndex'> | undefined;
    /** Named boundaries or active rungs, for a host's explicit level selector. */
    alertLevels?(drawing: Drawing): readonly AlertDrawingLevel[];
    /**
     * Anchors the tool needs before it is complete. `0` means free-form: the
     * drawing finishes on double-click.
     */
    points: number;
    /**
     * Sample the cursor continuously while the pointer is held, and finish on
     * release: one press-drag-release gesture is one stroke. Brushes want this;
     * without it a `points: 0` tool collects a vertex per click, which is
     * polyline behaviour and never terminates on its own.
     */
    freehand?: boolean;
    /**
     * Hold Shift to snap the free end of a two-anchor tool to 45 degree steps
     * on screen, while placing and while dragging a handle. The line family
     * sets it; a tool whose second anchor is not the far end of a line (a
     * rectangle's opposite corner) leaves it off, since a locked diagonal is
     * not what Shift means there.
     */
    angleLock?: boolean;
    /**
     * The tool can be anchored to the viewport (`Drawing.space`). Declare it
     * only when `draw` and `distance` read the screen anchors (`pts`) and never
     * `drawing.points`, which is empty in that space, and when nothing the tool
     * prints is a price or a time: a label that reads the axis under a pinned
     * shape would change every time the chart panned. `expand`, `constrain` and
     * `alertValue` apply to data space only.
     */
    viewport?: boolean;
    /**
     * The box the drawing covers at `pts`, its anchors in media px (the space
     * of {@link HitContext}). A drawing anchored to the viewport is always
     * painted, hit-tested and moved with this box inside its pane's plot, so it
     * can be seen and grabbed at every edge and after the chart resizes, and no
     * drag, nudge or handle can take it out. Declare it on a `viewport` tool
     * that paints beyond its anchors, as the text note and the table do from
     * their one corner, and the rectangle and the ellipse do for a label above
     * them or wider than they are; without it the box is the anchors' own
     * bounds. A handle dragged toward an edge the box reaches first stops short
     * of it, so the other anchors stay where they are.
     */
    bounds?(pts: readonly ScreenPoint[], drawing: Drawing): {
        x0: number;
        y0: number;
        x1: number;
        y1: number;
    };
    /**
     * Turn the anchors actually clicked into the full anchor set. Lets a tool drop
     * a complete, immediately editable default from fewer clicks: the position
     * tools place a 1:1 box off a single click, which the user then drags, rather
     * than demanding entry/target/stop be clicked in turn.
     *
     * The returned points become the drawing's anchors, so each one stays a
     * draggable handle.
     */
    expand?(clicked: readonly DrawingPoint[], ctx: ExpandContext): DrawingPoint[];
    /**
     * Keep the anchors consistent after one of them moves: a handle drag
     * (`handle` is its index) or a patch to `points` (`handle` is null). Returns
     * the anchors to store. The position tools use it to keep the stop and the
     * target on opposite sides of the entry, so dragging the stop up through
     * the entry of a long turns the trade into a short instead of piling both
     * levels on one side. Pure: the same input always yields the same output.
     */
    constrain?(points: readonly DrawingPoint[], handle: number | null): DrawingPoint[];
    /**
     * Keyboard shortcut that arms this tool, as `'Alt+T'` / `'Shift+Alt+F'`:
     * modifiers in `Ctrl`, `Alt`, `Shift` order, then a single key. Hosts render
     * it beside the tool's name in a palette and bind it with
     * `matchDrawingShortcut`; the library itself installs no listener, since only
     * the host knows whether the chart has focus or a dialog is open.
     */
    shortcut?: string;
    /** Merged under the caller's style when a drawing is created. */
    defaultStyle?: DrawingStyle;
    /**
     * Merged under the caller's text when a drawing is created. The text tool
     * uses it for its placeholder content and default size; a shape leaves it
     * unset so it carries no label until the user gives it one.
     */
    defaultText?: DrawingText;
    /**
     * The fields a settings panel offers for this tool. Every built-in declares
     * one; a custom tool without it gets the plain line fields.
     */
    settings?: SettingsSchema;
    draw(c: DrawContext): void;
    /**
     * Distance in media px from the cursor to the shape, or `null` for a miss.
     * Return `0` for "inside" so a filled region is grabbable anywhere.
     */
    distance(x: number, y: number, c: HitContext): number | null;
}

/** Time-anchored analysis drawings use the owning pane's bars, never a tier cache. */

declare const ANCHORED_VWAP: DrawingTool;
declare const FIXED_RANGE_VOLUME_PROFILE: DrawingTool;

/** Pure OHLCV analysis. Inputs are sorted by ascending UTC timestamp. */

/** Partial results have unavailable anchor history, missing volume or unusable prices. */
type AnalysisStatus = 'ready' | 'partial' | 'missing-volume' | 'zero-volume' | 'empty' | 'invalid-data';
type AnchoredVwapSource = 'hlc3' | 'close' | 'hl2' | 'ohlc4';
interface AnchoredVwapOptions {
    source?: AnchoredVwapSource;
}
interface AnchoredVwapPoint {
    time: number;
    value: number;
    /** Volume-weighted population standard deviation of the selected source. */
    deviation: number;
    /** Do not connect a line across an omitted observation. */
    breakBefore: boolean;
}
interface AnchoredVwapResult {
    status: AnalysisStatus;
    /** The requested anchor precedes the earliest supplied bar. */
    historyPartial: boolean;
    points: AnchoredVwapPoint[];
    missingVolumeBars: number;
    invalidPriceBars: number;
}
interface FixedRangeVolumeProfileOptions {
    /** Requested price rows, rounded and clamped to 4..200. Fewer rows are used when prices cannot represent distinct edges. Default 48. */
    rows?: number;
    /** Percentage of volume in the contiguous area around the POC, 1..100. Default 70. */
    valueArea?: number;
}
interface AnalysisProfileRow {
    low: number;
    high: number;
    volume: number;
    valueArea: boolean;
}
interface FixedRangeVolumeProfileResult {
    status: AnalysisStatus;
    /** The requested range begins before the earliest supplied bar. */
    historyPartial: boolean;
    rows: AnalysisProfileRow[];
    totalVolume: number;
    /** Midpoint of the largest row. Ties choose the lower price. */
    poc: number | null;
    valueAreaLow: number | null;
    valueAreaHigh: number | null;
    missingVolumeBars: number;
    invalidPriceBars: number;
}
/**
 * Accumulate from the first bar at or after the anchor, including the live tail.
 * Missing volume is omitted and reported; zero volume adds no statistical weight.
 * Recompute on each call so mutations of the forming candle are always visible.
 */
declare function anchoredVwapAnalysis(bars: readonly Bar[], anchorTime: number, options?: AnchoredVwapOptions): AnchoredVwapResult;
/**
 * Estimate a volume histogram over the inclusive timestamp range. Each candle's
 * volume is uniform over low..high; this is not trade-by-trade volume. Difference
 * arrays distribute interior rows without visiting each row per candle. Row
 * lookup is bounded by 200 edges: O(bars log(rows) + rows).
 */
declare function fixedRangeVolumeProfileAnalysis(bars: readonly Bar[], fromTime: number, toTime: number, options?: FixedRangeVolumeProfileOptions): FixedRangeVolumeProfileResult;

/**
 * Clipboard transfer for drawings: the layer between the drawing model and the
 * OS clipboard.
 *
 * Two things make this more than `JSON.stringify`:
 *
 * 1. **The OS clipboard is shared with everything else on the machine.** A
 *    paste can arrive from a spreadsheet, another application, or a hand-edited
 *    copy of our own payload. So the payload is written under a namespaced key
 *    and everything read back is validated field by field before it can reach
 *    the model. Foreign text is *not ours* and is ignored, never an exception
 *    the host has to catch on every Ctrl+V.
 * 2. **Clipboard access is async and permission-gated.** `navigator.clipboard`
 *    rejects when the document is not focused, the page is not secure, or the
 *    user denied the permission. Copy must still work inside the tab, so every
 *    write also lands in a module-level in-memory clipboard which is shared by
 *    every controller in the page. That is what makes chart-to-chart paste work
 *    with the permission denied, and it is why the memory store is a singleton
 *    rather than per-controller state.
 *
 * The payload body is a drawings document (`DrawingsDocument`), so a copy made
 * by a 1.9.x build (a version 1 body carrying the old style-bag text fields)
 * is upgraded by the same migration a saved layout goes through.
 *
 * A `paneIndex` in the payload counts the price pane as 0 and the study panes
 * after it in their order, whatever slot the price pane holds on the chart it
 * came from: `DrawingController` converts on copy and on paste, so a drawing
 * copied beside the candles pastes beside the candles on a chart that keeps
 * its price pane below its studies. On a chart with the price pane on top,
 * the default, that is the chart's own slot, as it always was.
 */

/**
 * Top-level key of the JSON payload. Namespaced so a paste of arbitrary text,
 * or of JSON belonging to some other application, is recognisable as not ours
 * by looking at one property.
 */
declare const DRAWING_CLIPBOARD_KEY = "openalgo-charts/drawings";
/**
 * The newest payload version this build reads. Tracks the model version, since
 * the body *is* a drawings document: version 1 carried 1.9.x drawings,
 * version 2 carries the split text and levelled fibs, and version 3 adds the
 * interval range a drawing is shown on. A payload is written as the lowest
 * version that holds it, so a copy with no range still pastes into a build
 * that reads only version 2, which refuses anything newer.
 */
declare const DRAWING_CLIPBOARD_VERSION: number;
/** The async slice of `navigator.clipboard` this module uses. */
interface ClipboardPort {
    writeText(text: string): Promise<void>;
    readText(): Promise<string>;
}
/** Test seam: drop whatever the in-memory clipboard is holding. */
declare function clearMemoryClipboard(): void;
/** `navigator.clipboard` when the browser exposes it, else null. */
declare function systemClipboard(): ClipboardPort | null;
/** Deep copy of the persisted fields, so a clone shares nothing with its source. */
declare function cloneDrawing(d: Drawing): Drawing;
/** Serialise drawings into the namespaced payload written to the clipboard. */
declare function encodeClipboardPayload(drawings: readonly Drawing[]): string;
/**
 * Validate one entry into a drawing with no id. Returns null for anything that
 * cannot be rendered: an unknown tool would throw inside the controller, and a
 * NaN anchor produces a drawing that can never be drawn or hit-tested again.
 * Accepts the 1.9.x shape too, lifting its text fields into `text`.
 */
declare function sanitizeDrawing(value: unknown): Omit<Drawing, 'id'> | null;
/**
 * Parse clipboard text into drawings, or null when the text is not ours or is
 * not usable. All-or-nothing on purpose: a payload with one corrupt entry is a
 * corrupt payload, and pasting the other nine silently would be worse than
 * pasting none.
 */
declare function decodeClipboardPayload(text: unknown): Omit<Drawing, 'id'>[] | null;
interface DrawingClipboardOptions {
    /** Where to read and write. Defaults to `navigator.clipboard` when present. */
    port?: ClipboardPort | null;
    /**
     * Keep a copy in the shared in-memory clipboard so copy and paste keep
     * working when the OS clipboard is unavailable or the permission is refused.
     * Default true. Set false to make a failed system write a failed copy, which
     * a host that must not silently lose cross-tab transfer may prefer.
     */
    fallbackToMemory?: boolean;
}
/**
 * The clipboard as the controller sees it: drawings in, drawings out, no
 * exceptions escaping. Every failure mode (no clipboard API, refused
 * permission, foreign text, corrupt payload) reduces to `false` or `null`.
 */
declare class DrawingClipboard {
    private _port;
    private _fallback;
    /** Why the last operation did not reach the OS clipboard, if it did not. */
    private _lastError;
    constructor(options?: DrawingClipboardOptions);
    /** Swap the port at runtime (a host granting permission later, or a test). */
    setPort(port: ClipboardPort | null): void;
    /**
     * Turn the in-memory backstop off or on. With it off, a refused system write
     * is a failed copy, which is what a host wants when a cut that cannot reach
     * the OS clipboard must not delete the drawing.
     */
    setFallbackToMemory(enabled: boolean): void;
    /**
     * Why the OS clipboard was not used by the last call, or null when it was.
     * Set on a successful copy too, when the payload reached memory but not the
     * system clipboard: that copy works in this tab and will not appear in
     * another, which is exactly what a host wants to be able to tell the user.
     */
    lastError(): string | null;
    /**
     * Write drawings out. Resolves true when the payload is retrievable by a
     * later `read()` (through the system clipboard, or through memory when the
     * fallback is on), false when it is not, so a caller doing a cut knows
     * whether it is safe to delete the original.
     */
    write(drawings: readonly Drawing[]): Promise<boolean>;
    /**
     * Read drawings back, or null when there is nothing of ours to paste. The
     * system clipboard wins when it holds our payload, so a copy made in another
     * tab beats a stale in-tab one; memory is consulted when the read fails or
     * returns something that is not ours. That last case can paste an older copy
     * after the user has copied unrelated text elsewhere, which is the deliberate
     * trade: losing the copy whenever a browser refuses the read half of the
     * permission would be the worse surprise.
     */
    read(): Promise<Omit<Drawing, 'id'>[] | null>;
}

/**
 * The public shapes of the drawing controller: the slice of the chart it
 * drives, its options, and the events and edit options its methods take.
 * Kept apart from the controller so the class reads as behaviour, and so a
 * module that needs one of these (the gestures read the gesture switches)
 * names it without importing the controller itself.
 */

/**
 * The slice of the chart this controller needs.
 *
 * Declared structurally rather than as `Chart` on purpose. Each tier ships its
 * own bundled `.d.ts`, so naming the class here made the draw tier re-declare
 * `Chart`, and because `Chart` has private members, TypeScript treats the two
 * declarations as *different* types. A TS consumer passing the chart from
 * `createChart()` got "separate declarations of a private property", which made
 * the tier unusable from TypeScript at all. An interface with no private
 * members is structural, so the real `Chart` satisfies it with nothing to cast.
 */
interface DrawingChartHost {
    readonly isDestroyed?: boolean;
    /**
     * The event bus, with the names and payloads of `ChartEventMap`. The
     * controller listens for `click`, `crosshair:move`, `drag`, `drag:end`,
     * `drag:cancel`, `dblclick` and `hover` (`{ id }`, the hit id under the pointer
     * whenever it changes), which is what drives the hover state: the chart has
     * already hit-tested the move, so the controller reads its answer rather than
     * testing a second time. A host that never emits `hover` has drawings that
     * select and drag but do not light up. It also follows `data:context`,
     * `drawings:restore`, `pick:start`, `pick:end`, `paneRemoved`, `paneMoved`,
     * `objects:change` and `indicatorRemoved`; a host that moves or removes panes
     * without `paneRemoved` and `paneMoved` leaves drawings on the old slots. It
     * emits the `draw:*` and `drawing:*` names this tier adds to `ChartEventMap`.
     */
    on(event: string, handler: (payload: unknown) => void): () => void;
    emit(event: string, payload: unknown): void;
    addPrimitive(primitive: IPrimitive, paneIndex?: number): void;
    removePrimitive(primitive: IPrimitive): void;
    readonly dataLayer: DataLayer;
    getVisibleLogicalRange(): {
        from: number;
        to: number;
    } | null;
    drawingState(): unknown;
    setDrawingState(state: unknown): void;
    setPlacementMode?(active: boolean): void;
    /**
     * Optional, and used only to move a drawing by a fixed screen distance (a
     * paste offset, an arrow-key nudge, a multi-drag across panes). Going
     * through pixels rather than adding a price delta keeps the offset the same
     * visible nudge on a log scale as on a linear one, and the same on an RSI
     * pane as on the price pane. A host without them still gets the time half.
     */
    priceToCoordinate?(price: number, paneIndex?: number): number | null;
    coordinateToPrice?(y: number, paneIndex?: number): number | null;
    /**
     * Optional, the time-axis half of the same conversion. coordinateToTime also
     * keeps previews and freehand strokes active in empty space beyond the bars.
     * Without these methods a horizontal nudge assumes the default bar spacing.
     */
    timeToCoordinate?(time: number): number;
    coordinateToTime?(x: number): number;
    /**
     * Optional, for drawings anchored to the viewport (`space: 'viewport'`):
     * the time axis, which turns a data anchor's time into a place on the plot
     * and back when a drawing is pinned or unpinned.
     */
    readonly timeScale?: {
        indexToX(index: number): number;
        xToIndex(x: number): number;
    };
    /**
     * Optional, for drawings anchored to the viewport: a pane's plot in
     * container px, as `Chart.plotRect` reports it, which is what a viewport
     * anchor is a fraction of. A host without it still paints them, since the
     * layer reads the plot size from its render context, but cannot place,
     * move or convert them.
     */
    plotRect?(paneIndex: number): PlotRect | null;
    /**
     * Optional. It keeps a paste from a chart with more panes than this one
     * landing on a pane the user cannot see: adding a primitive creates the pane
     * it names, so without this a drawing copied out of an indicator pane would
     * conjure an empty pane in a single-pane chart. An entry with `priceToY` and
     * `yToPrice`, as a chart pane has, also lets an alert read a drawing on a
     * pane the chart maps no price for, one collapsed to its header strip.
     */
    panes?(): readonly unknown[];
    /**
     * Optional. Slot of the price pane, the one pane whose drawings the magnet
     * snaps to candle prices and a drawing link shares. It moves when a host
     * puts the price pane below its studies; without it the price pane is slot 0.
     */
    primaryPaneIndex?(): number;
    /**
     * Optional, all three, for the magnet: the price series' bars, the studies
     * and a series' style. The chart reports no bar while a drawing is in hand,
     * so a handle or a shape snaps against `primaryBars`; a study pane snaps
     * to the values of the studies plotted on it, skipping a hidden plot. A
     * host without them snaps placements on the price pane only.
     */
    primaryBars?(): readonly Bar[];
    indicators?(): readonly IndicatorApi[];
    seriesStyle?(series: SeriesApi): {
        readonly visible?: boolean;
        readonly color?: string;
    } | null;
    /**
     * Optional, both: a pane's series band, back to front, and the call that
     * paints a layer directly above one of its entries. Without them no drawing
     * can be placed in the series band, and one saved there paints by its
     * `zIndex`.
     */
    seriesStack?(paneIndex: number): readonly string[];
    setPrimitiveStackAbove?(primitive: IPrimitive, above: string | null): boolean;
    /**
     * Optional: the host's data context, whose `interval` decides which
     * drawings with an interval range are shown (`Drawing.intervals`). Read
     * when the controller is built and on every `data:context`, whose payload
     * is the context itself; a host without it that emits `data:context` is
     * followed through the payload alone. With no interval every drawing shows.
     * The instrument is declared too, as the chart's context carries it, so one
     * host object also satisfies `DrawingLinkChart`.
     */
    getDataContext?(): {
        readonly symbol?: string;
        readonly exchange?: string;
        readonly interval?: string;
    } | undefined;
}
interface DrawingControllerOptions {
    /**
     * Snap new anchors to the O/H/L/C of the bar under the cursor. `'strong'`
     * always takes the nearest of the four; `'weak'` only when one sits within
     * a few pixels of the pointer, so a click on open space stays where it was
     * made. `true` means `'strong'` and `false` means `'off'`, which is what
     * the boolean meant before the modes existed. Default `'off'`. While a
     * tool is armed the layer paints a ring where the next click will land.
     */
    magnet?: boolean | MagnetMode;
    /** Style merged under every tool's own defaults. */
    defaultStyle?: DrawingStyle;
    /** Stay in the active tool after finishing a drawing. Default false. */
    stayInDrawingMode?: boolean;
    /** Undo depth. Default 50. */
    historyLimit?: number;
    /**
     * Where copy and paste move text. Defaults to `navigator.clipboard`; pass a
     * port to route through a host's own transfer, or `null` to stay in the
     * process-local clipboard entirely.
     */
    clipboard?: ClipboardPort | null;
    /**
     * Whether a refused or failing clipboard write still lands in the in-process
     * clipboard. Defaults to true, which is what makes copy and paste work between
     * two charts on a page where the browser has denied clipboard permission.
     *
     * Pass false for a host that would rather a failed copy be a failed copy: with
     * it off, `cut` leaves the drawing alone when the write does not land, so a
     * shape is never destroyed for a transfer that did not happen.
     */
    clipboardFallbackToMemory?: boolean;
    /**
     * How far a pasted or duplicated copy lands from its original, in bars along
     * time and in screen pixels down the price axis. A copy that lands exactly on
     * top of the original reads as nothing having happened. Defaults: 2 bars,
     * 16 px.
     */
    pasteOffsetBars?: number;
    pasteOffsetPixels?: number;
    /**
     * Draw the anchor of every study input that declares one (a `price` input
     * with a `timeKey` and `anchor: true`): a handle at the point the pair
     * names that drags both as one settings change and one step of this undo
     * history. Default true; false draws none, and the inputs are still edited
     * in settings and picked with `Chart.beginPick('point')`. Read when the
     * controller is built: `setOptions` does not change it.
     */
    inputAnchors?: boolean;
    /**
     * The pointer gestures a held modifier key starts, each on unless set to
     * false here, for a host whose own chart gestures already use that key.
     * `setOptions` merges it flag by flag.
     */
    gestures?: DrawingGestureOptions;
}
/**
 * The modifier gestures `DrawingControllerOptions.gestures` turns off. Ctrl
 * means Cmd as well, for macOS. None of them is a key chord, so none can
 * shadow one of `matchDrawingShortcut` or `keyToDrawingAction`.
 */
interface DrawingGestureOptions {
    /**
     * Ctrl held while placing an anchor or dragging a handle or a shape: the
     * strong magnet for as long as it is held, whatever `magnet` is set to.
     */
    snapModifier?: boolean;
    /**
     * Shift+click on empty chart space, with no tool armed: a ruler from the
     * click to the pointer, gone on the next click or `cancel()`. A preview,
     * never a drawing: not saved, not an undo step, never linked.
     */
    measure?: boolean;
    /**
     * Ctrl plus a drag on empty chart space, with no tool armed: a box that
     * selects every drawing whose geometry it touches, as a click there would
     * find it, leaving out drawings that are hidden, locked or unselectable.
     * Shift held as well adds them to the selection. While Ctrl is held over
     * empty space the chart is in placement mode, so the press is not a pan.
     */
    boxSelect?: boolean;
    /**
     * Alt (Option) plus a drag on a drawing's body: the drawing stays where it
     * is and a copy of it moves instead, as one undo step. The whole movable
     * selection is copied, as a plain drag would move it, and each copy is the
     * user's own drawing, with no policy and no link lineage. A copy waits for
     * the pointer to travel three pixels, so a jitter leaves none behind.
     */
    dragCopy?: boolean;
}
/** How `DrawingController.setTool` arms a tool. */
interface DrawingPlacementOptions {
    /**
     * The space the placed drawing is anchored in. `'viewport'` pins it to the
     * screen: the anchors are placed where they are clicked, as usual, and kept
     * as fractions of the pane's plot from then on. Only a tool that declares
     * `viewport` accepts it. Default `'data'`.
     */
    space?: DrawingSpace;
}
/**
 * What `drawing:change` reports happened to the listed ids. The list is empty
 * for a step of the undo history that changed no drawing, a study input
 * anchor's drag and its undo or redo, so a control showing whether Undo is
 * available still refreshes.
 */
type DrawingChangeKind = 'add' | 'update' | 'remove' | 'reorder' | 'undo' | 'redo';
/** The `drawing:change` payload. */
interface DrawingChangeEvent {
    ids: string[];
    kind: DrawingChangeKind;
    /** Set on a change another chart's link applied; it records no step here. */
    linked?: true;
    /**
     * The undo step this change was recorded as, on the change that closes it.
     * Absent for everything the history does not hold: a host's forced edit, a
     * linked commit, a restore, and a move along the branches (`undo`, `redo`).
     * {@link DrawingController.historySteps} lists the step by this number.
     */
    step?: number;
}
/**
 * Payload of `draw:add`, `draw:update` and `draw:remove`: the drawing itself.
 * `history` is set when an undo or redo step applied the change.
 */
interface DrawingEvent {
    drawing: Drawing;
    history?: true | undefined;
}
/** Payload of `draw:copy`, `draw:cut`, `draw:paste` and `draw:preview`. */
interface DrawingListEvent {
    drawings: Drawing[];
}
/** Payload of `drawing:select` (the whole selection, in pick order) and `draw:preview-clear`. */
interface DrawingIdsEvent {
    ids: string[];
}
/** Payload of `draw:select` (the primary selection) and `drawing:hover`; null when there is none. */
interface DrawingIdEvent {
    id: string | null;
}
/** Payload of `draw:tool`: the active tool, null when none; `space` only for a screen-pinned placement. */
interface DrawingToolEvent {
    tool: string | null;
    space?: 'viewport' | undefined;
}
/** Payload of `draw:measure` and `draw:eraser`: the mode switched on or off. */
interface DrawingModeEvent {
    active: boolean;
}
/** Options for a call that changes, groups or deletes drawings. */
interface DrawingEditOptions {
    /**
     * Reach drawings whose policy sets `editable: false` as well, and the
     * groups that hold them. Without it they are left exactly as they are,
     * which is what keeps every control a host wires to the user off them. The
     * host that placed such a drawing passes it to move, restyle, regroup or
     * retire it. A forced call is the host's own act, not the user's, so it
     * records no undo step, and every step already recorded takes it too: no
     * later undo or redo reverses it, and a step it leaves with nothing to do
     * is dropped.
     *
     * Cost: taking a call into the recorded steps is one pass over the undo
     * and redo history, parsing and rewriting both snapshots of every step, so
     * it grows with the number of recorded steps (see `historyLimit`) times
     * the drawing count. A forced delete, a forced grouping call, a forced
     * patch to a drawing the user may edit or one that carries `zIndex`, any
     * patch that carries `policy` and a linked chart's change of policy make
     * that pass. A forced patch to a read-only drawing that carries neither
     * `policy` nor `zIndex` (a level the host trails on every tick) makes
     * none: history cannot reach that drawing's content while it stays
     * read-only, so the patch is held and goes in with the next pass, which a
     * change of its policy always makes.
     */
    force?: boolean;
}

/** Where a study's price input is picked and anchored: a pane, and a scale on it. */
interface StudyInputTarget {
    paneIndex: number;
    priceScaleId: PriceScaleId;
}
/**
 * One anchor move as an undo step, walked by the drawing history or by the
 * host's timeline it was handed to (`DrawingController.delegateInputAnchorSteps`).
 * Each call writes the other side's point, and is false, writing nothing, once
 * the study no longer holds the point it starts from.
 */
interface InputAnchorStep {
    undo(): boolean;
    redo(): boolean;
}
/**
 * The pane and scale a study's `price` input is picked on and anchored to,
 * or null when that cannot be told. An explicit `pick` target with both
 * halves is used as given. Otherwise the study's own plots decide: the price
 * belongs where they are drawn, which must be one scale on one pane (and
 * inside any half the descriptor did name), since a price read off the wrong
 * scale is a different number.
 */
declare function studyInputTarget(chart: {
    panes?(): readonly unknown[];
}, study: IndicatorApi, key: string): StudyInputTarget | null;

/**
 * Drawing controller: the interaction and persistence layer over
 * `DrawingLayer`. It is **headless**: no DOM, no toolbar. A host sets the
 * active tool (from its own button, a shortcut, a command palette) and the
 * controller runs placement, selection, dragging, undo, and serialisation.
 *
 * It listens on the chart's event bus (`click`, `crosshair:move`, `drag`,
 * `drag:end`) rather than `subscribeClick`/`subscribeDrag`, which stay the
 * host's for its own order lines.
 *
 * Selection is a list. Every method that edits takes the whole list into one
 * undo entry, so a multi-drag, a batch delete or a paste of ten shapes is one
 * Ctrl+Z, which is what the hand that did it expects.
 */

declare class DrawingController {
    private readonly _chart;
    private _opts;
    private readonly _clipboard;
    /** Every conversion between data space and the screen. */
    private readonly _screen;
    /** The gestures that make no drawing: the temporary measure, box select and the eraser. */
    private readonly _gestures;
    /** Every pane's drawing layers; see pane-layers.ts. */
    private readonly _layerSet;
    private _drawings;
    private _groups;
    private _nextGroup;
    private readonly _linkedPreviews;
    private _destroyed;
    private _tool;
    /** The space the armed tool places in; data whenever no tool is armed. */
    private _toolSpace;
    private _pending;
    private _pendingPane;
    /** Selected ids, in the order they were picked. The first is the primary. */
    private _selection;
    /** The drawing under the pointer, from the chart's own hit-test. */
    private _hovered;
    /** Shift as of the last pointer report: what angle lock reads mid-preview. */
    private _shift;
    /** Ctrl or Cmd as of the last pointer report: the strong magnet while held. */
    private _strong;
    /** Placement mode as this controller last set it. */
    private _placing;
    /** The chart interval drawings are shown for; see `interval()`. */
    private _interval;
    /** The device behind the last pointer report, for target sizing. */
    private _pointerKind;
    /** Undo, redo and the host's own edits; see drawing-history.ts. */
    private readonly _history;
    /** The drag of a drawing or its handle; see drawing-drag.ts. */
    private readonly _drag;
    private readonly _off;
    private _anchors;
    /** The host's timeline an anchor step goes to instead of this history; see `delegateInputAnchorSteps`. */
    private _anchorSteps;
    private _lastCursor;
    /**
     * Bar under the cursor, carried by the crosshair event, with the time the
     * crosshair reported for it: the magnet lands anchors on that bar's values
     * at that bar's time.
     */
    private _lastBar;
    constructor(chart: DrawingChartHost, options?: DrawingControllerOptions);
    /**
     * Arm a tool for placement, or pass null to return to the cursor. With
     * `options.space` set to `'viewport'` the drawing it places is pinned to the
     * screen; a tool that cannot be (see `DrawingTool.viewport`) throws.
     */
    setTool(toolId: string | null, options?: DrawingPlacementOptions): void;
    /** The space the armed tool places in: `'data'` unless it was armed for the viewport. */
    activeToolSpace(): DrawingSpace;
    /** `draw:tool`, carrying the space only when it is not the default, as the payload always has. */
    private _emitTool;
    /**
     * Ask the chart to stop panning and report gestures as anchor placement.
     * Guarded so a base bundle predating `setPlacementMode` still loads the tier.
     */
    private _setPlacementMode;
    /**
     * The armed tool's placement mode, or a gesture's. The gestures are asked even with a tool
     * armed: their answer is also how they read the next press, and a stale one would take it.
     */
    private _placementWanted;
    activeTool(): string | null;
    setOptions(patch: DrawingControllerOptions): void;
    /** The snap mode in force, after the boolean form has been folded. */
    magnetMode(): MagnetMode;
    /**
     * Whether a temporary measure (Shift+click on empty space) is on the chart.
     * `draw:measure` (`{ active }`) fires when one starts and when it goes. A
     * host routing Escape to `cancel()` only while placing passes this too.
     */
    measuring(): boolean;
    /**
     * Eraser mode: a click on a drawing deletes it, and a drag deletes every
     * drawing it crosses, as one undo step on release. What the user could not
     * delete by selecting it (read-only, locked, unselectable or hidden) stays.
     * While it is on the chart is in placement mode, so a drag erases rather
     * than pans. Turning it on disarms any tool; arming a tool, `cancel()` and
     * `setEraser(false)` turn it off. `draw:eraser` (`{ active }`) fires on
     * each change.
     */
    setEraser(active: boolean): void;
    /** Whether eraser mode is on. */
    erasing(): boolean;
    /**
     * The drawing under the pointer, selected or not, as the chart's hit-test
     * last reported it. What a host passes as `hasTarget` to the key mapping,
     * and what a context menu opens on.
     */
    hovered(): string | null;
    /** The clipboard behind copy / cut / paste, for a host reporting failures. */
    clipboard(): DrawingClipboard;
    /** Named drawing sets, returned as detached records. */
    groups(): readonly DrawingGroup[];
    /**
     * Group live drawings, replacing any previous membership for those ids. A
     * read-only drawing's group is the host's, so it is left where it is
     * unless `options.force` is set.
     */
    createGroup(name: string, ids: readonly string[], options?: DrawingEditOptions): DrawingGroup | null;
    /** Rename a group. One that holds a read-only drawing is the host's, and needs `options.force`. */
    renameGroup(id: string, name: string, options?: DrawingEditOptions): boolean;
    /**
     * Remove a group, optionally deleting all its drawings as one edit. One
     * that holds a read-only drawing is the host's, and needs `options.force`.
     */
    removeGroup(id: string, removeDrawings?: boolean, options?: DrawingEditOptions): boolean;
    /**
     * Make a grouping edit. The host's forced one is its own act, so every
     * recorded step takes it as well, and no undo or redo reverses it.
     */
    private _regroup;
    /** The live group `id`, unless it holds a read-only drawing and the call is not forced. */
    private _group;
    /**
     * Move one step through the rendered stack, within the slot it paints in:
     * its side of the series, or the entry it is placed above. `placeInStack`
     * moves it between slots.
     */
    reorder(id: string, direction: -1 | 1): boolean;
    /**
     * Move a drawing directly above or below `target` in its pane's paint
     * order, as one undo step. Next to another drawing it joins the slot that
     * drawing paints in; above a series-band entry (`chart.seriesStack`) it is
     * placed on that entry, under the drawings already there; below one it goes
     * on top of the slot under that entry, the drawings behind the series when
     * the entry is the first. A slot's drawings are renumbered, below the series
     * up to -1 and elsewhere from 0. False, with nothing recorded, for a target
     * on another pane, one the host cannot report, or a move that changes nothing.
     * Like `reorder`, it is outside a drawing's policy.
     */
    placeInStack(id: string, target: DrawingStackTarget, where: 'above' | 'below'): boolean;
    /** A pane's series band as the chart reports it, or none on a host that cannot. */
    private _entries;
    /** The drawings of one slot of a pane, in paint order. */
    private _slotMembers;
    private _remapPanes;
    /**
     * Every drawing, in list order, those the interval hides included. That is
     * creation order until a z-order call moves one; the list is the tie-break
     * for equal `zIndex`, so it is also the paint order within a band.
     */
    drawings(): readonly Drawing[];
    get(id: string): Drawing | undefined;
    /**
     * The chart interval drawings are shown for: the data context's `interval`
     * when the controller was built, then as each `data:context` announces it,
     * or null when the host names none (or a blank one), and then every drawing
     * shows. Each drawing's `intervals` range is read against it.
     */
    interval(): string | null;
    /**
     * Whether a drawing is on the chart at the chart's interval: false for an
     * unknown id and for one whose `intervals` range leaves that interval out.
     * `visible` is not read; it is the user's own switch.
     */
    shownOnInterval(id: string): boolean;
    /**
     * The ids of the drawings the chart's interval leaves off the chart, in
     * model order. One pass for a caller asking about every drawing, as the
     * object inventory does on each refresh: `shownOnInterval` per id would
     * look each one up.
     */
    hiddenOnInterval(): string[];
    /** The test of whether a drawing is shown at the chart's interval, resolved once for a pass over many. */
    private _shownFilter;
    /**
     * Follow a change of the chart's interval: what it hides leaves the layers
     * and the selection, what it shows comes back. The layers are listed again
     * only when that set changed, since a list under the series costs the
     * series a repaint and a new symbol on the same interval changes nothing.
     */
    private _followInterval;
    get isDestroyed(): boolean;
    /** Apply a linked commit without adding to this chart's local undo history. */
    applyLinkedDrawing(id: string, drawing: Drawing | null): void;
    /** A linked drag paints over its committed drawing without changing saved state. */
    setLinkedPreview(id: string, drawing: Drawing | null): void;
    /** Reorder related drawings inside their existing slots, preserving unrelated local order. */
    reorderLinkedDrawings(ids: readonly string[]): void;
    /** Supported numeric levels, independent of whether the queried time lies on the shape. */
    alertInfo(id: string): AlertDrawingInfo;
    /** Numeric drawing value using the actual pane projection and collapsed time axis. */
    valueAt(id: string, time: number, level?: string): AlertDrawingValue | undefined;
    /**
     * Add a fully-specified drawing (import, or a host-authored one). One that
     * carries a restriction is the host's, and placing it is not a step the
     * user can take back, so it is not recorded.
     */
    add(drawing: DrawingInput): Drawing;
    /**
     * Append one drawing without touching history or the layers. Split out so a
     * paste of several drawings is a single undo step rather than one per shape.
     */
    private _insert;
    private _mintId;
    /**
     * Patch one drawing. False when there is no such drawing, or when it is
     * read-only to the user and `options.force` is not set. A patch that
     * carries `policy` is the host's, and like a forced one records no step.
     *
     * Also false when the patch asks for a `space` the drawing could not be
     * moved to (a tool without viewport support, or a pane with no place on
     * screen, folded or hidden behind a maximized pane): the rest of the patch
     * still applies, and the false tells a host that the drawing is not in the
     * space it asked for, which a control showing that space has to say.
     */
    update(id: string, patch: DrawingPatch, options?: DrawingEditOptions): boolean;
    /**
     * Patch several drawings as one undo entry: a colour change across a
     * multi-selection is one edit to the user, so it is one Ctrl+Z too. Ids that
     * no longer exist are skipped, and so are read-only drawings unless
     * `options.force` is set; nothing is recorded when nothing is left, or
     * when every patch is the host's (forced, or carrying `policy`).
     */
    updateMany(patches: ReadonlyArray<{
        id: string;
        patch: DrawingPatch;
    }>, options?: DrawingEditOptions): void;
    /**
     * A patch as it applies to `d`. A change of space comes out complete, with
     * the anchors of the new space (converted at the view on screen unless the
     * patch gives them), or is left out when it cannot be made; anchors of the
     * space the drawing will not be in are left out too, so one drawing never
     * carries two positions.
     */
    private _spacePatch;
    private _applyPatch;
    /** Delete one drawing. A read-only one goes only with `options.force`. */
    remove(id: string, options?: DrawingEditOptions): boolean;
    /**
     * Delete several drawings as one undo entry. Unknown ids are ignored, and
     * read-only drawings stay unless `options.force` is set.
     */
    removeMany(ids: readonly string[], options?: DrawingEditOptions): void;
    /**
     * The delete shared by `remove`, `removeMany`, `cut` and `clear`. Returns
     * what went, and records nothing when nothing did.
     */
    private _removeIds;
    /** Delete every drawing the user may, as one undo entry; `options.force` takes the read-only ones too. */
    clear(options?: DrawingEditOptions): void;
    /**
     * Replace the selection, or with `additive` toggle each id into it (the
     * shift-click gesture). Ids that name nothing, or a drawing whose policy
     * says it cannot be selected, are ignored, so the selection only ever holds
     * drawings that are there to act on. Pass null to clear.
     */
    select(id: string | readonly string[] | null, additive?: boolean): void;
    private _selectable;
    /** The primary selection: the first id picked, or null. */
    selected(): string | null;
    /** Every selected id, in the order they were picked. */
    selection(): readonly string[];
    private _setSelection;
    private _emitChange;
    setZIndex(id: string, z: number): void;
    /** In front of every other drawing in its slot: its side of the series, or the entry it is placed above. */
    bringToFront(id: string): void;
    /** Behind every other drawing in its slot: its side of the series, or the entry it is placed above. */
    sendToBack(id: string): void;
    /**
     * Under the series (`zIndex` -1), out of the series band when it was placed
     * in it. A no-op for a drawing already there.
     */
    sendBehindSeries(id: string): void;
    /**
     * Over the series (`zIndex` 0), out of the series band when it was placed
     * in it. A no-op for a drawing already there.
     */
    bringAboveSeries(id: string): void;
    private _crossSeries;
    /** The other drawings sharing `d`'s pane and slot. */
    private _band;
    private _reorder;
    /**
     * Move drawings by a screen distance, `dx` right and `dy` down in media px,
     * as one undo entry. Pixels rather than data units so an arrow key moves a
     * shape the same visible amount on every pane and scale. Locked and
     * read-only drawings stay put.
     */
    nudge(ids: readonly string[], dxPx: number, dyPx: number): void;
    /**
     * Clone drawings, offset like a paste so the copies are visibly new, and
     * select the clones. One undo entry. Ids that name nothing are ignored. A
     * clone is the user's own drawing, so it carries no policy.
     */
    duplicate(ids: readonly string[]): Drawing[];
    /**
     * Put drawings on the clipboard. Defaults to the selection; pass an id or a
     * list of ids to copy something else. Resolves false when there was nothing
     * to copy, or when the payload could not be stored anywhere.
     */
    copy(target?: string | readonly string[] | null): Promise<boolean>;
    /**
     * Copy, then delete. The delete happens **only** after the clipboard write
     * resolves successfully, so a refused write leaves the model exactly as it
     * was rather than destroying a drawing that went nowhere. A read-only
     * drawing cannot be deleted, so it is not cut either: it stays, uncopied.
     */
    cut(target?: string | readonly string[] | null): Promise<boolean>;
    /**
     * Paste whatever is on the clipboard into this chart, offset from the
     * original so the copy is visibly a second object, and select the result.
     * Each pasted drawing is a fresh object with a fresh id, never a second
     * reference to the one copied, so editing the paste cannot alter its source
     * (or the clipboard).
     *
     * Anything that is not our payload (foreign text, a truncated or hand-edited
     * copy, a newer format) pastes nothing and resolves to an empty array: a
     * paste shortcut must not throw at the host because the user last copied a
     * spreadsheet cell.
     */
    paste(): Promise<Drawing[]>;
    /** Resolve an id list to live drawings; defaults to the selection. */
    private _targets;
    /** The slot this chart keeps its price pane in, read at each use because a host can move it. */
    private _pricePane;
    /**
     * Drawings as the clipboard carries them: panes counted price pane first,
     * the study panes after it in their order, the way a portable template
     * counts them. A price pane at the top, where every build before the move
     * kept it, is written exactly as before, and a drawing copied beside the
     * candles pastes beside the candles on any chart in any arrangement.
     */
    private _portable;
    /**
     * Fold a clipboard pane onto a pane this chart actually has: clamped to the
     * pane count in clipboard order, so a study drawing from a taller stack
     * lands on the last study pane rather than on the price pane, then put in
     * this chart's own slots around its price pane.
     */
    private _clampPane;
    /**
     * A copy's anchors, offset from the original's in its own space. A viewport
     * copy moves the paste offset in pixels on both axes, as a fraction of the
     * pane it lands on, so it reads the same on a chart of any size, and stays
     * on that pane's plot when the original sits at its edge.
     */
    private _offsetAnchors;
    /** Nudge every anchor so a pasted copy is not hidden under its original. */
    private _offsetPoints;
    /**
     * Move a study's input anchor (a price input declared with `timeKey` and
     * `anchor: true`) to `point` as the user, the way dragging its handle does:
     * the time snaps to the bar under it, both halves stay inside the bounds the
     * inputs declare, a study the user may not configure refuses it, and the move
     * is one undo step in this history. For a host control that sets the point
     * another way, such as a point pick on the chart, so Undo takes it back like
     * a drag. A point written through the study's settings instead (a settings
     * dialog's Pick point) is one step of this history as well, unless it is
     * written inside `untracked` or forced on a study the user may not
     * configure, which are the host's own. False when the study has no anchor for `key` (or the controller was
     * built without input anchors), the study refuses, or it already holds the point.
     */
    moveInputAnchor(studyId: string, key: string, point: {
        time: number;
        price: number;
    }): boolean;
    /**
     * Hand the step each study input anchor move makes (a handle dragged, a
     * `moveInputAnchor`) to a timeline of the host's instead of this history,
     * until the returned function gives them back. For a host keeping one undo
     * history for the whole chart that already records the settings patch the
     * move writes, as the widget's `ChartHistory` does: held here as well, one
     * move would be taken back twice, and a press here would reach a step the
     * other timeline had already taken back. `record` is called once the patch
     * is written, in the same turn, with the step's own `undo` and `redo`
     * (each false once the settings have moved on from it); this history
     * records nothing for it and emits no `drawing:change`. A move made inside
     * `untracked` is the host's own and reaches neither. A later call takes
     * the steps from an earlier one. A point written to the settings some other
     * way, which this history otherwise holds as a step of its own, is not
     * handed over: the host's timeline sees that write itself.
     */
    delegateInputAnchorSteps(record: (step: InputAnchorStep) => void): () => void;
    undo(): boolean;
    redo(): boolean;
    /** Drop selected ids the model no longer holds, or that can no longer be selected. */
    private _pruneSelection;
    canUndo(): boolean;
    canRedo(): boolean;
    /**
     * The steps each branch holds, oldest first, by the number `drawing:change`
     * reported them under. For a host that keeps one timeline across drawings
     * and its own edits: a step missing from both branches has been taken away
     * (a reset, a trim, a host edit that left it nothing to do), and pressing
     * undo for it would reach an older step instead. A step still being
     * recorded, a drag in progress, is not listed until its change closes it.
     */
    historySteps(): {
        undo: number[];
        redo: number[];
    };
    /**
     * Run `fn` as the host's own act. An edit it makes records no undo step and
     * leaves both branches as they are, and every step already recorded takes
     * it in, the way a forced call does, so no later undo or redo reverses it.
     * Unlike `force` it reaches no read-only drawing, and `undo` or `redo`
     * inside it still moves along the branches. For a host keeping one timeline
     * across drawings and its own changes, whose own changes are never steps.
     */
    untracked<T>(fn: () => T): T;
    /**
     * Serialisable document, the same shape `ChartState.drawings` carries.
     * Transient drawings (`policy.persistent` false) are left out, and so is
     * their group membership.
     */
    toJSON(): DrawingsDocument;
    /** `drawings` as a document, with the groups narrowed to them. */
    private _document;
    /** Every drawing, transient ones too: an undo in the session reaches them. */
    private _historyText;
    /**
     * Replace every drawing. Accepts a {@link DrawingsDocument} or a 1.9.x bare
     * `Drawing[]`; both go through the migration, so an old save upgrades on
     * load. Clears the selection and history.
     */
    fromJSON(data: unknown): void;
    destroy(): void;
    private _onCrosshair;
    /** The chart's hit-test answer for the pointer position, whenever it changes. */
    private _onHover;
    private _setHovered;
    /** Remember the device behind a report and size the layers' targets for it. */
    private _notePointer;
    /**
     * Let a tool turn the clicked anchors into its full set (the position tools
     * build a 1:1 box off one click). Identity for tools without the hook.
     */
    private _expand;
    private _isFreehand;
    /**
     * Append one sample to the stroke in progress. Points arriving closer than a
     * bar-eighth apart in time carry no shape and would bloat the saved drawing,
     * so they collapse into the last one: a pointer can fire far faster than the
     * stroke actually changes direction.
     */
    private _inkPoint;
    /** Commit `pts` as a drawing of the armed tool and leave placement. */
    private _commit;
    /**
     * Commit the stroke a freehand gesture built, if it has any extent. The
     * samples are thinned first: a pointer reports every few px, and a stroke
     * kept whole costs a time and price conversion per sample on every frame
     * and a row per sample in every save, for a curve the eye cannot tell from
     * the thinned one. Thinning happens in screen space, since the tolerance is
     * a pixel one; a host that cannot map to pixels keeps every sample.
     */
    private _finishFreehand;
    /**
     * End a variable-anchor shape (polyline, path) at the anchors placed so far.
     * Those tools declare `points: 0`, so nothing else can ever complete them;
     * without this they collected vertices forever. Bound to double-click, and
     * public so a host can offer Esc / Enter too. No-op when there is nothing
     * placeable, so a stray double-click costs nothing.
     */
    finish(): boolean;
    /**
     * Abandon whatever is being placed: the anchors so far are dropped and,
     * unless the controller stays in drawing mode, the tool is disarmed too, so
     * one Escape returns the chart to the cursor the way a finished shape
     * would. With nothing pending, an armed tool is simply disarmed. Returns
     * whether anything changed, so a host can let the key fall through when it
     * did nothing.
     */
    cancel(): boolean;
    /**
     * Remove the last anchor placed on a variable-anchor tool (polyline, path)
     * still being drawn: the Backspace of placement. A fixed-anchor tool has
     * nothing to pop, since its anchors commit the moment the last one lands,
     * and a freehand stroke is one gesture rather than a list. Returns whether
     * an anchor went.
     */
    popAnchor(): boolean;
    private _onClick;
    private _placePoint;
    /**
     * Where a click at `point` actually lands for the armed tool: on the 45
     * degree lock while Shift holds the free end of a line, else on the magnet
     * when it pulls, else where it was. The lock wins over the magnet because a
     * snapped price would bend the exact angle the lock exists to give.
     */
    private _aimPoint;
    /**
     * The free end of a two-anchor line under angle lock, or null when the lock
     * does not apply: no Shift, a tool without the flag, no anchor yet to
     * measure from, or a host that cannot map pixels (the lock is a screen
     * angle, so there is nothing to lock to in data space).
     */
    private _lockedPoint;
    /**
     * Where the magnet lands `point`, or null when it does not pull: the
     * nearest O/H/L/C of `bar` on the price pane, the nearest plotted value on
     * a study pane (`snap.ts`). Ctrl held pulls as the strong magnet whatever
     * the mode. A placement reads the bar the chart reported under the
     * pointer; a drag, which reports none, passes the bar under its time.
     */
    private _snapPoint;
    /** Shift and Ctrl (Cmd) as the last pointer report carried them: the angle lock and the strong magnet. */
    private _noteKeys;
    /** Roll back an interrupted drag and leave the pre-gesture undo/redo stacks intact. */
    cancelDrag(): boolean;
    /**
     * The drawing's anchors in container media px, the space `timeToCoordinate`
     * and `priceToCoordinate` answer in, whichever space it is anchored in. What
     * a host places an overlay by (an inline editor, a popover). Null for an
     * unknown id, or when the drawing's pane has no place on screen (folded to
     * a strip, or hidden behind a maximized pane).
     */
    screenPoints(id: string): ScreenPoint[] | null;
    /** Push the current list into each pane's layers and into the chart state. */
    private _sync;
}

/**
 * Drawings shared between linked charts of one instrument: a drawing made,
 * moved or removed on one chart is made, moved or removed on the others,
 * each with its own undo history. A shared drawing carries a lineage mark in
 * its props, so the link finds it again after a restore.
 */

/** Both fields must be known. A host may use its feed namespace as the exchange. */
interface DrawingLinkContext {
    symbol?: string;
    exchange?: string;
}
/** Resolve a host-owned identity without modifying the chart's data context. */
type DrawingLinkContextSource = DrawingLinkContext | (() => DrawingLinkContext | null | undefined);
interface DrawingLinkOptions {
    enabled?: boolean;
}
interface DrawingLinkChart extends Pick<DrawingChartHost, 'on' | 'primaryPaneIndex'> {
    readonly isDestroyed?: boolean;
    getDataContext?(): Readonly<DrawingLinkContext> | undefined;
}
/**
 * Existing local drawings stay local until `share` is called. Persisted shared
 * lineage reconnects across restores. Only price-pane drawings participate; time anchors
 * cross the boundary unchanged so each interval uses its own coordinate map.
 *
 * The price pane is each chart's own, in whatever slot that chart keeps it: one
 * chart can hold it at the top and another below its studies. A shared drawing
 * is therefore kept on slot 0 and put on each member's price pane as it is
 * applied there, never copied across by slot.
 *
 * A drawing pinned to the viewport never participates, and pinning a shared one
 * takes it out of the link on that chart alone: the other charts keep their copy.
 */
declare class DrawingLinkGroup {
    private readonly _members;
    private _enabled;
    private _broadcasting;
    private _destroyed;
    constructor(options?: DrawingLinkOptions);
    options(): Required<DrawingLinkOptions>;
    add(chart: DrawingLinkChart, controller: DrawingController, context?: DrawingLinkContextSource): void;
    setOptions(options: DrawingLinkOptions): void;
    /** Report the current instrument. Unknown identity stops sharing immediately. */
    setContext(chart: DrawingLinkChart, context: DrawingLinkContext | undefined): void;
    /** Share selected ids, or every existing eligible drawing when ids are omitted. */
    share(chart: DrawingLinkChart, ids?: readonly string[]): number;
    has(chart: DrawingLinkChart): boolean;
    /** Committed drawings remain on peers; transient previews are discarded. */
    remove(chart: DrawingLinkChart): void;
    destroy(): void;
    /** The slot this member keeps its price pane in, read at each use because it moves. */
    private _pricePane;
    /** A shared drawing as it lands on one member: on that member's own price pane. */
    private _onPricePane;
    private _eligible;
    private _create;
    private _reconnect;
    private _change;
    /**
     * A shared drawing pinned to the screen leaves the link on this chart. Its
     * lineage mark goes too, or a restore would find the peers' copy by it and
     * pull the drawing back to where they keep it.
     */
    private _unlink;
    /**
     * An unbound drawing that carries this chart's lineage mark, while a peer
     * still holds that lineage, is one that left the link by being pinned and
     * came back by undo: the undo put the mark back but no binding. It joins
     * again, as the newest edit, so its state goes to the peers the way any
     * other undo on a linked drawing does. If the peers have since deleted
     * their copies it stays on this chart alone and loses the mark, since an
     * edit never brings back another chart's deletion, and a later restore
     * must not apply that deletion here. A mark no peer holds is left for a
     * peer that has yet to join, as after any restore.
     */
    private _rejoin;
    private _commit;
    private _preview;
    private _syncOrder;
    private _orderFrom;
    private _clearPreview;
    private _clearMemberPreviews;
    private _detachDrawings;
}
declare function createDrawingLinkGroup(options?: DrawingLinkOptions): DrawingLinkGroup;

/**
 * Drawings per instrument: one drawing document for each instrument a chart
 * shows, kept in a store the host chooses, and swapped as the chart moves
 * from one instrument to another.
 *
 * A trader's levels belong to the symbol they were drawn on. The controller
 * holds one document and knows nothing of instruments (the engine has no
 * instrument concept), so without this a trend line drawn on one symbol stays
 * on screen for every symbol loaded after it, and deleting it anywhere
 * deletes it everywhere.
 *
 * The rules this keeps, each of which a host would otherwise have to get right
 * on its own:
 *
 * - **The instrument comes from the chart's data context.** A `data:context`
 *   naming another symbol or exchange saves the outgoing document and loads
 *   the incoming one, in the same turn, so nothing that listens after it (an
 *   alert, a link group) ever sees one instrument's drawings under another's
 *   name. An interval or a data variant is the same instrument, and a context
 *   that names no instrument changes nothing.
 * - **A swap is not an edit.** It records no undo step, and it clears the
 *   drawing undo history the way any `fromJSON` does: a step taken on one
 *   instrument describes that instrument's drawings, and taking it back on
 *   another would put one of them there. A drag in progress is put back first
 *   and a shape half placed is dropped, with the tool left armed.
 * - **The whole document swaps**, drawings pinned to the viewport included: a
 *   group or a stacking order can span both spaces, and a note pinned to the
 *   screen is nearly always about the instrument on it. Transient drawings
 *   (`policy.persistent` false) are the host's and are not saved; a host
 *   keeping one per instrument places it again after the swap.
 * - **Every committed change is written at once**, only when the saved
 *   document actually changed, so a reload loses nothing and a stream of
 *   updates to transient drawings writes nothing. An instrument left with no
 *   drawings has its entry removed rather than an empty one kept.
 * - **A store that fails loses nothing for the session.** A refused write is
 *   reported through `onError` and the document is held in memory, read back
 *   from there, and written again with the next change.
 * - **Anything else that replaces the whole document** (a host's `fromJSON`,
 *   a chart layout restored) is the current instrument's new document, and is
 *   written as such.
 * - **A stored document of an earlier version is read through the migration**
 *   and written back only when something in it changes, in the version that
 *   holds it then: a version 2 document stays version 2 until a drawing in it
 *   is given an interval range, and a version 3 one keeps its ranges.
 *
 * The store is synchronous on purpose: a swap happens inside the context
 * change, and an answer arriving later would land one instrument's drawings
 * on whatever the chart shows by then. A host whose drawings live on a
 * server keeps a synchronous copy here and syncs it in the background.
 */

/** An instrument as a chart's data context names it. */
interface DrawingInstrument {
    symbol?: string;
    exchange?: string;
}
/**
 * Where the documents live, one per instrument key. The host's choice:
 * `memoryDrawingStore` (the default), `webStorageDrawingStore` over a web
 * storage the host hands it, or its own. Synchronous; a throw from any
 * method, or `set` returning false, is a failure the helper reports.
 */
interface DrawingDocumentStore {
    /** The document stored under `key`, or null (or undefined) when there is none. */
    get(key: string): unknown;
    /** Store `document` under `key`. False, or a throw, when the write did not land. */
    set(key: string, document: DrawingsDocument): boolean | void;
    /** Forget whatever is stored under `key`. */
    remove(key: string): void;
}
/** The three members of a web storage `webStorageDrawingStore` uses. */
interface DrawingTextStorage {
    getItem(key: string): string | null;
    setItem(key: string, value: string): void;
    removeItem(key: string): void;
}
/** The slice of the chart `InstrumentDrawings` needs. */
interface InstrumentDrawingsChart {
    readonly isDestroyed?: boolean;
    on(event: string, handler: (payload: unknown) => void): () => void;
    getDataContext?(): Readonly<DrawingInstrument> | undefined;
}
/** A store operation that failed, as `InstrumentDrawingsOptions.onError` receives it. */
interface InstrumentDrawingsError {
    operation: 'read' | 'write' | 'remove';
    key: string;
    error: unknown;
}
interface InstrumentDrawingsOptions {
    /** Where the documents live. Default: `memoryDrawingStore()`, for the life of the page. */
    store?: DrawingDocumentStore;
    /**
     * The key an instrument's document is stored under, or null for a context
     * that names no instrument. Default `instrumentDrawingsKey`. A host that
     * wants drawings per data variant too, or one namespace per chart, says so here.
     */
    key?: (instrument: DrawingInstrument) => string | null;
    /**
     * Which document wins when the helper starts on an instrument the store
     * already holds one for while the controller holds another. `'stored'`
     * (default) loads the stored one, which is right for a chart built fresh.
     * `'live'` keeps the controller's, even an empty one, and writes it to the
     * store, for a host that has just loaded that instrument's drawings itself
     * (a layout, a chart rebuilt from the one it replaces). Either way, drawings the
     * controller holds for an instrument the store has nothing for are kept as
     * that instrument's: that is how a document saved before drawings were per
     * instrument is attached to the instrument it was drawn on.
     */
    prefer?: 'stored' | 'live';
    /** Called for every store operation that failed. The document stays in memory for the session. */
    onError?: (error: InstrumentDrawingsError) => void;
}
/**
 * The default key: the exchange and the symbol, `NSE:INFY`, or the symbol
 * alone when there is no exchange. Each part is URI-encoded, so a colon
 * inside a symbol cannot make two instruments one key. Null when the symbol
 * is missing or blank. Case is kept: the host decides whether `infy` and
 * `INFY` are one instrument, and the widget uppercases symbols itself.
 */
declare function instrumentDrawingsKey(instrument: DrawingInstrument | null | undefined): string | null;
/** A store in memory, for the life of the page. Each document is kept as text, so no caller shares an object with it. */
declare function memoryDrawingStore(): DrawingDocumentStore;
/**
 * A store over a web storage the host hands it (the page's `localStorage`,
 * or anything with the same three members), one JSON entry per instrument
 * under `prefix`. A text that does not parse throws from `get`, which the
 * helper reports and treats as no drawings.
 */
declare function webStorageDrawingStore(storage: DrawingTextStorage, prefix: string): DrawingDocumentStore;
/**
 * Attach a document saved before drawings were kept per instrument (one set
 * for the whole chart) to the instrument it was saved with, so no one loses
 * the drawings they already have. It is written only when the store holds
 * nothing for that instrument yet and the document has something in it; an
 * instrument that already has its own keeps it. True when it was written.
 */
declare function migrateUnscopedDrawings(store: DrawingDocumentStore, key: string | null, document: unknown): boolean;
/**
 * Keeps a drawing controller's document per instrument. Build it once the
 * controller exists; it follows the chart's data context from then on.
 *
 * ```ts
 * const scoped = new InstrumentDrawings(chart, draw, {
 *   store: webStorageDrawingStore(localStorage, 'my-app:drawings:'),
 * });
 * chart.setDataContext({ symbol: 'INFY', exchange: 'NSE', interval: '5m' });
 * ```
 */
declare class InstrumentDrawings {
    private readonly _draw;
    private readonly _store;
    private readonly _keyOf;
    private readonly _onError;
    private _key;
    /** The current instrument's document as last written or loaded, to skip a write that changes nothing. */
    private _text;
    /** Documents whose write failed, by key: what the store should hold, kept for the session. */
    private readonly _held;
    private _swapping;
    private _destroyed;
    private readonly _off;
    constructor(chart: InstrumentDrawingsChart, controller: DrawingController, options?: InstrumentDrawingsOptions);
    get isDestroyed(): boolean;
    /** The key of the instrument whose drawings are on the chart, or null before the chart has named one. */
    key(): string | null;
    /**
     * Move to another instrument now, without waiting for the chart's data
     * context: for a host that does not publish one, or that switches its
     * drawings ahead of its data. A later context naming the same instrument
     * changes nothing. Null, or an instrument with no key, changes nothing.
     */
    setInstrument(instrument: DrawingInstrument | null): void;
    /**
     * The saved document of an instrument: the drawings on the chart for the
     * current one, the stored document for any other, or null when it has
     * none (or no key). A copy either way.
     */
    document(instrument: DrawingInstrument): DrawingsDocument | null;
    /**
     * Replace one instrument's drawings. On the current instrument that is the
     * controller's `fromJSON`, which clears its undo history; on any other, the
     * stored document, shown when the chart next moves there. For a layout
     * that brings the drawings of an instrument the chart is about to show.
     */
    setDocument(instrument: DrawingInstrument, document: unknown): void;
    /** Write the current instrument's document now, and any the store refused before. False while one is still refused. */
    save(): boolean;
    /** Stop following the chart. The documents stay where they are. */
    destroy(): void;
    private _keyFor;
    /** The drawings the controller would save, as text. */
    private _live;
    /** Take `key` as the current instrument, with the drawings `prefer` picks. */
    private _start;
    private _follow;
    private _swap;
    /** Put `document` on the controller without the change it announces being written back. */
    private _load;
    private _changed;
    /** Write the current document when it changed, or always when `force`. */
    private _saveCurrent;
    private _write;
    /** The stored document for `key`, migrated to the current shape, or null. A held one comes first. */
    private _read;
    private _report;
}
/** `new InstrumentDrawings(chart, controller, options)`, as a function. */
declare function createInstrumentDrawings(chart: InstrumentDrawingsChart, controller: DrawingController, options?: InstrumentDrawingsOptions): InstrumentDrawings;

/**
 * The per-tool settings schema: what a host may show in a drawing's settings
 * dialog, expressed as dot paths into the drawing (`style.color`,
 * `text.fontSize`, `props.foo`, `intervals.from`, `zIndex`, `space`) with a
 * control kind and a label.
 *
 * The rule the whole module exists for: a tool declares only fields its `draw`
 * actually reads. A schema is not a wish list. A host renders exactly what is
 * declared, so an entry with nothing behind it becomes a control that does
 * nothing, which is worse than no control at all.
 *
 * Pure: no DOM, no registry. The registry lookup (`drawingSettingsSchema`)
 * lives in registry.ts so this file has no import that could loop back here.
 */

declare const LINE_STYLE_OPTIONS: ReadonlyArray<{
    value: string;
    label: string;
}>;
declare const ALIGN_OPTIONS: ReadonlyArray<{
    value: string;
    label: string;
}>;
declare const VALIGN_OPTIONS: ReadonlyArray<{
    value: string;
    label: string;
}>;
declare const TEXT_POSITION_OPTIONS: ReadonlyArray<{
    value: string;
    label: string;
}>;
/** The two coordinate spaces, as a host offers them: follow the bars, or stay on screen. */
declare const SPACE_OPTIONS: ReadonlyArray<{
    value: DrawingSpace;
    label: string;
}>;
/**
 * Font stacks a host can offer. Values are real CSS stacks, so a drawing's
 * `text.fontFamily` needs no translation before it reaches `ctx.font`; a host
 * may still accept any stack the user types.
 */
declare const FONT_OPTIONS: ReadonlyArray<{
    value: string;
    label: string;
}>;
declare const COLOR_FIELD: SettingsField;
declare const LINE_WIDTH_FIELD: SettingsField;
declare const LINE_STYLE_FIELD: SettingsField;
/** Colour, width and dash: what every stroked tool reads through `applyStroke`. */
declare const LINE_FIELDS: readonly SettingsField[];
declare const FILL_FIELDS: readonly SettingsField[];
declare const EXTEND_FIELDS: readonly SettingsField[];
/**
 * Where the drawing is anchored: to time and price, or to the screen. Declared
 * only by tools that set `viewport`, since the controller converts nothing
 * else. Writing it converts the anchors at the view on screen, so the drawing
 * keeps its place and only stops (or starts) following the bars.
 */
declare const SPACE_FIELD: SettingsField;
/**
 * The intervals a drawing is shown on (`Drawing.intervals`), as two interval
 * codes, each end included and either one left empty for no limit. No tool
 * declares them, since the engine cannot list a host's intervals: a host that
 * offers visibility per interval adds them to a tool's schema with
 * `composeSettings`, and `applyDrawingSettings` writes the pair as the patch's
 * `intervals`, with both ends cleared meaning every interval.
 */
declare const INTERVAL_FIELDS: readonly SettingsField[];
declare const SHOW_LABELS_FIELD: SettingsField;
declare const LEVEL_FIELDS: readonly SettingsField[];
/**
 * The face of any text a tool prints, including readouts it composes itself
 * (a fib level's ratio, a measure's chip, a horizontal line's price tag).
 * Tools that print nothing do not declare these.
 */
declare const FONT_FIELDS: readonly SettingsField[];
declare const TEXT_VALUE_FIELD: SettingsField;
/** The full surface of the text tool: content, face, layout, plate. */
declare const TEXT_FIELDS: readonly SettingsField[];
/**
 * A label attached to a shape: it sits inside or just above the outline, so
 * it has an alignment and a position but no plate of its own (the shape is
 * the plate) and wraps to the shape's width rather than a width of its own.
 */
declare const SHAPE_TEXT_FIELDS: readonly SettingsField[];
/**
 * An annotation's plate (note, balloon, comment, signpost, price note): always
 * drawn, so there is no `background` toggle, only its colour and opacity, plus
 * an optional border.
 */
declare const PLATE_TEXT_FIELDS: readonly SettingsField[];
/**
 * Build a schema from field lists and single fields. Later duplicates of a
 * path are dropped so a tool can layer a group over a shared list without
 * showing one control twice.
 */
declare function composeSettings(parts: ReadonlyArray<readonly SettingsField[] | SettingsField>, opts?: {
    textIsContent?: boolean;
}): SettingsSchema;
/** The value at a dot path, or `undefined` when unset or the path is not one the model has. */
declare function readDrawingSetting(d: Drawing, path: string): unknown;
/**
 * Every field of a schema read off a drawing, keyed by path. Unset fields use
 * their declared default value, or remain `undefined` when no default exists.
 */
declare function readDrawingSettings(d: Drawing, schema: SettingsSchema): Record<string, unknown>;
/**
 * A raw control value made into what the field's kind stores, or `undefined`
 * when it cannot be. Form inputs hand back strings; a settings panel should
 * not have to know that `fontSize` is a number and `bold` a boolean.
 */
declare function coerceSettingValue(field: SettingsField, raw: unknown): unknown;
/**
 * Turn `{ 'style.color': '#f00', 'text.value': 'Hi', zIndex: 2 }` into the
 * partial a controller's `update` accepts. Bags come back whole
 * (`style: { ...d.style, color }`), so it does not matter whether the
 * controller merges or replaces them. A value of `undefined` removes the key,
 * which is how a host offers "reset to default". The `intervals` pair comes
 * back whole as well, as the range to replace the drawing's; with neither end
 * left it is `{}`, which a controller takes as no range, every interval.
 *
 * With a `schema`, each value is coerced to its field's kind and any path the
 * schema does not declare is dropped: a host can hand over its form state as
 * is. Without one, values are written verbatim. Paths the model has no home
 * for are ignored either way.
 */
declare function applyDrawingSettings(d: Drawing, patch: Record<string, unknown>, schema?: SettingsSchema): Partial<Drawing>;

/**
 * The drawing-tool registry. Same philosophy as the chart-type and indicator
 * registries: a tool is a descriptor, the layer just runs it, and
 * `registerDrawingTool` makes a custom one first-class. The built-ins are
 * listed and registered in tools.ts; everything that only needs to look a
 * tool up imports this module, not the catalogue.
 */

declare function registerDrawingTool(tool: DrawingTool): void;
declare function getDrawingTool(id: string): DrawingTool;
declare function hasDrawingTool(id: string): boolean;
declare function registeredDrawingTools(): DrawingTool[];
/** Keyboard event fields a shortcut is matched against. */
interface ShortcutEvent {
    key: string;
    /** The physical key (`KeyT`): under Alt it names the letter when `key` is not one, as macOS Option types a symbol. Since 2.5.10. */
    code?: string;
    altKey?: boolean;
    ctrlKey?: boolean;
    metaKey?: boolean;
    shiftKey?: boolean;
}
/**
 * The id of the tool whose `shortcut` matches this key event, or `null`.
 *
 * Pure, so a host can bind one `keydown` listener and decide for itself when
 * shortcuts apply. The library installs no listener, because only the host
 * knows whether the chart has focus, a dialog is open, or the user is typing.
 *
 * Modifiers must match exactly: `Alt+T` will not fire for `Ctrl+Alt+T`, so a
 * tool shortcut cannot shadow a browser or host chord. `metaKey` (Cmd) is
 * treated as Ctrl, which is what a Mac user expects.
 */
declare function matchDrawingShortcut(e: ShortcutEvent): string | null;
/**
 * Every registered tool that has a shortcut, as `id -> shortcut`. Built as
 * entries, so an id such as `__proto__` is a key like any other rather than an
 * assignment to the object's prototype.
 */
declare function drawingShortcuts(): Record<string, string>;
/**
 * The settings a host may show for a tool: the tool's own declaration, else
 * the line fields, which every stroked custom tool reads. This is a registry
 * lookup, which is why it lives here and not in schema.ts: that module stays
 * free of any import that could loop back into this one.
 */
declare function drawingSettingsSchema(toolId: string): SettingsSchema;

/**
 * Built-in drawing tools: the line family, shapes, curves, freehand ink and
 * cycles, plus the catalogue of every built-in in registration order. The
 * other families live beside it (fib-tools.ts, measure-tools.ts,
 * annotation-tools.ts and the advanced, pattern and analysis modules), what
 * they paint with in tool-paint.ts, and the registry in registry.ts.
 *
 * `draw` receives anchors already in device px; `distance` receives them in
 * media px, the same space as the incoming cursor.
 *
 * Text lives in `drawing.text` (a `DrawingText`), never in the style bag: a
 * shape's outline colour and its label colour are two decisions, and a tool
 * that prints only a readout (a fib ladder, a measure chip) still reads its
 * face from there. Ladders read `FibLevel[]` and stroke each level in its own
 * colour, taken from the shared palette in levels.ts.
 *
 * Every tool declares `settings`: the fields a host may show for it. The
 * schema is a contract, not a wish list. A field is declared only when the
 * tool reads it, because a control with nothing behind it is a defect.
 */

declare const TREND_LINE: DrawingTool;
declare const RAY: DrawingTool;
declare const EXTENDED_LINE: DrawingTool;
declare const ARROW: DrawingTool;
declare const HORIZONTAL_LINE: DrawingTool;
declare const HORIZONTAL_RAY: DrawingTool;
declare const VERTICAL_LINE: DrawingTool;
declare const CROSS_LINE: DrawingTool;
declare const RECTANGLE: DrawingTool;
declare const ELLIPSE: DrawingTool;
declare const PARALLEL_CHANNEL: DrawingTool;
/**
 * Path: click each vertex, double-click to finish, arrowhead on the last leg.
 * The arrow is what separates it from `polyline`: a path points somewhere.
 */
declare const PATH: DrawingTool;
/**
 * Brush: freehand ink. Press, drag, release. The anchors are the thinned
 * samples of the gesture and the spline between them is what is stroked, so
 * the ink stays a curve after the thinning that keeps a stroke small.
 */
declare const BRUSH: DrawingTool;
/**
 * Rotated rectangle: anchors 0 and 1 lay out one edge (and so the rotation),
 * and anchor 2 sets the depth perpendicular to it. An axis-aligned `rectangle`
 * cannot follow a trend channel; this can.
 */
declare const ROTATED_RECTANGLE: DrawingTool;
/**
 * Double curve: an S through three anchors. `curve` bends one way off a single
 * control; this mirrors that control about the midpoint so the second half bends
 * back, which is the shape a rounded top-then-bottom actually needs.
 */
declare const DOUBLE_CURVE: DrawingTool;
/**
 * Cyclic lines: vertical lines repeating at the interval the two anchors set,
 * for reading a rhythm forward off a measured swing.
 */
declare const CYCLIC_LINES: DrawingTool;
/**
 * Time cycles: semicircles of the anchors' width repeating along the axis, the
 * classic cycle-projection overlay.
 */
declare const TIME_CYCLES: DrawingTool;
/**
 * Sine line: a full wave between the anchors. The horizontal span is one
 * period, the vertical offset its amplitude.
 */
declare const SINE_LINE: DrawingTool;
/**
 * Circle from centre + a radius handle. The radius is measured in *pixels*, so
 * it stays a circle on screen instead of the ellipse the two axes' differing
 * scales would otherwise produce.
 */
declare const CIRCLE: DrawingTool;
declare const TRIANGLE: DrawingTool;
/** Free-form polyline: click each vertex, double-click to finish. */
declare const POLYLINE: DrawingTool;
/** Arc through three anchors: the middle one is on the curve, not a handle. */
declare const ARC: DrawingTool;
/** Quadratic curve: the middle anchor is a control handle, off the curve. */
declare const CURVE: DrawingTool;
/** Highlighter: the brush with a fat, translucent stroke. */
declare const HIGHLIGHTER: DrawingTool;
declare const BUILTIN_DRAWING_TOOLS: readonly DrawingTool[];
/** Register every built-in tool. Idempotent; called on tier import. */
declare function registerBuiltinDrawingTools(): void;

/**
 * The fibonacci and gann family: retracement and extension, the channel, time
 * zone and fan, the gann fan and box. Ladders read `FibLevel[]` and stroke each
 * level in its own colour, taken from the shared palette in levels.ts.
 */

declare const FIB_RETRACEMENT: DrawingTool;
declare const FIB_EXTENSION: DrawingTool;
/** Fib channel: fib levels spread across a trend leg, parallel to it. */
declare const FIB_CHANNEL: DrawingTool;
/**
 * Fib time zone: vertical lines at Fibonacci multiples of the base leg's width.
 * Its levels count bars rather than divide a leg, so they carry no convention
 * colour and fall back to the drawing's own.
 */
declare const FIB_TIME_ZONE: DrawingTool;
/** Rays from the anchor at fib fractions of the leg: the speed resistance fan. */
declare const FIB_FAN: DrawingTool;
/**
 * Gann fan: rays from one anchor at the classic price/time angles. A level's
 * ratio is price per unit of time, so the 1x1 is 1, the 1x2 is 0.5 and the
 * 2x1 is 2; each angle keeps its own colour from the cycle palette.
 */
declare const GANN_FAN: DrawingTool;
/**
 * Gann box: the drawn rectangle divided at its levels on both axes, plus the
 * 1x1 diagonal. Ratio 0 is the first anchor's price and time, ratio 1 the
 * second's, so the box reads like a retracement laid over a date range.
 */
declare const GANN_BOX: DrawingTool;

/**
 * Measurement and positions: the measure box, the price and date ranges, the
 * forecast and the long and short position calculators. Measurers are tinted
 * by direction, so they read width, dash and opacity but not a colour.
 */

declare const MEASURE: DrawingTool;
declare const LONG_POSITION: DrawingTool;
declare const SHORT_POSITION: DrawingTool;
/**
 * Measurers. `measure` reports price *and* time together; these two constrain it
 * to one axis, which is what you want when the other one is noise: sizing a
 * retracement without caring how long it took, or counting bars to an event
 * without caring where price went.
 */
declare const PRICE_RANGE: DrawingTool;
declare const DATE_RANGE: DrawingTool;
/**
 * Position forecast: project a move from an anchor. Two anchors give the
 * projected leg; the shape extends the same slope past the target as a dashed
 * cone, so the drawing says "if this continues" rather than just marking a line.
 */
declare const FORECAST: DrawingTool;

/**
 * Annotations: the text tool, the price label, callout and flag, the plate
 * notes (note, balloon, comment, signpost, price note), the table and the
 * arrow marks. Every one of them puts a human sentence, or a mark pointing at
 * a bar, on the chart.
 */

declare const TEXT: DrawingTool;
/**
 * Price label: a pill of the anchored price with a tail pointing at the bar.
 * Reads its value off the anchor, so dragging it re-reads rather than going
 * stale the way a typed-in text would. Text, when given, replaces the price.
 */
declare const PRICE_LABEL: DrawingTool;
/**
 * Callout: a text bubble on its own anchor with a tail back to the point it
 * annotates, so the note can sit clear of the price action it refers to.
 */
declare const CALLOUT: DrawingTool;
/** Flag mark: a pennant on a pole at one anchor, for tagging a bar. */
declare const FLAG_MARK: DrawingTool;
/**
 * Note: a pin at the bar with its text to the upper right.
 *
 * The pin is the point being annotated and the plate is the annotation, which
 * is why the two are drawn as separate marks joined by a stem rather than as
 * one bubble: the reader needs to see exactly which bar is meant, and a bubble
 * wide enough to hold a sentence covers several.
 */
declare const NOTE: DrawingTool;
/**
 * Balloon: a speech bubble sitting above its anchor, tail pointing down.
 *
 * One anchor rather than the callout's two: a balloon is for saying something
 * *about this bar*, so letting the bubble be dragged away from what it labels
 * would only invite it to drift.
 */
declare const BALLOON: DrawingTool;
/**
 * Comment: a small square-cornered box with a tail off its bottom left.
 *
 * Deliberately plainer than the balloon: a chart that says something at every
 * other bar needs one of these marks to be quiet, and the shape is the only
 * thing distinguishing them once both are the user's own colour.
 */
declare const COMMENT: DrawingTool;
/**
 * Signpost: a post standing on the bar with its plate at the top.
 *
 * The one annotation anchored to *time* rather than to a level: the post is
 * vertical so the plate can clear the price action entirely while the foot
 * still names an exact bar. Useful for events, which happen at a moment and
 * not at a price.
 */
declare const SIGNPOST: DrawingTool;
/**
 * Price note: the anchored price, with the user's text under it.
 *
 * The price is read off the anchor rather than typed, so dragging the note
 * re-reads it. A typed price is a number that was true once, which on a chart
 * is worse than no number at all.
 */
declare const PRICE_NOTE: DrawingTool;
/**
 * Table: rows of text in a grid, anchored top-left.
 *
 * Cells come from the text: a newline starts a row and a pipe separates
 * columns, so a whole table is one editable string and survives `getState`
 * with no new shape in the drawing model. The first row is drawn as a header
 * because a table on a chart is nearly always labelled.
 */
declare const TABLE: DrawingTool;
declare const ARROW_UP: DrawingTool;
declare const ARROW_DOWN: DrawingTool;
declare const ARROW_LEFT: DrawingTool;
declare const ARROW_RIGHT: DrawingTool;

/**
 * The shared level palette for the Fibonacci and Gann family.
 *
 * Every ladder tool (retracement, extension, channel, fan, time zone, the Gann
 * fan and box) strokes a list of ratios, and each ratio has a colour the eye
 * expects: 0.618 reads as one thing across every tool, not a different hue per
 * tool. Before this module each tool would have restated those colours, and
 * restated colours drift. This is the one statement; tools take their defaults
 * from it and hosts seed a newly added level from it.
 *
 * Pure: no DOM, no registry. Safe to import from a worker or a settings panel.
 */

/**
 * The colour of a ratio that is an anchor rather than a level (0, 1, 2, 3 are
 * where the measured leg starts and ends, or whole multiples of it) and of any
 * ratio the convention does not name. Neutral on purpose: it is the reference
 * the coloured levels are read against.
 */
declare const LEVEL_NEUTRAL = "#787b86";
/**
 * The conventional colour for a Fibonacci ratio. Anchors (0, 1, 2, 3) and any
 * ratio without a convention come back {@link LEVEL_NEUTRAL}, so a host can call
 * this for every new level and always get a usable colour.
 */
declare function levelColor(ratio: number): string;
/**
 * A short sequence of colours that stay distinguishable side by side, for
 * tools whose levels are positions rather than ratios: the angles of a Gann
 * fan, the repeats of a cycle. Index `i` wraps, so any count is safe.
 */
declare const CYCLE_PALETTE: readonly string[];
/** The `i`th cycle colour. Wraps in both directions; a negative index is fine. */
declare function cycleColor(i: number): string;
/** `0.618` as `61.8%`: the text a level prints when it carries no label. */
declare function formatRatio(ratio: number): string;
/**
 * The label for a Gann angle stored as a price-per-time ratio: `1` is the 1x1,
 * `0.5` the 1x2, `2` the 2x1. Ratios that are not clean reciprocals fall back
 * to the ratio itself, which is still true, just less idiomatic.
 */
declare function gannLabel(ratio: number): string;
/** A mutable copy of a default ladder, for a fresh drawing's own `style.levels`. */
declare function cloneLevels(levels: readonly FibLevel[]): FibLevel[];
/** Retracement and extension: the classic ladder, coloured by convention. */
declare const DEFAULT_FIB: readonly FibLevel[];
/** Speed fan: rays at the fractions of the leg. No 0 ray, it would be flat. */
declare const DEFAULT_FIB_FAN: readonly FibLevel[];
/** Gann box: quarters plus the golden pair, applied to both axes of the box. */
declare const DEFAULT_GANN_BOX: readonly FibLevel[];
/**
 * Gann fan: the classic angles as price-per-time ratios, 1x8 up to 8x1. These
 * are positions in a sequence rather than Fibonacci ratios, so they take the
 * cycle palette.
 */
declare const DEFAULT_GANN_FAN: readonly FibLevel[];
/**
 * Time zones count bars along the Fibonacci sequence, so `ratio` here is a
 * multiple of the anchor leg, not a fraction of it. No colour is set: there is
 * no convention for "the 13 line", so every zone takes the drawing's own colour
 * until the user gives one its own.
 */
declare const DEFAULT_FIB_TIME_ZONE: readonly FibLevel[];

/**
 * Glyphs for the drawing tools and for host chrome, as path data.
 *
 * The engine ships no DOM, and this does not change that: an icon here is a
 * string of SVG path commands, not an element. The host still builds its own
 * toolbar, flyouts and rail; what it no longer has to do is draw every glyph
 * before it can show them. `icon-svg.ts` turns these into markup strings for a
 * host that wants the convenience; this file is the single source both of them
 * and every other surface read from.
 *
 * That was the real cost of leaving them out. Every adopter drew their own set,
 * each drifted on stroke weight, grid and visual density independently, and the
 * result read as sixty icons rather than as one set, however carefully any
 * single glyph was made. A shipped set is worth more than a better glyph.
 *
 * # Two tiers, one line
 *
 * Tool glyphs live on a 24-unit grid and are shown at 20px in the rail. Host
 * chrome (undo, close, the settings tab rail) is shown at 16px beside them on
 * a registry of its own, drawn on a 16-unit grid, because a 24-grid glyph
 * scaled to 16 loses its detail. Each tier's stroke is set so that it lands
 * on the same line on screen at its display size: 1.6 units at 20px and 1.3
 * units at 16px are both about 1.3px, so a lock under the tools reads as part
 * of the same set, only smaller.
 *
 * Through 2.6.2 both tiers drew a 2-unit stroke at their grid size (24px and
 * 16px), which put every edge on a pixel boundary. Crisp, but a line half
 * again as wide on tool glyphs a fifth larger is nearly twice the ink, and a
 * rail of them read as heavy beside the chart it frames: on screen, weight is
 * noticed long before edge sharpness.
 * Earlier still the chrome stroke was 1.5 at 16px, the lightest of the three,
 * and the tool rail beside it the heaviest; one line across both is the rule
 * that survived every revision.
 *
 * # The grid
 *
 * Every glyph is authored to the same constraints, and `tests/draw-icons.test.ts`
 * enforces all of them mechanically, because a set of this size cannot be kept
 * consistent by review:
 *
 *  - **One viewBox per tier.** 24 by 24 for tools, 16 by 16 for chrome, so a
 *    host sets the size once per surface.
 *  - **A margin all round.** Live area 2 to 22 on the tool grid, 2 to 14 on the
 *    chrome grid, so the ink stops short of the box. Without it, glyphs that
 *    happen to reach the edge look larger than their neighbours and the rail
 *    reads as ragged.
 *  - **Integer coordinates.** Parallel strokes keep an even gap and a family
 *    of glyphs shares its anchor points, so siblings line up in a list.
 *  - **One stroke weight.** Different weights across identically sized boxes
 *    is the single most visible tell of a set assembled rather than drawn.
 *  - **The whole glyph in its path.** The marks that tell siblings apart (a
 *    segment ends in two dots, a ray starts at one, a path ends in a head)
 *    are small closed shapes drawn in the path itself, so a host that renders
 *    the path data and nothing else still gets every one of them. An accent
 *    registry repeats those marks for a fill on top: a dot then paints solid
 *    rather than as a ring, which adds weight, never identity.
 *
 * # Rendering
 *
 * Tool glyphs are drawn for 20px and chrome glyphs for 16px, which is what
 * `iconSvg` and `chromeIconSvg` default to. A host that shows a glyph larger
 * scales the line with it; one that wants the same line at another size sets
 * the stroke to keep it near 1.3px.
 */
/** The viewBox every glyph is authored in. */
declare const ICON_VIEWBOX = "0 0 24 24";
/**
 * The CSS pixel size a tool glyph is drawn for: the rail, a flyout row and
 * `iconSvg` by default.
 */
declare const ICON_SIZE = 20;
/**
 * Stroke width in viewBox units: about 1.3px at `ICON_SIZE`. Paths carry no
 * presentation attributes, so a host that wants another weight overrides this
 * once rather than editing glyphs.
 */
declare const ICON_STROKE = 1.6;
/**
 * The attribute bag a tier hands its host. Structural, so the markup builders
 * in `icon-svg.ts` take either tier's bag through one signature.
 */
interface IconAttrs {
    readonly viewBox: string;
    readonly fill: 'none';
    readonly stroke: 'currentColor';
    readonly strokeWidth: number;
    readonly strokeLinecap: 'round';
    readonly strokeLinejoin: 'round';
}
/** Attributes a host should apply to the `<svg>`, so every set matches. */
declare const ICON_ATTRS: {
    readonly viewBox: "0 0 24 24";
    readonly fill: "none";
    readonly stroke: "currentColor";
    readonly strokeWidth: 1.6;
    readonly strokeLinecap: "round";
    readonly strokeLinejoin: "round";
};
/**
 * Path data by tool id, plus a few keys a host toolbar needs that are not
 * themselves tools (`cursor`, the group headers).
 *
 * Values are the `d` attribute of one `<path>`. Multiple subpaths are joined
 * into the same string rather than split across elements, so a host renders one
 * node per glyph, and that node is the whole glyph: its dots and heads are in
 * the string as outlines. `DRAWING_TOOL_ACCENTS` repeats those marks for a
 * host that also wants them filled.
 */
declare const DRAWING_TOOL_ICONS: Readonly<Record<string, string>>;
/**
 * The fills of the tool glyphs' marks, by tool id: anchor dots, pole caps and
 * arrowheads. Optional: most glyphs have none.
 *
 * Every mark here is also drawn, as an outline, in the glyph's own path, so a
 * host that renders only `DRAWING_TOOL_ICONS` still shows each one. This adds
 * the fill: a second `<path>` painted in `currentColor` with no stroke of its
 * own, after the glyph's path. The markup builders in `icon-svg.ts` add it; a
 * host wrapping the path data itself adds
 * `<path d="..." fill="currentColor" stroke="none"/>` for the same result.
 */
declare const DRAWING_TOOL_ACCENTS: Readonly<Record<string, string>>;
/**
 * The glyph for a tool, or `undefined` when it has none.
 *
 * Undefined rather than a placeholder: a host that renders an empty box has a
 * visible gap to fix, while one handed a question mark ships it.
 */
declare function drawingToolIcon(toolId: string): string | undefined;
/** The filled accent for a tool glyph, or `undefined` when it has none. */
declare function drawingToolAccent(toolId: string): string | undefined;
/** Every id this set covers, for a host building a palette from it. */
declare function drawingToolIconIds(): string[];
/** The viewBox every chrome glyph is authored in. */
declare const CHROME_ICON_VIEWBOX = "0 0 16 16";
/** The CSS pixel size a chrome glyph is drawn for, and `chromeIconSvg`'s default. */
declare const CHROME_ICON_SIZE = 16;
/**
 * Chrome stroke width in viewBox units: about 1.3px at `CHROME_ICON_SIZE`, the
 * line the tool tier draws at its own size, so the two rails read as one set.
 */
declare const CHROME_ICON_STROKE = 1.3;
/** Attributes a host should apply to the `<svg>` around a chrome glyph. */
declare const CHROME_ICON_ATTRS: {
    readonly viewBox: "0 0 16 16";
    readonly fill: "none";
    readonly stroke: "currentColor";
    readonly strokeWidth: 1.3;
    readonly strokeLinecap: "round";
    readonly strokeLinejoin: "round";
};
/**
 * Path data for host chrome: the buttons around the chart rather than the
 * tools in it. Same rules as the tool tier (the whole glyph in one path, its
 * marks included; no presentation attributes; integer coordinates) on the 16
 * grid with a 2..14 live area.
 *
 * The set was drawn for a 2px line, where a 16px glyph has room for about four
 * parallel strokes with a pixel between them, and keeps that air at the finer
 * line it is shown with now: two
 * sliders rather than three, a lens and a pupil as small solid dots rather
 * than rings, and no plus squeezed inside a square.
 *
 * The pair `cursor` / `magnet` / `text` also exist in the tool registry, at 24.
 * They are drawn twice on purpose: a rail button and a toolbar button are
 * different sizes, and the whole point of a second grid is not scaling one
 * drawing to both.
 *
 * The tier also carries a glyph for each chart type and transform, under
 * `chart-<id>` (see `chartTypeIcon`), so a chart-type menu reads from the
 * same set as the buttons around it. A layout picker's tiles are derived
 * rather than listed (`layoutIconPath`).
 */
declare const CHROME_ICONS: Readonly<Record<string, string>>;
/**
 * The fills of the chrome glyphs' marks, as `DRAWING_TOOL_ACCENTS` is for the
 * tools: each mark is outlined in the glyph's path, and this paints it solid.
 * At 16px a ring of radius 1 is mostly line already, so the fill matters
 * most for the rings of radius 2: the slider knobs, a head, a line's markers.
 */
declare const CHROME_ICON_ACCENTS: Readonly<Record<string, string>>;
/**
 * Chrome glyphs meant to be painted solid. Paths carry no presentation
 * attributes, so the fill is applied by the wrapper (`chromeIconSvg` does it,
 * a host with its own wrapper reads this set) and the registry stays pure.
 */
declare const CHROME_ICON_FILLED: ReadonlySet<string>;
/** The chrome glyph for an id, or `undefined` when there is none. */
declare function chromeIcon(id: string): string | undefined;
/** The filled accent for a chrome glyph, or `undefined` when it has none. */
declare function chromeIconAccent(id: string): string | undefined;
/** Every id the chrome tier covers. */
declare function chromeIconIds(): string[];
/**
 * The chrome glyph for a chart type or a transform, by its id: a registered
 * type (`candlestick`, `point-figure`) or one of the transforms that render
 * as candles (`heikin-ashi`, `renko`, `range-bars`, `line-break`). It is the
 * `chart-<id>` entry of `CHROME_ICONS`, so `chromeIconSvg('chart-' + id)`
 * draws the same picture. `undefined` for a type a host registered itself,
 * which has no glyph until the host draws one.
 */
declare function chartTypeIcon(type: string): string | undefined;
/** One chart's place in a layout glyph: a cell, and how many rows and columns it spans. */
interface LayoutIconSlot {
    readonly row: number;
    readonly column: number;
    /** Default 1. */
    readonly rowSpan?: number;
    /** Default 1. */
    readonly columnSpan?: number;
}
/**
 * A chrome glyph for a grid of charts: the frame, split into `rows` by
 * `columns` cells, with the dividers inside a spanning slot left out, so a
 * chart that spans two cells reads as one pane. Cells no slot claims are
 * drawn as their own panes. Without `slots`, every cell is one chart.
 *
 * Derived rather than drawn per layout, so a host's layout picker gets a
 * tile for every layout it offers, uneven ones included, without an entry in
 * this registry for each. The path is on the chrome grid and to its rules
 * (whole units, the 2..14 live area, no presentation attributes), and draws
 * through the chrome attribute bag like any `CHROME_ICONS` value.
 *
 * One to four a side: twelve units split four ways leaves clear space
 * between two lines even at the 2px stroke the set was drawn for, and five
 * would close it up. Throws a
 * `RangeError` for anything else, and for a slot outside the grid or over
 * another. A chart grid takes up to eight a side, so a caller that draws the
 * current layout falls back to the `layout` glyph past four.
 */
declare function layoutIconPath(rows: number, columns: number, slots?: readonly LayoutIconSlot[]): string;

/** Prefix of every `<symbol id>` in the sprite, and of every `<use href>`. */
declare const ICON_SYMBOL_PREFIX = "oac-icon-";
/** Options shared by the inline builders. */
interface IconSvgOptions {
    /**
     * Width and height. A number is CSS pixels; a string is written verbatim, so
     * `'1em'` sizes the glyph with the surrounding text. Default: the size the
     * tier is drawn for (`ICON_SIZE` for tools, `CHROME_ICON_SIZE` for chrome).
     */
    size?: number | string;
    /** Stroke width in viewBox units. Default: the tier's `*_STROKE`. */
    stroke?: number;
    /** A class for the host's own styling. */
    className?: string;
}
/** Options for `toolCursor`. */
interface ToolCursorOptions {
    /** Cursor image size in CSS pixels. Default 20. Browsers reject images over 128. */
    size?: number;
    /** The pointer's active pixel, from the image's top-left. Default: the centre. */
    hotspot?: readonly [number, number];
    /** Glyph colour. Default white. */
    color?: string;
    /**
     * Halo colour. Default: black under a light `color`, white under a dark one,
     * judged from a hex colour. A non-hex `color` gets a black halo unless this
     * is set.
     */
    halo?: string;
    /** The keyword after the image, for a browser that will not take it. Default `crosshair`. */
    fallback?: string;
}
/**
 * A complete inline `<svg>` for one tool glyph, stroked in `currentColor`.
 *
 * ```ts
 * button.innerHTML = iconSvg('trend-line');
 * button.innerHTML = iconSvg('trend-line', { size: '1em' });
 * ```
 */
declare function iconSvg(id: string, opts?: IconSvgOptions): string;
/**
 * A complete inline `<svg>` for one chrome glyph, on the 16 grid. Glyphs in
 * `CHROME_ICON_FILLED` are painted solid as well as stroked.
 */
declare function chromeIconSvg(id: string, opts?: IconSvgOptions): string;
/**
 * One hidden `<svg>` of `<symbol>`s, for a host that shows many glyphs and
 * wants each path in the document once. Inject it once, then place glyphs with
 * `iconUse`. Symbols carry the paths and viewBox, and no stroke: the weight
 * inherits from the `<svg>` around each `<use>`, so one sprite serves every
 * weight. An accent carries its fill, the one attribute it cannot inherit,
 * and no stroke, since the glyph's own path outlines it.
 *
 * Default: every tool glyph. Repeated ids collapse to one symbol, since a
 * duplicate element id is an invalid document.
 */
declare function iconSprite(ids?: readonly string[]): string;
/** An `<svg>` that references a symbol from `iconSprite`. Same frame as `iconSvg`. */
declare function iconUse(id: string, opts?: IconSvgOptions): string;
/**
 * A CSS `cursor` value carrying the tool's glyph, for the canvas while that
 * tool is the active one.
 *
 * The glyph is drawn twice: once in `halo`, one pixel wider on each side, and
 * once in `color` on top with its marks filled, so it reads on a light chart
 * and a dark one without the host choosing per theme. The value is
 * `url("data:image/svg+xml,...") x y, fallback`, ready for `canvas.style.cursor`.
 */
declare function toolCursor(id: string, opts?: ToolCursorOptions): string;

/**
 * A `DrawingLayer` renders the drawings of one pane and answers the chart's
 * hit-test. Anchors live in `{ time, price }`, so the layer maps them through
 * the *fractional* index helpers: the gapless axis means an anchor can sit
 * between bars (inside a collapsed weekend) or past the last one (a forward
 * projection), and whole-index lookups have nothing to return there.
 *
 * Each pane carries a `bottom` layer for drawings with a negative `zIndex`
 * (under the series), a `top` one for the rest, and a `series` layer for each
 * series-band entry a drawing is placed above (`stackAbove`), which the chart
 * paints right after that entry's series. The top one takes over handles and
 * hit-testing for the others through `setBelow`, so a shape parked under the
 * candles still shows its grab handles above them and a click lands on
 * whatever the eye sees on top: the top layer's bodies first, then each layer
 * under it in the order they paint, front to back.
 *
 * Hit ids are `draw:<id>` for the body and `draw:<id>#<n>` for anchor `n`, so
 * the controller can tell "move the whole shape" from "move this handle".
 *
 * A viewport drawing (`space: 'viewport'`) skips the time and price mapping:
 * its anchors are fractions of this pane's plot and scale by the plot size in
 * the render context, which is all that pan and zoom never touch. The one step
 * added is keeping its box inside the plot. Everything after the projection
 * (paint, handles, hit-testing) is shared, so the two spaces cannot drift
 * apart in how they are grabbed.
 *
 * The hover ring, the magnet ring and the hover handles are all overlay
 * state: they change on every pointer move, so they must only ever cost the
 * top canvas. The layer that paints them is the one whose `requestUpdate`
 * the chart maps to the cursor tier, which is why the bottom layer of a pair
 * stores that state and asks for nothing.
 *
 * A hit test asks a tool for its distance only where the drawing can be: each
 * layer keeps a box per drawing (src/draw/hit-index.ts) and a pointer outside
 * it costs four comparisons. The answer is the one a test of every drawing
 * gives, because a box covers every point its tool would report as a hit.
 */

/**
 * Where a layer paints: under the series, in the series band (placed above an
 * entry by the controller, and over the series while it is not), or on top.
 */
type DrawingLayerOrder = 'bottom' | 'series' | 'top';
/** The pointer kinds the layer sizes its targets for. */
type DrawingPointerKind = 'mouse' | 'touch' | 'pen';
/**
 * Paint order: by `zIndex`, ties by array position. `Array.prototype.sort` is
 * stable, which is what makes the tie rule hold without a second key.
 */
declare function sortByZIndex(drawings: readonly Drawing[]): Drawing[];
declare class DrawingLayer implements IPrimitive {
    private readonly _order;
    /** In paint order. */
    private _drawings;
    private _host;
    private _selected;
    /** The drawing under the pointer, whether or not it is selected. */
    private _hovered;
    /** Where the next click will land while the magnet is pulling it. */
    private _snap;
    /** Anchors of the in-progress drawing, plus the live cursor point. */
    private _preview;
    /** The layers under this one, front to back, whose handles and hits this layer answers for. */
    private _below;
    /** The layer that has taken over this one's handles and hit-testing. */
    private _above;
    /** Sizes grab targets for the device last seen. */
    private _touch;
    /** Hit boxes per drawing, kept while nothing that places them changes. */
    private readonly _index;
    /** Paint positions of the selected, unlocked, shown drawings, or null until asked again. */
    private _selectedAt;
    constructor(order?: DrawingLayerOrder);
    attached(host: PrimitiveHost): void;
    detached(): void;
    zOrder(): ZOrder;
    /** Drawings overlay the price range; they never drive it. */
    autoscaleInfo(): null;
    /**
     * The drawings this layer paints and answers for. The hit boxes kept for
     * the previous list all go, so a drawing edited in place must be handed
     * over again, as the controller does after every edit, for a hover to see
     * where it now is.
     */
    setDrawings(drawings: readonly Drawing[]): void;
    setSelected(ids: readonly string[]): void;
    /**
     * The drawing under the pointer. Its handles paint at a lighter weight than
     * a selection's, as the hint that it can be grabbed. Repaints only from the
     * layer that paints handles: the adopted bottom layer keeps the id for its
     * own bookkeeping and asks for nothing, since its repaint costs the series.
     */
    setHovered(id: string | null): void;
    hovered(): string | null;
    /**
     * The point the magnet will snap the next click to, or null when it is not
     * pulling. Painted as a hollow ring so the user sees the anchor land before
     * committing to it.
     */
    setSnapPoint(point: DrawingPoint | null): void;
    snapPoint(): DrawingPoint | null;
    /**
     * Size the grab targets for the device in use. Touch doubles the radii; a
     * mouse or a pen keeps the desktop sizes. The render context may carry the
     * same fact (`pointerType`), and when it does that wins per call, so a
     * first touch on a fresh chart is already sized right.
     */
    setPointerType(kind: DrawingPointerKind | null | undefined): void;
    setPreview(drawing: Drawing | null): void;
    /**
     * Adopt the layers under this one, front to back: the order they paint in,
     * reversed. From then on this layer paints the handles of all of them and
     * answers `hitTest` for all of them, and the adopted layers paint bodies
     * only. Handles have to live up here: a selected drawing under the series
     * would otherwise have its handles buried under the candles, and a handle
     * you cannot see is a handle you cannot grab. Pass null to release.
     */
    setBelow(layer: DrawingLayer | readonly DrawingLayer[] | null): void;
    /** Repaint for overlay-only state, from the layer that paints it. */
    private _requestOverlay;
    /** Whether targets are sized for a fingertip on this call. */
    private _isTouch;
    private _grabRadius;
    private _handleRadius;
    private _points;
    /** Selected, unlocked drawings of this layer, in paint order. */
    private _handled;
    /**
     * The hovered drawing of this layer, when it is one that could be grabbed
     * and is not already showing selection handles.
     */
    private _hoverHandled;
    draw(ctx: CanvasRenderingContext2D, rc: PrimitiveRenderContext): void;
    private _drawingContext;
    private _drawContent;
    private _drawPlacementGuide;
    /**
     * Anchor handles. A hover shows them light (thin, translucent) as a hint
     * that the shape can be grabbed; a selection shows them at full weight.
     */
    private _drawHandles;
    /** The magnet ring: hollow, in the axis text colour, at the snap point. */
    private _drawSnapRing;
    hitTest(x: number, y: number, rc: PrimitiveRenderContext): PrimitiveHit | null;
    /**
     * Paint positions of the drawings `_handled` lists, less the registry check,
     * which is read live: a tool registered after the list was made makes its
     * drawings handled from then on, with no new list to say so.
     */
    private _selectedPositions;
    /**
     * A drawing's body box: none for one that cannot be hit, everywhere for a
     * tool the box table does not know, else its tool's box at this radius.
     */
    private _measure;
    private _hitHandle;
    private _hitBody;
}

/**
 * Keyboard editing for drawings, as a pure mapping from a key event to the
 * action it means. The engine installs no listener: only the host knows whether
 * the chart has focus, whether a dialog is open, or whether the user is typing
 * into a text box, so the host reads the event, asks this function, and calls
 * the matching `DrawingController` method itself.
 *
 * Modifiers must match exactly, the same rule `matchDrawingShortcut` applies:
 * a chord with an extra modifier belongs to the host or the browser, and an
 * editing action must never shadow it. `Meta` (Cmd) counts as `Ctrl`.
 */
/** The key-event fields the mapping reads. A DOM `KeyboardEvent` satisfies it. */
interface DrawingKeyEvent {
    key: string;
    ctrlKey?: boolean;
    metaKey?: boolean;
    shiftKey?: boolean;
    altKey?: boolean;
}
interface DrawingKeyContext {
    /** At least one drawing is selected. */
    hasSelection: boolean;
    /**
     * A drawing is under the pointer without being selected. Copy, cut,
     * duplicate and delete act on it when nothing is selected; nudge does not,
     * because moving something the user has not picked reads as a glitch.
     */
    hasTarget: boolean;
    /** The user is typing (a text box is open). Every key is theirs. */
    editingText: boolean;
    /**
     * A tool is armed. Escape, Enter and Backspace then belong to placement
     * (cancel, finish, drop the last anchor) rather than to the selection.
     */
    placing?: boolean;
}
type DrawingKeyAction = {
    type: 'undo';
} | {
    type: 'redo';
} | {
    type: 'copy';
} | {
    type: 'cut';
} | {
    type: 'paste';
} | {
    type: 'duplicate';
} | {
    type: 'delete';
}
/** Move the selection by a screen distance; `dx` right and `dy` down, in px. */
 | {
    type: 'nudge';
    dx: number;
    dy: number;
}
/** Placement, while a tool is armed: leave it, close the shape, drop the last anchor. */
 | {
    type: 'cancel';
} | {
    type: 'finish';
} | {
    type: 'popAnchor';
};
/** How far an arrow key moves the selection, plain and with Shift. */
declare const NUDGE_STEP_PX = 1;
declare const NUDGE_STEP_SHIFT_PX = 10;
declare function keyToDrawingAction(e: DrawingKeyEvent, ctx: DrawingKeyContext): DrawingKeyAction | null;

/**
 * The drawing migration: 1.9.x to 2.0, and document version 2 to 3.
 *
 * Hosts persist `toJSON()` verbatim, and a 1.9.x host has done so for a long
 * time: a bare `Drawing[]` per pane in local storage, for a great many users.
 * Version 2 moved every text field out of the style bag, made `levels` a list
 * of {@link FibLevel} rather than bare ratios, and gave each drawing a
 * `zIndex`. Reading an old save through the new types would lose the text and
 * the levels silently, and the user would open 2.0 to a chart with holes in
 * it. This is the one place the old shape is understood, so nothing
 * downstream needs to.
 *
 * Rules, in the order of what they protect:
 *
 * - A drawing that can be rendered is kept. An optional field with the wrong
 *   type is dropped on its own; only an entry with no usable tool, anchors or
 *   pane is dropped, because it could never be drawn again anyway. The
 *   clipboard is stricter (a paste is all-or-nothing) and gates before it
 *   calls this.
 * - Ids and array order survive. The list order is the paint order, and a
 *   host may hold ids of its own against the drawings.
 * - Unknown fields are dropped, so the style bag stays closed and a stray key
 *   never reaches the model.
 * - An unregistered tool is not a reason to drop anything: a plugin tool may
 *   register after the layout is restored. This module never consults the
 *   registry, which also keeps it pure.
 * - A viewport drawing keeps its `viewportPoints` and carries no data
 *   anchors; one without usable viewport anchors is dropped, since falling
 *   back to its data anchors would put it somewhere it never was. Data is the
 *   default space and is never written out, so a document with no viewport
 *   drawing comes back exactly as it went in.
 * - Garbage yields an empty document. This runs on the load path, where an
 *   exception would take the whole chart down with it.
 *
 * Version 3 added one optional field, the interval range a drawing is shown
 * on, so a version 2 document needs no upgrade and comes back unchanged,
 * version and all. A range is kept when it names at least one bound, and the
 * document out is version 3 only when a drawing kept one. A document from a
 * later version is read the same lenient way: what this build knows is kept
 * and the rest dropped, since refusing it would empty the chart on load.
 *
 * Idempotent on a version 2 or 3 document, so a host can call it on every load.
 */

/**
 * Upgrade saved drawings to the current document shape. Accepts a 1.9.x
 * `Drawing[]` or a {@link DrawingsDocument} of any version. Every field is
 * validated, the 1.9.x text keys are lifted out of the style bag into `text`,
 * bare level ratios become coloured levels, and array order is kept. An entry
 * that could never be rendered is dropped; anything unparseable yields an
 * empty document rather than an error. The result is version 3 when a drawing
 * carries an interval range and version 2 otherwise.
 */
declare function migrateDrawings(input: unknown): DrawingsDocument;

/**
 * Whether a drawing is shown on a chart whose interval is `interval`. A
 * drawing with no range is shown on every interval. For a host marking the
 * drawings a range hides (an objects panel, a settings preview) without a
 * controller at hand; `DrawingController.shownOnInterval` answers for the
 * chart the controller is on.
 */
declare function drawingShownOnInterval(drawing: Pick<Drawing, 'intervals'>, interval: string | null | undefined): boolean;

/**
 * The draw tier's events on the chart bus, added to the base's `ChartEventMap`
 * by declaration merging. The base declares the map and cannot name a
 * `Drawing` without importing this tier, so the names arrive with the tier: a
 * host that imports `openalgo-charts/draw` (or the widget, which does) gets
 * `chart.on('draw:add', ...)` typed.
 *
 * Two families, two granularities, both kept. `draw:*` reports per drawing and
 * per tool or gesture: `draw:add`, `draw:update` and `draw:remove` fire once for
 * each drawing touched and carry it, and `draw:select` names the primary
 * selection. `drawing:*` reports per mutation and per selection change:
 * `drawing:change` fires once after the per-drawing events with every id, and
 * `drawing:select` carries the whole selection. A host that renders one
 * drawing's properties listens to the first; one that refreshes a list or an
 * undo button listens to the second.
 */

declare module 'openalgo-charts' {
    interface ChartEventMap {
        /** A drawing tool became active, or none is (`tool: null`). */
        'draw:tool': DrawingToolEvent;
        /** A drawing was created, once per drawing. */
        'draw:add': DrawingEvent;
        /** A drawing's anchors, style, text or flags changed, once per drawing. */
        'draw:update': DrawingEvent;
        /** A drawing was deleted, once per drawing. */
        'draw:remove': DrawingEvent;
        /** The selection changed; `id` is the primary (first picked) id. Fires with `drawing:select`. */
        'draw:select': DrawingIdEvent;
        /** A copy reached the clipboard (copies, not the live drawings). */
        'draw:copy': DrawingListEvent;
        /** A cut wrote to the clipboard and then deleted. */
        'draw:cut': DrawingListEvent;
        /** The drawings a paste created, after their own `draw:add`. */
        'draw:paste': DrawingListEvent;
        /** Where a drag is carrying drawings, before it commits (copies). */
        'draw:preview': DrawingListEvent;
        /** A drag preview ended, committed or not. */
        'draw:preview-clear': DrawingIdsEvent;
        /** `fromJSON` or a chart restore replaced every drawing. */
        'draw:restore': EmptyEvent;
        /** A controller was destroyed and let go of the chart. */
        'draw:destroy': {
            controller: DrawingController;
        };
        /** The measure gesture started or ended. */
        'draw:measure': DrawingModeEvent;
        /** The eraser was switched on or off. */
        'draw:eraser': DrawingModeEvent;
        /** One per mutation, after the per-drawing `draw:*` events. */
        'drawing:change': DrawingChangeEvent;
        /** The whole selection, when it changed. */
        'drawing:select': DrawingIdsEvent;
        /** The unselected drawing under the pointer changed. */
        'drawing:hover': DrawingIdEvent;
    }
}

/**
 * Hit-test geometry for the drawing tier. Everything works in media px and
 * returns a distance, so `DrawingLayer` can pick the nearest shape under the
 * cursor with one comparison rather than each tool inventing its own tolerance.
 */

/** Distance from a point to a line segment. */
declare function distToSegment(px: number, py: number, a: ScreenPoint, b: ScreenPoint): number;
/** Distance to an infinite line through `a` and `b`. */
declare function distToLine(px: number, py: number, a: ScreenPoint, b: ScreenPoint): number;
/** Distance to a polyline's nearest segment. */
declare function distToPolyline(px: number, py: number, pts: readonly ScreenPoint[]): number;
/** Normalised rect corners from two opposite anchors. */
declare function rectOf(a: ScreenPoint, b: ScreenPoint): {
    x0: number;
    y0: number;
    x1: number;
    y1: number;
};
/**
 * The axis-aligned box around any number of points. What a shape's label is
 * laid out against when the shape itself is not a rectangle (a triangle, a
 * rotated rectangle, a circle's centre plus radius).
 */
declare function boundsOf(pts: readonly ScreenPoint[]): {
    x0: number;
    y0: number;
    x1: number;
    y1: number;
};
/**
 * Distance to a rectangle: 0 anywhere inside when `filled`, otherwise the
 * distance to the nearest edge. An unfilled rect must stay grabbable only by
 * its outline, or it would swallow every click in its interior.
 */
declare function distToRect(px: number, py: number, a: ScreenPoint, b: ScreenPoint, filled: boolean): number;
/**
 * Distance to an axis-aligned ellipse inscribed in the box `a`..`b`. Uses the
 * normalised radius, scaled back by the smaller semi-axis: exact enough for a
 * grab tolerance and far cheaper than the true closest-point solve.
 */
declare function distToEllipse(px: number, py: number, a: ScreenPoint, b: ScreenPoint, filled: boolean): number;
/** Clamp a segment's endpoints out to the plot edges, keeping its slope. */
declare function extendSegment(a: ScreenPoint, b: ScreenPoint, width: number, left: boolean, right: boolean): [ScreenPoint, ScreenPoint];

/**
 * Ramer-Douglas-Peucker thinning. Keeps the first and last points and, between
 * them, only those that pull the stroke more than `epsilonPx` away from the
 * chord of their kept neighbours; every dropped point lies within `epsilonPx`
 * of the returned polyline. Kept points come back at their exact input
 * coordinates, in input order. An `epsilonPx` that is not positive returns a
 * copy. The result never aliases its input, so a caller may mutate it.
 */
declare function rdpSimplify(points: ReadonlyArray<ScreenPoint>, epsilonPx: number): ScreenPoint[];
/**
 * Appends a Catmull-Rom spline through `points` to the current path as cubic
 * beziers: one `moveTo`, then a `bezierCurveTo` per segment that ends exactly
 * on the next point. The caller owns `beginPath` and `stroke`, so the same
 * spline can also outline a filled ribbon. Two points degrade to a straight
 * `lineTo`; fewer emit nothing.
 *
 * `tension` scales the tangent at each point: 0.5 is the classic spline and 0
 * collapses every segment to a straight line. Values outside 0..1 are clamped,
 * since past 1 the tangents grow long enough for a segment to fold back on
 * itself.
 */
declare function catmullRom(ctx: CanvasRenderingContext2D, points: ReadonlyArray<ScreenPoint>, tension?: number): void;
/**
 * A line width scaled by pointer pressure. Pressure 0.5, which a mouse
 * reports while a button is down, returns `baseWidth` unchanged, so a stroke
 * drawn without a pen is the width the user configured. A pen sweeps the
 * width from `baseWidth * (1 - range)` at no pressure to
 * `baseWidth * (1 + range)` at full pressure. Pressure and `range` are
 * clamped to 0..1, and a pressure that is not a number (a synthetic event)
 * counts as 0.5.
 */
declare function pressureWidth(baseWidth: number, pressure: number, range?: number): number;

/** Channels, pitchforks and dedicated line tools. Geometry stays in media pixels. */

declare const ADVANCED_LINE_TOOLS: readonly DrawingTool[];

/** Screen-space curves and price/time constructions for advanced drawings. */

declare const ADVANCED_GEOMETRY_TOOLS: readonly DrawingTool[];

/** Labeled price patterns with shared paint and hit geometry. */

/** Pattern drawing descriptors. Registration is owned by the draw-tier catalog. */
declare const PATTERN_DRAWING_TOOLS: readonly DrawingTool[];

declare const DRAW_TIER: "draw";

export { ADVANCED_GEOMETRY_TOOLS, ADVANCED_LINE_TOOLS, ALIGN_OPTIONS, ANCHORED_VWAP, ARC, ARROW, ARROW_DOWN, ARROW_LEFT, ARROW_RIGHT, ARROW_UP, type AnalysisProfileRow, type AnalysisStatus, type AnchoredVwapOptions, type AnchoredVwapPoint, type AnchoredVwapResult, type AnchoredVwapSource, BALLOON, BRUSH, BUILTIN_DRAWING_TOOLS, CALLOUT, CHROME_ICONS, CHROME_ICON_ACCENTS, CHROME_ICON_ATTRS, CHROME_ICON_FILLED, CHROME_ICON_SIZE, CHROME_ICON_STROKE, CHROME_ICON_VIEWBOX, CIRCLE, COLOR_FIELD, COMMENT, CROSS_LINE, CURVE, CYCLE_PALETTE, CYCLIC_LINES, type ClipboardPort, DATE_RANGE, DEFAULT_FIB, DEFAULT_FIB_FAN, DEFAULT_FIB_TIME_ZONE, DEFAULT_GANN_BOX, DEFAULT_GANN_FAN, DOUBLE_CURVE, DRAWING_CLIPBOARD_KEY, DRAWING_CLIPBOARD_VERSION, DRAWING_STATE_VERSION, DRAWING_TOOL_ACCENTS, DRAWING_TOOL_ICONS, DRAW_TIER, type DrawContext, type Drawing, type DrawingChangeEvent, type DrawingChangeKind, type DrawingChartHost, DrawingClipboard, type DrawingClipboardOptions, DrawingController, type DrawingControllerOptions, type DrawingDocumentStore, type DrawingEditOptions, type DrawingEvent, type DrawingGestureOptions, type DrawingGroup, type DrawingIdEvent, type DrawingIdsEvent, type DrawingInput, type DrawingInstrument, type DrawingIntervalRange, type DrawingKeyAction, type DrawingKeyContext, type DrawingKeyEvent, DrawingLayer, type DrawingLayerOrder, type DrawingLinkChart, type DrawingLinkContext, type DrawingLinkContextSource, DrawingLinkGroup, type DrawingLinkOptions, type DrawingListEvent, type DrawingModeEvent, type DrawingPatch, type DrawingPlacementOptions, type DrawingPoint, type DrawingPointerKind, type DrawingPolicy, type DrawingSpace, type DrawingStackTarget, type DrawingStyle, type DrawingText, type DrawingTextStorage, type DrawingTool, type DrawingToolEvent, type DrawingValueContext, type DrawingsDocument, ELLIPSE, EXTENDED_LINE, EXTEND_FIELDS, type ExpandContext, FIB_CHANNEL, FIB_EXTENSION, FIB_FAN, FIB_RETRACEMENT, FIB_TIME_ZONE, FILL_FIELDS, FIXED_RANGE_VOLUME_PROFILE, FLAG_MARK, FONT_FIELDS, FONT_OPTIONS, FORECAST, type FibLevel, type FieldGroup, type FieldKind, type FixedRangeVolumeProfileOptions, type FixedRangeVolumeProfileResult, GANN_BOX, GANN_FAN, HIGHLIGHTER, HORIZONTAL_LINE, HORIZONTAL_RAY, type HitContext, ICON_ATTRS, ICON_SIZE, ICON_STROKE, ICON_SYMBOL_PREFIX, ICON_VIEWBOX, INTERVAL_FIELDS, type IconAttrs, type IconSvgOptions, type InputAnchorStep, InstrumentDrawings, type InstrumentDrawingsChart, type InstrumentDrawingsError, type InstrumentDrawingsOptions, LEVEL_FIELDS, LEVEL_NEUTRAL, LINE_FIELDS, LINE_STYLE_FIELD, LINE_STYLE_OPTIONS, LINE_WIDTH_FIELD, LONG_POSITION, type LayoutIconSlot, MEASURE, type MagnetMode, NOTE, NUDGE_STEP_PX, NUDGE_STEP_SHIFT_PX, PARALLEL_CHANNEL, PATH, PATTERN_DRAWING_TOOLS, PLATE_TEXT_FIELDS, POLYLINE, PRICE_LABEL, PRICE_NOTE, PRICE_RANGE, RAY, RECTANGLE, ROTATED_RECTANGLE, SHAPE_TEXT_FIELDS, SHORT_POSITION, SHOW_LABELS_FIELD, SIGNPOST, SINE_LINE, SPACE_FIELD, SPACE_OPTIONS, type ScreenPoint, type SettingsField, type SettingsSchema, type ShortcutEvent, type StudyInputTarget, TABLE, TEXT, TEXT_FIELDS, TEXT_POSITION_OPTIONS, TEXT_VALUE_FIELD, TIME_CYCLES, TREND_LINE, TRIANGLE, type ToolCursorOptions, VALIGN_OPTIONS, VERTICAL_LINE, type ViewportPoint, anchoredVwapAnalysis, applyDrawingSettings, boundsOf, catmullRom, chartTypeIcon, chromeIcon, chromeIconAccent, chromeIconIds, chromeIconSvg, clearMemoryClipboard, cloneDrawing, cloneLevels, coerceSettingValue, composeSettings, createDrawingLinkGroup, createInstrumentDrawings, cycleColor, decodeClipboardPayload, distToEllipse, distToLine, distToPolyline, distToRect, distToSegment, drawingSettingsSchema, drawingShortcuts, drawingShownOnInterval, drawingToolAccent, drawingToolIcon, drawingToolIconIds, encodeClipboardPayload, extendSegment, fixedRangeVolumeProfileAnalysis, formatRatio, gannLabel, getDrawingTool, hasDrawingTool, iconSprite, iconSvg, iconUse, instrumentDrawingsKey, keyToDrawingAction, layoutIconPath, levelColor, matchDrawingShortcut, memoryDrawingStore, migrateDrawings, migrateUnscopedDrawings, pressureWidth, rdpSimplify, readDrawingSetting, readDrawingSettings, rectOf, registerBuiltinDrawingTools, registerDrawingTool, registeredDrawingTools, sanitizeDrawing, sortByZIndex, studyInputTarget, systemClipboard, toolCursor, webStorageDrawingStore };
