/** Library version string. Matches package.json (including locally prepared releases). */
declare const VERSION = "2.6.6";
/** Returns the current library version. */
declare function version(): string;

/**
 * Invalidation model (ARCHITECTURE.md §3.2).
 *
 * A single global level is too coarse for multi-pane indicators and trade
 * overlays, so the mask carries a **global level + a per-pane map**. Multiple
 * invalidations within one frame raise the same mask, each level only ever
 * going up. It also keeps a queue of time-scale operations that nothing in the
 * chart reads, deprecated with the methods that fill and fold it (see
 * {@link TimeScaleOp}).
 */
/** How much of a pane (or the whole chart) must be repainted this frame. */
declare const InvalidationLevel: {
    /** Nothing to do. */
    readonly None: 0;
    /** Repaint only the top (overlay) canvas: the crosshair and the top-layer primitives. */
    readonly Cursor: 1;
    /** Repaint the base canvas at the current scales: series moved or changed, no rescale. */
    readonly Light: 2;
    /** Recompute scales/ticks then repaint everything. */
    readonly Full: 3;
};
type InvalidationLevel = (typeof InvalidationLevel)[keyof typeof InvalidationLevel];
/** Per-pane invalidation entry. `autoScale` requests a price-axis rescale. */
interface PaneInvalidation {
    level: InvalidationLevel;
    autoScale: boolean;
}
/**
 * Discrete time-scale operations a mask can queue, kept for compatibility.
 * The mask queues and merges them, and the chart never reads the queue: pan,
 * zoom, fit and reset change the shared time scale directly, and the kinetic
 * and wheel-zoom glides are stepped at the top of the render loop's frame
 * (ARCHITECTURE.md §3.2).
 *
 * @deprecated Removed in 3.0.0. A queued operation does nothing; change the time scale directly with
 * `chart.timeScale`, `chart.fitContent` or `chart.setVisibleLogicalRange`.
 */
type TimeScaleOp = {
    type: 'fitContent';
} | {
    type: 'applyBarSpacing';
    value: number;
} | {
    type: 'applyRightOffset';
    value: number;
} | {
    type: 'reset';
};
declare class InvalidateMask {
    private _globalLevel;
    private readonly _panes;
    private _timeScaleOps;
    constructor(globalLevel?: InvalidationLevel);
    get globalLevel(): InvalidationLevel;
    /** Raise the chart-wide level (monotonic: it only ever increases). */
    invalidateGlobal(level: InvalidationLevel): void;
    /** Raise a single pane's level without touching the others. */
    invalidatePane(paneIndex: number, invalidation: PaneInvalidation): void;
    paneInvalidation(paneIndex: number): PaneInvalidation | undefined;
    panes(): ReadonlyMap<number, PaneInvalidation>;
    /**
     * Queue an operation. The chart never reads the queue; see {@link TimeScaleOp}.
     *
     * @deprecated Removed in 3.0.0. The operation does nothing; change the time scale directly.
     */
    addTimeScaleOp(op: TimeScaleOp): void;
    /**
     * The operations queued so far, which nothing in the chart reads.
     *
     * @deprecated Removed in 3.0.0 with {@link TimeScaleOp}.
     */
    timeScaleOps(): readonly TimeScaleOp[];
    isEmpty(): boolean;
    /**
     * Fold another mask into this one.
     *
     * @deprecated Removed in 3.0.0. The chart raises one mask per frame and never folds two, and a
     * host holds only the mask `chart.invalidate` hands it; raise levels on that one.
     */
    merge(other: InvalidateMask): void;
}

/**
 * A single `<canvas>` element with media/bitmap sizing. Constructed only in a
 * browser; the size math above is the part exercised by unit tests.
 */
declare class CanvasLayer {
    readonly element: HTMLCanvasElement;
    readonly ctx: CanvasRenderingContext2D;
    private _mediaWidth;
    private _mediaHeight;
    private _dpr;
    /** The device-pixel box the browser last reported for this canvas, 0 when none is trusted. */
    private _deviceWidth;
    private _deviceHeight;
    constructor(doc: Document, zIndex: number);
    get mediaWidth(): number;
    get mediaHeight(): number;
    get pixelRatio(): number;
    /** Resize backing buffer + CSS box. No-op if nothing changed. */
    resize(mediaWidth: number, mediaHeight: number, dpr: number): void;
    /**
     * Size the backing store to the device pixels the browser reports the box
     * covers (`devicePixelContentBoxSize`). `media x dpr` is only an estimate:
     * a box that starts part way into a device pixel is snapped by the browser
     * to one pixel more or fewer, and a backing store one pixel off is stretched
     * across the box, blurring everything on it. The media size and the ratio
     * are unchanged, so drawing code keeps scaling by `dpr`.
     *
     * A report more than a pixel away from `media x dpr` is not a snapped box
     * but a different scale, and is refused: an emulated device scale reports
     * the box at the host's real ratio while `devicePixelRatio` says another,
     * and a store sized to it would crop the chart.
     *
     * Returns true when the bitmap changed, which clears it: the caller repaints.
     */
    setDeviceSize(width: number, height: number): boolean;
    /**
     * Drop the reported box, for a report the caller could not match to this
     * canvas's current size: `resize` then estimates until the next report.
     */
    forgetDeviceSize(): void;
    /** A reported device size, if it is whole, positive and within a pixel of `estimate`. */
    private _reported;
    /**
     * A device size last reported, if a box of `estimate` device pixels can
     * still snap to it: the floor or the ceiling of the estimate, which for a
     * whole size is the one less than a pixel away.
     */
    private _snapsTo;
    /** Clear the whole bitmap and reset the transform to bitmap (device-px) scope. */
    clearBitmap(): void;
}

interface PriceRange {
    min: number;
    max: number;
}
/**
 * Price-scale mode, each one a coordinate transform (see `PriceScale._t`):
 * `linear` is the identity, `logarithmic` is log10, and `percentage` /
 * `indexed-to-100` rebase every price against a baseline: percent change from
 * it, or the baseline rebased to 100. The rebasing pair needs a baseline from
 * the data before it can transform anything, see `PriceScale.setBaseline`.
 *
 * Overlay scales are supported in every mode: a series added with
 * `priceScaleId: ''` gets a hidden scale with its own autoscale, see
 * `Pane._scaleFor`.
 */
type PriceScaleMode = 'linear' | 'logarithmic' | 'percentage' | 'indexed-to-100';
/**
 * The four price-scale modes, in the order a menu lists them. The one list:
 * restore, the hit prefilter and the settings schema read it, so a fifth mode
 * is added here and in the type above.
 */
declare const PRICE_SCALE_MODES: readonly PriceScaleMode[];
/** Whether a mode rebases prices against a baseline rather than mapping them directly. */
declare function isRebasing(mode: PriceScaleMode): boolean;
interface PriceScaleOptions {
    /** Fraction of pane height kept empty at top/bottom (default 0.1 each). */
    marginTop: number;
    marginBottom: number;
    /**
     * Instrument tick size (minMove), e.g. 0.05. 0 infers it from the range.
     *
     * A property of the instrument and not of the axis, so it belongs only on a
     * scale that quotes one. An oscillator's pane reads in its own units and is
     * left at 0 there, which is why `Chart.setPriceScaleOptions` withholds it
     * from those panes rather than broadcasting it like the rest of this block.
     */
    minMove: number;
    /**
     * Least decimals a scale with no tick will print. 0 leaves the span to
     * decide alone, which is right for a price axis and too coarse for a bounded
     * oscillator: see `precision()`.
     *
     * Ignored once `minMove` is set, because a declared tick is a stronger
     * statement about the instrument than a floor is about the axis.
     */
    minPrecision: number;
    /** Linear, logarithmic or rebased (percentage / indexed-to-100) mapping between price and y. */
    mode: PriceScaleMode;
    /** Flip the axis (price increases downward): for spread/short views. */
    inverted: boolean;
}
declare const DEFAULT_PRICE_SCALE_OPTIONS: PriceScaleOptions;
/**
 * Pure: compute a price range from data extremes plus top/bottom margins.
 * Returns a padded [min,max]; widens a degenerate (flat) range so it's drawable.
 *
 * The margins are fractions of the **pane height**, so the data band occupies
 * the `1 - marginTop - marginBottom` left between them. Padding the data span
 * by the margin instead (which is what this did) makes the reserved space
 * depend on how tall the data happens to be: an overlay asking for `0.82` to
 * sit in the bottom 18% got `high + 0.82 * span`, leaving its bars 55% of the
 * pane. The difference only shows at large margins; at the 0.1 default the two
 * readings land within a few percent of each other.
 */
declare function autoscaleRange(low: number, high: number, marginTop: number, marginBottom: number): PriceRange;
declare class PriceScale {
    private _options;
    private _height;
    private _min;
    private _max;
    private _autoScale;
    /**
     * True once a real range has been applied. The default 0..1 is a placeholder,
     * not a measurement: anything converting between y and price before that would answer
     * confidently with nonsense.
     */
    private _scaled;
    /**
     * Baseline for the rebasing modes. null is "nothing measured yet", the same
     * idea as `_scaled`: until the data supplies one there is no percent change
     * to report, so the transform stays the identity.
     */
    private _baseline;
    /**
     * A range declared by whoever owns this scale rather than measured from the
     * data: an oscillator's 0..100, a signal's -1..1. It is remembered, not just
     * applied, because "fit this axis" has a different answer on a declared axis:
     * the fit *is* the declared range, and a request to auto-fit has to be able
     * to find its way back to it. See `setFixedRange`.
     */
    private _fixedRange;
    private _fixedRangeOwner;
    /** Host writes can keep the same numeric range and still take control of the view. */
    private _manualRangeOverride;
    private _priceFormatter;
    constructor(options?: Partial<PriceScaleOptions>);
    get options(): PriceScaleOptions;
    /** Merge partial options (minMove, mode, inverted, margins) at runtime. */
    setOptions(opts: Partial<PriceScaleOptions>): void;
    setHeight(height: number): void;
    get height(): number;
    setPriceRange(range: PriceRange): void;
    /** Internal renderer projection update, preserving host ownership of the view. */
    setComputedRange(range: PriceRange): void;
    private _writePriceRange;
    /** Whether a real price range has been applied (see `_scaled`). */
    get scaled(): boolean;
    /**
     * Set the baseline the rebasing modes (`percentage`, `indexed-to-100`) quote
     * against. Ignored by `linear` and `logarithmic`.
     *
     * The baseline is *data*, not geometry, so the scale cannot find it alone:
     * the autoscale pass supplies the first value of the visible range each
     * frame. That is what makes panning re-base, so the axis always reads as
     * change measured from the left edge of what is on screen.
     *
     * Pass null when there is nothing to measure (no series, no visible bars).
     * A rebasing mode without a baseline falls back to the identity transform
     * and behaves exactly like `linear`, rather than answering confidently with
     * nonsense before the first frame.
     *
     * Note what a rebase does *not* do: it is affine over a range held in price
     * units, so it relabels the pane rather than reshaping it. One series looks
     * the same as it does on a linear scale, by definition (percent change is a
     * straight-line function of price). Two instruments become comparable when
     * each sits on its own scale with its own baseline, which is the overlay
     * mechanism, not a second transform here.
     */
    setBaseline(value: number | null): void;
    /** The baseline last supplied, or null if none (see `setBaseline`). */
    get baseline(): number | null;
    priceRange(): PriceRange;
    /** Whether the range tracks the data (true) or has been set manually (false). */
    get autoScale(): boolean;
    setAutoScale(on: boolean): void;
    /**
     * Declare the range this scale must hold, or pass null to withdraw it.
     *
     * It is what an indicator that owns its pane asks for (RSI 0..100, a signal
     * line -1..1): a band whose meaning is in the numbers themselves, so fitting
     * it to the values on screen would destroy the reading rather than improve
     * it. The scale goes manual while one is held, because the range is declared
     * and not measured, and `setAutoScale(true)` returns to it.
     *
     * Withdrawing (null) leaves the range where it is; the caller decides whether
     * the scale goes back under autoscale, because it is the one that knows
     * whether anything is left on the pane to measure.
     */
    setFixedRange(range: PriceRange | null): void;
    /** Internal host helper: install a study default without taking over a host-controlled range. */
    setOwnedFixedRange(owner: object, range: PriceRange): boolean;
    /** Internal host helper: withdraw only this owner's default, preserving later host range writes. */
    clearOwnedFixedRange(owner: object): boolean;
    /** Internal host helper: ownership is identity-based, independent of equal numeric endpoints. */
    ownsFixedRange(owner: object): boolean;
    /** Internal host metadata for saving an owned default and any later manual view override. */
    ownedFixedRangeState(owner: object): {
        manual: boolean;
    } | null;
    /** Whether an unused scale retains configuration beyond a measured automatic range. */
    hasConfiguration(): boolean;
    /** The declared range this scale is holding, or null. See `setFixedRange`. */
    get fixedRange(): PriceRange | null;
    /**
     * Forget the measured range and go back to the placeholder.
     *
     * Called when a scale loses its last series: the range it holds described
     * something that is no longer on the chart, and whatever arrives next may
     * plot nothing at all. An indicator whose entire output is a table plots no
     * values, and inheriting a departed oscillator's 0..100 left its pane
     * labelled with a ladder it had no prices for.
     *
     * A manually scaled axis is left alone: the user set that range, and nothing
     * would recompute it if it were thrown away.
     */
    reset(): void;
    /**
     * Manually scale the visible range around its centre. `factor` > 1 widens the
     * range (compress / zoom out), < 1 narrows it (expand / zoom in). Switches the
     * scale to manual mode so autoscale stops overriding it.
     */
    scaleAroundCenter(factor: number): void;
    /** Scale around a screen coordinate, retaining its price in every scale mode. */
    scaleAtY(y: number, factor: number): void;
    /**
     * Pan the visible range vertically by `dy` media px (dragging the plot up/down).
     * Works in transformed space so it's correct for log scales, and respects
     * `inverted`. Switches to manual mode so autoscale stops overriding it.
     */
    panByPixels(dy: number): void;
    /**
     * Recompute the visible range from data extremes + configured margins.
     *
     * The range stays in *price* units in every mode, so the rebasing modes need
     * no separate pass: their transform is affine with a positive scale factor,
     * which maps a margin of 10% of the price span onto a margin of 10% of the
     * percent span. Log pads in log space (`scaleAutoscaleRange`), so the data
     * keeps the same share of the pane there too.
     */
    autoscale(low: number, high: number, progress?: number): boolean;
    /**
     * The baseline actually in force, or null when this mode does not rebase or
     * the value it was given cannot carry one.
     *
     * Zero and negative baselines are rejected on purpose. Percent change from
     * zero is undefined (every price is an infinite move away), and a negative
     * baseline flips the sign of the whole transform, so a rising price would
     * draw *downward* on an axis that still labels itself normally. Falling back
     * to the identity is recoverable and obvious on screen; an axis that
     * silently runs backwards is neither. Instruments that legitimately cross
     * zero (spreads, oscillators) have no percent-of-baseline reading to want.
     */
    private _rebase;
    /**
     * Coordinate transform for the active mode: identity for linear, log10 for
     * log, and a rebase against the baseline for percentage/indexed-to-100.
     * The rebasing pair share one ladder: percent change is the index minus the
     * 100 it is rebased to.
     */
    private _t;
    private _tInv;
    /** Price to y (media px). A higher price gives a smaller y (top of pane), unless inverted. */
    priceToY(price: number): number;
    /** y (media px) to price. */
    yToPrice(y: number): number;
    /**
     * Decimal precision implied by minMove (or the visible range if unset).
     *
     * While a rebase is in force the labels are percent/index points, where a
     * price tick size means nothing: precision comes from the transformed span
     * instead, with two decimals as the floor traders expect of a percentage
     * ("+3.42%") and more only when the visible band is tighter than that.
     *
     * A scale carrying no tick is not quoting an instrument, so the span is all
     * there is to go on. That alone reads too coarse on a bounded oscillator: an
     * RSI spanning 0 to 100 implies a step of 1 and prints a whole-number ladder,
     * so a reading of 62.24 lands on a rung labelled "62" and a trader comparing
     * it to a 70 level is reading a number that has been rounded past the part
     * they care about. `minPrecision` is the floor for that case, and it is the
     * same two decimals the percent branch above already settles on for the same
     * reason.
     */
    precision(): number;
    /**
     * Tick prices for the axis ladder, at most `maxTicks` of them. Linear and log
     * get the nice ladder over the price range, exactly what the axis renderer
     * used to build for itself.
     *
     * The rebasing modes need it built in *label* space: a nice price is an ugly
     * percentage, and a ladder of "+3.47%, +6.94%" is not a ladder. So the nice
     * values are chosen over the transformed range and mapped back to the prices
     * the axis positions with.
     */
    ticks(maxTicks?: number): number[];
    /** Snap a price to the instrument tick size (no-op if minMove is 0). */
    snapToTick(price: number): number;
    /**
     * Override the numeric formatting used for axis tick labels, the last-price
     * tag, and price-line labels (e.g. a currency or percent format). Pass null
     * to restore the default tick-size-aware `toFixed`.
     */
    setPriceFormatter(fn: ((price: number) => string) | null): void;
    /**
     * Format a price for axis/label display. While a rebase is in force the
     * label is the rebased value, "+3.42%" or "103.42", and it outranks a custom
     * price formatter: a currency prefix on a percent change would read as money
     * that is not there. The formatter takes over again the moment the mode does.
     */
    format(price: number): string;
    /** Clamp a y to the pane (used by crosshair/order dragging). */
    clampY(y: number): number;
}

interface LogicalRange {
    from: number;
    to: number;
}
interface TimeScaleOptions {
    barSpacing: number;
    minBarSpacing: number;
    maxBarSpacing: number;
    /** Empty bars of space kept to the right of the latest bar. */
    rightOffset: number;
}
declare const DEFAULT_TIME_SCALE_OPTIONS: TimeScaleOptions;
declare class TimeScale {
    private _barSpacing;
    private _rightOffset;
    private readonly _minBarSpacing;
    private readonly _maxBarSpacing;
    private _width;
    private _baseIndex;
    constructor(options?: Partial<TimeScaleOptions>);
    setWidth(width: number): void;
    get width(): number;
    get barSpacing(): number;
    setBarSpacing(value: number): void;
    /** Bound a pending zoom target without changing the displayed viewport. */
    constrainBarSpacing(value: number): number;
    get rightOffset(): number;
    setRightOffset(value: number): void;
    /** Logical index of the latest bar; the right edge anchors to baseIndex+rightOffset. */
    setBaseIndex(index: number): void;
    private _rightEdgeIndex;
    /** Logical index to x (media px), bar center. */
    indexToX(index: number): number;
    /** x (media px) to fractional logical index. */
    xToIndex(x: number): number;
    /** Currently visible logical index range (fractional, unclamped to data). */
    visibleRange(): LogicalRange;
    /** Alias of `visibleRange()` for naming parity with common charting APIs. */
    getVisibleLogicalRange(): LogicalRange;
    /**
     * Set the visible logical range (best effort): pick a bar spacing so the span
     * fills the width and anchor the right edge at `range.to`. Bar spacing is
     * clamped to [min,max], so an extreme span lands at the nearest zoom. Fires the
     * change handler so a host that mutates the scale directly still repaints.
     */
    setVisibleLogicalRange(range: LogicalRange): void;
    /** Fired after changed navigation or any valid explicit range request; layout/data setters stay silent. */
    setChangeHandler(fn: ((before: LogicalRange) => void) | null): void;
    private _onChange;
    private _notifyChange;
    /**
     * Pan by a pixel delta. Positive `dx` drags chart content to the right
     * (revealing older bars), matching a natural left-button drag.
     */
    scrollByPixels(dx: number): void;
    /**
     * Zoom around an anchor x (the cursor): change bar spacing by `factor`
     * while keeping whatever logical index sits under `focusX` pinned there.
     * `factor` > 1 zooms in (wider bars).
     */
    zoomAtX(focusX: number, factor: number): void;
    /** Choose bar spacing so `barCount` bars fit the width, anchored at the right edge. */
    fitContent(barCount: number): void;
}

/**
 * Internal time is always **UTC seconds** (integer). Feed adapters convert
 * broker formats (IST strings, epoch ms) to this at the edge; see ARCHITECTURE.md §4.0.
 */
type UTCSeconds = number;
/** The original, caller-supplied time value, echoed back untouched in callbacks. */
type OriginalTime = number | string;
/** A single OHLC(V) bar. `volume` is optional (not all feeds carry it). */
interface Bar {
    time: UTCSeconds;
    open: number;
    high: number;
    low: number;
    close: number;
    volume?: number;
    /**
     * Open interest: contracts outstanding at the end of this bar. Optional,
     * because only a derivatives feed carries it; on a cash instrument it is
     * absent, and absent is not zero. Zero is a real reading on a contract nobody
     * holds.
     *
     * **It is a level, not a flow, and that is the whole reason it needs saying
     * here.** Volume is a quantity traded *during* the bar, so folding five
     * one-minute bars into a five-minute bar adds five volumes together. Open
     * interest is a position *as at* the bar, so the same fold takes the last
     * one and adding them would produce a number five times too large that still
     * looks entirely plausible on a chart.
     *
     * Every aggregation path in this library therefore treats the two
     * differently: `mergeBars`, the higher-timeframe fold in `securitySeries`,
     * and the tick-to-bar builders all sum volume and carry the latest open
     * interest. A transform that maps one source bar to one output bar passes it
     * through; one that invents bars from price alone (Renko, Point and Figure)
     * does not carry it at all, because a synthetic bar has no instant to be the
     * position as at.
     */
    oi?: number;
    /**
     * Per-bar colour override, honoured by every Family-A renderer: candles and
     * OHLC bars take it on body, border and wick together, histogram and column
     * on the bar, and line, step, area and the HLC-area close line split their
     * stroke into runs at the bars where it changes. Baseline is the exception,
     * its stroke is already split by the above/below-base rule.
     * A MACD histogram is four colours by momentum, and a conditional study is
     * two. Neither is expressible with one colour for the whole series.
     */
    color?: string;
    /**
     * Wick colour for this bar alone, over `color`. A study that paints candles
     * with a translucent body and a solid wick cannot say so with one colour,
     * and without this a per-bar override always set the two together. Read by
     * the candle renderers only; absent means the wick follows `color`.
     */
    wickColor?: string;
    /** Border colour for this bar alone, on the same terms as `wickColor`. */
    borderColor?: string;
}
/** A single value point (for line/area/baseline series). */
interface LinePoint {
    time: UTCSeconds;
    value: number;
    /** Per-bar colour override; carried through to the Bar. */
    color?: string;
}
/** A whitespace point: occupies a logical index for alignment but draws nothing. */
interface Whitespace {
    time: UTCSeconds;
}
declare function isWhitespace(p: Bar | LinePoint | Whitespace): p is Whitespace;
/** Any item a series accepts: an OHLC bar, a value point, or a whitespace gap. */
type SeriesDataItem = Bar | LinePoint | Whitespace;
/**
 * Normalize any series data item into an internal OHLC bar:
 * - a `Bar` passes through untouched;
 * - a `LinePoint` `{ time, value }` becomes a flat OHLC bar (open=high=low=close=value);
 * - a `Whitespace` `{ time }` becomes a NaN bar, which the line renderer draws as a
 *   gap and autoscale skips.
 */
declare function toBar(item: SeriesDataItem): Bar;

/**
 * One band of a price-dependent tick schedule. Valid prices inside a band are
 * whole multiples of its tick, counted from zero rather than from `from`.
 */
interface TickBand {
    /**
     * Lowest price the band covers, inclusive; it ends where the next band
     * starts. The first band has none: it covers every lower price, zero and
     * negative prices included. A floor is a price limit, not a tick rule.
     */
    readonly from?: number;
    readonly tick: number;
}
/**
 * A validated, ordered tick schedule: the increment an instrument trades in
 * at each price. Every boundary must be a multiple of the ticks on both of its
 * sides, so the boundary is itself a valid price, rounding within a band is
 * also the nearest valid price overall, and no step ever skips a boundary.
 */
declare class TickSchedule {
    readonly bands: readonly TickBand[];
    /**
     * The finest grid every scheduled price lies on: the greatest common divisor
     * of the ticks. It is the smallest tick whenever the ticks nest, and finer
     * when they do not, because 20.05 on a 0.05 band is no multiple of 0.02.
     */
    readonly minMove: number;
    private readonly _digits;
    /** Validated like any host metadata: a plain JSON list is enough. */
    constructor(bands: readonly TickBand[]);
    /** The band a price falls in; an exact boundary belongs to the band it starts. */
    private _band;
    private _at;
    /** The tick at `price`, NaN when it is not finite. */
    tickAt(price: number): number;
    /**
     * The nearest valid price, NaN when `price` is not finite. A price halfway
     * between two valid prices, as written in decimal, rounds toward positive
     * infinity.
     */
    round(price: number): number;
    /**
     * The valid price `ticks` whole ticks from `price` (rounded first). Up from a
     * boundary uses the band that starts there, down from it the band below.
     */
    step(price: number, ticks: number): number;
}

/**
 * Chart theme (palette). A single object drives chart chrome (background, grid,
 * axes, crosshair), series defaults (up/down, line, area gradient, last price),
 * and the trade layer (buy/sell, profit/loss). Renderers read theme colors when
 * a per-series style field is absent, so one theme restyles the whole chart.
 */
interface ChartTheme {
    background: string;
    grid: string;
    axisText: string;
    axisLine: string;
    /** 1px rule between stacked panes. Subtle: it separates, it does not divide. */
    paneSeparator: string;
    crosshair: string;
    /** Axis label font size in px (default 11). */
    axisFontSize?: number;
    /** Grid line dash style (default 'solid'). */
    gridStyle?: 'solid' | 'dashed' | 'dotted';
    /** Crosshair line dash style (default 'dashed'). */
    crosshairStyle?: 'solid' | 'dashed' | 'dotted';
    /** Crosshair line width in device px (default 1 = hairline). */
    crosshairWidth?: number;
    /** Background of the crosshair value tags (defaults to `crosshair`). */
    crosshairLabelBackground?: string;
    /** Show the crosshair price/time value tags (default true). */
    crosshairLabelVisible?: boolean;
    upColor: string;
    downColor: string;
    wickUpColor: string;
    wickDownColor: string;
    lineColor: string;
    areaTopColor: string;
    areaBottomColor: string;
    baselineTopLine: string;
    baselineTopFill: string;
    baselineBottomLine: string;
    baselineBottomFill: string;
    lastPriceUp: string;
    lastPriceDown: string;
    lastPriceText: string;
    buy: string;
    sell: string;
    profit: string;
    loss: string;
}
declare const darkTheme: ChartTheme;
declare const lightTheme: ChartTheme;
declare const DEFAULT_THEME: ChartTheme;

/**
 * Primitive / plugin API (ARCHITECTURE.md §8). The extension point that keeps
 * the core small and powers markers, events, indicators, and the trade layer.
 * A primitive draws on a pane, optionally contributes to autoscale, and
 * optionally hit-tests for hover/drag.
 */

type ZOrder = 'bottom' | 'normal' | 'top';
interface PrimitiveRenderContext {
    timeScale: TimeScale;
    /** Explicitly bound scale, or the pane's right scale for an unbound primitive. */
    priceScale: PriceScale;
    /** The pane's primary visible price series scale, including a moved left axis. */
    readoutPriceScale?: PriceScale;
    dataLayer: DataLayer;
    plotWidth: number;
    plotHeight: number;
    /** Width of the bound scale's axis column; zero for hidden scales. */
    priceAxisWidth: number;
    /** Axis placement for price labels. Absent retains the default right axis. */
    priceAxisSide?: 'left' | 'right' | 'hidden';
    /**
     * Plot-relative media-px x of the bound column's inner edge. Defaults to 0 on
     * the left and plotWidth on the right. An explicit offset confines axis tags
     * to this column without shifting the primitive's plot coordinates.
     */
    priceAxisOffset?: number;
    dpr: number;
    theme: ChartTheme;
    /**
     * The pane's primary price series, for a primitive that needs what price
     * actually did rather than just the scales: a forecast scoring itself, say.
     * Lazy, so nothing pays for it unless asked. Absent on synthetic contexts.
     */
    bars?: () => readonly Bar[];
    /** externalId of the primitive hit under the pointer (hover state), if any. */
    hoverId?: string | null;
    /** Optional subtarget identity; leaves the primitive's external click ID unchanged. */
    hoverKey?: string | null;
    /** externalId of the line being dragged (active state), if any. */
    dragId?: string | null;
}
interface PrimitiveHit {
    externalId: string;
    /** Distinguishes hover regions which share one external click ID. Omission uses externalId. */
    hoverKey?: string;
    zOrder: ZOrder;
    /**
     * Pixel distance from the cursor (smaller wins ties before z-order). Never
     * negative: zero is a hit on the shape itself, and the pane stops asking
     * once the front band has one.
     */
    distance: number;
    cursor?: string;
    /** Coordinate scale for a bound primitive's drag prices. Unbound hits omit it. */
    priceScale?: PriceScale;
    /**
     * Arm a drag on press. Price lines set `cursor: 'ns-resize'` and move on one
     * axis; anything that moves on **both** (a drawing anchor, a whole shape)
     * declares it here, and the drag callbacks receive time as well as price.
     */
    draggable?: boolean;
    /** Opt into Escape cancellation without a release. Listen for `drag:cancel` to discard the preview. */
    cancelOnEscape?: boolean;
    /**
     * The primitive that paints what was hit, when it is not the one answering:
     * a drawing layer answers for the layers under it. The chart ranks the hit
     * by where that primitive paints, so a series painted over it takes the
     * context menu. Omission means the answering primitive painted it.
     */
    paintedBy?: IPrimitive;
}
/** Injected when a primitive is attached; lets it request a repaint. */
interface PrimitiveHost {
    requestUpdate(): void;
}
/**
 * Where chart furniture lives, as distinct from pane furniture.
 *
 * A price line belongs to a pane. A brand mark, a corner clock or a session
 * badge belongs to the CHART: it should sit at an edge of the whole stack, and
 * follow that edge as indicator panes come and go. `chart-bottom` is the common
 * case, and it also survives maximize, which hides the other panes entirely and
 * would otherwise take a price-pane watermark with it.
 *
 * `primary-pane` is the third kind: furniture that describes the price, such
 * as a symbol badge, follows the primary price pane to whatever slot it is
 * moved to, and the pane maximized over it while it is hidden.
 */
type PrimitiveAnchor = 'chart-top' | 'chart-bottom' | 'primary-pane';
/** Passed to `addPrimitive` instead of a pane index to anchor to the chart. */
interface PrimitivePlacement {
    anchor: PrimitiveAnchor;
}
interface IPrimitive {
    /** Layer order vs series: 'bottom' (behind), 'normal' (over), 'top' (overlay). */
    zOrder(): ZOrder;
    draw(ctx: CanvasRenderingContext2D, rc: PrimitiveRenderContext): void;
    /** Optional: expand the pane's autoscale range so this primitive isn't clipped. */
    autoscaleInfo?(): {
        min: number;
        max: number;
    } | null;
    /**
     * Optional: run once per frame after every scale on the pane has been
     * measured, and before anything is painted.
     *
     * `draw` is too late for anything that has to change a scale, because the
     * price axis is painted near the top of `paintBase` while primitives draw
     * further down: a range corrected in `draw` labels its axis one frame late,
     * and on a static chart that frame never comes. A comparison overlay lining
     * its scale up with the pane's own is the case this exists for.
     */
    afterAutoscale?(): void;
    /** Optional: topmost hit under (x,y) in media px (relative to the pane plot). */
    hitTest?(x: number, y: number, rc: PrimitiveRenderContext): PrimitiveHit | null;
    /**
     * Optional: the box, in the coordinates `hitTest` receives, outside which it
     * answers null, or null when nothing of this primitive can be hit at all.
     * Edges are inclusive, and an unbounded side may be infinite.
     *
     * A pointer move then asks `hitTest` only of primitives whose box it is in,
     * so a pane carrying hundreds of them pays for the few near the pointer.
     * The pane asks for the box once and reuses it while nothing it can follow
     * changes: the time scale and the bars' times, the pane's price scales, its
     * size and axes, the pixel ratio, the hover and drag state, and the
     * primitive's own state, which it announces through `requestUpdate`. A box
     * that follows anything else, such as bar prices, has to request an update
     * when that changes, or leave this out. Work the box out from the same
     * geometry `hitTest` answers from, `rc` or what the last `draw` recorded: a
     * box asked for between a change and the frame that paints it is asked for
     * again once that frame is painted. Absent, every pointer move asks
     * `hitTest`.
     */
    hitBounds?(rc: PrimitiveRenderContext): {
        left: number;
        top: number;
        right: number;
        bottom: number;
    } | null;
    attached?(host: PrimitiveHost): void;
    detached?(): void;
}
/** Pick the best hit across primitives: nearest distance, then z-order priority. */
declare function bestHit(hits: readonly (PrimitiveHit | null)[]): PrimitiveHit | null;

/**
 * Unified style bag for all Family-A series types (ARCHITECTURE.md §6A). Each
 * renderer reads the fields it needs; per-type defaults are filled by the
 * chart-type registry. Keeping one optional-field interface avoids a sprawling
 * discriminated union at the rendering boundary.
 */

interface SeriesStyle {
    upColor?: string;
    downColor?: string;
    borderUpColor?: string;
    borderDownColor?: string;
    wickUpColor?: string;
    wickDownColor?: string;
    borderVisible?: boolean;
    wickVisible?: boolean;
    /** Paint candle bodies. Off leaves outline and wick. Default true. */
    bodyVisible?: boolean;
    /**
     * Has no effect: no renderer reads it, and a `candlestick` series with it set
     * draws filled candles.
     *
     * @deprecated Removed in 3.0.0. Add the series as `'hollow-candle'` (or switch it with
     * `chart.setSeriesType`), which draws up candles as outlines and down candles filled.
     */
    hollow?: boolean;
    /**
     * Color bars by close-versus-previous-close instead of close-versus-own-open,
     * which is how most terminals paint a bar. Body, border and wick switch
     * together. Default false.
     */
    colorByPreviousClose?: boolean;
    /** Whether the series is drawn and counted in autoscale. Default true. */
    visible?: boolean;
    /** Optional label carried with the series (for host-drawn legends). */
    title?: string;
    /** Show the dashed horizontal last-price line across the plot. Default true. */
    priceLineVisible?: boolean;
    /**
     * Show this series' current value as a tag on the price axis. Default true.
     *
     * Each visible left or right scale labels its own series, including indicator
     * overlays, comparison lines and studies. Hidden scales have no tags. The
     * instrument's tag is the up/down coloured one that also carries the bar
     * countdown; the rest are drawn in their own plot colour. A series whose
     * current value is not a number draws no tag, so a study that is `na` right
     * now says nothing rather than showing a stale reading.
     */
    lastValueVisible?: boolean;
    /**
     * Decimal places for every price the scale this series maps to formats: the
     * axis ticks, the last-value tag, the crosshair label and the drawing-tool
     * labels. It overrides the precision the price scale infers from the tick
     * size or the visible range. Undefined is the "Default" entry of the
     * Precision dropdown: keep inferring. Valid range 0 to 8.
     *
     * It does not reach a legend row: a `PaneLegend` is handed finished strings,
     * so whoever builds those readings owns their formatting.
     */
    precision?: number;
    /**
     * Draw every point this many bars to the right of the bar it belongs to
     * (negative: left). The data is untouched and keeps its own times; only the
     * painted position moves, so the last `barOffset` points of a series land in
     * the right margin, past the newest bar. Autoscale follows what is drawn in
     * view. Default 0.
     */
    barOffset?: number;
    color?: string;
    lineWidth?: number;
    /** Line dash style for line/step/area/HLC series. Default 'solid'. */
    lineStyle?: 'solid' | 'dashed' | 'dotted';
    step?: boolean;
    markers?: boolean;
    /** Draw only the markers, with no connecting line (Parabolic SAR, scatter). */
    markersOnly?: boolean;
    markerRadius?: number;
    areaTopColor?: string;
    areaBottomColor?: string;
    baseValue?: number;
    topColor?: string;
    bottomColor?: string;
    /**
     * Stroke for the top edge of an HLC area's band. Undefined leaves the edge
     * unstroked, which is how the band has always drawn.
     */
    highColor?: string;
    /** Stroke for the bottom edge of an HLC area's band. See `highColor`. */
    lowColor?: string;
    closeColor?: string;
    base?: number;
    /**
     * Fallback box size for stacking P&F X/O glyphs. Columns from
     * `PointFigureTransform` carry their own `boxSize`, which wins, so set this
     * only for hand-built column data.
     */
    boxSize?: number;
    /** Kagi thick (yang) line color. */
    thickColor?: string;
    /** Kagi thin (yin) line color. */
    thinColor?: string;
}

/**
 * Axis label rendering (ARCHITECTURE.md §6, §5.3). Price axis (right strip) and
 * time axis (bottom strip). Time labels switch from clock to date at a day
 * boundary in the chart's configured zone, IST by default; gaps are already
 * collapsed by the logical-index time scale.
 *
 * Past the tick ladders, the file also owns the chrome that lives on the axis
 * strips: the countdown to the next bar close inside the last-price tag, the
 * crosshair's date pill, the corner session clock, and the priority rule that
 * decides which price-axis label survives a pile-up. All of it is opt-in, and an
 * axis nobody configured draws what it always drew.
 */

/**
 * Boundary class of a time-axis label, passed to a custom `timeFormatter` as a
 * hint so a host can render adaptive labels (year at year boundaries, month at
 * month boundaries, day otherwise, clock intraday): parity with common
 * `tickMarkFormatter(time, tickMarkType)` APIs.
 */
type TickMarkType = 'year' | 'month' | 'day' | 'time' | 'timeWithSeconds';
interface AxisStyle {
    textColor: string;
    lineColor: string;
    font: string;
}
/**
 * A clock reading UTC seconds (fractional allowed).
 *
 * Live chrome takes one of these rather than calling `Date.now()` itself: a test
 * has to hold time still, replay runs on the time of the bar being replayed, and
 * a terminal that trusts the exchange's clock over the browser's has to be able
 * to substitute its own. A renderer that reached for the global clock could
 * serve none of the three.
 */
type ClockSource = () => number;
/** The countdown row inside the last-price tag. Absent or `visible: false` draws no row. */
interface BarCountdownOptions {
    /** Draw the row. Unset or false leaves the tag exactly one line, as before. */
    visible?: boolean;
    /** UTC seconds now. Never `Date.now()` reached for here, see `ClockSource`. */
    now: ClockSource;
    /** Open time (UTC seconds) of the last bar on the chart. */
    lastBarTime: number;
    /** Bar interval in seconds, e.g. from `medianBarInterval`. */
    intervalSec: number;
}
/** The corner session clock. Absent or `visible: false` leaves the corner empty. */
interface SessionClockOptions {
    /** Draw the clock. Unset or false draws nothing at all. */
    visible?: boolean;
    /** UTC seconds now. Never `Date.now()` reached for here, see `ClockSource`. */
    now: ClockSource;
    /** IANA zone the clock reads in; unset means the shipped default. */
    timezone?: string;
    /** Second row carrying the zone's offset from UTC. Default true. */
    showOffset?: boolean | undefined;
}

/**
 * The Canvas tab's crosshair controls. Same precedence as the rest of the
 * canvas block (see grid.ts): set overrides the theme, unset falls through.
 */
interface CrosshairOptions {
    /** Line colour. Unset falls back to `theme.crosshair`. */
    color?: string;
    /** Line dash. Unset falls back to `theme.crosshairStyle`, then dashed. */
    style?: CanvasLineStyle;
    /** Line width in device px (1 = hairline). Unset falls back to the theme. */
    width?: number;
}
/** Resolved crosshair line style, ready for `drawCrosshair`. */
interface CrosshairStyle {
    color: string;
    width: number;
    dash: number[];
}
/** Fold the crosshair options over the theme. */
declare function resolveCrosshairStyle(theme: Pick<ChartTheme, 'crosshair' | 'crosshairStyle' | 'crosshairWidth'>, opts: CrosshairOptions | undefined, dpr: number): CrosshairStyle;

/** Dash style shared by the grid, crosshair and any other chrome line. */
type CanvasLineStyle = 'solid' | 'dashed' | 'dotted';
/**
 * Dash pattern in device px for a line style. `solid` (and an unset style)
 * yields an empty array, which is what `setLineDash` wants for a plain line.
 */
declare function dashPattern(style: CanvasLineStyle | undefined, dpr: number): number[];
interface GridOptions {
    /** Target spacing between grid lines, in media px. */
    spacing: number;
    /** Draw the vertical (time) lines. Default true. */
    vertLines?: boolean;
    /** Draw the horizontal (price) lines. Default true. */
    horzLines?: boolean;
    /** Vertical line colour. Unset falls back to `theme.grid`. */
    vertColor?: string;
    /** Horizontal line colour. Unset falls back to `theme.grid`. */
    horzColor?: string;
    /** Vertical line dash. Unset falls back to `theme.gridStyle`, then solid. */
    vertStyle?: CanvasLineStyle;
    /** Horizontal line dash. Unset falls back to `theme.gridStyle`, then solid. */
    horzStyle?: CanvasLineStyle;
    /** Line width in media px. Default 1. */
    lineWidth?: number;
}
/** Colour + dash for one axis of the grid. */
interface GridAxisStyle {
    color?: string;
    /** Dash pattern (device px already applied by the caller's dpr). */
    dash?: number[];
}
interface GridStyle {
    color: string;
    lineWidth: number;
    /** Optional dash pattern (device px already applied by the caller's dpr). */
    dash?: number[];
    /** Vertical-line overrides; each field falls back to the flat pair above. */
    vert?: GridAxisStyle;
    /** Horizontal-line overrides; each field falls back to the flat pair above. */
    horz?: GridAxisStyle;
}
/**
 * Fold the canvas grid options over the theme into the style `drawGrid` takes.
 * `theme.grid` / `theme.gridStyle` are the defaults for both axes; a per-axis
 * option replaces one of them without disturbing the other.
 */
declare function resolveGridStyle(theme: Pick<ChartTheme, 'grid' | 'gridStyle'>, opts: Partial<GridOptions> | undefined, dpr: number): GridStyle;
/** The "Scales: Text" and "Scales: Lines" controls. */
interface ScaleCanvasOptions {
    /** Tick label colour. Unset falls back to `theme.axisText`. */
    textColor?: string;
    /** Tick label size in px, clamped to the dialog's 10..14 range. */
    fontSize?: number;
    /** Axis line and tick colour. Unset falls back to `theme.axisLine`. */
    lineColor?: string;
}
/** Bounds of the "Scales: Text" size control. */
declare const SCALE_FONT_MIN = 10;
declare const SCALE_FONT_MAX = 14;
/**
 * Fold the scale options over the theme into the axis renderer's style.
 *
 * Only the *option* is clamped to 10..14: a theme is code, not a dialog, and a
 * host that set `axisFontSize: 16` deliberately keeps it.
 */
declare function resolveScaleStyle(theme: Pick<ChartTheme, 'axisText' | 'axisLine' | 'axisFontSize'>, opts: ScaleCanvasOptions | undefined): AxisStyle;
/** Plot-area margins, in percent of pane height (the dialog's units). */
interface PlotMarginOptions {
    top?: number;
    bottom?: number;
}
/**
 * Percent -> the fraction `PriceScaleOptions.marginTop/marginBottom` already
 * store. There is no second margin state: the price scale stays the owner, this
 * only converts the dialog's units. Each side is capped at 49% so the pair
 * always leaves a band for the data (the scale itself only guards the sum with
 * a 1% floor, which would squash the plot to a line).
 */
declare function resolvePlotMargins(opts: PlotMarginOptions | undefined): {
    marginTop?: number;
    marginBottom?: number;
};
/**
 * Everything the settings dialog's Canvas tab can set, in one block.
 *
 * No watermark field: the mark is a primitive the host owns and configures
 * (`LogoWatermark`), so a switch here would be a stored value nothing reads.
 * Same rule as the settings schema itself, see model/chart-settings.ts.
 */
interface CanvasOptions {
    grid?: Partial<GridOptions>;
    crosshair?: CrosshairOptions;
    scales?: ScaleCanvasOptions;
    /** Plot-area margins in percent of pane height. */
    margins?: PlotMarginOptions;
}

/**
 * Price-level family, built on the primitive API (ARCHITECTURE.md §8). A price
 * level is a horizontal line at a meaningful price **and** a matching tag on the
 * price axis, and the two halves toggle independently.
 *
 * WHY one options group per level instead of a list of lines and a list of
 * labels: the two halves of a level are one idea. Reference terminals split them
 * across separate menus, so a level's line and its label drift apart and a user
 * ends up with an axis tag for a line that is not drawn. Here `line` and `label`
 * are two flags on the same group, which cannot come apart.
 *
 * WHY the session comes from the bar gaps and never from a calendar midnight:
 * the overnight break is the only thing in the data that says where a trading
 * day ends, and a fixed midnight lands mid-session for half the world's
 * exchanges. `sessionStartFlags` already reads it back (and falls back to the
 * calendar only when the series shows no readable break, which is the right
 * answer for daily bars). That bug was fixed in 1.2.0; this file does not
 * reintroduce it.
 *
 * WHY levels with no data draw nothing rather than drawing at zero: extended
 * hours need a phase the historical feed does not carry, and bid/ask need a live
 * quote it does not carry either. A missing value is `null`, never `0`, so the
 * level stays inert and a host can render its control disabled with the state
 * still visible (`available(kind)`), which is what the UI standard asks for.
 *
 * WHY there is no `autoscaleInfo`: a previous close a gap away from the session
 * in view would stretch the range and flatten the bars the user is actually
 * looking at. A reference level that scrolls off the top is the lesser harm, and
 * it is what the last-price line already does.
 */

type PriceLevelKind = 'previousClose' | 'sessionHigh' | 'sessionLow' | 'lastPrice' | 'preMarketOpen' | 'preMarketClose' | 'postMarketOpen' | 'postMarketClose' | 'bid' | 'ask';
/** Every level, in the order a settings panel should list them. */
declare const PRICE_LEVEL_KINDS: readonly PriceLevelKind[];
/** One level's appearance. `line` and `label` are the two independent toggles. */
interface PriceLevelStyle {
    /** Draw the horizontal line across the plot. */
    line: boolean;
    /** Draw the matching tag on the price axis. */
    label: boolean;
    /** Line and tag colour. Unset falls through to the theme (see `defaultColor`). */
    color?: string;
    /** Line width in media px. Default 1. */
    lineWidth?: number;
    /** Line dash. Default per kind. */
    lineStyle?: CanvasLineStyle;
    /** Axis tag text. Default: the price, formatted by the scale. */
    text?: string;
    /**
     * Fraction of the plot width the line spans, measured from the price axis.
     * 1 = full width (default), the same meaning `PriceLine` gives it.
     */
    extentFromRight?: number;
}
/**
 * Which part of the trading day a bar belongs to. Only a host knows this: the
 * OHLC feed carries no phase, and guessing it from the clock would be wrong for
 * every exchange but the one guessed for.
 */
type MarketPhase = 'pre' | 'regular' | 'post';
/** Classify a bar into a market phase; null/undefined means "unknown". */
type MarketPhaseFn = (bar: Bar) => MarketPhase | null | undefined;
/** A live top-of-book quote. Either side may be absent. */
interface PriceLevelQuote {
    bid?: number | null;
    ask?: number | null;
}
interface PriceLevelsOptions {
    /** Per-level overrides, merged over the defaults. */
    levels?: Partial<Record<PriceLevelKind, Partial<PriceLevelStyle>>>;
    /** IANA zone for the calendar fallback when the bars show no session break. */
    timezone?: string;
    /** Extended-hours classifier. Absent leaves the four extended levels inert. */
    marketPhase?: MarketPhaseFn | null;
    /** Live quote, as a value or a callback read each frame. Absent leaves bid/ask inert. */
    quote?: PriceLevelQuote | (() => PriceLevelQuote | null | undefined) | null;
}
/** Each level's price for the current context, or null when it has no data. */
type PriceLevelValues = Record<PriceLevelKind, number | null>;
interface PriceLevelInput {
    bars: readonly Bar[];
    /**
     * UTC seconds at the right edge of the viewport: the session containing it is
     * "the session in view", and the level before it is its previous session.
     * Default (and anything past the last bar) anchors on the last bar.
     */
    anchorTime?: number;
    timezone?: string;
    marketPhase?: MarketPhaseFn | null;
    quote?: PriceLevelQuote | null;
}
/**
 * Every level's price for one viewport, or null where the data cannot say.
 *
 * Pure, so the levels are testable without a canvas, and callable by a host that
 * wants the numbers for a readout rather than for a line.
 */
declare function computePriceLevels(input: PriceLevelInput): PriceLevelValues;
/**
 * The last-price level as the series style already expresses it.
 *
 * That level is not duplicated here: `priceLineVisible` and `lastValueVisible`
 * are exactly this family's `line` and `label` under older names, and the pane
 * draws them. These two functions are the translation, so a host can present the
 * last price in the same shape as every other level and write the answer back to
 * the series it belongs to.
 */
declare function lastPriceLevelFromSeriesStyle(style: Pick<SeriesStyle, 'priceLineVisible' | 'lastValueVisible'>): Pick<PriceLevelStyle, 'line' | 'label'>;
/** The series-style patch a last-price level group means. */
declare function seriesStyleForLastPriceLevel(level: Pick<PriceLevelStyle, 'line' | 'label'>): Pick<SeriesStyle, 'priceLineVisible' | 'lastValueVisible'>;
declare class PriceLevels implements IPrimitive {
    private readonly _levels;
    private _timezone;
    private _phase;
    private _quote;
    private _values;
    private _host;
    constructor(options?: PriceLevelsOptions);
    attached(host: PrimitiveHost): void;
    detached(): void;
    zOrder(): ZOrder;
    /** One level's resolved style, for a host rendering its controls. */
    level(kind: PriceLevelKind): Readonly<PriceLevelStyle>;
    /** Patch one level. The two toggles move independently: neither implies the other. */
    setLevel(kind: PriceLevelKind, patch: Partial<PriceLevelStyle>): void;
    /** Patch several levels, plus the shared options, in one repaint. */
    setOptions(options: PriceLevelsOptions): void;
    /** Feed a live top-of-book quote; null clears it and the bid/ask levels go inert. */
    setQuote(quote: PriceLevelQuote | null): void;
    /**
     * Level prices as of the last frame. Values depend on the viewport, so they
     * are computed while drawing; before the first frame every level reads null.
     * A host that needs them without a chart calls {@link computePriceLevels}.
     */
    values(): Readonly<PriceLevelValues>;
    /**
     * Whether a level has data in the current context. False is the signal to
     * render its control disabled rather than to hide it: "no previous session
     * yet" is information, an absent checkbox is not.
     */
    available(kind: PriceLevelKind): boolean;
    draw(ctx: CanvasRenderingContext2D, rc: PrimitiveRenderContext): void;
}

/**
 * Which part of the trading day an instant falls in, and what a status readout
 * says about "now", read from a session calendar rather than from a clock.
 *
 * A calendar lays each local date out as windows of hours: the regular session,
 * the pre-open and post-close hours around it, extended hours of their own, and
 * a whole-date holiday. Where they overlap one wins by a fixed precedence, and
 * everything left over is closed. The answer is a partition of time into spans,
 * which is the one shape every reader needs: a phase is the span an instant
 * sits in, a status is that span and the one after it, and a shade paints the
 * spans in view without asking about every bar.
 */

/**
 * The part of the trading day an instant falls in. `regular` is the session
 * itself; `pre` and `post` are the hours the calendar trades before the first
 * opening and after the last close of a trading date; `extended` is extended
 * hours of their own, such as an overnight session; `holiday` is a date the
 * calendar's exceptions close although its weekly sessions would open; and
 * `closed` is every other hour.
 */
type SessionPhase = 'pre' | 'regular' | 'post' | 'extended' | 'closed' | 'holiday';
/** One stretch of time in a single phase. */
interface SessionPhaseSpan {
    readonly phase: SessionPhase;
    /** UTC seconds, inclusive. */
    readonly start: number;
    /** UTC seconds, exclusive. */
    readonly end: number;
}
/**
 * Anything that can lay its phases out over a range: `SessionCalendar`,
 * `Instrument`, or a host's own hours. The spans are consecutive, never
 * overlap, and cover the range from its start to its end.
 */
interface SessionPhaseSource {
    phaseSpans(fromUtcSeconds: number, toUtcSeconds: number): readonly SessionPhaseSpan[];
}
/** The market at one instant, as a status readout shows it. */
interface MarketStatus {
    /** The phase at the instant asked about. */
    readonly phase: SessionPhase;
    /**
     * When the phase next changes, in UTC seconds, or null when it holds for the
     * next 14 days: a round-the-clock calendar, or a long closure. A readout
     * that has to follow the market sets its timer for this instant.
     */
    readonly changesAt: number | null;
    /** The phase from `changesAt`, or null with it. */
    readonly nextPhase: SessionPhase | null;
    /**
     * The opening of the regular session in force, or else of the next one, in
     * UTC seconds. Null when none opens within 14 days, or when the session in
     * force has run for more than two days, as a round-the-clock one does.
     */
    readonly opensAt: number | null;
    /** The close of that same session, or null when it is not within 14 days. */
    readonly closesAt: number | null;
}
/**
 * The market status at an instant, from anything that lays out phases:
 * `SessionCalendar`, `Instrument`, or the calendar a chart was given, read
 * back from `chart.dataLayer.sessionCalendar`. Null when the source is absent
 * or cannot lay out phases, such as a host's hours with `sessionFrom` alone.
 * Pass `Date.now() / 1000` for "now". A calendar with a window whose boundary
 * a daylight-saving change removes throws, as its `sessionAt` does on that
 * date. A status lays out the days around the instant, so it throws from
 * about fifteen days before that date to four after, and a readout catches it.
 */
declare function marketStatusAt(source: {
    phaseSpans?: SessionPhaseSource['phaseSpans'];
} | null | undefined, utcSeconds: number): MarketStatus | null;
/**
 * A `PriceLevels` phase classifier from a calendar, so the four extended-hours
 * levels need no host code: a bar in the pre-open reads `pre`, and so on.
 * Extended hours of their own, closed hours and holidays read as unknown,
 * since those levels describe the hours either side of one session.
 *
 * The levels ask about every bar of a session on every frame, so the spans of
 * about a week are kept and walked in time order, and the calendar is asked
 * again only when a bar falls outside them. A calendar that cannot answer
 * leaves the phase unknown rather than stopping the frame.
 */
declare function calendarMarketPhase(source: SessionPhaseSource): MarketPhaseFn;

interface InstrumentCalendar {
    /** HHMM-HHMM[:days], with opening weekdays 1 (Sunday) through 7. */
    readonly sessions: readonly string[];
    /** Local opening dates replace weekly sessions. An empty list closes that date. */
    readonly exceptions?: Readonly<Record<string, readonly string[]>>;
    /**
     * Whole minutes of pre-open trading before each trading date's first
     * regular opening, 0 to 1439. Counted from the opening, so a shortened day,
     * a special session and a closed date move it with them.
     */
    readonly preMarketMinutes?: number;
    /** Whole minutes of post-close trading after each trading date's last regular close, 0 to 1439. */
    readonly postMarketMinutes?: number;
    /**
     * Extended hours of their own, such as an overnight session, written like
     * `sessions`. They open on their own days, except on a date the exceptions
     * close.
     */
    readonly extendedHours?: readonly string[];
}
/** Host-supplied market rules, separate from observations and saved user preferences. */
interface InstrumentMetadata {
    readonly symbol: string;
    readonly exchange: string;
    readonly timezone: string;
    /** The minimum move: with `tickBands`, the schedule's common grid. */
    readonly priceTick: number;
    readonly pricePrecision: number;
    /**
     * Price-dependent ticks, supplied by the host from its venue's rules. Absent
     * means one tick at every price, which is `priceTick`.
     */
    readonly tickBands?: readonly TickBand[];
    /** In the order adapter's units. This is a grid, never a lot conversion factor. */
    readonly quantityStep: number;
    readonly intervals: readonly string[];
    readonly calendar: InstrumentCalendar;
    readonly hasOpenInterest?: boolean;
}
interface InstrumentSession {
    /** Local opening date, including for an overnight window. */
    readonly date: string;
    /** UTC seconds, inclusive. */
    readonly open: number;
    /** UTC seconds, exclusive. */
    readonly close: number;
}
/**
 * Trading hours on their own: the calendar part of {@link InstrumentMetadata}
 * with the zone its times are written in, for a host that knows a venue's
 * hours but has no tick or quantity rules to supply.
 */
interface SessionCalendarSpec extends InstrumentCalendar {
    /** IANA zone the session windows and exception dates are read in. */
    readonly timezone: string;
}
/**
 * Validated, detached trading hours: the calendar an {@link Instrument}
 * carries, without the price and quantity rules. Apply one to a chart so
 * times past the last bar follow the venue's sessions, and read it with the
 * same `sessionAt` and `sessionFrom`.
 */
declare class SessionCalendar {
    /** The validated IANA zone. */
    readonly timezone: string;
    /** A frozen copy of the windows and exceptions, detached from the input. */
    readonly calendar: InstrumentCalendar;
    private readonly _hours;
    constructor(input: unknown);
    /** The window active at an instant. Closed dates and breaks return null, never inferred hours. */
    sessionAt(utcSeconds: number): InstrumentSession | null;
    /**
     * The window active at an instant, or else the next one to open, looking at
     * most about a year ahead. Null when nothing opens in that time.
     */
    sessionFrom(utcSeconds: number): InstrumentSession | null;
    /** The part of the trading day an instant falls in. */
    phaseAt(utcSeconds: number): SessionPhase;
    /**
     * The phases from one instant to another, as consecutive spans clipped to
     * the range, which may be at most 400 days long.
     */
    phaseSpans(fromUtcSeconds: number, toUtcSeconds: number): readonly SessionPhaseSpan[];
    /** The market status at an instant: pass `Date.now() / 1000` for now. */
    marketStatusAt(utcSeconds: number): MarketStatus;
    /**
     * Lay the chart's time axis past the last bar out in these hours, and
     * repaint, so a drawing already placed there moves to the time it now
     * means: `chart.setSessionCalendar(this)`, after checking the chart is
     * alive. `chart.dataLayer.setSessionCalendar` sets the same without asking
     * for a frame, for a host about to load bars or move the view anyway.
     */
    applyTo(chart: Chart): void;
}
/** Validated, detached rules. Construction does not change global intervals or chart defaults. */
declare class Instrument {
    readonly metadata: InstrumentMetadata;
    /**
     * The validated `tickBands`, or null for a constant tick. A constant tick
     * has no schedule, so it keeps a single snapping rule, `priceTick`, on every
     * path: the order constraints, chart drags and anything the host builds.
     */
    readonly tickSchedule: TickSchedule | null;
    private readonly _hours;
    constructor(input: unknown);
    /** Provider tokens are exact: a feed may distinguish monthly M from minute m. */
    supportsInterval(code: string): boolean;
    /** Formats display only. The source bars retain their unrounded values. */
    formatPrice(value: number): string;
    /** Resolve an active window. Closed dates and breaks return null, never inferred hours. */
    sessionAt(utcSeconds: number): InstrumentSession | null;
    /**
     * The window active at an instant, or else the next one to open, looking at
     * most about a year ahead. Null when nothing opens in that time.
     */
    sessionFrom(utcSeconds: number): InstrumentSession | null;
    /** The part of the trading day an instant falls in. */
    phaseAt(utcSeconds: number): SessionPhase;
    /**
     * The phases from one instant to another, as consecutive spans clipped to
     * the range, which may be at most 400 days long.
     */
    phaseSpans(fromUtcSeconds: number, toUtcSeconds: number): readonly SessionPhaseSpan[];
    /** The market status at an instant: pass `Date.now() / 1000` for now. */
    marketStatusAt(utcSeconds: number): MarketStatus;
    /** The host clears old bars and owns source loading; only metadata is applied here. */
    applyTo(chart: Chart, interval: string): void;
}

/**
 * Shared data layer (ARCHITECTURE.md §4.1). One per chart. Merges all series by
 * time onto a single logical-index space (0..N-1) so price + volume + indicator
 * panes stay aligned, and so non-trading gaps collapse (an absent time simply
 * has no logical index). Per-series rows are addressable by that shared index.
 */

type SeriesId = number;
/**
 * Trading hours the time axis follows past the last bar. `Instrument` and
 * `SessionCalendar` both satisfy it.
 */
interface SessionCalendarSource {
    /** The window active at `utcSeconds`, or else the next one to open; null when none opens. */
    sessionFrom(utcSeconds: number): InstrumentSession | null;
    /**
     * Optional: the phases of the trading day over a range, which session
     * shading and `marketStatusAt` read from the chart's calendar. The axis
     * never calls it. `Instrument` and `SessionCalendar` provide it.
     */
    phaseSpans?(fromUtcSeconds: number, toUtcSeconds: number): readonly SessionPhaseSpan[];
}
interface IndexedBar {
    index: number;
    bar: Bar;
}
declare class DataLayer {
    private readonly _series;
    private _sortedTimes;
    private readonly _indexByTime;
    /** How many series hold each time on the axis; a time leaves the axis at zero. */
    private readonly _timeRefs;
    private _nextId;
    /** Bumped whenever the time axis changes, so the future plan knows to look again. */
    private _version;
    private _calendar;
    private _future;
    /**
     * Trading hours for the time axis to the right of the last bar, or null for
     * none. With a calendar, the bar after a session's last one is the next
     * session's first, across a night, a weekend or a closed date, so an anchor
     * placed there lands on a time the market will print. Without one, or when
     * the recent bars do not sit in its sessions, the axis continues at the
     * median of the recent bar spacing.
     *
     * This changes where times fall but asks for no frame: the data layer has
     * no way to reach the chart, so a drawing already placed past the last bar
     * stays where it was painted until something repaints. `chart.setSessionCalendar`
     * sets the same and repaints, and so do `SessionCalendar.applyTo` and
     * `Instrument.applyTo`, which go through it; call this directly only where
     * bars or a view change follow anyway.
     */
    setSessionCalendar(calendar: SessionCalendarSource | null): void;
    /** The calendar the time axis follows past the last bar, or null. */
    get sessionCalendar(): SessionCalendarSource | null;
    /** Register a new series; returns its id. */
    createSeries(): SeriesId;
    removeSeries(id: SeriesId): void;
    /**
     * Bulk-load (full replace) one series' data. Input is sorted and
     * de-duplicated by time by a private `sortedUniqueByTime`.
     *
     * The shared index is rebuilt only when the set of times changes, and then
     * only where it changed. Most whole writes do not change the set: a study's
     * plots carry the source's own times, which the source put in the index
     * before any study recomputed, and a colour overlay on the source keeps its
     * times. Ten studies on a long history used to pay for the index two dozen
     * times per tick while it came out unchanged.
     */
    setSeriesData(id: SeriesId, bars: readonly Bar[]): void;
    /**
     * Upsert bars into a series by time (used for history paging / backfill /
     * out-of-order corrections, ARCHITECTURE.md §4.2). Existing times are
     * replaced; new times are inserted; the result stays time-sorted.
     *
     * Prepending older bars shifts every existing logical index up by the
     * inserted count: callers preserve the viewport by re-reading `baseIndex`
     * (the invariant `rightEdge − index` is unchanged, so visible bars don't move).
     */
    addBars(id: SeriesId, bars: readonly Bar[]): void;
    /**
     * Apply a single live bar (ARCHITECTURE.md §4.2 hot path). Returns the kind of
     * change so the chart auto-scrolls only on a genuine right-edge append:
     * - `'append'`: newer than the last bar (advances baseIndex)
     * - `'replace'`: same time as the last bar (intra-bar tick) or an existing time
     * - `'insert'`: an older time inserted into history (late / out-of-order)
     */
    update(id: SeriesId, bar: Bar): 'append' | 'replace' | 'insert';
    private _appendTime;
    /** Number of logical indices (distinct time points across all series). */
    get length(): number;
    /** Logical index of the latest real bar (length - 1), or -1 if empty. */
    get baseIndex(): number;
    indexToTime(index: number): number | undefined;
    timeToIndex(time: number): number | undefined;
    /**
     * Fractional logical index to UTC seconds, interpolating between bars and
     * extrapolating past either edge.
     *
     * `indexToTime` only answers for indices that have a bar. Anything anchored to
     * an arbitrary x (a drawing endpoint, a cursor readout, a projection to the
     * right of the last bar) needs a time for positions *between* bars too, which
     * the gapless axis (§5.3) makes common: everything a weekend or a session
     * break collapsed away lands there. Returns NaN when there is no data.
     *
     * Past the last bar the whole indices are the bar times still to come: from
     * the session calendar when one is set (`setSessionCalendar`), otherwise one
     * median recent spacing apart. Never the last gap alone, which is the gap most
     * likely to be a night or a weekend. Left of the first bar the first gap
     * applies.
     */
    indexToTimeFloat(index: number): number;
    /** UTC seconds to fractional logical index. The inverse of `indexToTimeFloat`. */
    timeToIndexFloat(time: number): number;
    /**
     * The future plan for the current last bars and calendar. A rebuild that
     * leaves the recent bars as they were (a page of older history, a gap filled
     * well to the left) keeps the plan and the times it already generated.
     */
    private _plan;
    /**
     * A series' bars, time-sorted, with no per-call allocation: the read path
     * for anything that recomputes over full history (indicators, transforms).
     * The array is live: treat it as read-only.
     */
    seriesBars(id: SeriesId): readonly Bar[];
    /** All bars of a series paired with their shared logical index. */
    indexedBars(id: SeriesId): IndexedBar[];
    /**
     * Bars of a series whose logical index lies within [fromIndex, toIndex].
     * Binary-searches the (time-sorted) series into the visible time window instead
     * of scanning all bars, so a full repaint costs O(log n + visible) per series,
     * not O(total bars) - the hot path called for autoscale and drawing every frame.
     */
    visibleBars(id: SeriesId, fromIndex: number, toIndex: number): IndexedBar[];
    /** The last bar of a series with its shared logical index, in O(1). */
    lastIndexedBar(id: SeriesId): IndexedBar | null;
    /** Count one more series holding `time`; true when no series held it before. */
    private _retain;
    /**
     * Swap a series' bars and bring the time axis along. The counts say which
     * times entered or left the union, so the work is the series' own change,
     * never every series the chart holds: re-sending the same times touches no
     * index at all, times past the right edge are appended, and anything else
     * is merged in by `_rebuild`.
     */
    private _replaceBars;
    /**
     * Merge `added` (sorted, new to the axis) into it and drop the times no
     * series holds any longer. Indices before the first change keep their value,
     * so a change near the right edge reindexes only what moved. A time that
     * does not order (NaN) cannot be merged, and the union is sorted afresh.
     */
    private _rebuild;
}

/**
 * Chart-type registry (ARCHITECTURE.md §6A). Every series type registers a
 * descriptor: how to draw it and how it contributes to autoscale. The core
 * iterates descriptors, so adding a style is one registration and no core change.
 * The time-indexed types register here; the transform tier registers the ones
 * it draws from its own elements (Point & Figure, Kagi).
 */

type SeriesType = 'candlestick' | 'hollow-candle' | 'volume-candle' | 'bar' | 'high-low' | 'line' | 'line-markers' | 'step' | 'area' | 'hlc-area' | 'baseline' | 'column' | 'histogram' | 'point-figure' | 'kagi';
interface DrawItem {
    x: number;
    bar: Bar;
    /**
     * Close of the bar left of the visible range, carried on the first item only.
     * Previous-close candle colouring is the one renderer that reads it, because
     * it is the one that needs a reference the visible window does not hold.
     */
    prevClose?: number;
    /**
     * Set only on the neighbour a `connectsBars` renderer gets beyond each edge
     * of the view: the x (media px) of the plot edge between it and the bars in
     * view, 0 on the left and the plot width on the right. That neighbour can
     * sit millions of pixels off screen (a level held for a session on an axis
     * of five-second bars), so a renderer cuts the segment to it at this x,
     * plus whatever its stroke needs, rather than draw it all the way out, and
     * paints nothing of the neighbour's own (a marker, a body).
     */
    edgeX?: number;
}
interface SeriesRenderContext {
    plotHeight: number;
    maxVolume: number;
    theme: ChartTheme;
}
interface RendererEntry {
    defaultStyle: SeriesStyle;
    /** True for the price series whose last close drives the last-price line. */
    isPriceSeries: boolean;
    /**
     * True for a renderer that draws a segment from each bar to the next (a
     * line, a step, an area, kagi). Its items then carry the nearest bar beyond
     * each edge of the view as well, marked with `edgeX`, so the segment that
     * crosses an edge is drawn: a sparse line (a level held for an hour, a
     * position that steps twice a session) would otherwise lose the part that
     * enters from off screen. Such a renderer cuts that segment at `edgeX`.
     */
    connectsBars?: boolean;
    /**
     * Paint the items on a bitmap-scope 2D context: the bars in view, and for a
     * `connectsBars` renderer the neighbour beyond each edge (`DrawItem.edgeX`).
     * On screen the pane reaches this through its render backend
     * (src/render/backend.ts); the 2D backend calls it as is, and the vector
     * export calls it directly on the serialising target.
     */
    draw(ctx: CanvasRenderingContext2D, items: readonly DrawItem[], toY: (v: number) => number, barSpacing: number, dpr: number, style: SeriesStyle, rc: SeriesRenderContext): void;
    /** Min/max price contribution of one bar to autoscale. */
    extents(bar: Bar, style: SeriesStyle): {
        min: number;
        max: number;
    };
}
/** Register a chart type. `type` accepts a built-in `SeriesType` or any custom string. */
declare function registerChartType(type: SeriesType | (string & {}), entry: RendererEntry): void;
declare function getChartType(type: SeriesType | (string & {})): RendererEntry;
declare function registeredChartTypes(): string[];

/**
 * Series markers (ARCHITECTURE.md §8.1): buy/sell signals and shapes anchored
 * to bars. Visible-range culled, per-bar stacked, four discrete sizes.
 *
 * Markers that carry text and sit above or below the bar (or on a pane edge)
 * are laid out in lanes: a label is wider than a bar, so on a whipsaw the
 * signals on neighbouring bars landed on each other. Each one is pushed
 * outward just past any earlier one on its side whose box it overlaps, and
 * one that overlaps nothing stays where it always was. A label moves at most
 * a few of its own heights: where labels outnumber the room, as at a wide
 * zoom, the rest overlap on their bars rather than stack out of the pane.
 */

/**
 * `labelUp` / `labelDown` are text plates with a tail, for named signals ("Buy",
 * "Sell") rather than bare glyphs. The tail points *at* the anchor price and the
 * body sits clear of it: `labelUp`'s tail points up so its body hangs below the
 * anchor, `labelDown` is the mirror. Both require `text`.
 */
type MarkerShape = 'arrowUp' | 'arrowDown' | 'circle' | 'square' | 'triangleUp' | 'triangleDown' | 'diamond' | 'flag' | 'text' | 'labelUp' | 'labelDown' | 'cross' | 'xcross';
/**
 * `paneTop` and `paneBottom` pin the glyph to the edge of the plot rather than
 * to a price, so a squeeze dot or a session flag sits in a fixed row whatever
 * the scale does. They need no bar under them and no `price`.
 */
type MarkerPosition = 'aboveBar' | 'belowBar' | 'inBar' | 'atPrice' | 'paneTop' | 'paneBottom';
type MarkerSize = 'tiny' | 'small' | 'medium' | 'big';
interface SeriesMarker {
    time: number;
    position: MarkerPosition;
    price?: number;
    shape: MarkerShape;
    size: MarkerSize;
    color: string;
    text?: string;
    /** Overrides marker-matching text or contrasting label text without changing the glyph fill. */
    textColor?: string;
    /** Positive finite CSS pixels. Defaults to max(9, the size preset), independently of bar spacing. */
    fontSize?: number;
    /** CSS font-family list. Defaults to system-ui, sans-serif. */
    fontFamily?: string;
    /** Label plates default to semibold; other marker text defaults to normal. */
    bold?: boolean;
    italic?: boolean;
    /** Multiline row alignment inside the centered text block. Defaults to center. */
    textAlign?: 'left' | 'center' | 'right';
    id?: string;
}
/** Base glyph size in CSS px for a marker size preset. */
declare function markerSizePx(size: MarkerSize): number;
/** Effective glyph px, clamped so it never exceeds the current bar spacing. */
declare function effectiveMarkerPx(size: MarkerSize, barSpacing: number): number;
declare function drawShape(ctx: CanvasRenderingContext2D, shape: MarkerShape, cx: number, cy: number, px: number, color: string): void;
/**
 * A signal label: rounded plate, contrasting text, and a tail that points at
 * `anchorY`. All coordinates are bitmap px: the caller has already applied dpr.
 * `up` puts the tail on the top edge and the body below the anchor.
 *
 * `text` may carry `\n`: the plate widens to the longest row and grows
 * downward-and-upward about its own centre, so it stays centred on `cx` and the
 * tail keeps meeting the anchor price.
 */
declare function drawLabel(ctx: CanvasRenderingContext2D, up: boolean, cx: number, anchorY: number, text: string, color: string, fontPx: number): void;
declare class SeriesMarkers implements IPrimitive {
    private readonly _seriesId;
    private readonly _fallbackBars;
    private readonly _priceScale;
    private _markers;
    private _host;
    private _lastPositions;
    /** The widest row of each text mark measured so far, for this marker set and `_widthDpr`. */
    private _widths;
    private _widthDpr;
    /** The same widths by font and string, kept across marker sets. */
    private readonly _fontWidths;
    private _anyStyled;
    /**
     * @param seriesId The series whose pane and price scale the marks live on.
     * @param fallbackBars Bars to position against where that series has none.
     *
     * The second argument exists because a marker's series decides *where* it is
     * drawn while the bar under it decides *how high*, and those are not always
     * the same row of data. An indicator that draws one line in an uptrend and
     * another in a downtrend has a gap in each, and a mark that lands in a gap
     * had no bar to measure from and was dropped without a word. The caller
     * passes the instrument's own bars, which have no gaps.
     */
    constructor(seriesId: SeriesId, fallbackBars?: () => readonly Bar[], priceScale?: () => PriceScale);
    attached(host: PrimitiveHost): void;
    detached(): void;
    zOrder(): ZOrder;
    setMarkers(markers: readonly SeriesMarker[]): void;
    /**
     * The widest row of a mark's text in its own font, measured only for marks a
     * paint lays out, never the whole history. A study sets its marks again on
     * every recompute, with new objects and the same few strings, so a width is
     * also kept by font and string: setting the font is the costly part.
     */
    private _textWidth;
    draw(ctx: CanvasRenderingContext2D, rc: PrimitiveRenderContext): void;
    hitTest(x: number, y: number): PrimitiveHit | null;
}

/**
 * Series records (ARCHITECTURE.md §4.3). A series references its rows in the
 * shared DataLayer by id, names a registered chart type, and carries a style
 * bag. The chart-type registry (§6A) supplies the renderer + autoscale extents,
 * so the core never switches on type.
 */

/**
 * Which price axis a series maps to. 'right' (default) and 'left' each draw an
 * axis and autoscale independently; '' is a hidden overlay scale (no axis, its
 * own autoscale) used to pin a volume histogram inside the price pane.
 * `overlay:name` creates an independent scale, hidden by default and shared only
 * by series using that same name on this pane. Chart.setPriceAxisPlacement can
 * display an overlay or change any scale's side without changing its identity.
 */
type PriceScaleId = 'right' | 'left' | '' | `overlay:${string}`;
/**
 * Value formatting for a price scale (its axis labels and crosshair tag):
 * `price` (tick-size precision), `volume` (compact 1.2K / 3.4M / 5.6B),
 * `percent` (a `%` suffix at a fixed precision), or a `custom` formatter.
 *
 * It lives here rather than beside `addSeries` because an indicator plot names
 * one too, and the model tier cannot reach into the core.
 *
 * `percent` suffixes the value as it stands and does **not** scale it: a study
 * that already returns 0..100 reads `62.24%`, and one that returns a 0..1
 * fraction reads `0.62%`. Scaling here would put the axis and the plotted value
 * into disagreement, which is the one thing a formatter must never do.
 */
type PriceFormat = {
    type: 'price';
    precision?: number;
    minMove?: number;
} | {
    type: 'volume';
} | {
    type: 'percent';
    precision?: number;
} | {
    type: 'custom';
    formatter: (value: number) => string;
};
interface SeriesRecord {
    dataId: SeriesId;
    type: SeriesType;
    style: SeriesStyle;
    scaleId: PriceScaleId;
}
/**
 * Explicit state of the supplied bar, including count-driven bars. For
 * `setData`, describes the last bar. `auto` returns to interval/clock inference.
 * Empty datasets have no confirmation override.
 */
interface BarConfirmationOptions {
    confirmation?: 'auto' | 'forming' | 'confirmed';
}
/** Updates are live by default; historical corrections never count as live. */
interface SeriesUpdateOptions extends BarConfirmationOptions {
    source?: 'live' | 'history';
}
/** Source metadata offered by native charts and optionally by custom indicator hosts. */
interface SeriesDataState {
    /** Stable identity within the host; revisions belong to this source only. */
    readonly sourceId: number;
    readonly revision: number;
    /** Advances whenever a previously calculated history prefix may have changed. */
    readonly historyRevision: number;
    readonly provenance: 'history' | 'live' | 'replay';
    readonly change: 'reset' | 'prepend' | 'append' | 'replace' | 'correction';
    readonly confirmation?: 'forming' | 'confirmed';
    readonly confirmationSource?: 'provider' | 'replay';
}
/** Public handle returned by `chart.addSeries(...)`. */
interface SeriesApi {
    /**
     * Replace historical data. Accepts OHLC bars, `{ time, value }` points, or
     * `{ time }` gaps. Omitted confirmation clears any previous tail override.
     */
    setData(bars: readonly SeriesDataItem[], options?: BarConfirmationOptions): void;
    /** Merge older data (history paging); same item shapes as `setData`. */
    prependData(bars: readonly SeriesDataItem[]): void;
    /**
     * Update one item. Replacing or appending the tail is live by default;
     * older corrections are historical. Same item shapes as `setData`.
     * Confirmation applies only when the supplied item is the tail; metadata
     * on older corrections cannot alter tail confirmation. An omitted override
     * survives same-tail writes, but clears when a new tail is appended.
     */
    update(bar: SeriesDataItem, options?: SeriesUpdateOptions): void;
    /** Current bars for this series (sorted old -> new, normalized to OHLC). Handy for computing the next live update. */
    getData(): Bar[];
    /** Merge a partial style into the series and repaint (recolor, `{ visible:false }` to hide, ...). */
    applyOptions(style: Partial<SeriesStyle>): void;
    /** Remove the series from its pane and free its data rows. */
    remove(): void;
    /** The price scale this series maps to (call `.setOptions({ marginTop, marginBottom })` on it). */
    priceScale(): PriceScale;
    /**
     * Create a markers layer (buy/sell signals, shapes) bound to this series.
     *
     * `fallbackBars` positions a mark whose time this series has no point for,
     * which happens whenever the series is drawn with gaps. Without it such a
     * mark is dropped silently.
     */
    createMarkers(fallbackBars?: () => readonly Bar[]): SeriesMarkers;
}

/**
 * Data variants: which of a provider's series for one instrument a chart shows.
 *
 * Regular and extended session, adjusted and raw prices, and a quote currency
 * or unit are each a different series from the provider, not a view of one
 * series. Extended hours add bars that the regular series does not have, and
 * an adjusted history rewrites every price before a corporate action. So a
 * variant is identity, like the symbol: it is part of every request, every
 * cache key and every saved chart, and a change of variant is a change of
 * source.
 *
 * Nothing here converts, adjusts or filters bars. A variant is served by the
 * provider that declares it (`DataFeed.dataVariants`) or it is unsupported;
 * there is no local fallback, because a series derived here would look exactly
 * like the provider's own and be wrong in ways nobody could see.
 */

/** Regular trading hours only, or with the provider's pre and post market sessions. */
type DataSession = 'regular' | 'extended';
/** Prices adjusted by the provider for corporate actions, or as traded. */
type DataAdjustment = 'adjusted' | 'raw';
/** One field of a variant, as a snapshot or an error names it. */
type DataVariantDimension = 'session' | 'adjustment' | 'currency' | 'unit';
/**
 * A provider series for one instrument. An absent field is the provider's
 * default for it, which is itself an identity: `{}` asks for exactly what a
 * request without a variant always got, and `{ session: 'regular' }` asks for
 * regular hours explicitly, whatever the provider's default is.
 */
interface DataVariant {
    session?: DataSession;
    adjustment?: DataAdjustment;
    /** The currency the provider quotes the series in, as it names it (`'USD'`). Compared exactly. */
    currency?: string;
    /** The unit the provider quotes the series in, as it names it (`'per lot'`). Compared exactly. */
    unit?: string;
}
/**
 * What a provider can serve for one instrument and interval. A dimension that
 * is absent or empty serves only the provider's default, so a variant must
 * leave it out.
 */
interface DataVariantCapabilities {
    sessions?: readonly DataSession[];
    adjustments?: readonly DataAdjustment[];
    currencies?: readonly string[];
    units?: readonly string[];
}
/** What `DataFeed.dataVariants` is asked about. */
interface DataVariantQuery {
    symbol: string;
    exchange: string;
    interval: string;
    signal?: AbortSignal;
}
/**
 * Validate a variant and return a frozen copy that holds only the fields it
 * names, always in the same order, or undefined for the provider's default.
 * Throws a TypeError for anything else: an unknown session or adjustment, a
 * blank or overlong currency or unit, or a field this build does not know,
 * which could only be a variant from a newer build that this one would
 * silently serve wrong.
 */
declare function normalizeDataVariant(input: unknown): Readonly<DataVariant> | undefined;
/**
 * One string per variant, for keys and comparisons: empty for the provider's
 * default, so a key that appends it is byte-identical to the key before
 * variants existed. Field order in the input does not matter.
 */
declare function dataVariantKey(variant?: DataVariant): string;
/**
 * The first field of `variant` the provider did not declare, or null when it
 * can serve all of it. The default variant is always served. Without a
 * declaration only the default is: a provider that never said it serves
 * extended hours cannot be trusted to have honoured the request, and a
 * regular series labelled extended is worse than no chart.
 */
declare function unsupportedDataVariant(capabilities: DataVariantCapabilities | undefined, variant?: DataVariant): DataVariantDimension | null;
/** The error a provider or the loading controller raises for an undeclared variant. */
declare function dataVariantError(dimension: DataVariantDimension, variant?: DataVariant): Error;
/** The two chart members `publishDataContext` needs. */
interface DataContextTarget {
    getDataContext(): Readonly<ChartDataContext> | undefined;
    setDataContext(context: ChartDataContext | undefined): void;
}
/**
 * Give a chart a data context whose variant counts as part of its source.
 *
 * `Chart.setDataContext` treats a change of symbol, exchange, interval or
 * variant as a new source: it aborts requested bars, restarts source revisions
 * and tells studies to read the context again. For a `Chart` this is therefore
 * `setDataContext` with the variant normalized, and `data:context` fires once.
 *
 * A target that does not compare the variant, such as a host's own
 * `DataContextTarget`, would ignore a context that differs only there and go
 * on reporting the old one. For it this passes through a context with a
 * different interval first, which the target does treat as a source change,
 * then sets the real one. The instrument never changes on the way, so event
 * markers, linked drawings and news stay with it. On that detour
 * `data:context` fires twice: once for the passing context, whose interval is
 * cleared, then for the real one. That context is marked, and the library's
 * requested studies, Tier 2 studies and alert controller wait for the real one
 * rather than fetch, report or save for it. A host's own listener sees both.
 */
declare function publishDataContext(chart: DataContextTarget, context: ChartDataContext | undefined): void;

/**
 * Study policies: what a user may do with a study. A host that places a study
 * of its own (a strategy's signal line, a session band, a study a desk
 * mandates) needs restrictions the user cannot lift, one at a time, while the
 * host itself keeps changing the study. It mirrors the drawing policy: each
 * flag defaults to true, so a study without a policy behaves as it always has.
 */
/**
 * `false` restricts the matching user action. Every native call that performs
 * one treats the caller as the user unless it passes `{ force: true }`
 * ({@link IndicatorEditOptions}); a restore always applies.
 */
interface IndicatorPolicy {
    /** `false`: no legend close button, inventory remove or plain `remove` call takes it off the chart. */
    removable?: boolean;
    /** `false`: no settings button, settings dialog, `setSettings` or scale assignment changes it. */
    configurable?: boolean;
    /** `false`: it keeps its pane and its place in the pane's stack. Other objects still move past it. */
    movable?: boolean;
    /** `false`: left out of the object inventory (`ChartObjects`) and every panel built on it. */
    listed?: boolean;
}
/** `force: true` marks a call as the owning host's, which a study's policy does not restrict. */
interface IndicatorEditOptions {
    force?: boolean;
}
/**
 * Validate a policy and keep only its restrictions, so an unrestricted study
 * saves nothing and two policies that mean the same compare equal. A flag
 * that is not a boolean throws; a key this build does not know is dropped,
 * the way the drawing migration keeps only its own flags.
 */
declare function parseIndicatorPolicy(input: unknown): IndicatorPolicy;

/**
 * Horizontal price line primitive (ARCHITECTURE.md §8). The reusable base for
 * order/SL/TP/alert/indicator-level lines: a line across the plot plus a fixed
 * price-axis tag and an optional broker-style segmented pill group on the
 * line ([badge][qty][label][close]) with hover / dragging states (the chart
 * passes `hoverId`/`dragId` on the render context) and a drag ghost at the
 * pre-drag price via `setDragGhost`. Interaction semantics are unchanged from
 * the classic tag: the close segment hit-tests as `${id}::close`, everything else drags.
 */

interface PriceLineOptions {
    price: number;
    color: string;
    /** Line thickness in media px. Default 1. */
    lineWidth?: number;
    /**
     * Two-state shorthand for `lineStyle`: `true` draws `'dashed'`. A retained
     * form rather than a deprecated one (COMPATIBILITY.md lists why); reach for
     * `lineStyle` when a line needs `'dotted'`.
     */
    dashed?: boolean;
    /**
     * Dash style, the three-way form of `dashed`. Set, it wins over the boolean;
     * unset, the boolean still decides, so a line built before this existed draws
     * exactly as it did.
     */
    lineStyle?: CanvasLineStyle;
    /** Price-axis tag text. Defaults to the formatted price. Hidden scales omit the tag. */
    label?: string;
    /** Solid colored badge segment at the start of the pill group (e.g. 'BUY', 'TP', 'SL'). */
    badge?: string;
    /** Quantity segment rendered as a neutral box after the badge. */
    qty?: string | number;
    /** Info text segment (order type, price, P&L ...): the classic left tag text. */
    leftLabel?: string;
    /**
     * Fraction of the plot width the line spans, measured from the right (price)
     * axis. 1 = full width (default); 0.3 = only the rightmost 30%, like a
     * partial-width order line. Visible scales also carry a price tag.
     */
    extentFromRight?: number;
    /** Draw a close (cancel) segment at the end of the pill group; hit-tests as `${id}::close`. */
    closeButton?: boolean;
    /** Stable id returned by hit-test (for click/drag routing). */
    id: string;
    /** Cursor hint when hovered (e.g. 'ns-resize' for draggable lines). */
    cursor?: string;
}
declare class PriceLine implements IPrimitive {
    private _opts;
    private _host;
    private _ghostPrice;
    /** Pill-group geometry from the last draw (media px) for hit-testing. */
    private _group;
    constructor(opts: PriceLineOptions);
    attached(host: PrimitiveHost): void;
    detached(): void;
    get price(): number;
    /** Move the line; schedules a repaint via the host. */
    setPrice(price: number): void;
    /** Update the info segment text (e.g. live position P&L); repaints. */
    setLeftLabel(text: string): void;
    /**
     * Restyle in place; repaints. `id` is the hit-test handle the chart routes
     * clicks and drags through, so it is not patchable: swapping it under a
     * live drag would strand the gesture.
     *
     * A last-price line is the case this exists for: it has to follow the tick
     * direction, and only `setPrice` was updatable, so the colour was stuck at
     * whatever it was constructed with.
     */
    setOptions(patch: Partial<Omit<PriceLineOptions, 'id'>>): void;
    /**
     * Show a dimmed reference line at the pre-drag price while the user drags
     * (pass the original price on drag start, null on drag end to clear).
     */
    setDragGhost(price: number | null): void;
    options(): Readonly<PriceLineOptions>;
    zOrder(): ZOrder;
    autoscaleInfo(): {
        min: number;
        max: number;
    } | null;
    draw(ctx: CanvasRenderingContext2D, rc: PrimitiveRenderContext): void;
    hitTest(x: number, y: number, rc: PrimitiveRenderContext): PrimitiveHit | null;
}

/**
 * Pane legend (ARCHITECTURE.md §8): the row at the top-left
 * of a pane: a color swatch, the source's name, its parameters, the value under
 * the crosshair, and inline action buttons before the changing readings.
 *
 * Drawn on the canvas rather than in the DOM, like `BuySellButtons` and
 * `DomLadder`, so it composites into screenshots and costs no DOM per pane.
 * Buttons hit-test as `${id}::close`, `${id}::hide`, and `${id}::settings`, so
 * the host routes them through the same `subscribeClick` path as order pills.
 *
 * Rows stack: several legends on one pane offset each other vertically, which
 * the host does by giving each a `row` index.
 *
 * This row is also the chart's status line, so `statusLine` carries the
 * per-field switches a settings dialog expects (logo, title, market status,
 * chart values, bar change, volume, last day change, background). Every switch
 * defaults to the behaviour that predates it, so a caller that passes none sees
 * the row it always saw. Fields the primitive cannot compute (a logo bitmap,
 * whether the market is open, the change since yesterday's close) arrive
 * through `status`; with no source they draw nothing at all rather than a
 * placeholder.
 */

type PaneLegendAction = 'hide' | 'settings' | 'source' | 'up' | 'down' | 'collapse' | 'maximize' | 'close';
/**
 * One reading on a legend row. Multi-plot sources show one per plot, each in
 * that plot's own color (an MA ribbon's four averages, MACD's three lines):
 * a single string in a single color cannot say which number is which.
 */
interface LegendValue {
    /** Dimmed prefix, e.g. `O` / `H` / `Vol`. */
    label?: string;
    text: string;
    /** Defaults to the row's `valueColor`, then `color`, then the theme text. */
    color?: string;
    /**
     * Which status-line switch owns this reading. Untagged readings are the
     * source's own last value, governed by `statusLine.lastValueLabel`.
     */
    field?: LegendField;
    /** Higher values retain this whole reading on narrow rows. Series readings default to 1; status metadata ranks lower. */
    priority?: number;
}
/**
 * Status-line groups a host can feed and switch off independently. The legend
 * never derives these: it tags what the host hands it, so one switch hides one
 * group and leaves the rest of the row alone.
 */
type LegendField = 'ohlc' | 'change' | 'volume' | 'openInterest';
/**
 * Which name the title shows. `description` and `ticker` come from `status`;
 * with neither supplied the title falls back to `title`, which always exists.
 */
type LegendTitleMode = 'symbol' | 'description' | 'ticker';
/**
 * The parts of the status line the primitive has no way to know. The host
 * supplies what it has; anything missing is simply not drawn.
 */
interface LegendStatusData {
    /** Already-decoded logo (an `<img>`, an `ImageBitmap`, a canvas). */
    logo?: CanvasImageSource;
    /** Long name for `titleMode: 'description'`, e.g. `Apple Inc.`. */
    description?: string;
    /** Exchange ticker for `titleMode: 'ticker'`, e.g. `NASDAQ:AAPL`. */
    ticker?: string;
    /** Session state, e.g. `{ text: 'Market open', color: '#26a69a' }`. */
    marketStatus?: LegendValue;
    /** Change against the previous close, e.g. `{ text: '+1.20 (+0.75%)' }`. */
    lastDayChange?: LegendValue;
}
/**
 * A snapshot, or a getter the legend calls each frame, so live fields (market
 * status, day change) can change without the host patching options at tick
 * speed. Returning `null` means "nothing to show".
 */
type LegendStatusSource = LegendStatusData | (() => LegendStatusData | null);
/**
 * Per-field switches for the status line. Existing reading fields default to
 * on; open interest and the background plate default to off. A missing reading
 * draws nothing whether its switch is on or off.
 */
interface LegendStatusLineOptions {
    /** Symbol logo, when `status` supplies one. */
    logo?: boolean;
    /** The bold name. Also the "name label" switch for an indicator row. */
    title?: boolean;
    /** Which name the title shows. Default `symbol`. */
    titleMode?: LegendTitleMode;
    /** Session state from `status.marketStatus`. */
    marketStatus?: boolean;
    /** The OHLC readout: readings tagged `field: 'ohlc'`. */
    chartValues?: boolean;
    /** Change over the hovered bar: readings tagged `field: 'change'`. */
    barChange?: boolean;
    /** Readings tagged `field: 'volume'`. */
    volume?: boolean;
    /** Readings tagged `field: 'openInterest'`. Off by default. */
    openInterest?: boolean;
    /** Change since the previous close, from `status.lastDayChange`. */
    lastDayChange?: boolean;
    /**
     * The source's own reading (untagged values). This is the scales-and-lines
     * "last value label" control, which lands here because the legend is what
     * draws that number on this row.
     */
    lastValueLabel?: boolean;
    /**
     * Plate behind the row's text, for legibility over candles. Off by default:
     * the row has never had one, and turning it on is a deliberate choice.
     */
    background?: boolean;
    /** Plate opacity, 0..1. Default 0.8, matching the hover plate. */
    backgroundOpacity?: number;
    /** Plate color. Defaults to the theme background. */
    backgroundColor?: string;
}
interface PaneLegendOptions {
    /** Explicit false suppresses OI without discarding its saved switch. Set by an owning chart. */
    hasOpenInterest?: boolean;
    /** Stable id; buttons hit-test as `${id}::close` etc. */
    id: string;
    /** Bold source name, e.g. `RSI`. */
    title: string;
    /** Dimmed parameter summary after the title, e.g. `14 close`. */
    params?: string;
    /** Swatch color; omitted draws no swatch. */
    color?: string;
    /**
     * Color for the live value. Defaults to `color`, then the theme's text, so a
     * row can tint its reading (an up/down change) without being forced to show a
     * swatch in that same color.
     */
    valueColor?: string;
    /** Vertical slot on the pane (0 = topmost). */
    row?: number;
    /**
     * Which inline action buttons to draw, left to right. Each hit-tests as
     * `${id}::<action>`:
     *  - `up` / `down`: move this pane one slot (`::up` / `::down`)
     *  - `hide`: toggle visibility (`::hide`)
     *  - `settings`: open this source's settings (`::settings`)
     *  - `source`: show the code this source was written from (`::source`)
     *  - `collapse`: fold this pane to a header strip, or open it again (`::collapse`)
     *  - `maximize`: expand this pane to fill the chart (`::maximize`)
     *  - `close`: remove the source, and its pane if it empties (`::close`)
     *
     * When only some actions fit, the end of this list stays visible.
     * Defaults to `['hide', 'settings', 'close']`. A chart drawing a study's row
     * adds the pane's own controls to it: on the first row of a pane other than
     * the price pane it puts `up`, `down`, `collapse` and `maximize` before
     * `close` (last when the row has none), and it leaves out `close` or
     * `settings` where the study's policy refuses them.
     */
    actions?: readonly PaneLegendAction[];
    /** Rendered as hidden (dimmed, eye hollow). */
    hidden?: boolean;
    /** False removes this row and its hit areas. Independent of the dimmed study visibility state. */
    visible?: boolean;
    /** Rendered as maximized (the maximize glyph becomes restore). */
    maximized?: boolean;
    /** Rendered as collapsed (the collapse glyph points the other way). */
    collapsed?: boolean;
    /** Text size in media px. Default 11. */
    font?: number;
    /**
     * Square side of one action button in media px. Default 16.
     *
     * The row grows to hold it, so raising this moves every legend row below it
     * down by the same amount and nothing overlaps. It is a chart-wide setting
     * for that reason: two legends on one pane with different button sizes would
     * stack against different row heights and collide.
     */
    iconSize?: number;
    /** Left inset from the plot edge in media px. Default 8. */
    left?: number;
    /** Top inset in media px. Default 6. */
    top?: number;
    /** Per-field status-line switches. Patching merges field by field. */
    statusLine?: LegendStatusLineOptions;
    /** Host-supplied status-line data (logo, names, market state, day change). */
    status?: LegendStatusSource;
}
declare class PaneLegend implements IPrimitive {
    private _opts;
    private _host;
    private _values;
    /** Button geometry from the last draw, in media px, for hit-testing. */
    private _buttons;
    /** Right edge of the drawn row, in media px. */
    private _width;
    private _plotWidth;
    private _plotHeight;
    private _suppressed;
    constructor(opts: PaneLegendOptions);
    attached(host: PrimitiveHost): void;
    detached(): void;
    zOrder(): ZOrder;
    autoscaleInfo(): null;
    /** @internal Chart display policy, kept separate from the row's own options. */
    setSuppressed(on: boolean): void;
    private _clearGeometry;
    /** A single live reading after the params (typically crosshair-driven). */
    setValue(text: string, color?: string): void;
    /** One reading per plot, each in its own color. */
    setValues(values: readonly LegendValue[]): void;
    setOptions(patch: Partial<PaneLegendOptions>): void;
    /** Resolve the host's status data for this frame; `{}` when it has none. */
    private _status;
    options(): PaneLegendOptions;
    draw(ctx: CanvasRenderingContext2D, rc: PrimitiveRenderContext): void;
    hitTest(x: number, y: number): PrimitiveHit | null;
}

/**
 * A grid overlay pinned to a corner of the pane rather than to bars.
 *
 * Seasonality heatmaps, performance summaries and signal scoreboards are all
 * the same shape: a small table of coloured cells that stays put while the
 * chart pans underneath. That makes this a screen-space primitive like the
 * watermark and the pane legend, not a series: it has no time anchor, takes no
 * part in autoscale, and survives a zoom untouched.
 */

type TablePosition = 'top-left' | 'top-center' | 'top-right' | 'middle-left' | 'middle-center' | 'middle-right' | 'bottom-left' | 'bottom-center' | 'bottom-right';
interface TableCell {
    /** Explicit newlines form a block with 1.2em line spacing. */
    text: string;
    /** Plain-text hover detail. Newlines are preserved; a merged cell uses its anchor's tooltip. */
    tooltip?: string;
    /** Cell fill. Transparent when omitted, so the pane shows through. */
    bgColor?: string;
    /** Text colour. Derived from `bgColor` for contrast when omitted. */
    textColor?: string;
    align?: 'left' | 'center' | 'right';
    /** Overrides the table's `fontSize` for this cell, for a heading row. */
    fontSize?: number;
    bold?: boolean;
    italic?: boolean;
    /** CSS font-family list. Defaults to `system-ui, sans-serif`. */
    fontFamily?: string;
    /** Aligns the text block within the cell, with 4px edge padding. Defaults to middle. */
    verticalAlign?: 'top' | 'middle' | 'bottom';
    /** Positive safe integer, default 1. The top-left cell supplies a merged cell's content. */
    colSpan?: number;
    /** Positive safe integer, default 1. Spans stay inside the existing rectangular grid. */
    rowSpan?: number;
}
interface ChartTableOptions {
    position: TablePosition;
    /** Gap from the pane edge, media px. */
    margin: number;
    /**
     * Column width in media px. A per-column array sizes each one separately.
     *
     * `'auto'` measures instead: every column takes the width of its own widest
     * cell at the size that cell will be drawn at. A grid whose cells are
     * sentences rather than numbers cannot be sized by a caller who has not
     * measured the text, and a fixed width there is a column of readings running
     * into the column beside it.
     */
    cellWidth: number | readonly number[] | 'auto';
    cellHeight: number;
    /**
     * Type size in media px, or `'auto'` to fit each cell: as large as its row
     * allows, shrunk until its text also fits its column. A stretched grid with
     * one long label would otherwise either clip that cell or be sized down as a
     * whole to suit it.
     */
    fontSize: number | 'auto';
    /** Grid line colour. Omit to draw no grid. */
    borderColor?: string;
    borderWidth: number;
    /** Outer frame, independent of cell borders. Omit to draw no frame. */
    frameColor?: string;
    /** Outer frame width in media px. Defaults to 1 when a frame color is supplied. */
    frameWidth?: number;
    /**
     * Table width as a percentage of the plot, 0 or omitted to size from
     * `cellWidth` instead. Column proportions are preserved, so a per-column
     * `cellWidth` array still controls the relative widths, and the percentage
     * only decides the total.
     */
    widthPercent?: number;
    /** Table height as a percentage of the plot, 0 or omitted to size from `cellHeight`. */
    heightPercent?: number;
    /**
     * Relative row heights, one per row, defaulting to 1. A separator row is the
     * reason this exists: stretched to fill a pane, an equal split makes a rule
     * between two sections as tall as the sections themselves.
     */
    rowWeights?: readonly number[];
    /** Backdrop behind the whole grid, drawn before the cells. */
    background?: string;
    /** Hit-test id, so a host can route clicks the way it does for other primitives. */
    id?: string;
}
declare const DEFAULT_CHART_TABLE_OPTIONS: ChartTableOptions;
/** Top-left corner of the grid for a position keyword, in media px. */
declare function tableOrigin(position: TablePosition, margin: number, w: number, h: number, plotW: number, plotH: number): {
    x: number;
    y: number;
};
declare class ChartTable implements IPrimitive {
    private _rows;
    private _grid;
    private _opts;
    private _host;
    /** Last drawn rect in media px, for hit-testing without recomputing layout. */
    private _rect;
    private readonly _hitId;
    private _revision;
    private _hits;
    private _size;
    constructor(options?: Partial<ChartTableOptions>);
    attached(host: PrimitiveHost): void;
    detached(): void;
    zOrder(): ZOrder;
    options(): Readonly<ChartTableOptions>;
    setOptions(patch: Partial<ChartTableOptions>): void;
    /** Replace the grid atomically. Ragged rows share the maximum column count; covered cells are ignored. */
    setRows(rows: readonly (readonly TableCell[])[]): void;
    rows(): readonly (readonly TableCell[])[];
    private _clearHits;
    private _sameSize;
    draw(ctx: CanvasRenderingContext2D, rc: PrimitiveRenderContext): void;
    private _drawTooltip;
    hitTest(x: number, y: number, rc?: PrimitiveRenderContext): PrimitiveHit | null;
}

/**
 * Shaded band between two indicator plots: the Ichimoku cloud, a Bollinger
 * channel, a Keltner envelope.
 *
 * A pair of lines is not the same picture as a filled region: the fill is what
 * makes "price is above the cloud" or "the cloud flipped" readable at a glance,
 * and which side is on top is itself the signal. So the band is drawn as two
 * runs (one where A leads, one where B does) and the crossings between them
 * are split at the exact intersection rather than at the nearest bar, or the
 * colours would bleed a bar past every flip.
 */

/**
 * Vertical gradient across the band, graded between two prices rather than
 * two pixel rows.
 *
 * The stops are anchored in PRICE, not in pixels, because that is what the
 * shading is claiming: a band graded from its 70 level down to its 30 means
 * those levels, and a viewport-relative gradient would slide off them the
 * moment the pane is panned or the scale re-fits.
 */
interface FillGradient {
    /** Price the top stop sits at. Omitted: the band's own highest value. */
    topValue?: number;
    /** Price the bottom stop sits at. Omitted: the band's own lowest value. */
    bottomValue?: number;
    topColor: string;
    bottomColor: string;
}
interface IndicatorFillOptions {
    /** Fill colour where the first series is above the second. */
    colorUp: string;
    /** Fill colour where the second is above the first. */
    colorDown: string;
    /** 0..1. Defaults to 0.12: a band must not drown the candles it sits behind. */
    opacity?: number;
    /**
     * Set to grade the band instead of flat-filling it, in place of
     * `colorUp`/`colorDown`. A point carrying its own `color` still overrides it.
     * Unset leaves the two-colour fill exactly as it was.
     */
    gradient?: FillGradient | undefined;
}
/** One bar's pair of values; `null` where either plot has no value yet. */
interface FillPoint {
    /** Logical index on the shared time axis (fractional is fine). */
    index: number;
    a: number | null;
    b: number | null;
    /**
     * Paints this bar onward in one colour, for a band shaded by something other
     * than which line leads: trend state, a regime, a third series. The run is
     * split at the bar where the colour changes, or at the preceding intersection
     * if the plots cross into that bar. Overrides both point and band gradients.
     */
    color?: string | undefined;
    /**
     * Price-anchored gradient starting at this bar, or at the preceding intersection
     * when the plots cross into it. Undefined uses the band's gradient.
     */
    gradient?: FillGradient | undefined;
}
declare class IndicatorFill implements IPrimitive {
    private _points;
    private _opts;
    private _host;
    private _visible;
    constructor(options: IndicatorFillOptions);
    attached(host: PrimitiveHost): void;
    detached(): void;
    /** Behind the series, so the lines and candles stay crisp on top of it. */
    zOrder(): ZOrder;
    /** The plots it spans already drive the scale; the fill must not widen it. */
    autoscaleInfo(): null;
    setPoints(points: readonly FillPoint[]): void;
    setOptions(patch: Partial<IndicatorFillOptions>): void;
    setVisible(on: boolean): void;
    draw(ctx: CanvasRenderingContext2D, rc: PrimitiveRenderContext): void;
}

/**
 * Which bars a study computes on, when its chart applies a transform
 * (`Chart.setSeriesTransform`).
 *
 * `'chart'`, the default, is the bars the chart draws: a Renko chart's bricks,
 * a Heikin Ashi chart's candles. That is what a study computed on before the
 * chart could transform, when a host fed it the elements, so a study nobody
 * sets reads what it always read. `'underlying'` is the bars the host feeds.
 * On a chart with no transform the two are the same bars.
 *
 * A study on the underlying bars still draws on the chart's elements, or its
 * points would put the host's times on the shared axis and scatter the bricks
 * (data-layer.ts). So its values are read at the source bar each element was
 * completed on, which is the value a trader could have read when that element
 * formed, and a mark it dates at a source bar moves to the element that bar
 * completed into. Heikin Ashi keeps one candle per bar at the bar's own time,
 * so there the values need no reading across at all.
 */

/** The bars a study computes on: the chart's (default), or the host's under a transform. */
type IndicatorBarSource = 'chart' | 'underlying';

/**
 * Indicator runtime (ARCHITECTURE.md §8). Turns an `IndicatorDescriptor` into
 * live chart objects: one series per plot, optional reference levels, an
 * optional fixed pane range, and recomputes them when the source data or the
 * settings change.
 *
 * It adds **no rendering code**. Every plot names a registered chart type, so
 * indicators draw through the same Family-A renderers as any other series.
 */

/** The slice of the chart the runtime needs. Keeps this module testable alone. */
interface IndicatorHost {
    validateIndicatorSettings?(id: string, descriptor: IndicatorDescriptor, settings: Readonly<IndicatorSettings>): void;
    studyOutput?(source: IndicatorStudySource): Readonly<IndicatorStudyOutput> | undefined;
    indicatorOutputChanged?(id: string, refresh: boolean): void;
    indicatorRecompute?(id: string, refresh: boolean, fallback: () => void): void;
    assignIndicatorScale?(id: string, series: readonly {
        api: SeriesApi;
        scaleId: PriceScaleId;
    }[], primitives: readonly {
        primitive: IPrimitive;
        scaleId: PriceScaleId;
    }[], commit: () => void): boolean;
    bindIndicatorPrimitiveScale?(primitive: IPrimitive, scaleId: PriceScaleId): void;
    setIndicatorRange?(id: string, paneIndex: number, scaleId: PriceScaleId, range: {
        min: number;
        max: number;
    } | null, series: readonly SeriesApi[]): void;
    /** Forget a disposed instance, including disposal through its public handle. */
    indicatorRemoved?(instanceId: string, failedOwnedPane?: number): void;
    /** Optional instrument identity and source-range notifications. */
    dataContext?(): Readonly<ChartDataContext> | undefined;
    subscribeDataChanges?(listener: (change: IndicatorDataChange) => void): () => void;
    /** Add the pane-legend row (name + inline up/down/hide/maximize/close). */
    addIndicatorLegend(opts: {
        id: string;
        title: string;
        params: string;
        color?: string;
        row: number;
        paneIndex: number;
        /** The descriptor's `hasSource`, so the row can offer a source button. */
        hasSource?: boolean;
    }): PaneLegend;
    removeIndicatorLegend(legend: PaneLegend): void;
    /** How many legends already sit on this pane, so rows stack. */
    legendRowsOn(paneIndex: number): number;
    /** The instrument's own series, for a descriptor anchoring marks to price. */
    primarySeries?(): SeriesApi | null;
    /**
     * A series for one plot. The study writes it with `setData` for a whole pass
     * and, from 2.5.8, with `update` for the points a live tick moved (older
     * points included), so a host's own series must implement both as documented.
     */
    addIndicatorSeries(type: string, paneIndex: number, style: Record<string, unknown> | undefined, priceScaleId: string | undefined, 
    /** Axis/crosshair formatting for the scale this plot maps to. */
    priceFormat?: PriceFormat): SeriesApi;
    setIndicatorSeriesType?(series: SeriesApi, type: string): boolean;
    /**
     * Add a reference level. One options object rather than seven positional
     * arguments: the list grew a width and a dash style in 1.7.1, and a call site
     * of seven bare values is where the next one gets passed in the wrong slot.
     */
    addIndicatorLevel(level: {
        price: number;
        color: string;
        /**
         * Kept for hosts predating `lineStyle`; always `lineStyle === 'dashed'`.
         *
         * @deprecated Removed in 3.0.0. Read `lineStyle` (since 1.7.1), which
         * also carries `'dotted'`. The instance resolves it before calling the
         * host, so it is never absent.
         */
        dashed: boolean;
        lineWidth: number;
        lineStyle: IndicatorLineStyle;
        label: string;
        id: string;
    }, paneIndex: number): PriceLine;
    removeIndicatorLevel(line: PriceLine): void;
    /** Attach a band drawn behind the plots (an Ichimoku cloud). */
    addIndicatorFill(fill: IndicatorFill, paneIndex: number): void;
    removeIndicatorFill(fill: IndicatorFill): void;
    /**
     * Detach a signal-marker layer. There is no matching `add`: the layer comes
     * from `series.createMarkers()` on a plot's own series, or on the instrument's
     * for marks sent to the candles, so it already lands in the right pane.
     * Removing a series does not remove its primitives, hence this.
     */
    removeIndicatorMarkers(markers: SeriesMarkers): void;
    /** Attach a corner-pinned summary grid to a pane, and detach it again. */
    addIndicatorTable(paneIndex: number): ChartTable;
    removeIndicatorTable(table: ChartTable): void;
    /**
     * Attach an arbitrary primitive to a pane, and detach it again. Carries both
     * the descriptor's drawing layer and whatever a Tier-2 `attach` lifecycle
     * wants to paint, so those two do not need a host method each.
     *
     * Optional, like `timezone`, so a host predating it still satisfies this
     * interface: an indicator that draws simply draws nothing there.
     */
    addIndicatorPrimitive?(primitive: IPrimitive, paneIndex: number): void;
    removeIndicatorPrimitive?(primitive: IPrimitive): void;
    /**
     * Recompute whatever the host has marked stale, before a caller reads a value.
     *
     * Optional, like `timezone`, so a host predating it still satisfies this
     * interface: one that recomputes eagerly has nothing to flush.
     */
    flushIndicators?(): void;
    /**
     * Restore the host's study stack after settings replace series or attached
     * visuals. Also called from inside a calculation pass when that pass first
     * creates a layer for an output target (see `IndicatorOutputTarget`), so the
     * layer takes its study's place among the targeted layers instead of landing
     * above every later study's. During that call the instances report their
     * targeted layers alone, so nothing else moves. It must only restack:
     * recomputing studies from it would run them in the middle of another
     * study's pass.
     */
    resourcesChanged?(): void;
    /** A study's policy changed: the chart redraws its legend buttons and the inventory. */
    policyChanged?(): void;
    /** Bars of the primary price series: the calculation input. */
    sourceBars(): readonly Bar[];
    /**
     * The host's bars under a transformed primary series and the source bar each
     * of `sourceBars` was completed on (null when they are one to one), or null
     * when the chart draws its bars as given. Optional; without it a study on the
     * underlying bars computes on `sourceBars`, and one on the chart's bars is
     * never told they are transformed (`IndicatorCalcContext.transformed`).
     */
    underlyingBars?(): {
        bars: readonly Bar[];
        sourceIndex: readonly number[] | null;
    } | null;
    /** Optional mutation metadata; absent hosts retain the legacy timestamp heuristic. */
    sourceState?(): SeriesDataState | undefined;
    /** Selected candle after native or linked hover; absent means latest. */
    legendIndex?(): number | undefined;
    /** Index of a fresh pane for an indicator that wants its own. */
    nextPaneIndex(): number;
    /**
     * Current slot of the chart's price pane, where an on-chart study and every
     * `overlay` plot, band, table and price-anchored mark draws. Read at each
     * use, since the price pane can be moved below the studies. Optional so a
     * host predating it still satisfies this interface; absent means slot 0.
     */
    primaryPaneIndex?(): number;
    /**
     * The chart's configured IANA zone. Optional so a host predating the option
     * still satisfies this interface; absent means the shipped default.
     *
     * A descriptor is handed bars and settings and never the chart, so this is
     * how the calendar an anchor resets on (a VWAP session, a seasonality month)
     * reaches the calculation. See `IndicatorInstance._descriptorSettings`.
     */
    timezone?(): string;
    /**
     * The instrument and timeframe on screen, when the host knows them. The
     * host can supply an explicit `dataContext` instead. Without either hook,
     * a descriptor sees `undefined` rather than a guessed identity.
     */
    symbol?(): string | undefined;
    interval?(): string | undefined;
    /** Chart wall clock in UTC seconds. Absent means the system clock. */
    now?(): number;
    /**
     * Publish an indicator's per-bar colours onto the **primary price series**,
     * or withdraw them with `null`. `owner` is the instance id: a host holds one
     * overlay at a time and only lets its current owner withdraw it, so a second
     * publisher taking over does not get cleared by the first one's teardown.
     *
     * Optional, like `timezone`: a host that does not implement it simply gives a
     * `barColors` descriptor nowhere to publish, and the indicator's own plots are
     * unaffected.
     */
    setBarColors?(colors: readonly (string | null)[] | null, owner: string): void;
    /** Emit on the chart's event bus (indicator alerts, and `attach`'s own events). */
    emit?(event: string, payload: unknown): void;
    /**
     * Bars of another instrument or interval, from wherever the host keeps its
     * history. Optional: without it the attach context's `requestBars` rejects,
     * which a study reads as "not available on this chart".
     */
    requestBars?(request: IndicatorBarsRequest): Promise<readonly Bar[]>;
    requestSnapshot?(request: IndicatorSnapshotRequest): Promise<RequestedBarsSnapshot>;
    requestState?(): Readonly<IndicatorRequestState>;
    subscribeRequestChanges?(listener: () => void): () => void;
    /**
     * Tick size of the named pane's price scale, or undefined when none is set.
     * Optional so a host predating it still satisfies this interface.
     *
     * Per pane, and the panes genuinely differ: a pane that does not quote the
     * instrument has no tick to report. The price pane (`primaryPaneIndex`) is
     * the one to ask for the instrument's own step.
     */
    tickSize?(paneIndex: number): number | undefined;
    /**
     * Write a number the way the price axis of that pane writes it.
     *
     * The legend sits inches from the axis and names the same quantity, so the
     * two disagreeing is the reading a user has to reconcile themselves. Deriving
     * the format here from a tick got that wrong twice over: a study pane carries
     * no tick at all, so a percentage read `0.618` beside an axis saying `0.62`,
     * and a price pane's tick alone misses the precision floor and the host's own
     * formatter, so a volume study read seven digits where its axis said `1.20M`.
     *
     * Asking the scale removes the second opinion. Optional so a host driving
     * this module alone still works, falling back to the magnitude ladder.
     */
    formatPrice?(paneIndex: number, value: number, series?: SeriesApi): string | undefined;
    /** Pin a pane's price scale to a fixed range, or release it with `null`. */
    setPaneRange(paneIndex: number, range: {
        min: number;
        max: number;
    } | null): void;
}
/** Public handle returned by `chart.addIndicator(...)`. */
interface IndicatorApi {
    /** Whole-study scale override, or null for descriptor assignments. */
    priceScaleId(): PriceScaleId | null;
    /**
     * Move local price resources together; null restores descriptor assignments.
     * False, and nothing changes, for a study whose policy is not `configurable`
     * unless `options.force` is set.
     */
    setPriceScale(scaleId: PriceScaleId | null, options?: IndicatorEditOptions): boolean;
    /** Effective scale for a declared plot, or null for an unknown key. */
    plotPriceScaleId(plotKey: string): PriceScaleId | null;
    /** Detached explicit per-plot assignments, before descriptor and study defaults. */
    plotPriceScaleIds(): Readonly<Record<string, PriceScaleId>>;
    /**
     * Atomically patch plot assignments; null clears an override. Invalid patches
     * return false, and so does a study that is not `configurable` unless forced.
     */
    setPlotPriceScales(assignments: Readonly<Record<string, PriceScaleId | null>>, options?: IndicatorEditOptions): boolean;
    /** The restrictions the host set, only the flags that are false. */
    policy(): Readonly<IndicatorPolicy>;
    /**
     * Replace the policy; null lifts every restriction. The host's act, never
     * restricted itself. Throws on a flag that is not a boolean.
     */
    setPolicy(policy: IndicatorPolicy | null): void;
    /** External data state, or null for a study without a managed lifecycle. */
    dataStatus(): Readonly<IndicatorDataStatus> | null;
    /** Observe changes; immediately receives the current managed status, if any. */
    subscribeDataStatus(listener: (status: Readonly<IndicatorDataStatus>) => void): () => void;
    /** Retry external history when the descriptor supplies a retry action. */
    retryData(): void;
    /** Unique instance id (several instances of one indicator can coexist). */
    readonly id: string;
    /** The descriptor id, e.g. `'macd'`. */
    readonly indicatorId: string;
    /** Display name. */
    readonly name: string;
    /** Pane the indicator drew into. */
    readonly paneIndex: number;
    /** Current settings (a copy). */
    settings(): IndicatorSettings;
    /**
     * Merge a settings patch, recompute, and restyle. False, and nothing changes,
     * when the study is removed, or is not `configurable` and `options.force` is not set.
     */
    setSettings(patch: Readonly<IndicatorSettings>, options?: IndicatorEditOptions): boolean;
    /** The series backing one plot key, for direct styling. */
    series(plotKey: string): SeriesApi | undefined;
    /** Latest computed values (a reference: do not mutate). */
    values(): IndicatorValues;
    /** Whether the plots are drawn (the legend's eye toggle). */
    visible(): boolean;
    /** Show or hide every plot without removing the instance. */
    setVisible(on: boolean): void;
    /** This indicator's legend row, or null if it has none. */
    legend(): PaneLegend | null;
    /** Refresh the legend readings for a bar index; omit for the latest bar. */
    updateLegendValues(index?: number): void;
    /** The bars it computes on: the chart's, or on a transformed chart the host's underlying bars. */
    barSource(): IndicatorBarSource;
    /**
     * Compute on the bars the chart draws (`'chart'`, the default) or the bars
     * the host feeds (`'underlying'`), whose values are then read at the bar
     * each drawn element was completed on. The two are the same bars on a chart
     * with no transform. False when unchanged, or for a study that is not
     * `configurable` unless `options.force` is set; any other value throws.
     */
    setBarSource(source: IndicatorBarSource, options?: IndicatorEditOptions): boolean;
    /**
     * Remove every series, level, and legend row this indicator created. False
     * when it is already gone, or is not `removable` and `options.force` is not set.
     */
    remove(options?: IndicatorEditOptions): boolean;
}

/**
 * The payload of the chart's `data:update` event (see `ChartEventMap`): a
 * primary-source mutation, emitted after indicator invalidation. Every chart
 * emits it, alerts or not; the type is declared here because the alert
 * controller was its first reader.
 */
interface ChartDataUpdate {
    kind: 'update' | 'reset' | 'prepend';
    time?: number;
}
/**
 * Minimum headless chart surface needed to evaluate alerts. A host announces
 * its writes with `data:update`; one `update` may append several bars, and the
 * controller judges each of them in turn.
 */
interface AlertChartHost {
    primaryBars(): readonly Bar[];
    /** Owning price scale for primary-price drag snapping, including a left axis. */
    primarySeries?(): Pick<SeriesApi, 'priceScale'> | null;
    getDataContext(): Readonly<ChartDataContext> | undefined;
    on(event: string, callback: (payload: unknown) => void): () => void;
    emit(event: string, payload: unknown): void;
    /** Flushes computed study values. Only required by indicator-source alerts. */
    indicators?(): readonly (Pick<IndicatorApi, 'id' | 'paneIndex' | 'series' | 'values'> & Partial<Pick<IndicatorApi, 'indicatorId'>>)[];
    addPrimitive?(primitive: IPrimitive, paneIndex?: number): void;
    removePrimitive?(primitive: IPrimitive): void;
    alertState?(): AlertsDocument | undefined;
    setAlertState?(document: AlertsDocument | undefined): void;
    /**
     * A price rounded to the tick the pane's scale is written in.
     *
     * Optional, and unrounded is the fallback: a host that declares no tick has
     * nothing to round to, and inventing one would move a price somebody chose.
     */
    snapPrice?(paneIndex: number, price: number): number;
    /**
     * The instrument's tick schedule, or null for a constant tick. A dragged
     * price alert lands on the band its price falls in. Optional; absent keeps
     * the source scale's one tick.
     */
    tickSchedule?(): TickSchedule | null;
    /**
     * Slot of the price pane, where a price alert draws and where a drawing
     * needs no input plot. It moves when a host puts the price pane below its
     * studies. Optional; absent means slot 0.
     */
    primaryPaneIndex?(): number;
}
type AlertCondition = 'crossing' | 'crossingUp' | 'crossingDown' | 'greaterThan' | 'lessThan' | 'enteringRange' | 'leavingRange' | 'matches';
type AlertPolicy = 'onBarClose' | 'onTouch';
type AlertRepeat = 'once' | 'everyTime';
type AlertState = 'armed' | 'triggered' | 'expired' | 'disabled';
/** Prices are in primary-series units. Range conditions require upperPrice. */
interface PriceAlertSource {
    kind: 'price';
    price: number;
    upperPrice?: number | undefined;
}
/** A threshold in plot units, anchored to one specific study instance. */
interface IndicatorAlertSource {
    kind: 'indicator';
    instanceId: string;
    plotKey: string;
    value: number;
    upperValue?: number | undefined;
}
interface BarConditionAlertSource {
    kind: 'barCondition';
    id: string;
}
interface DrawingAlertSource {
    kind: 'drawing';
    drawingId: string;
    level?: string;
    /** Required for a drawing on a study pane, so prices are not compared to different units. */
    input?: {
        instanceId: string;
        plotKey: string;
    };
}
type AlertSource = PriceAlertSource | IndicatorAlertSource | BarConditionAlertSource | DrawingAlertSource;
/** Missing values, an absent anchor or a paused/context-mismatched chart are unavailable. */
interface AlertAvailability {
    available: boolean;
    reason?: string | undefined;
    paneIndex?: number;
}
interface AlertDrawingValue {
    price: number;
    upperPrice?: number;
    paneIndex: number;
}
interface AlertDrawingLevel {
    id: string;
    title: string;
}
interface AlertDrawingInfo extends AlertAvailability {
    levels: readonly AlertDrawingLevel[];
}
/** Implemented by the optional drawing tier; the alert engine never imports it. */
interface AlertDrawingProvider {
    get(id: string): unknown;
    valueAt(id: string, time: number, level?: string): AlertDrawingValue | undefined;
    alertInfo(id: string): AlertDrawingInfo;
}
interface BarConditionContext {
    /** Only the prefix through index is exposed, including for confirmed-bar checks. */
    bars: readonly Bar[];
    index: number;
}
interface BarCondition {
    id: string;
    title: string;
    /** Runs at the alert's chosen policy, never for loaded history. */
    when(context: BarConditionContext): boolean;
}
/**
 * Evaluation belongs to the instrument, interval and data variant present when
 * the alert was armed. Fixed price levels remain visible on other intervals and
 * sessions of the same instrument quoted in the same currency and unit,
 * labelled with where they evaluate and paused until that is displayed again.
 */
interface AlertScope {
    symbol?: string;
    exchange?: string;
    interval?: string;
    /** The series the chart showed (extended hours, raw prices, a currency). Absent for the provider's default. */
    variant?: DataVariant;
}
interface AlertInput {
    id?: string;
    source: AlertSource;
    condition?: AlertCondition;
    /** Defaults to onBarClose. An intrabar touch may disappear from final history. */
    policy?: AlertPolicy;
    repeat?: AlertRepeat;
    state?: 'armed' | 'disabled';
    title?: string;
    message?: string | undefined;
    cooldownSeconds?: number;
    /** UTC seconds. At this instant the alert expires before it can trigger. */
    expiresAt?: number | undefined;
    /** Opaque host routing data. The controller never interprets or delivers it. */
    payload?: unknown;
}
type AlertPatch = Partial<Omit<AlertInput, 'id'>>;
/** Lifecycle record. Snapshots detach mutable configuration, retaining opaque payloads. */
interface Alert extends Omit<AlertInput, 'id' | 'condition' | 'policy' | 'repeat' | 'state' | 'title' | 'cooldownSeconds'> {
    id: string;
    condition: AlertCondition;
    policy: AlertPolicy;
    repeat: AlertRepeat;
    state: AlertState;
    title: string;
    cooldownSeconds: number;
    scope: AlertScope;
    lastTriggeredAt?: number;
    lastTriggeredTime?: number;
    /** Newest confirmed bar already judged, including a nonmatch. */
    lastClosedTime?: number;
    /** Newest intrabar match consumed, including one suppressed by cooldown. */
    lastTouchedTime?: number;
}
/** Portable trader records. JSON persistence rejects unsupported host payloads. */
interface AlertsDocument {
    version: 1;
    alerts: Alert[];
}
/** Shared delivery fields for trader and indicator-authored alerts. */
interface AlertEventPayload {
    alertId: string;
    title: string;
    message?: string;
    /** Source bar UTC seconds and source index, not delivery wall-clock time. */
    time: number;
    index: number;
}
interface AlertTriggeredPayload extends AlertEventPayload {
    price: number;
    alert: Alert;
}
/** Payload of `alert:created`, `alert:updated` and `alert:expired`: a copy of the alert as it now stands. */
interface AlertChangeEvent {
    alert: Alert;
}
/**
 * Payload of `alert:removed`. `reason` is `'removed'` for `remove`, `'drawing-removed'`
 * when its drawing was deleted, and `'drawing-missing'`, `'indicator-missing'` or
 * `'plot-missing'` when a restore could not find what the alert watches.
 */
interface AlertRemovedEvent extends AlertChangeEvent {
    reason: string;
}
/** Payload of `alert:error`: a bar condition threw while the alert was evaluated. */
interface AlertErrorEvent extends AlertChangeEvent {
    error: unknown;
}
/** Payload of `alerts:changed`: a drag on the chart committed a new level. */
interface AlertsChangedEvent {
    id: string;
    reason: 'dragged';
}
/** Payload of `alerts:restored`: every alert after a validated replacement. */
interface AlertsRestoredEvent {
    alerts: Alert[];
}
interface AlertControllerOptions {
    /** Delivery and expiry clock in UTC seconds. Defaults to Date.now() / 1000. */
    now?: () => number;
    drawings?: AlertDrawingProvider;
    /** PriceLine visuals are enabled on chart hosts; disable for a model-only consumer. */
    visuals?: boolean;
    /**
     * Whether a spent alert keeps its line. Defaults to `'show'`.
     *
     * A triggered or expired alert is no longer watching anything, and the two
     * defensible things to do with its line are opposites. Keeping it is this
     * library's default and is why each lifecycle state has its own badge and
     * colour: the line says what became of the level, which is worth knowing on a
     * chart somebody has just come back to.
     *
     * `'hide'` is for the host where that reading does not pay. A terminal left
     * open through a session accumulates levels that will never fire again, and
     * past a certain number the ones still watching are the hardest to pick out
     * of them. The alert itself is untouched either way: it stays in `list()`,
     * keeps its lifecycle state, still refuses to fire twice, and gets its line
     * back if a host re-arms it. Only the drawing goes.
     */
    spentLines?: 'show' | 'hide';
}

/**
 * A descriptor or input validator can report a condition the user can fix.
 * Calculation failures also surface on the instance's data status, allowing
 * the host to display the reason while other studies continue drawing.
 */
declare class IndicatorInputError extends Error {
    constructor(message: string);
}

/**
 * Indicator registry (ARCHITECTURE.md §6A, §8). The sibling of the chart-type
 * registry: that one answers *"how do I paint an array of bars"*, this one
 * answers *"what do I compute, what does it plot, and what can a user tune"*.
 *
 * A descriptor is data, not code-in-the-core: the chart never switches on an
 * indicator id. Each `plot` names a registered **chart type**, so indicators
 * ride the existing Family-A renderers and add no drawing code at all.
 *
 * The built-in descriptors live in the lazy `openalgo-charts/indicators` tier;
 * only the registry and the runtime ship in the base bundle, so an app that
 * plots its own maths pays nothing for the catalog.
 */

/** Which price a calculation reads from each bar. */
type IndicatorSource = 'open' | 'high' | 'low' | 'close' | 'hl2' | 'hlc3' | 'ohlc4' | 'volume';
/** A scalar study output, aligned with the primary source bars. */
interface IndicatorStudySource {
    readonly kind: 'indicator';
    readonly instanceId: string;
    readonly plotKey: string;
}
/** Committed scalar output supplied by a host that schedules study dependencies. */
interface IndicatorStudyOutput {
    generation: number;
    revision: number;
    /** Changes whenever an earlier output prefix may have changed. */
    historyRevision: number;
    source?: Readonly<SeriesDataState>;
    available: boolean;
    values: readonly (number | null)[];
}
/** A setting value an input condition compares with. */
type IndicatorInputConditionValue = string | number | boolean;
/**
 * A test over a study's current settings, read by a settings form for
 * `activeWhen` and `visibleWhen`.
 *
 * `is` holds when the setting under `key` equals the value, or one of the
 * values when it is a list; `isNot` is its negation. `all` holds when every
 * nested condition does and `any` when at least one does, so `all` of nothing
 * holds and `any` of nothing does not. Values compare strictly: the number 5
 * is not the text '5'. It is data rather than a callback so a descriptor that
 * a compiler emits, or one saved as JSON, can carry it.
 */
type IndicatorInputCondition = {
    key: string;
    is: IndicatorInputConditionValue | readonly IndicatorInputConditionValue[];
} | {
    key: string;
    isNot: IndicatorInputConditionValue | readonly IndicatorInputConditionValue[];
} | {
    all: readonly IndicatorInputCondition[];
} | {
    any: readonly IndicatorInputCondition[];
};
/**
 * How a settings form presents an input. The calculation never reads these:
 * a hidden or inactive input keeps its value, and `calc` still receives it.
 */
interface IndicatorInputPresentation {
    /**
     * The input can be edited only while this holds; otherwise the form shows it
     * disabled, with its value readable and a reason naming what it depends on.
     * Also inactive while an input the condition reads is hidden or inactive,
     * because a setting nobody can change cannot be what enables another.
     */
    activeWhen?: IndicatorInputCondition;
    /**
     * The input is shown only while this holds; otherwise it leaves the form and
     * the tab order, and any draft in it is kept for when it returns. Also
     * hidden while an input the condition reads is hidden.
     */
    visibleWhen?: IndicatorInputCondition;
    /**
     * Consecutive inputs sharing this id sit on one row, the first one's label
     * leading it: a length beside its source, a switch beside the value and
     * colour it turns on. A multi-line text input always takes a row of its own.
     */
    inline?: string;
}
/**
 * One tunable input. New typed values are validated before study mutations;
 * established input kinds retain their descriptor's calculation contract.
 *
 * `tooltip` is help text for the row. A label has to stay short enough to fit a
 * dense panel, which leaves nowhere to say what a parameter actually does, and a
 * ported study whose every input carried an explanation arrives here with that
 * explanation dropped. A settings UI renders it as a hover affordance beside the
 * label; the core ignores it.
 *
 * Every variant also takes the presentation fields of
 * {@link IndicatorInputPresentation}: `activeWhen`, `visibleWhen` and `inline`.
 */
type IndicatorInput = IndicatorInputPresentation & ({
    key: string;
    type: 'number';
    label: string;
    default: number;
    min?: number;
    max?: number;
    step?: number;
    group?: string;
    tooltip?: string;
} | {
    key: string;
    type: 'boolean';
    label: string;
    default: boolean;
    group?: string;
    tooltip?: string;
} | {
    key: string;
    type: 'color';
    label: string;
    default: string;
    group?: string;
    tooltip?: string;
} | {
    key: string;
    type: 'text';
    label: string;
    default: string;
    group?: string;
    tooltip?: string;
} | {
    key: string;
    type: 'symbol';
    label: string;
    default: string;
    exchangeKey?: string;
    group?: string;
    tooltip?: string;
} | {
    key: string;
    type: 'session';
    label: string;
    default: string;
    group?: string;
    tooltip?: string;
} | {
    key: string;
    type: 'multiline';
    label: string;
    default: string;
    group?: string;
    tooltip?: string;
} | {
    key: string;
    type: 'price';
    label: string;
    default: number;
    min?: number;
    max?: number;
    step?: number;
    pick?: boolean | {
        paneIndex?: number;
        priceScaleId?: PriceScaleId;
    };
    group?: string;
    tooltip?: string;
    /**
     * Key of a declared `timestamp` input this price pairs with: together
     * they name one point, a bar time and a price. A host picks both from
     * one click (`Chart.beginPick('point')`) and writes them in one settings
     * patch, so the study never computes on the time of one pick and the
     * price of another. A timestamp pairs with one price at most.
     */
    timeKey?: string;
    /**
     * With `timeKey`: a handle on the chart at the point, on the same pane
     * and scale a pick of this price uses, that drags the time and the price
     * together as one settings change and one undo step. The drawing tier's
     * controller draws it; a study the user may not configure keeps it still.
     */
    anchor?: boolean;
}
/** Absolute UTC seconds. Independent of the chart timezone and legacy wall-clock `time` inputs. */
 | {
    key: string;
    type: 'timestamp';
    label: string;
    default: number;
    min?: number;
    max?: number;
    step?: number;
    pick?: boolean;
    group?: string;
    tooltip?: string;
} | {
    key: string;
    type: 'select';
    label: string;
    default: string;
    options: readonly {
        label: string;
        value: string;
    }[];
    group?: string;
    tooltip?: string;
} | {
    key: string;
    type: 'source';
    label: string;
    default: IndicatorSource;
    allowStudyOutputs?: boolean;
    group?: string;
    tooltip?: string;
}
/**
 * A timeframe code (`'5m'`, `'1d'`), for a study that folds the chart's bars
 * up to a coarser interval. A settings UI renders it as a select over the
 * registered intervals, so the value is always one the engine can bucket by;
 * a free text box would accept `'5min'` and leave the study computing on a
 * code it cannot resolve. An empty default means "the chart's own interval".
 */
 | {
    key: string;
    type: 'interval';
    label: string;
    default: string;
    group?: string;
    tooltip?: string;
}
/**
 * A wall-clock instant in the chart's zone, written `YYYY-MM-DD HH:MM` (the
 * time part optional), for an anchor a user picks by date: the start of an
 * anchored VWAP, an event to measure from. It is carried as that string, not
 * as UTC seconds, so a layout saved in one zone restores to the same wall
 * clock in another, and `zonedStringToUtcSeconds` turns it into a bar time.
 */
 | {
    key: string;
    type: 'time';
    label: string;
    default: string;
    group?: string;
    tooltip?: string;
});
/** Dash pattern for a level, a drawing, or a plot. */
type IndicatorLineStyle = 'solid' | 'dashed' | 'dotted';
/** Line-style options, for a settings UI's Style tab. */
declare const INDICATOR_LINE_STYLES: readonly {
    label: string;
    value: string;
}[];
/** Settings keys the runtime derives for a plot's appearance. */
declare function plotStyleKeys(plot: IndicatorPlot): {
    color: string;
    width: string;
    lineStyle: string;
    opacity: string;
    type: string;
};
/**
 * Per-plot appearance inputs, generated from the descriptor rather than
 * hand-written on each one: every indicator gets colour, opacity, thickness,
 * and line style for free, and a settings UI can render them as a "Style" tab
 * beside the descriptor's own `inputs`.
 *
 * Defaults come from the plot's declared style (and its legacy `colorKey`), so
 * an indicator that already ships colours keeps them.
 */
declare function indicatorStyleInputs(descriptor: IndicatorDescriptor): IndicatorInput[];
/**
 * Chart types a plot can be re-rendered as. A moving average is a line by
 * default, but the same column of numbers reads better as a histogram or an
 * area depending on what you are looking for, and a descriptor cannot know
 * which. Restricted to the types that make sense for a single value column.
 */
declare const INDICATOR_PLOT_STYLES: readonly {
    label: string;
    value: string;
}[];
/** Canonical option list for a `type: 'source'` input, for settings UIs. */
declare const INDICATOR_SOURCES: readonly {
    label: string;
    value: IndicatorSource;
}[];
type IndicatorSettings = Record<string, unknown>;
/** A shaded band between two of an indicator's output columns. */
interface IndicatorFillSpec {
    /** Plot keys or unplotted calculated columns; unplotted columns use the band's local scale. */
    between: readonly [string, string];
    /** Colour where the first plot is above the second. */
    colorUp?: string;
    /** Colour where the second is above the first. */
    colorDown?: string;
    /** Settings keys holding those colours, so the band is restyleable. */
    colorUpKey?: string;
    colorDownKey?: string;
    /** 0..1. Defaults to 0.12. */
    opacity?: number;
    /** A price-anchored gradient for the whole band, resolved after each calculation. */
    gradient?: FillGradient | ((ctx: {
        bars: readonly Bar[];
        values: IndicatorValues;
        settings: Readonly<IndicatorSettings>;
    }) => FillGradient | undefined);
    /** Per-bar gradient takes precedence over the whole-band gradient. Undefined uses the band default. */
    gradientBy?(ctx: {
        index: number;
        a: number | null;
        b: number | null;
        values: IndicatorValues;
        settings: Readonly<IndicatorSettings>;
    }): FillGradient | undefined;
    /** Per-bar color takes precedence over both gradients and the up/down colors. */
    colorBy?(ctx: {
        index: number;
        a: number | null;
        b: number | null;
        values: IndicatorValues;
        settings: Readonly<IndicatorSettings>;
    }): string | undefined;
    /**
     * Draw the band on the price pane even though the indicator owns a pane of
     * its own. The pair with `IndicatorPlot.overlay`: a study can already send
     * one plot to the candles, and a band between two such plots belongs beside
     * them rather than in the study pane the fill would otherwise land in.
     * Ignored for an `'onchart'` descriptor, which is on the price pane already.
     */
    overlay?: boolean;
}
/** A named summary grid owned by one indicator instance. */
interface IndicatorTableSpec {
    /** Stable, nonempty identity, unique within this instance's table list. */
    id: string;
    rows: readonly (readonly TableCell[])[];
    options?: Partial<ChartTableOptions>;
    /** Keep this grid on the price pane when the indicator uses another pane. */
    overlay?: boolean;
}
/**
 * What `colorParts` answers: a candle plot's colour split three ways, which is
 * how a study paints a wick in full colour over a translucent body. `body` is
 * the bar's colour (and the only part a line, histogram or column reads); a
 * part left undefined falls back to `colorBy`, then to the plot's own colour.
 */
type PlotBarColor = {
    body?: string;
    wick?: string;
    border?: string;
};
/** One plotted line/band/histogram. `type` is any registered chart type. */
interface IndicatorPlot {
    /** Key into the `calc` result. */
    key: string;
    /** Registered chart type used to draw it ('line', 'histogram', 'area', ...). */
    type: SeriesType;
    /** Legend title. */
    title: string;
    /** Style overrides merged onto the chart type's defaults. */
    style?: SeriesStyle;
    /** Price axis for this plot. Defaults to 'right'. */
    priceScaleId?: PriceScaleId;
    /**
     * Value formatting for the axis and crosshair tag of the scale this plot maps
     * to: `percent` for a ratio study, `volume` for a cumulative one, `custom` for
     * anything else.
     *
     * Like `style.precision`, this is a property of the **price scale**, not of the
     * series, so it belongs to a plot that owns its pane. Setting it on an
     * `'onchart'` plot reformats the instrument's own axis, which is almost never
     * what a study wants.
     *
     * `percent` suffixes the value as it stands and does not scale it, so a study
     * returning a 0..1 fraction should keep returning it and read `0.62%`. Scaling
     * inside `calc` to make the axis read better changes the plotted value, and the
     * legend, the crosshair and every downstream calculation with it.
     */
    priceFormat?: PriceFormat;
    /**
     * Draw this one plot on the price pane even though the indicator owns a pane
     * of its own. An oscillator that also wants a signal
     * band or a stop line sitting on the candles is the case: the study belongs in
     * its own pane, one of its columns belongs on price, and splitting it into two
     * indicators would make the user configure the same inputs twice.
     *
     * Ignored for an `'onchart'` descriptor, which is already on the price pane.
     */
    overlay?: boolean;
    /**
     * Draw the column shifted this many bars to the right (negative: left). The
     * column itself stays one value per bar and `calc` returns exactly what it
     * always did; only where each value is painted moves. Positive is what a
     * displaced cloud or a projected channel wants: the last `offset` values land
     * in the right margin, past the newest candle, where no bar exists to hold
     * them. It shifts the drawn series only. A fill between two plots with the
     * same offset follows; the legend reads the value drawn under the cursor.
     */
    offset?: number;
    /**
     * Settings key holding this plot's color, so a settings change restyles the
     * series without a full rebuild.
     */
    colorKey?: string;
    /**
     * Four `calc` keys to draw this plot as bar-shaped elements instead of one
     * value per bar: candles, hollow candles, OHLC bars, high-low.
     *
     * A single column cannot express those at all, and the alternative (a second
     * result shape for `calc`) would fork the contract every descriptor and every
     * helper is written against. Naming four columns inside the *same*
     * `IndicatorValues` keeps one shape: a smoothed Heikin-Ashi overlay, a
     * higher-timeframe candle, a synthetic spread instrument each return four
     * ordinary columns and point at them from here.
     *
     * The named columns must all exist and be bar-aligned, or `addIndicator`
     * throws. `key` stays the series identity and the legend reading falls back to
     * the `close` column.
     */
    ohlc?: {
        open: string;
        high: string;
        low: string;
        close: string;
    };
    /**
     * Per-bar colour, for plots whose meaning changes bar to bar: a MACD
     * histogram is four colours by sign and direction, a conditional study two.
     * Return `undefined` to fall back to the plot's own colour.
     *
     * Reaches the renderer as `Bar.color`, so every Family-A plot type honours
     * it: histogram, column, candles, OHLC bars, line, step and area.
     */
    colorBy?(ctx: {
        value: number;
        index: number;
        values: IndicatorValues;
        settings: IndicatorSettings;
    }): string | undefined;
    /**
     * Per-bar colour split three ways, for a candle plot whose wick or border
     * should not follow its body: a solid wick over a translucent body, a
     * border in the trend colour. Takes precedence over `colorBy` for the parts
     * it names; a part it leaves undefined falls back to `colorBy`, then to the
     * plot's own colour. A value plot (line, histogram, column) reads `body` only.
     */
    colorParts?(ctx: {
        value: number;
        index: number;
        values: IndicatorValues;
        settings: IndicatorSettings;
    }): PlotBarColor | undefined;
}
/** A horizontal reference level (RSI 70/30, Stochastic 80/20, a zero line). */
interface IndicatorLevel {
    price: number;
    color?: string;
    title?: string;
    /**
     * Two-state shorthand for `lineStyle`. `lineStyle` wins when both are given,
     * and with neither a level draws dashed. A retained form rather than a
     * deprecated one (COMPATIBILITY.md lists why).
     */
    dashed?: boolean;
    lineWidth?: number;
    lineStyle?: IndicatorLineStyle;
}
/** One end of an indicator drawing: a time on the shared axis, a price on the pane's scale. */
interface DrawAnchor {
    time: number;
    price: number;
}
/**
 * Where one returned drawing, marker or shading column goes, when the study's
 * own layer is the wrong place for it. A study in its own pane still has
 * things to say about the candles (a supply zone, a buy signal, a regime), and
 * a study whose plots sit on two axes has shapes and marks measured on each.
 * Naming no target keeps the output in the study's own layer, exactly as
 * before.
 *
 * Each distinct target gets a layer of its own, owned by the instance: it
 * hides with the study, is released with it, and is released as soon as a
 * calculation returns nothing for that target. Marks sent to the series the
 * study's own marks already anchor to join that layer instead, so marks at
 * one bar stack rather than overlap, unless that series is an `overlay` plot
 * of a study in its own pane (see `plot`). A study's layers stack in a fixed
 * order: its own marks, its marker targets, its own shapes, its drawing
 * targets, its own shading, then its shading targets, each kind's targets
 * taking the price pane first and then the plots in declaration order.
 * Shading paints behind every series on its pane, so on any one pane it sits
 * under the candles, the plots, the marks and the shapes whatever its slot;
 * the order decides which study's shading covers another's. A targeted layer
 * created after the study was added is put back in that order among the
 * targeted layers on its pane, below those of the studies added after it. No
 * other layer moves for it, so an output that names no target stacks exactly
 * as before.
 */
interface IndicatorOutputTarget {
    /**
     * A declared plot key. A shape is drawn on that plot's pane and measured on
     * its effective price scale; a marker is anchored to that plot's series, so
     * `aboveBar` and `belowBar` read its values, and where it has none, the
     * candle's whenever that plot is on the price pane and on the candles' scale,
     * first plot or not. A shading column fills that plot's pane and is bound to
     * its effective scale, which has no price to measure but keeps the layer on
     * the axis the plot uses, so a reassignment never leaves a column held for
     * nothing. Each follows the plot through a scale reassignment or a study
     * move. An `overlay` plot takes it to the price pane.
     */
    plot?: string;
    /**
     * The price pane, in the instrument's own units, staying there when the
     * study moves. A shape is measured on the scale that pane quotes prices on
     * (its crosshair readout, which is the candles' own scale on whichever axis
     * they sit) and holds no axis itself, so the price axis stays free to move.
     * A marker is anchored to the instrument's candles, so `belowBar` sits
     * under the low; it is drawn once the chart has a primary series. A shading
     * column fills the price pane's full height and binds no axis. Naming a
     * plot as well is rejected: a plot already decides its pane.
     */
    overlay?: boolean;
}
/**
 * One column of per-bar shading a study's `background` returns in its list
 * form, optionally sent to another pane or plot. `colors` is what the plain
 * form returns: one entry per bar, `null` where nothing is shaded.
 */
interface IndicatorBackgroundSpec extends IndicatorOutputTarget {
    colors: readonly (string | null)[];
}
/** A signal marker a study returns, optionally sent to another pane or plot. */
type IndicatorMarker = SeriesMarker & IndicatorOutputTarget;
/**
 * A free-standing shape an indicator paints in its own pane, anchored to time
 * and price rather than to a bar index.
 *
 * Plots, levels and markers each answer a different question and none of them
 * answers this one: a pivot-to-pivot trendline, a supply zone, an order block,
 * a measured-move projection are all geometry between two arbitrary points, and
 * a column of one value per bar cannot express any of them. Anchors are times,
 * so a shape stays put when history is paged in and every logical index shifts.
 * Any shape can name an {@link IndicatorOutputTarget} to be drawn elsewhere.
 */
type IndicatorDrawing = IndicatorOutputTarget & ({
    kind: 'line';
    from: DrawAnchor;
    to: DrawAnchor;
    color?: string;
    lineWidth?: number;
    lineStyle?: IndicatorLineStyle;
    /** Continue the line past its anchor to the pane edge. */
    extendLeft?: boolean;
    extendRight?: boolean;
} | {
    kind: 'box';
    from: DrawAnchor;
    to: DrawAnchor;
    /** Border colour. Omit `fillColor` to draw an outline only. */
    color?: string;
    fillColor?: string;
    /** Fill alpha, 0..1. Defaults to 0.12. */
    opacity?: number;
    lineWidth?: number;
    /** Caption drawn on a plate at the centre of the box; `\n` splits lines. */
    text?: string;
    textColor?: string;
    /** Positive finite CSS pixels. Defaults to 11. */
    fontSize?: number;
    /** CSS font-family list. Defaults to ui-sans-serif, system-ui, sans-serif. */
    fontFamily?: string;
    bold?: boolean;
    italic?: boolean;
    /** Multiline row alignment inside the plate. Defaults to left. */
    textAlign?: 'left' | 'center' | 'right';
    /** Plate placement inside the box. Defaults to center and middle. */
    align?: 'left' | 'center' | 'right';
    verticalAlign?: 'top' | 'middle' | 'bottom';
    /**
     * Detail shown on a plate while the pointer rests on the box, and gone
     * when it leaves; `\n` splits lines. A zone that carries its size, its
     * age and what formed it cannot print all of that on the box without
     * hiding the candles under it, so the caption names it and this explains
     * it. The box becomes hit-testable, and `id` (or the tooltip text) is
     * what `subscribeClick` reports for it.
     */
    tooltip?: string;
    /** Hit id, for `subscribeClick`. Defaults to the tooltip text. */
    id?: string;
} | {
    kind: 'label';
    at: DrawAnchor;
    /** `\n` splits lines. */
    text: string;
    /** Plate fill. */
    color?: string;
    textColor?: string;
    /** Positive finite CSS pixels. Defaults to 11. */
    fontSize?: number;
    /** CSS font-family list. Defaults to ui-sans-serif, system-ui, sans-serif. */
    fontFamily?: string;
    bold?: boolean;
    italic?: boolean;
    /** Multiline row alignment inside the plate. Defaults to left. */
    textAlign?: 'left' | 'center' | 'right';
    /** Which edge of the plate sits on the anchor. Defaults to 'center'. */
    align?: 'left' | 'center' | 'right';
    /** Which vertical plate edge sits on the anchor. Defaults to middle. */
    verticalAlign?: 'top' | 'middle' | 'bottom';
    /** Hover detail, as on a box. */
    tooltip?: string;
    /** Hit id, for `subscribeClick`. Defaults to the tooltip text. */
    id?: string;
} | {
    kind: 'polyline';
    points: readonly DrawAnchor[];
    /**
     * Straight segments by default. Smooth interpolates anchors in screen
     * space with half-chord tangents and can overshoot their price range.
     */
    curve?: 'linear' | 'smooth';
    color?: string;
    lineWidth?: number;
    /** Close the path back to the first point (a triangle, a wedge). */
    closed?: boolean;
    fillColor?: string;
    /** Fill alpha, 0..1. Defaults to 0.12. */
    opacity?: number;
});
/** `calc` output: one array per plot key, aligned 1:1 with the input bars. */
type IndicatorValues = Record<string, readonly (number | null)[]>;
/** Per-instance scratch owned by the descriptor (Tier-2 data lands here). */
type IndicatorStore = Record<string, unknown>;
/** Why this calculation ran. Revisions count source mutations, not provider ticks executed. */
interface IndicatorExecutionContext {
    /** Stable source-series identity within this host. */
    sourceId: number;
    provenance: 'history' | 'live' | 'replay';
    change: 'initial' | 'reset' | 'prepend' | 'append' | 'replace' | 'correction' | 'refresh';
    revision: number;
    historyRevision: number;
    confirmationSource: 'provider' | 'replay' | 'clock' | 'unknown' | 'empty';
}
/**
 * The fourth, optional argument to `calc` (and the sixth to `calcTail`): what
 * the calculation cannot read off the bars themselves.
 *
 * It is optional so that every descriptor written against `calc(bars, settings,
 * store)` keeps its exact signature and its exact behaviour, which is the whole
 * point: a calculation that ignores the context computes what it always did.
 */
interface IndicatorCalcContext {
    /** Resolves declared, opted-in study inputs without recursively flushing the chart. */
    resolveSource?(source: IndicatorStudySource): readonly (number | null)[];
    /** Native mutation provenance. Older custom hosts may omit it. */
    execution?: IndicatorExecutionContext;
    /**
     * Where the last bar stands, so a study can act once per bar rather than once
     * per tick, or refuse to signal off a bar that is still moving.
     */
    barState: {
        /** A live execution sees a newer tail than the previous calculation, including coalesced appends. */
        isNew: boolean;
        /**
         * The last bar's declared duration or calendar period has elapsed on the
         * chart clock. Count-driven and unknown intervals cannot be confirmed by
         * the clock. Without an interval, retains the legacy last-gap estimate
         * (a single bar is confirmed). Explicit provider or replay state takes
         * precedence over that estimate. Empty history is confirmed.
         */
        isConfirmed: boolean;
        /** A live feed is driving updates, rather than a one-off history load. */
        isRealtime: boolean;
        /** Index of the last bar, `bars.length - 1` (-1 when there are none). */
        lastIndex: number;
    };
    /** The instrument, when the host knows one. See `IndicatorAttachContext`. */
    symbol?: string;
    /** The timeframe (`'5m'`, `'1d'`), on the same terms as `symbol`. */
    interval?: string;
    /** The chart's IANA zone, the calendar its axis is labelled in. */
    timezone: string;
    /**
     * True when the bars `calc` receives are a transform's elements (Renko
     * bricks, Heikin Ashi candles) rather than the bars the host feeds, so their
     * times do not mark out the clock. Absent otherwise.
     */
    transformed?: boolean;
    /** Chart wall clock in UTC seconds, the clock the countdown row reads. */
    now(): number;
    /**
     * The instrument's tick size, from the **price pane's** `minMove`.
     *
     * The price pane and not the indicator's own, because `calc` runs on the
     * instrument's bars whichever pane the plot lands in, and a study pane is not
     * quoted in the instrument's tick: an RSI is a dimensionless 0..100 band, so
     * its scale carries no tick at all to read.
     *
     * `undefined` when the host has not told the chart what it is, which is the
     * honest answer rather than a guessed 0.01: an indicator sizing a range in
     * ticks has to tell "one paisa" apart from "nobody said".
     */
    tickSize?: number;
}
/** What an alert's `when` predicate is handed, for the bar it is judging. */
interface IndicatorAlertContext {
    bars: readonly Bar[];
    values: IndicatorValues;
    settings: Readonly<IndicatorSettings>;
    /** The bar being evaluated. */
    index: number;
}
/** Delivery frequency for eligible live calculations after chart batching. */
type IndicatorAlertFrequency = 'everyUpdate' | 'oncePerBar' | 'onBarClose' | 'once';
/**
 * A condition the runtime watches, declared by the descriptor rather than wired
 * up by the host: the indicator is the only thing that knows what a crossover of
 * its own columns means.
 *
 * Evaluated once per bar, for bars that are new since the last evaluation, so
 * adding the indicator to a loaded chart fires nothing for history.
 */
interface IndicatorAlertSpec {
    /** Stable within the descriptor, e.g. `'cross-up'`. */
    id: string;
    /** Short human label, e.g. `'MACD crossed up'`. */
    title: string;
    /**
     * Omitted retains evaluation only when a new bar is appended. Explicit
     * policies also observe qualifying updates within a bar. once is spent only
     * by a delivered live event and lasts for this instance's lifetime.
     * Historical loads, replay and settings-only recalculation never deliver.
     */
    frequency?: IndicatorAlertFrequency;
    /**
     * Longer text for a notification; defaults to `title`. A function is handed
     * the same context `when` judged, so the message can carry the bar's own
     * numbers: the price it crossed at, the histogram reading, a JSON body for a
     * webhook. It runs only for a bar `when` accepted.
     */
    message?: string | ((ctx: IndicatorAlertContext) => string);
    when(ctx: IndicatorAlertContext): boolean;
}
/**
 * Bars of another instrument or interval, supplied by the host on request.
 *
 * The engine is handed one symbol's bars and owns no transport, so a study
 * that compares against a benchmark, or a Tier-2 provider that needs a second
 * series, asks the host through this and the host answers from wherever it
 * keeps history. `from` and `to` are UTC seconds. Caller cancellation and the
 * instance lifetime both bound each request. Managed studies cancel their
 * data-setting generations while preserving requests across style changes.
 */
interface IndicatorBarsRequest {
    symbol: string;
    exchange?: string;
    interval: string;
    from: number;
    to: number;
    /** The provider series to answer from; absent is its default. See `inheritedDataVariant`. */
    variant?: DataVariant;
    signal?: AbortSignal;
}
type IndicatorBarsProvider = (request: IndicatorBarsRequest) => Promise<readonly Bar[]>;
/** Requested observations and their known availability, aligned one-to-one. */
interface RequestedBarsSnapshot {
    /** Finite, strictly increasing opening times. Observations are never compacted. */
    bars: readonly Bar[];
    /** UTC seconds at or after opening; null means availability is unknown. */
    availableAt: readonly (number | null)[];
    /** Explicit confirmation, independent of a clock or the next observed opening. */
    confirmed: readonly boolean[];
}
/** An optional historical knowledge cutoff, separate from opening-time bounds. */
interface IndicatorSnapshotRequest extends IndicatorBarsRequest {
    /** The provider must supply values as known then, or reject if unsupported. */
    asOf?: number;
}
/** Native provider access with optional explicit requested-data metadata. */
interface IndicatorBarsProviderAccess {
    requestBars: IndicatorBarsProvider;
    requestSnapshot?(request: IndicatorSnapshotRequest): Promise<RequestedBarsSnapshot>;
}
/** Current native request identity and availability boundary. */
interface IndicatorRequestState {
    source?: Readonly<SeriesDataState>;
    providerRevision: number;
    /** Host announcements of changed external data, independently of price ticks. */
    dataRevision: number;
    supportsSnapshots: boolean;
    /** Legacy replay has no known availability cutoff. */
    replay?: {
        time: number;
        asOf?: number;
        forming: boolean;
    };
}
/** Payload of the `'indicator:alert'` event on the chart's own bus. */
interface IndicatorAlertPayload extends AlertEventPayload {
    /** Descriptor id, e.g. `'macd'`. */
    indicatorId: string;
    /** Instance id, so a host can tell three EMAs apart. */
    instanceId: string;
    message: string;
}
/** Optional instrument identity and capability supplied by the host. */
interface ChartDataContext {
    symbol?: string;
    exchange?: string;
    interval?: string;
    /** Instrument capability, independent of readings: false unsupported, absent unknown. */
    hasOpenInterest?: boolean;
    /** Which of the provider's series the chart shows; absent is its default. Set it with `publishDataContext`. */
    variant?: Readonly<DataVariant>;
}
/** Source identity changed, or the available source-bar range changed. */
type IndicatorDataChange = 'context' | 'range';
/** Observable state of an indicator's external data lifecycle. */
type IndicatorDataStatus = {
    state: 'loading' | 'ready' | 'empty' | 'unsupported';
} | {
    state: 'error';
    error: unknown;
};
/** What an indicator's `attach` lifecycle can reach. */
interface IndicatorAttachContext {
    /** Optional host identity, independent of the indicator's own settings. */
    dataContext?(): Readonly<ChartDataContext> | undefined;
    /** Read bars and identity again when the host publishes a change. */
    subscribeDataChanges?(listener: (change: IndicatorDataChange) => void): () => void;
    /** Publish status and an explicit retry action for this instance. */
    setDataStatus?(status: IndicatorDataStatus): void;
    setDataRetry?(retry: (() => void) | null): void;
    /** Instance lifetime. Aborted on removal, preserved across style changes. */
    signal?: AbortSignal;
    /** Current settings (live: read at call time, not captured). */
    settings(): Readonly<IndicatorSettings>;
    /** The bars the study computes on now: the chart's, or the underlying bars for a study set to them (`IndicatorApi.setBarSource`). */
    bars(): readonly Bar[];
    /** Re-run `calc` and repaint: call when external data arrives. */
    requestRecompute(): void;
    /** Scratch this instance owns; the same object `calc` receives. */
    store: IndicatorStore;
    /**
     * The instrument the chart is showing, when the host knows it.
     *
     * Hosts can supply this through `IndicatorHost` or Chart's explicit data
     * context. Without a configured identity it stays undefined. External
     * studies read `dataContext` to include the exchange as well.
     */
    symbol?(): string | undefined;
    /** The chart's timeframe (`'5m'`, `'1d'`), on the same terms as `symbol`. */
    interval?(): string | undefined;
    /** The chart's IANA zone, the one its axis is labelled in. */
    timezone?(): string;
    /** Chart wall clock in UTC seconds, the same clock the countdown row uses. */
    now?(): number;
    /**
     * Ask the host for another instrument's (or interval's) bars. Always present
     * under `chart.addIndicator`; it rejects when the host has registered no
     * provider (`chart.setBarsProvider`), so a study can treat the rejection as
     * "unsupported here" and say so through `setDataStatus`.
     */
    requestBars?(request: IndicatorBarsRequest): Promise<readonly Bar[]>;
    /** Request explicit confirmation and availability without inferring either from raw bars. */
    requestSnapshot?(request: IndicatorSnapshotRequest): Promise<RequestedBarsSnapshot>;
    /** Native source, provider and replay identity, read at request time. */
    requestState?(): Readonly<IndicatorRequestState>;
    /** Includes source revisions, provider changes and within-bar replay clock movement. */
    subscribeRequestChanges?(listener: () => void): () => void;
    /** The pane this instance drew into. Moves when panes are reordered. */
    paneIndex?(): number;
    /** Attach a primitive to this indicator's pane, and detach it again. */
    addPrimitive?(p: IPrimitive): void;
    removePrimitive?(p: IPrimitive): void;
    /**
     * Emit on the chart's own event bus, the one `chart.on(name, cb)` listens to.
     *
     * The declarative `alerts` slot covers a condition read off the bars; this is
     * the imperative half, for an indicator whose signal arrives from outside the
     * calculation entirely (a subscription its `attach` opened).
     */
    emit?(event: string, payload: unknown): void;
}
/**
 * What `levels` is handed. It carries `bars` and `values` **and** spreads the
 * settings keys onto itself, so the built-ins written against the original
 * `levels(settings)` signature keep working unchanged: they read
 * `ctx.overbought` (or pass `ctx` to a `num(s, key, default)` helper) and find
 * exactly what they found before. A widened parameter is the only way a level
 * can be data-derived (yesterday's high, the session VWAP band), and that is a
 * whole class of level that could not be expressed at all before.
 *
 * The three data members are optional for the same backward-compatibility
 * reason, not because the runtime ever omits them: it always passes all three,
 * but a caller holding only a settings bag must still be able to invoke
 * `levels` directly. A descriptor that needs the data should default them
 * (`ctx.bars ?? []`).
 *
 * `settings`, `bars` and `values` are therefore reserved keys, the way
 * `timezone` already is in the settings a `calc` receives: an input declared
 * under one of those names is shadowed here.
 */
type IndicatorLevelContext = IndicatorSettings & {
    settings?: Readonly<IndicatorSettings>;
    bars?: readonly Bar[];
    values?: IndicatorValues;
};
interface IndicatorDescriptor {
    /** Registry key, e.g. `'macd'`. */
    id: string;
    /** Display name, e.g. `'MACD'`. */
    name: string;
    /** Grouping for a picker UI ('Trend', 'Momentum', 'Volume', 'Volatility'). */
    category?: string | undefined;
    /** `'onchart'` overlays the price pane; `'pane'` gets its own pane. */
    placement: 'onchart' | 'pane';
    /**
     * This indicator was written from code the host can show the user.
     *
     * Its legend row then carries a source button beside the gear, and pressing
     * it emits `indicatorSource` with the same payload `indicatorSettings`
     * carries. The engine does not hold the code and does not want to: a
     * descriptor may be compiled from a script, generated, or written by hand in
     * the host's own bundle, and only the host knows which of those it can put in
     * front of somebody. So this says a button is worth offering, and the host
     * decides what the button opens.
     *
     * Absent or false draws no button, which is every built-in study.
     */
    hasSource?: boolean;
    inputs: readonly IndicatorInput[];
    plots: readonly IndicatorPlot[];
    /**
     * Shaded bands between pairs of plots: the Ichimoku cloud, a Bollinger
     * channel. A pair of lines is not the same picture as a filled region: the
     * fill is what makes "price is above the cloud" readable at a glance, and
     * which side leads is itself the signal, hence the two colours.
     */
    fills?: readonly IndicatorFillSpec[];
    /**
     * Full recompute over every bar. Must return arrays the same length as
     * `bars` (use `null` for warmup gaps: the line renderer breaks across them
     * and autoscale skips them).
     *
     * Tier-1 indicators are pure functions of `(bars, settings)` and ignore
     * `store`. Tier-2 indicators (the ones with their own data) read the
     * external series their `attach` lifecycle put in `store`.
     */
    calc(bars: readonly Bar[], settings: Readonly<IndicatorSettings>, store: IndicatorStore, ctx?: IndicatorCalcContext): IndicatorValues;
    /**
     * Optional per-instance lifecycle, for indicators whose data is not derived
     * from the chart's bars (CVD, PCR, an external feed). Called once
     * when the instance is created; return a teardown function.
     *
     * Fetch into `ctx.store`, then call `ctx.requestRecompute()`: `calc` runs
     * again and reads what you stored.
     */
    attach?(ctx: IndicatorAttachContext): (() => void) | void;
    /**
     * Optional incremental path, called instead of `calc` when only the tail
     * changed (a live tick). Return values for indices `[fromIndex, bars.length)`
     * (the runtime splices them onto the previous result) or `null` to fall
     * back to a full `calc`.
     *
     * Without it every tick costs a full recompute. That is a few hundred
     * microseconds for one indicator over 50k bars, but it is O(n) per tick per
     * indicator, so implement this for anything meant to run in a busy live pane.
     */
    calcTail?(bars: readonly Bar[], settings: Readonly<IndicatorSettings>, fromIndex: number, previous: IndicatorValues, store: IndicatorStore, ctx?: IndicatorCalcContext): IndicatorValues | null;
    /**
     * Optional bar-anchored signal markers: a named "Buy"/"Sell" plate, an arrow
     * at a crossover. Runs after every `calc`, so it reads the values it just
     * produced rather than recomputing anything.
     *
     * A plot cannot express this: a plot is a column of prices drawn as a line or
     * histogram, whereas a signal is a discrete event with a label. Returning `[]`
     * (when a `showLabels`-style input is off, say) clears every layer. A mark
     * can name the price pane or a plot to anchor to instead of the default
     * (see {@link IndicatorOutputTarget}).
     */
    markers?(ctx: {
        bars: readonly Bar[];
        values: IndicatorValues;
        settings: Readonly<IndicatorSettings>;
    }): readonly IndicatorMarker[];
    /**
     * What `aboveBar` and `belowBar` are measured against.
     *
     * `'plot'`, the default, is this study's own first plot, which is right for a
     * mark that belongs to the line: an arrow on a moving average sits against
     * the average.
     *
     * `'price'` is the instrument's candles, so above is above the high and below
     * is below the low. That is what a buy or sell signal on an overlay study
     * means, and anchoring one to the study's own column instead puts it wherever
     * that column happens to sit: a study that anchors its marks to a mid-body
     * line draws every "below" mark through the middle of the candle.
     *
     * Ignored by a study in its own pane, which has no candles to measure
     * against, and ignored when the chart has no primary series yet. Both fall
     * back to the first plot rather than dropping the marker. It applies to the
     * marks that name no target; a study in its own pane sends a mark to the
     * candles with `overlay: true` on that mark.
     */
    markerAnchor?: 'plot' | 'price';
    /**
     * Optional summary grid pinned to a corner of the pane.
     *
     * Some studies are not a value per bar at all: a seasonality heatmap is a
     * matrix of monthly returns, a scoreboard is a handful of statistics. Those
     * have no place in `calc`, whose contract is one column per plot aligned to
     * the bars, so they come back through here instead. Runs after every `calc`.
     *
     * Return `null` (or a zero-row grid) to draw nothing, which is how a
     * `showTable`-style input should switch it off.
     */
    table?(ctx: {
        bars: readonly Bar[];
        values: IndicatorValues;
        settings: Readonly<IndicatorSettings>;
    }): {
        rows: readonly (readonly TableCell[])[];
        options?: Partial<ChartTableOptions>;
    } | null;
    /**
     * Multiple named grids, refreshed after each calculation. Stable IDs reuse
     * their grid; omitted IDs are removed. Return [] to remove all grids.
     * When provided, this hook takes precedence over the single `table` hook.
     */
    tables?(ctx: {
        bars: readonly Bar[];
        values: IndicatorValues;
        settings: Readonly<IndicatorSettings>;
    }): readonly IndicatorTableSpec[];
    /**
     * Optional free-standing shapes: trendlines between pivots, supply and
     * demand boxes, projection labels. Drawn in the indicator's pane unless a
     * shape names another pane or plot (see {@link IndicatorOutputTarget}). Runs
     * after every `calc`, like `markers` and `table`, and the returned list
     * replaces the previous one wholesale, so returning `[]` clears every layer.
     */
    draws?(ctx: {
        bars: readonly Bar[];
        values: IndicatorValues;
        settings: Readonly<IndicatorSettings>;
    }): readonly IndicatorDrawing[];
    /**
     * Optional per-bar shading behind everything else in the indicator's pane: a
     * full-height column per bar, `null` where nothing should be shaded.
     *
     * A regime study answers "which state is the market in right now", and that is
     * a property of the whole bar, not a price. Drawn as a plot it would need a
     * value to sit at and would fight the pane's autoscale; as a column behind the
     * candles it reads at a glance and costs the scale nothing.
     *
     * Return a list of {@link IndicatorBackgroundSpec} instead to shade somewhere
     * else as well: each column can name the price pane or a plot (see
     * {@link IndicatorOutputTarget}), one column per target, and a column naming
     * none shades the indicator's pane exactly as the plain form does.
     *
     * Runs after every `calc`. Return `[]` to clear the study's own shading and
     * release every targeted column.
     */
    background?(ctx: {
        bars: readonly Bar[];
        values: IndicatorValues;
        settings: Readonly<IndicatorSettings>;
    }): readonly (string | null)[] | readonly IndicatorBackgroundSpec[];
    /**
     * Optional recolouring of the **main price candles**, one entry per bar,
     * `null` to leave that bar with its own colour.
     *
     * Distinct from a plot's `colorBy`, which paints the indicator's own series: a
     * trend filter, a volatility regime or a higher-timeframe bias is a statement
     * about the price bars themselves, and drawing it as a second series beside
     * them says something weaker.
     *
     * Only one indicator's colours can be on the candles at a time; the most
     * recent publisher wins, and publishers run in `addIndicator` order, so the
     * winner is the same one from frame to frame. Removing it, or hiding it,
     * restores the bars' own colours.
     */
    barColors?(ctx: {
        bars: readonly Bar[];
        values: IndicatorValues;
        settings: Readonly<IndicatorSettings>;
    }): readonly (string | null)[];
    /**
     * Optional conditions the runtime watches on the descriptor's behalf, emitted
     * as `'indicator:alert'` on the chart's event bus with an
     * {@link IndicatorAlertPayload}. See {@link IndicatorAlertSpec}.
     */
    alerts?: readonly IndicatorAlertSpec[];
    /**
     * Optional horizontal reference levels drawn in the indicator's pane.
     * Recomputed after every `calc`, so a level derived from the data (the
     * previous day's high) tracks it. See `IndicatorLevelContext` for why the
     * argument still reads as a settings bag.
     */
    levels?(ctx: IndicatorLevelContext): readonly IndicatorLevel[];
    /**
     * Optional fixed price range for the indicator's own pane (RSI 0..100).
     * Applied only when the indicator creates its pane: two indicators sharing a
     * pane would otherwise fight over it.
     */
    range?(settings: Readonly<IndicatorSettings>): {
        min: number;
        max: number;
    } | null;
}
/** Register an indicator descriptor. Later registrations of the same id win. */
declare function registerIndicator(descriptor: IndicatorDescriptor): void;
declare function getIndicator(id: string): IndicatorDescriptor;
declare function hasIndicator(id: string): boolean;
declare function registeredIndicators(): IndicatorDescriptor[];
/** The descriptor's declared defaults as a settings object. */
declare function indicatorDefaults(descriptor: IndicatorDescriptor): IndicatorSettings;
/** Read one bar's value for a price source. */
declare function sourceValue(bar: Bar, source: IndicatorSource): number;
/** Read a price source or an explicitly resolved scalar study output. */
declare function sourceValues(bars: readonly Bar[], source: IndicatorSource): number[];
declare function sourceValues(bars: readonly Bar[], source: IndicatorSource | IndicatorStudySource, context?: Pick<IndicatorCalcContext, 'resolveSource'>): (number | null)[];

type PriceAxisSide = 'left' | 'right' | 'hidden';
/** Placement is independent of scale identity, formatting and numeric range. */
interface PriceAxisPlacement {
    side: PriceAxisSide;
    /** Zero is nearest the plot; hidden scales retain an order for restoration. */
    order: number;
}
/** A visible axis column. Chart.priceAxisLayout returns x in pane CSS coordinates. */
interface PriceAxisSlot {
    scaleId: PriceScaleId;
    side: 'left' | 'right';
    order: number;
    x: number;
    width: number;
}

/**
 * Series transforms the chart applies itself (ARCHITECTURE.md §6A, Family B).
 *
 * A price-driven chart type (Heikin Ashi, Renko, range bars, line break,
 * point and figure, Kagi) draws elements derived from the host's bars rather
 * than the bars themselves. Before 2.6.0 a host ran the transform and handed
 * the chart the result; a series given a transform here takes the host's bars
 * as they are, runs the transform, and draws its elements, tick by tick.
 *
 * The transforms themselves live in the lazy transform tier, which registers
 * them here on import, so a chart that never draws one never loads one. The
 * base holds only this registry and the shapes it names. A series with no
 * transform is exactly what it always was: nothing here runs for it, which is
 * why a host that still prepares its own elements is never transformed twice.
 */

/**
 * A transform choice: which one, and the options it was given. JSON-safe, so
 * saved state carries it as it is.
 */
interface SeriesTransformSpec {
    /** A registered transform: `heikin-ashi`, `renko`, `range-bars`, `line-break`, `point-figure` or `kagi` once the transform tier is imported. */
    type: string;
    /**
     * The options the transform declares (`SeriesTransformDefinition.inputs`),
     * by the names its class takes: `boxSize`, `range`, `lines`, `reversal`.
     * An omitted option takes its default, and a size left at 0 is taken from
     * the history loaded with each `setData`.
     */
    options?: Readonly<Record<string, number | string>>;
}
/**
 * One transformed series, as the chart drives it: the host's bars in, the
 * elements it draws out. The newest source bar is the forming one, so the
 * elements it produced are provisional and a tick replaces them; no state is
 * ever committed from it (CLAUDE.md, never cache the forming bar).
 */
interface SeriesTransformRun {
    /** Replace the source history: a load, or a new symbol. Sizes left automatic are resolved from it. */
    setData(bars: readonly Bar[]): void;
    /** Merge older bars in at the left edge, keeping the resolved sizes. */
    prepend(bars: readonly Bar[]): void;
    /**
     * Apply one source bar: the forming bar again, a newer bar, or a correction
     * of an older one. Returns the index of the first element that may differ
     * from before, so the chart writes the tail alone.
     */
    update(bar: Bar): number;
    /** The source bars, oldest first. Read-only: the run keeps it. */
    source(): readonly Bar[];
    /** The elements, oldest first, with strictly increasing times. Read-only. */
    elements(): readonly Bar[];
    /**
     * The source bar each element was completed on, the newest bar for an
     * element still forming; null when each source bar is one element at its
     * own time (Heikin Ashi).
     */
    sourceIndex(): readonly number[] | null;
}
/** What the transform tier registers for one transform. */
interface SeriesTransformDefinition {
    /** Display name, for a settings group or a menu. */
    name: string;
    /** The renderer its elements draw with. */
    renderer: SeriesType;
    /**
     * The options it takes, described the way a study describes its inputs, so
     * `create` checks a spec against them and a settings dialog renders
     * them. Only `number` and `select` inputs are used.
     */
    inputs: readonly IndicatorInput[];
    /**
     * Build a run. Throws a `TypeError` naming an option the transform does not
     * take or a value outside what its input allows, before anything is built:
     * how a host checks a spec from outside (a saved layout).
     */
    create(options: Readonly<Record<string, number | string>>): SeriesTransformRun;
}
declare function getSeriesTransform(type: string): SeriesTransformDefinition;
declare function registeredSeriesTransforms(): string[];

/**
 * Serialisable chart state: the keystone the persistence-shaped features hang
 * off (saved layouts, templates, an objects panel, favourites, drawings).
 *
 * The rule that shapes this type: **the chart serialises what the chart owns.**
 * Series *data* is the application's (it knows the symbol, the timeframe, and
 * the feed), so `restoreState` never recreates series. It restores the things
 * the chart is the source of truth for (viewport, grid, panes, price scales,
 * indicators) and reports the series it saw so an app can rebuild them itself
 * and re-apply their styling.
 */

/**
 * The newest state version this build reads and writes. Bumped when the shape
 * changes incompatibly; `restoreState` ignores unknown versions.
 *
 * Version 2 adds `primaryPane`. `getState` writes it only for a chart whose
 * price pane has moved from the top, and writes every other state as version
 * 1, unchanged, so a reader from before the move still opens it. A reader
 * that predates version 2 refuses a moved layout instead of laying the price
 * pane's scales, studies and drawings onto the study pane in slot 0.
 */
declare const CHART_STATE_VERSION = 2;
interface PriceScaleState {
    marginTop: number;
    marginBottom: number;
    minMove: number;
    /** Optional in older snapshots, which used the current scale's precision floor. */
    minPrecision?: number;
    mode: PriceScaleMode;
    inverted: boolean;
    /** False when the user (or an indicator's fixed range) pinned the scale. */
    autoScale: boolean;
    /** The pinned range, present only when `autoScale` is false. */
    range?: {
        min: number;
        max: number;
    };
    /** The declared auto-fit band, independent of a temporary manual range. */
    fixedRange?: {
        min: number;
        max: number;
    } | null;
    /** Study default ownership; manual records a host view override without losing the default. */
    indicatorRange?: {
        instanceId: string;
        manual: boolean;
    };
    /** Geometry paired with the saved range, so reopening at another size preserves its proportion. */
    ratioLock?: {
        barSpacing: number;
        height: number;
    };
    /** Independent axis side and order. Omission restores the scale identity's default placement. */
    placement?: PriceAxisPlacement;
}
interface PaneState {
    /** Relative height weight among panes. */
    weight: number;
    priceScale: PriceScaleState;
    /** Secondary scales only; the right scale remains in priceScale for older readers. */
    scales?: Partial<Record<PriceScaleId, PriceScaleState>>;
    /**
     * Folded to its header strip. Omission restores the pane open; the primary
     * price pane (see `ChartState.primaryPane`) is always open, whatever it says.
     */
    collapsed?: boolean;
}
/** Validate and detach a pane snapshot before either a chart or workspace accepts it. */
declare function parsePaneState(input: unknown, allowLegacyPartial?: boolean): PaneState;
/** A series descriptor: enough to rebuild the shell, never the data. */
interface SeriesState {
    type: string;
    style: SeriesStyle;
    paneIndex: number;
    priceScaleId: PriceScaleId;
    /**
     * The transform the chart applies to the series (`Chart.setSeriesTransform`),
     * written only for a series that has one, so every other series saves what it
     * always did. Like the rest of this descriptor it is the host's to reapply.
     */
    transform?: SeriesTransformSpec;
}
interface IndicatorState {
    indicatorId: string;
    /** Stable workspace identity. Dependency templates remap it when copied. */
    instanceId?: string;
    /** Settings keys carrying declared study references, for portable template remapping. */
    studyInputs?: readonly string[];
    /** Whole-study scale override. Omission retains the descriptor's plot assignments. */
    priceScaleId?: PriceScaleId;
    /** Explicit plot assignments, including plots drawn on the price pane. */
    plotPriceScaleIds?: Readonly<Record<string, PriceScaleId>>;
    settings: IndicatorSettings;
    paneIndex: number;
    /** Omitted by older layouts, which restore the indicator as visible. */
    visible?: boolean;
    /**
     * The restrictions its host set, written only when a flag is false, so an
     * unrestricted study saves exactly what it always did. Omission restores it
     * unrestricted.
     */
    policy?: IndicatorPolicy;
    /**
     * `'underlying'` for a study computing on the bars under a transformed chart,
     * written only then, so a study on the chart's bars saves what it always did.
     * Omission restores it on the chart's bars.
     */
    barSource?: IndicatorBarSource;
}
interface ChartState {
    version: number;
    /** Visible logical range at save time. */
    viewport?: {
        from: number;
        to: number;
    } | undefined;
    barSpacing?: number | undefined;
    grid?: {
        vertLines: boolean;
        horzLines: boolean;
    };
    crosshairMode?: 'normal' | 'magnet';
    crosshairSnapToBar?: boolean;
    /** Only the primary series contributes to its scale's auto-fit. Omission preserves the current preference. */
    priceOnlyAutoScale?: boolean;
    /** Collapse only study legend rows. Omission preserves the current preference. */
    indicatorLegendCollapsed?: boolean;
    panes?: PaneState[] | undefined;
    /**
     * Slot in `panes` of the primary price pane, present only when it is not the
     * first (version 2). Every `paneIndex` in the state, of a pane, a series, a
     * study or a drawing, is a visual slot counted from the top, so this says
     * which of them is the price pane. Omitted with `panes` present means slot
     * 0, the only place a price pane could be before it could move. Needs
     * `panes`, and must name one of them.
     */
    primaryPane?: number;
    /** Informational: `restoreState` does not recreate these (it has no data). */
    series?: SeriesState[];
    indicators?: IndicatorState[];
    /**
     * Instance id of the study the price source paints directly above, present
     * only when the source was moved over a study on its pane. Omission, as in
     * every layout from before the source could move, paints the source behind
     * the studies. Read with `indicators`: a restore without them leaves it be.
     */
    sourceAbove?: string;
    /**
     * Opaque slot the drawing tier fills. The base engine round-trips it
     * untouched, so an app that persists state keeps drawings for free once the
     * tier is loaded.
     */
    drawings?: unknown;
    alerts?: AlertsDocument | undefined;
}
/** Runtime choices for restoring configuration without serializing callbacks. */
interface ChartRestoreOptions {
    /**
     * Keep the current formatter callback or default formatter on selected live
     * scales while recreating study series. Numeric saved scale settings still
     * apply. Selectors must name existing panes/scales; no callbacks are serialized.
     */
    preserveScaleFormats?: readonly {
        paneIndex: number;
        scaleId: PriceScaleId;
    }[];
}
/** What `restoreState` actually applied, so a caller can finish the job. */
interface RestoreReport {
    /** True when the payload was a recognised, applicable state object. */
    applied: boolean;
    /** Series descriptors found in the state: the app rebuilds these itself. */
    series: SeriesState[];
    /** Indicator instances recreated. */
    indicators: number;
    /** Set when the payload was rejected. */
    reason?: string;
}

/**
 * The render backend port (ARCHITECTURE.md section 3). A pane paints its series
 * through one of these and draws everything else (background, grid, primitives,
 * axes) on a 2D context itself, so a backend that rasterises series on the GPU
 * plugs in without the pane, the registry or any renderer learning about it.
 *
 * The port is deliberately narrow. It covers the one hot path a GPU can take
 * over, the per-frame series pass, and nothing the 2D context already does
 * well. Text, dashed lines, gradients and the drawing tools stay on canvas 2D.
 *
 * Which backend a chart gets is decided once, at construction, by the
 * `renderer` chart option or an injected factory. This module keeps the
 * registry that resolves the option; the shipped 2D backend registers itself
 * from its own module, and a later tier registers `webgl2` the same way, so
 * this file never has to import either.
 */

/** The backends the port can name. Only `canvas2d` ships in the base tier. */
type RenderBackendKind = 'canvas2d' | 'webgl2';
/**
 * The `renderer` chart option. `'auto'` picks `webgl2` when a backend of that
 * kind is registered and available on this device, and `canvas2d` otherwise,
 * which is every chart until that tier is imported.
 */
type RendererChoice = RenderBackendKind | 'auto';
/**
 * Paints one pane's series. One instance per pane: a backend holds a context
 * on that pane's base canvas, and contexts do not travel between canvases.
 */
interface IRenderBackend {
    readonly kind: RenderBackendKind;
    /**
     * Take the pane's base data canvas. `ctx2d` is the 2D context the pane
     * already holds on it, when it holds one: the canvas-2D backend draws on
     * exactly that object, so a frame is one context's op stream and not two
     * contexts' interleaved. A backend that owns the canvas outright ignores it.
     */
    mount(canvas: HTMLCanvasElement, ctx2d: CanvasRenderingContext2D | null): void;
    /** The pane's media size and pixel ratio changed. Sizes in CSS px. */
    resize(widthPx: number, heightPx: number, dpr: number): void;
    /** Start a frame on the base canvas, clearing it first when asked. */
    beginFrame(clear: boolean): void;
    /**
     * Paint one series. Same contract as `RendererEntry.draw` minus the context:
     * `priceToY` and `items[].x` are media px, and the renderer snaps to device
     * pixels with `dpr` itself.
     *
     * `items` and the objects in it belong to the pane, which rewrites them in
     * place the next time it draws this series, so a backend that keeps any of
     * it past that (a batch held across frames, say) copies what it keeps.
     */
    drawSeries(entry: RendererEntry, items: readonly DrawItem[], priceToY: (price: number) => number, barSpacing: number, dpr: number, style: SeriesStyle, rc: SeriesRenderContext): void;
    /** Finish the frame: flush whatever the backend batched. */
    endFrame(): void;
    /**
     * A 2D context for what the backend does not draw natively (grid, axes,
     * primitives), or null when the backend paints those itself.
     */
    overlay2d(): CanvasRenderingContext2D | null;
    /** Release the context and any GPU resources. The pane removes the canvas. */
    destroy(): void;
    /**
     * The GPU device behind the backend, when it has one. The chart reads it
     * after a frame (`backendDegradation`) to learn that the context has been
     * lost or never came up, and moves every pane to `canvas2d` for the rest of
     * the session, announced by the 'renderer:fallback' chart event. A backend
     * without a device is never degraded, so the 2D backend leaves it unset.
     */
    readonly device?: RenderDevice;
}
/**
 * What the chart needs to know about a GPU device. `available` is whether it
 * ever got a usable context (a driver that cannot run the program counts as
 * never having one), and `lost` is the window between a context loss and its
 * restoration.
 */
interface RenderDevice {
    readonly available: boolean;
    readonly lost: boolean;
}
/**
 * Why a chart left its GPU backend: the context went away mid-session, or
 * the device turned out to be unusable after the backend was built (the
 * program failed to compile on a live context).
 */
type RendererFallbackReason = 'context-lost' | 'unavailable';
/**
 * Whether a backend is painting through its own 2D fallback rather than the
 * path it was chosen for, and why. Null for a healthy backend and always for
 * `canvas2d`, which has nothing to fall back from.
 */
declare function backendDegradation(backend: IRenderBackend): RendererFallbackReason | null;
/**
 * Builds a backend for one pane, or returns null to decline: a `webgl2`
 * factory on a device without WebGL2 says so here, and the chart falls back
 * to `canvas2d` rather than painting nothing.
 */
type RenderBackendFactory = () => IRenderBackend | null;
/** Register (or replace) the factory for a backend kind. */
declare function registerRenderBackend(kind: RenderBackendKind, factory: RenderBackendFactory): void;
/**
 * Drop a registered backend. The 2D backend is refused: it is the fallback
 * every other choice lands on, and a chart with no backend paints nothing.
 */
declare function unregisterRenderBackend(kind: RenderBackendKind): boolean;
declare function registeredRenderBackends(): RenderBackendKind[];
/**
 * Turn the `renderer` option into a factory the chart calls once per pane.
 *
 * An explicit kind that nothing has registered throws, the way an unloaded
 * chart type does: the caller asked for a tier they did not import, and a
 * silent 2D fallback would hide that. A registered factory that declines at
 * run time (no WebGL2 on this device) falls back to `canvas2d` instead,
 * because that is a property of the machine, not a mistake in the code.
 */
declare function resolveRenderBackend(choice?: RendererChoice): RenderBackendFactory;
/** One backend instance for the given choice; see `resolveRenderBackend`. */
declare function createRenderBackend(choice?: RendererChoice): IRenderBackend;

/**
 * A pane is one vertically-stacked drawing region (price pane, volume pane,
 * indicator pane). It owns a base + top canvas (ARCHITECTURE.md §3.1) and a
 * price scale, and renders its series against the shared time scale + DataLayer.
 *
 * The canvas pile, bottom to top, and what lands on each:
 *
 *   base canvas (2D context, z 0): background, grid, the series pass, the
 *     normal-layer primitives, the axis strip (ladder, last-price tag, value
 *     tags, trading pills). Repainted on Light and Full.
 *   top canvas (2D context, z 1): the crosshair and its tags, and the
 *     top-layer primitives (drawings over the series or lifted for a drag,
 *     legends, tables, trading buttons, trade markers). Repainted on Cursor.
 *     Order and position lines are normal-layer price lines on the base canvas.
 *
 * Both canvases are the pane's height. Their backing store is `media x dpr`,
 * or the device-pixel box the browser reports where it reports one
 * (`CanvasLayer.setDeviceSize`), and the chart puts every pane boundary on a
 * device pixel, so neither canvas is ever stretched by a fraction of a pixel.
 * Under the rule between panes (`setSeparator`) they start one CSS pixel down
 * at a whole-number ratio and at the pane's top at a fractional one.
 *
 * A GPU backend adds no canvas to the pile. It rasterises the series pass on
 * one page-wide offscreen surface shared by every pane of every chart and
 * blits the result into the base canvas at `endFrame`, between the grid and
 * the normal-layer primitives, exactly where the 2D backend would have
 * painted them. That is why the grid, the primitives, the axes and the
 * overlay keep painting on 2D contexts whatever the backend is, why
 * `takeScreenshot` and the export still read `base.element`, and why a
 * browser's cap on live WebGL contexts never limits how many panes a page
 * can show.
 */

interface PaneRenderContext {
    timeScale: TimeScale;
    dataLayer: DataLayer;
    /** Restrict the primary series' scale, wherever that series is placed. */
    priceOnlyAutoScale?: boolean;
    primaryDataId?: SeriesRecord['dataId'] | undefined;
    dpr: number;
    priceAxisWidth: number;
    /** Left inset (px) reserved chart-wide for a left price axis; 0/absent when none. */
    leftAxisWidth?: number;
    /** Width of one price column; omitted preserves the legacy single-column layout. */
    axisColumnWidth?: number;
    /** Empty panes show the default column only when the whole chart has no scale users. */
    emptyPriceAxis?: boolean;
    timeAxisHeight: number;
    /** Only the bottom pane draws the time axis. */
    showTimeAxis: boolean;
    /**
     * The pane is folded to its header strip: its legend rows draw and answer
     * the pointer, and the time axis when it is the bottom pane. Series, grid,
     * price axes and every other primitive keep their data and state but
     * neither paint nor hit-test until it opens again.
     */
    collapsed?: boolean;
    /**
     * Draw one OHLC-preserving stick per device-pixel column once bars are
     * narrower than a column (§4.4); false draws every bar at every zoom.
     */
    conflate: boolean;
    /** Columns this many sticks wide (1 = one stick, the default; higher = coarser). */
    conflationFactor: number;
    /** Active palette: drives chrome, series defaults, and trade colors. */
    theme: ChartTheme;
    /** Draw the vertical (time) grid lines. */
    showVertGrid: boolean;
    /** Draw the horizontal (price) grid lines. */
    showHorzGrid: boolean;
    /**
     * The settings dialog's Canvas block (grid, crosshair, scales, margins).
     * Named `canvasOptions` rather than `canvas` so it is never mistaken for the
     * canvas element. Every field is an override: unset falls back to the theme.
     */
    canvasOptions?: CanvasOptions;
    /** Optional custom time label formatter (UTC seconds -> string). Omitted, labels follow `timezone`. */
    timeFormatter?: ((utcSeconds: number, tickMark?: TickMarkType) => string) | undefined;
    /**
     * IANA zone the time axis and crosshair label in. Absent means the shipped
     * default ('Asia/Kolkata'); an explicit `timeFormatter` outranks it, because a
     * host that formats its own labels has already decided the question.
     */
    timezone?: string;
    /**
     * The corner clock between the two axis strips. Absent draws nothing, which
     * is the shipped chart: it is chrome a host asks for.
     */
    sessionClock?: SessionClockOptions | undefined;
    /**
     * The countdown row inside the last-price tag. Absent leaves the tag the one
     * line it has always been.
     */
    barCountdown?: BarCountdownOptions | undefined;
    /** externalId of the primitive under the pointer (hover visual state). */
    hoverId?: string | null;
    hoverKey?: string | null;
    /** externalId of the line currently being dragged (active visual state). */
    dragId?: string | null;
    /**
     * Fill the pane with the theme background before anything else. Absent means
     * yes, which is every on-screen frame; the vector export turns it off for a
     * document that is meant to sit on the host's own page.
     */
    paintBackground?: boolean;
    /**
     * The series a series-band entry ('source:primary', 'indicator:<id>') paints
     * last on this pane, or undefined when it has none here. A primitive placed
     * above that entry paints right after it.
     */
    stackSlot?(entry: string): SeriesRecord | undefined;
}
declare class Pane {
    readonly element: HTMLElement;
    readonly base: CanvasLayer;
    readonly top: CanvasLayer;
    /** The rule over the pane's top edge, shown on every pane but the one against the chart's top. */
    private readonly _separator;
    /**
     * What paints this pane's series onto `base`. Everything else on that canvas
     * (background, grid, axes, primitives) the pane draws itself on the 2D
     * context the backend hands back, so a GPU backend only has to own the one
     * pass that is worth moving. Replaced through `setBackend` when the chart
     * falls back to 2D for the session.
     */
    private _backend;
    private _rightScale;
    /** Extra scales created on demand: left axis and a hidden overlay (volume). */
    private _leftScale;
    private readonly _overlayScales;
    private readonly _axisLayout;
    /**
     * Scales whose price-per-bar ratio is pinned, with the geometry the ratio was
     * last held against. A lock stores that geometry rather than a number,
     * because the ratio lives in the scale's *transformed* span (log prices are
     * not linear in price) and only the scale itself can measure that. Every
     * later change in bar spacing or pane height is answered by the opposite
     * change in the visible span, which needs no such measurement.
     */
    private readonly _ratioLocks;
    /** Relative height weight within the chart (price=1, volume≈0.3). */
    weight: number;
    private readonly _series;
    private readonly _primitives;
    private readonly _primitiveScales;
    /** Primitives painted inside the series band, each with the entry it sits directly above. */
    private readonly _stackAbove;
    /**
     * The chart's price source when it is on this pane. It is the instrument the
     * readout, the last-price line and the rebasing modes describe wherever it
     * paints: moved over a study it is no longer the first series here.
     */
    private _source;
    private _destroyed;
    private _width;
    private _height;
    /** Each series' draw items, reused from frame to frame (src/render/draw-items.ts). */
    private readonly _drawItems;
    /** Hit boxes of the primitives that declare one, kept while nothing they follow changes (src/render/hit-boxes.ts). */
    private readonly _hitBoxes;
    /** Bumped by every change on this pane that can move a primitive's context: series, scales, axes, primitives. */
    private _hitEpoch;
    /**
     * `backend` defaults to the 2D one so a pane built on its own (tests, a host
     * composing panes by hand) paints the way it always has; the chart passes
     * whatever its `renderer` option resolved to.
     */
    constructor(doc: Document, backend?: IRenderBackend);
    get backend(): IRenderBackend;
    /**
     * Swap the backend for the rest of the session: the old one releases its
     * resources, the new one takes the base canvas and its context and is told
     * the current size, so it is ready for the very next frame. The chart calls
     * this for every pane at once when a GPU backend degrades, so a chart never
     * paints half its panes one way and half the other.
     */
    setBackend(next: IRenderBackend): void;
    /**
     * Why the backend is painting through its 2D fallback rather than the path
     * it was chosen for, or null. Read by the chart after each frame; the
     * answer is a property of the device, so it is the same for every pane
     * sharing it.
     */
    get backendDegradation(): RendererFallbackReason | null;
    /**
     * The 'right' scale: the pane's primary axis, and the one a series maps to
     * unless it names another. A getter over a field rather than a plain readonly
     * property because `moveSeriesScale` swaps the two side scales, and the range,
     * mode, margins, tick size and formatter all belong to the axis being moved.
     */
    get priceScale(): PriceScale;
    addSeries(record: SeriesRecord): void;
    /** The PriceScale for a scale id, creating the left/overlay scale on first use. */
    private _scaleFor;
    /**
     * The scale for an id, created if this pane has never used it. A host acting
     * on one axis (a price-axis menu) needs the scale a side *would* use, not
     * only the ones series happen to occupy. Unused scales retain their settings,
     * while axis labels and columns follow the series that use them.
     */
    scaleFor(id: PriceScaleId): PriceScale;
    /** True when a series or explicitly bound primitive uses the named scale. */
    usesScale(id: PriceScaleId): boolean;
    axisPlacement(id: PriceScaleId): PriceAxisPlacement;
    setAxisPlacement(id: PriceScaleId, side: PriceAxisSide, order?: number): boolean;
    restoreAxisPlacements(saved: ReadonlyMap<PriceScaleId, PriceAxisPlacement>): void;
    private _retainPlacedScales;
    /** Active columns, nearest the plot first on each side. Vacant scales keep their rank. */
    visibleAxes(emptyPriceAxis?: boolean): readonly {
        scaleId: PriceScaleId;
        side: 'left' | 'right';
        order: number;
    }[];
    /**
     * Column x positions relative to the plot, shared by rendering and chart input.
     * A collapsed pane has none: it draws no ladder, so no axis gesture may land on it.
     */
    axisSlots(ctx: PaneRenderContext): readonly PriceAxisSlot[];
    /**
     * Move every series and bound primitive on one side to the other, axis and all.
     *
     * The two scale objects are swapped rather than their state copied across:
     * the range, mode, margins, tick size and any custom formatter are all
     * properties of the axis being moved, and copying would have to enumerate
     * every one of them (and gain a field each time one is added). What is left
     * behind carries nothing, so it is reset: keeping its range would label the
     * vacated strip with a ladder for prices that are no longer on that side.
     *
     * Refuses when the target side already carries series or bound primitives.
     * One side draws one axis, so a move onto an occupied side could only mean stacking two ladders
     * in one strip or silently sending the sitting tenant the other way, and
     * neither is what "move this axis to the left" asks for.
     */
    moveSeriesScale(from: 'right' | 'left', to: 'right' | 'left'): boolean;
    /** Whether this scale's price-per-bar ratio is currently pinned. */
    ratioLocked(id: PriceScaleId): boolean;
    /**
     * Pin (or release) the price-per-bar ratio of one scale, against the geometry
     * in force right now. A locked scale is manual by definition: autoscaling it
     * would re-fit the data every frame and undo the ratio being held.
     *
     * Locking a scale nothing has measured does nothing: there is no ratio to
     * hold yet, and switching it to manual would strand it on the 0..1
     * placeholder with nothing left to measure it. Returns whether the scale
     * ended up in the state asked for; releasing always succeeds.
     */
    setRatioLock(id: PriceScaleId, on: boolean, barSpacing: number, plotHeight: number): boolean;
    /** Release every ratio lock on this pane (resetting the view drops them). */
    clearRatioLocks(): void;
    /** The price scale a series maps to (for the series handle's `priceScale()`). */
    scaleOf(record: SeriesRecord): PriceScale;
    /**
     * Every scale this pane has actually created. The left and overlay scales are
     * built on demand, so a caller applying a pane-wide setting (plot margins,
     * label precision) needs the live set rather than all three ids.
     */
    scales(): PriceScale[];
    /** Detached scale configuration, excluding data baselines and runtime formatters. */
    scaleStates(): Partial<Record<PriceScaleId, PriceScaleState>>;
    /**
     * Scales configured with a visible column, including temporarily vacant ones.
     * Hidden overlays are deliberately excluded from chart-wide axis settings.
     *
     * That scale is positioned by whoever created it and by nobody else. A volume
     * histogram sitting in the bottom fifth of the price pane is an overlay with
     * `marginTop: 0.82`, and a chart-wide plot-margin change that swept it up
     * with the visible axes replaced that 0.82 with the dialog's number: the bars
     * grew to fill most of the pane, and putting the dialog back where it started
     * wrote 0.1, not the 0.82 nobody had recorded. Destructive and unrecoverable,
     * from a control that only claims to move the plot inside its own axes.
     */
    axisScales(): PriceScale[];
    /** True when a series or explicitly bound primitive uses the left axis. */
    hasLeftScale(): boolean;
    /** Remove a series record if present; returns true if it was found. */
    removeSeries(record: SeriesRecord): boolean;
    series(): readonly SeriesRecord[];
    /** Reorder a subset of series without disturbing unrelated sources. */
    reorderSeries(ordered: readonly SeriesRecord[]): void;
    /** Name the chart's price source on this pane, or null when it is elsewhere or gone. */
    setSourceSeries(record: SeriesRecord | null): void;
    /** The price source when it shows a price here, else undefined. */
    private _shownSource;
    /** Move one series to just before `before`, or to the end for null: the price source taking its place. */
    moveSeries(record: SeriesRecord, before: SeriesRecord | null): void;
    /** Reorder owned visuals within each renderer layer. */
    reorderPrimitives(ordered: readonly IPrimitive[]): void;
    /** Transfer ownership without ending an attached primitive's lifetime. */
    transferPrimitive(primitive: IPrimitive, target: Pane): boolean;
    /**
     * Paint an attached primitive in the series band, directly above the entry
     * `above` names, instead of in its own z-order band; null puts it back. An
     * entry with no series on this pane leaves it in its own band.
     */
    setPrimitiveStackAbove(primitive: IPrimitive, above: string | null): boolean;
    /** The entry a primitive is placed above, or null for one in its own band. */
    primitiveStackAbove(primitive: IPrimitive): string | null;
    /**
     * The placed primitives this frame can honour, keyed by the series each
     * paints right after, or null when there are none (the common case, which
     * then costs nothing).
     */
    private _slotted;
    /**
     * Whether `primitive` paints under `record`: in the band behind the series,
     * or in the series band after an earlier series. The chart asks when a
     * drawing and a series are both under the pointer, so the one painted on top
     * takes the context menu.
     */
    paintsBelowSeries(primitive: IPrimitive, record: SeriesRecord, ctx: PaneRenderContext): boolean;
    /**
     * Bind an attached primitive without detaching it. Null restores the right scale.
     * The owner schedules layout and repaint after finishing its resource transaction.
     */
    bindPrimitiveScale(primitive: IPrimitive, scaleId: PriceScaleId | null): boolean;
    /** Explicit binding, or null for an unbound or unavailable primitive. */
    primitiveScaleId(primitive: IPrimitive): PriceScaleId | null;
    /** Primitives attached to this pane, in draw order. */
    primitives(): readonly IPrimitive[];
    addPrimitive(primitive: IPrimitive, host: PrimitiveHost): void;
    /** Whether this pane currently holds `primitive`. */
    hasPrimitive(primitive: IPrimitive): boolean;
    /** Remove a primitive if present; returns true if it was found. */
    removePrimitive(primitive: IPrimitive): boolean;
    /** Detach every primitive (lifecycle cleanup) and remove the pane element. */
    destroy(): void;
    private _primitiveContext;
    private _boundPrimitiveContext;
    /** What a frame draws and the pointer can reach: only the legend rows of a collapsed pane. */
    private _live;
    /**
     * Topmost primitive hit at media-px (x,y) relative to this pane's plot.
     * `except` is left out, for the chart's corner mark, which yields to
     * anything else at the point.
     *
     * What paints over the series (the overlay band and the front) beats what
     * paints with or behind it (a drawing or a primitive placed in the series
     * band, a drawing sent behind the series, a bottom primitive), whatever the
     * distance: a box under an order line gives the press to the line, as the
     * eye does. On either side the nearest wins, then the one painted later in
     * band order, as `bestHit` ranks them. A hit painted by a primitive placed
     * in the series band names it (`paintedBy`), so the chart can rank it
     * against a series painted over it too.
     *
     * Two things keep a pointer move from asking every primitive. One that
     * declares a hit box is skipped when the point is outside it; the boxes are
     * kept while nothing they depend on changes (`_checkHitBoxes`). And the
     * walk stops at an exact hit in the front band, which nothing after it can
     * outrank: a later primitive would need a smaller distance than zero or a
     * band above the front one, and the order the walk visits primitives in
     * only ever breaks exact ties, which the first hit wins.
     */
    hitTestPrimitives(x: number, y: number, ctx: PaneRenderContext, except?: IPrimitive | null): PrimitiveHit | null;
    /** After an on-screen paint: what the pane shows now is what the key describes (src/render/hit-boxes.ts). */
    private _paintedHitBoxes;
    /**
     * Read the key the kept hit boxes are checked against: what a primitive's
     * render context can map with (the time scale and the bars' times, every
     * price scale here, the pane's size and axes, the pixel ratio, the readout
     * scale, hover and drag, the theme, the session calendar) and this pane's
     * own count of changes.
     */
    private _checkHitBoxes;
    /**
     * Draw the rule over this pane's top edge in `color` at device pixel ratio
     * `dpr`, or none with a null colour.
     *
     * At a whole-number ratio it is the pane's 1 px top border, with the
     * canvases starting under it: 1 px is whole device pixels there, and it is
     * the layout, and the pane-local y, that every earlier release showed. At a
     * fractional ratio a 1 px border is 1.25 or 1.5 device pixels, which starts
     * the canvases part way into a pixel, and the browser then resamples the
     * whole pane and blends the rule into it. There the rule is a box
     * `hairlineHeight(dpr)` tall, one device pixel, laid over the canvases'
     * first rows, and the canvases start on the pane's own top.
     */
    setSeparator(color: string | null, dpr: number): void;
    resize(width: number, height: number, dpr: number): void;
    /**
     * Lay the pane out at a size without touching its canvases. The vector
     * export paints at a size of the caller's choosing and then puts the live
     * size back; resizing a canvas clears it, so going through `resize` would
     * blank the screen until the next frame.
     */
    setLayoutSize(width: number, height: number): void;
    /**
     * Give every scale on this pane its plot height. Height is a *layout*
     * property, but it used to be set only inside the autoscale pass, so any
     * conversion between y and price before the first paint divided by zero and returned
     * ±Infinity. Layout is when the height is actually known.
     */
    setScaleHeights(plotHeight: number): void;
    private _layout;
    /** Autoscale each active price scale from its own series (independent axes). */
    autoscale(ctx: PaneRenderContext, progress?: number): boolean;
    private _autoscaleScale;
    /**
     * Hold every locked scale's price-per-bar ratio against the geometry it is
     * being painted into. A bar is `barSpacing` px wide and the plot is
     * `plotHeight` px tall, so a fixed ratio means the visible price span moves
     * with height / barSpacing: zoom in on time and the same slope needs fewer
     * prices in view to keep drawing at the same angle.
     */
    private _applyRatioLocks;
    /**
     * Multiply a scale's visible span by `factor` around the middle of the pane,
     * in the scale's own transformed space so a log axis scales by decades rather
     * than by price. Reading the endpoints back through `yToPrice` is what keeps
     * this transform-agnostic: y is linear in transformed space by construction,
     * whichever mode the scale is in.
     */
    private _scaleSpan;
    /** Close of the first visible bar on this scale: the rebasing modes quote against it. */
    private _firstVisibleValue;
    /**
     * Paint background + grid + series + axes on the base canvas, or into
     * `target` when given: the vector export runs this exact pass into a
     * serialising context, so what it produces is the frame and not a
     * re-description of it. Only the pane's own canvas is cleared first; a
     * target starts empty by construction.
     */
    paintBase(ctx: PaneRenderContext, target?: CanvasRenderingContext2D): void;
    /**
     * The series pass of `paintBase`, inside the plot clip: every series at the
     * frame's level of detail, each followed by the primitives slotted above it.
     * It hands back what the axis strips draw afterwards.
     */
    private _paintSeriesPass;
    /** The tick ladder of each axis strip, with the value tags that fit beside the readout tag. */
    private _paintAxisLadders;
    /** Paint into one axis strip, clipped to it, in the coordinates its tags use. */
    private _inAxisSlot;
    /**
     * Paint the primitives placed directly above `record`. A backend that
     * batches series flushes first, so the batch so far lands under them and
     * the series after them on top, the way it keeps a 2D fallback type in order.
     */
    private _paintSlot;
    /**
     * Top (overlay) canvas: top-layer primitives + crosshair. Cheap repaint on
     * cursor moves. `cross.x` is the shared plot x (vertical line, drawn in every
     * pane for a global crosshair); `cross.yLocal` is the price-line y for the
     * hovered pane only (null elsewhere); `cross.showTimeTag` draws the date tag
     * on the bottom pane's axis strip.
     */
    paintTop(cross: {
        x: number;
        yLocal: number | null;
        showTimeTag: boolean;
    } | null, ctx: PaneRenderContext, target?: CanvasRenderingContext2D): void;
    /**
     * The scale this pane's price readout belongs to: the one the chart's price
     * source maps to while it shows here, else the one its first visible price
     * series maps to, falling back to the right scale. A pane whose series
     * sit on the left axis has nothing on the right one, and reading the
     * crosshair price off it would tag the cursor with the 0..1 placeholder.
     */
    private _readoutScale;
    /**
     * The scale a price quoted for this pane belongs to: the crosshair readout,
     * the price on a click or a drag, and the chart's coordinate API all mean
     * this one. It is the right scale in every layout that has not moved an axis.
     */
    readoutScale(): PriceScale;
    /** Media-px y of a price on this pane's readout scale. The inverse of `yToPrice`. */
    priceToY(price: number): number;
    /** Price at a media-px y on this pane (crosshair magnet, click/drag readout). */
    yToPrice(y: number): number;
}

/**
 * Frame scheduler (ARCHITECTURE.md §3.2). Coalesces many `requestFrame()` calls
 * within a single tick into one `onFrame` invocation. The rAF function is
 * injectable so the loop is deterministically testable without a browser.
 */
type RafScheduler = (cb: () => void) => number;
type RafCanceller = (handle: number) => void;

/**
 * Crosshair magnet snapping (ARCHITECTURE.md §6). Pure helpers so the snap
 * logic is unit-testable; the crosshair's position lives in the chart's input
 * routing (core/chart-input.ts) and its drawing in render/crosshair.ts.
 */

type CrosshairMode = 'normal' | 'magnet';

type ShortcutScope = 'hover' | 'global';
type ShortcutPreset = 'default' | 'alt';
interface KeymapEntry {
    command: string;
    label: string;
    /** Canonical combos that trigger this command. */
    combos: string[];
}
interface CustomShortcut {
    command: string;
    label?: string;
    combos: string | string[];
    onTrigger: () => void;
}
interface ShortcutManagerOptions {
    preset?: ShortcutPreset;
    /** Rebind (`string`/`string[]`) or unbind (`null`) a command, still listed. */
    overrides?: Record<string, string | string[] | null>;
    /** Commands to unbind entirely (still listed in `list()`). */
    disabledCommands?: string[];
    customShortcuts?: CustomShortcut[];
    /** `hover` fires while the pointer is over the chart (or it is focused); `global` always. */
    scope?: ShortcutScope;
    /** Persist rebinds/preset to localStorage. */
    persist?: boolean;
    storageKey?: string;
    /** Force platform (⌘ vs Ctrl). Auto-detected when omitted. */
    isMac?: boolean;
}
interface ShortcutTriggerEvent {
    command: string;
    combo: string;
    isCustom: boolean;
}
interface KeyLike {
    code?: string | undefined;
    key?: string;
    ctrlKey?: boolean | undefined;
    metaKey?: boolean | undefined;
    altKey?: boolean | undefined;
    shiftKey?: boolean | undefined;
}
/** Parse a combo string into ordered modifiers + a key code, or null if invalid. */
declare function parseCombo(combo: string): {
    mods: string[];
    key: string;
} | null;
/** Canonical form of a combo (ordered modifiers), or `''` when invalid. */
declare function normalizeCombo(combo: string): string;
declare function isValidCombo(combo: string): boolean;
/** Platform-aware display string, e.g. `Alt+KeyR` -> `⌥ R` (mac) or `Alt + R`. */
declare function formatCombo(combo: string, isMac?: boolean): string;
/** Build a canonical combo from a keyboard event (`Mod` = ⌘ on mac, Ctrl elsewhere). */
declare function eventToCombo(e: KeyLike, isMac?: boolean): string;
declare function isReservedCombo(combo: string): boolean;
/** The built-in commands and their default bindings. */
declare const DEFAULT_KEYMAP: KeymapEntry[];
/** Alternate preset - overrides a few default bindings. */
declare const ALT_PRESET: Record<string, string[]>;
/** The set of built-in command ids (everything in DEFAULT_KEYMAP). */
declare const BUILTIN_COMMANDS: ReadonlySet<string>;
interface ShortcutListItem {
    command: string;
    label: string;
    combos: string[];
    isCustom: boolean;
    isDisabled: boolean;
}
declare class ShortcutManager {
    scope: ShortcutScope;
    private readonly _isMac;
    private readonly _persist;
    private readonly _storageKey;
    private _preset;
    private _overrides;
    private _disabled;
    private readonly _custom;
    private _entries;
    private _reverse;
    private readonly _listeners;
    private readonly _changes;
    constructor(options?: ShortcutManagerOptions);
    private _presetCombos;
    private _effectiveCombos;
    private _rebuild;
    /** Resolve a keyboard event to a command id (or null). */
    resolve(event: KeyLike): string | null;
    /** Resolve a combo string to a command id (or null). */
    handleKey(combo: string): string | null;
    /** Run a custom command's handler (built-ins are executed by the chart). */
    runCustom(command: string): boolean;
    emitTrigger(command: string, combo?: string): void;
    on(cb: (e: ShortcutTriggerEvent) => void): () => void;
    /**
     * Called after every change to the keymap (a binding set, disabled or reset,
     * a preset, a custom shortcut), so a hint drawn from it can follow. Returns
     * the unsubscriber.
     */
    onChange(fn: () => void): () => void;
    setBinding(command: string, combo: string | string[]): boolean;
    disable(command: string): void;
    resetBinding(command: string): void;
    resetAll(): void;
    setPreset(preset: ShortcutPreset): void;
    addCustom(shortcut: CustomShortcut): void;
    list(): ShortcutListItem[];
    state(): {
        preset: ShortcutPreset;
        overrides: Record<string, string[]>;
        disabled: string[];
    };
    private _after;
    private _save;
    private _load;
    /** True when a key event targets a text field and should be ignored. */
    static shouldIgnore(target: unknown): boolean;
}

/** Time-anchored event badges. Event feeds and detail loading belong to the host. */

interface EventDetailField {
    label: string;
    value: string;
}
/** A run of text in a rich block, shown as text: nothing in it is read as markup. */
interface EventDetailSpan {
    text: string;
    strong?: boolean;
    em?: boolean;
    /** Opened in a new tab when it is an absolute http or https URL; any other value shows the text alone. */
    href?: string;
}
/** One plain run, or runs with emphasis and links. */
type EventDetailInline = string | readonly EventDetailSpan[];
/** Rich detail content as structure, never as markup, so a feed's text cannot run as code. */
type EventDetailBlock = {
    type: 'heading' | 'paragraph';
    text: EventDetailInline;
} | {
    type: 'list';
    items: readonly EventDetailInline[];
};
/** Detail content. Every string is plain text: none is interpreted as markup. */
interface ChartEventDetails {
    summary?: string;
    fields?: readonly EventDetailField[];
    /** Headings, paragraphs and lists shown after the fields. */
    blocks?: readonly EventDetailBlock[];
}
interface ChartEvent {
    /** Original UTC timestamp in seconds, including times between candles. */
    time: number;
    type: 'earnings' | 'dividend' | 'split' | 'news' | string;
    label: string;
    color?: string;
    id?: string;
    title?: string;
    details?: string | ChartEventDetails;
    group?: string;
}
interface EventGroup {
    id: string;
    label: string;
    parentId?: string;
    /** Local visibility. A hidden ancestor also hides this group. */
    visible?: boolean;
}
interface EventMarkersOptions {
    /** Combine nearby badges. Defaults to false. */
    clustering: boolean;
    /** Maximum distance between cluster members in CSS pixels, from 1 to 200. Default 18. */
    clusterRadius: number;
}
interface EventMarkerDetails {
    id: string;
    cluster: boolean;
    /** Independent copies, in timestamp order. */
    events: ChartEvent[];
}
declare class EventMarkers implements IPrimitive {
    private _entries;
    private _groups;
    private _options;
    private _host;
    private _positions;
    private _hits;
    private _providedIds;
    private _duplicateIds;
    private _width;
    private _height;
    constructor(options?: Partial<EventMarkersOptions>);
    attached(host: PrimitiveHost): void;
    detached(): void;
    zOrder(): ZOrder;
    /** Replace data atomically. Caller mutations never change installed events. */
    setEvents(events: readonly ChartEvent[]): void;
    events(): ChartEvent[];
    /** Replace the hierarchy; duplicate IDs, missing parents and cycles are rejected. */
    setGroups(groups: readonly EventGroup[]): void;
    groups(): EventGroup[];
    setGroupVisible(id: string, visible: boolean): void;
    /** Ungrouped events and group IDs not yet configured remain visible. */
    isGroupVisible(id: string): boolean;
    setOptions(options: Partial<EventMarkersOptions>): void;
    options(): EventMarkersOptions;
    /** Resolve only hits from the latest layout, with independent member details. */
    detailsForHit(externalId: string): EventMarkerDetails | null;
    draw(ctx: CanvasRenderingContext2D, rc: PrimitiveRenderContext): void;
    hitTest(x: number, y: number): PrimitiveHit | null;
    private _invalidate;
}

/**
 * Time navigator (ARCHITECTURE.md section 8): hover-revealed zoom, reset and
 * one-bar step controls just above the time axis.
 *
 * Invisible until the pointer nears the bottom of the chart, so a clean chart
 * stays clean. It fades in and out rather than snapping, which is what keeps it
 * from reading as a glitch when the cursor crosses the reveal band.
 *
 * Reveal is driven by an explicit `setPointer` from the chart, **not** by
 * `rc.hoverId`. Hover ids come from `bestHit`, which picks the nearest primitive,
 * so a drawing or an order line near the bottom of the chart would win the
 * hit and silently hide the controls. Pointer position is the honest input here;
 * hit-testing still owns the buttons themselves.
 */

/** Command each button runs. These are `Chart` shortcut command ids. */
type TimeNavigatorAction = 'zoomOut' | 'zoomIn' | 'resetScale' | 'panLeftBar' | 'panRightBar';
interface TimeNavigatorOptions {
    /** Prefix for hit ids. Lets a host run more than one. */
    id: string;
    /** Buttons, left to right. A `null` inserts a gap between groups. */
    buttons: readonly (TimeNavigatorAction | null)[];
    /** Button box size in media px. */
    size: number;
    /** Gap between buttons, and the wider gap a `null` produces. */
    gap: number;
    groupGap: number;
    /** Distance from the bottom of the plot to the bottom of the buttons. */
    bottomMargin: number;
    /**
     * Height of the reveal band above the plot bottom. The pointer anywhere in
     * this band brings the controls in.
     */
    revealHeight: number;
    /** Seconds the fade takes. 0 disables the animation. */
    fadeSeconds: number;
    /** Tooltip label overrides per action. */
    labels: Partial<Record<TimeNavigatorAction, string>>;
    /** Optional keyboard hint shown next to the label, e.g. `"Ctrl + −"`. */
    hints: Partial<Record<TimeNavigatorAction, string>>;
    /** Show the tooltip above the hovered button. */
    showTooltip: boolean;
    font: number;
    radius: number;
    zOrder: ZOrder;
}
declare const DEFAULT_TIME_NAVIGATOR_OPTIONS: TimeNavigatorOptions;
declare class TimeNavigator implements IPrimitive {
    private _opts;
    private _host;
    /** Button geometry from the last paint, in media px relative to the plot. */
    private _boxes;
    /** Pointer in plot-local media px, or null when it left the pane. */
    private _pointer;
    /** 0 hidden, 1 fully shown. Eased toward the target each frame. */
    private _opacity;
    private _lastFrameMs;
    private _now;
    constructor(opts?: Partial<TimeNavigatorOptions>, now?: () => number);
    attached(host: PrimitiveHost): void;
    detached(): void;
    zOrder(): ZOrder;
    options(): TimeNavigatorOptions;
    setOptions(patch: Partial<TimeNavigatorOptions>): void;
    /**
     * Tell the navigator where the pointer is, in plot-local media px. `null`
     * when it leaves. The chart calls this; hosts driving their own navigator
     * should too.
     */
    setPointer(p: {
        x: number;
        y: number;
    } | null): void;
    /** True when the pointer is inside the reveal band. */
    private _revealed;
    private _targetOpacity;
    /** Top of the reveal band, recomputed each paint from the plot height. */
    private _revealTop;
    /** Whether the fade is still running: the chart keeps painting while true. */
    animating(): boolean;
    draw(ctx: CanvasRenderingContext2D, rc: PrimitiveRenderContext): void;
    /** Which button the pointer is over, or null. */
    private _hoveredAction;
    private _button;
    private _tooltip;
    /**
     * Buttons hit-test only while visible, so a hidden navigator never steals a
     * click from the chart body underneath it.
     */
    hitTest(x: number, y: number): PrimitiveHit | null;
}

/**
 * Logo / brand watermark (ARCHITECTURE.md §8). Draws a small image (or an
 * already-decoded bitmap) faintly in a corner of the plot, the way charting
 * apps stamp a product/brand mark. Because it draws on the canvas it is captured
 * by `chart.takeScreenshot()`, and an optional `tint` recolors the opaque pixels
 * so a single-color logo reads on both dark and light themes.
 *
 * Pass a `src` (URL or data URI) or a preloaded `image` for custom artwork.
 * Without a source, the original OpenAlgo glyph draws without a network request.
 */

type WatermarkPosition = 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right' | 'center';
interface LogoWatermarkOptions {
    /** Image URL or data URI. Ignored when `image` is provided. */
    src?: string;
    /** A preloaded image/bitmap to draw (skips loading). */
    image?: CanvasImageSource & {
        width: number;
        height: number;
    };
    /** Corner (or center) to anchor to. Default `bottom-right`. */
    position?: WatermarkPosition;
    /** Gap from the plot edges in px. Default `12`. */
    margin?: number;
    /** Rendered logo height in px; width follows the source aspect. Default `28`. */
    height?: number;
    /** 0..1. Default `0.7`. */
    opacity?: number;
    /** Recolor the opaque pixels to this color (e.g. a faint theme gray). */
    tint?: string;
    /** Layer order vs the series. Default `top`. */
    zOrder?: ZOrder;
    /**
     * Text revealed to the right of the mark on hover: "Chart by OpenAlgo".
     * The mark alone is what sits on the chart at rest; the wording is only
     * needed when someone looks at it, so it unrolls rather than occupying the
     * corner permanently. Omit for a plain, non-interactive mark.
     */
    label?: string;
    /** Hit-test id, so the chart can report hover. Default `watermark`. */
    id?: string;
    /** Reveal duration in seconds. Default 0.18. */
    revealSeconds?: number;
    /** Label colour. Defaults to `tint`, then the theme's text. */
    labelColor?: string;
    /** Label size in media px. Default 12. */
    fontSize?: number;
    /**
     * Rounded plate behind the mark and label. Without one the wording sits
     * straight on the candles and is unreadable wherever the chart is busy.
     * Defaults to a translucent form of the theme background; `'none'` omits it.
     */
    background?: string;
    /** Plate border. Defaults to the theme's axis line. `'none'` omits it. */
    borderColor?: string;
    /** Plate corner radius in media px. Default 6. */
    radius?: number;
    /**
     * Plate padding around the lockup, in media px. A number pads both axes;
     * `{ x, y }` pads them separately. Default `{ x: 7, y: 4 }`, which keeps the
     * revealed label clear of the plate edge.
     *
     * Use it to size the resting plate exactly: a 40px mark with `padding: 2.5`
     * sits in a 45x45 square. The plate widens as the label unrolls; the padding
     * is what stays constant.
     */
    padding?: number | {
        x: number;
        y: number;
    };
    /**
     * Where the mark points. A canvas cannot hold an anchor, so this only marks
     * it as clickable: the hit reports a pointer cursor, and the host opens
     * {@link href} from its own click handler.
     *
     * Given a bare URL, `href()` appends UTM parameters naming the page the chart
     * is embedded in, so the referral is attributable. Pass a URL that already
     * has a query string to compose your own and skip that entirely.
     */
    href?: string;
    /** `utm_medium` for the composed link. Default `oac-link`. */
    utmMedium?: string;
    /** `utm_campaign` for the composed link. Default `oac-chart`. */
    utmCampaign?: string;
}
/** Top-left placement (in media px) of a `w x h` logo within a `plotW x plotH` plot. */
declare function watermarkRect(position: WatermarkPosition, margin: number, w: number, h: number, plotW: number, plotH: number): {
    x: number;
    y: number;
    w: number;
    h: number;
};
declare class LogoWatermark implements IPrimitive {
    private _opts;
    /** 0..1 reveal progress, eased toward hover state each frame. */
    private _reveal;
    private _lastFrameMs;
    /** Label width in media px, measured once the font is known. */
    private _labelW;
    private readonly _now;
    private _host;
    private _img;
    private _ready;
    private _autoHeight;
    private _tintCanvas;
    private _tintKey;
    constructor(opts?: LogoWatermarkOptions);
    attached(host: PrimitiveHost): void;
    detached(): void;
    zOrder(): ZOrder;
    autoscaleInfo(): null;
    /** Live restyle. Pass a new `src`/`image` to swap the logo. */
    setOptions(patch: Partial<LogoWatermarkOptions>): void;
    /**
     * The mark's box, in media px. The label unrolls to its right, so the hit
     * area is the mark alone: hovering the revealed text keeps it open because
     * the pointer is still within the widened box below.
     */
    private _rect;
    private _builtin;
    /**
     * The link to open, with attribution appended. A caller who supplied their
     * own query string gets it back untouched.
     */
    href(): string | undefined;
    hitTest(x: number, y: number, rc: PrimitiveRenderContext): PrimitiveHit | null;
    draw(ctx: CanvasRenderingContext2D, rc: PrimitiveRenderContext): void;
    /**
     * The single colour the mark and its label share. Either option can set it;
     * they follow each other so the pair never renders in two shades.
     */
    private _ink;
    /** Ease the reveal toward the hover state, asking for frames while moving. */
    private _advanceReveal;
    /** Recolor the opaque logo pixels to `color` at (dw x dh) device px, cached. */
    private _tinted;
}

/**
 * A word stamped faintly across the plot, to say what mode the chart is in.
 *
 * The case it exists for is replay: a chart showing a past session looks exactly
 * like a chart showing the present one, and a trader who forgets which they are
 * looking at can read a live decision off history. The mark is large, centred
 * and static on purpose. It is not decoration and not a brand: it is the answer
 * to "am I looking at the market or a recording of it", and it has to be
 * readable without being looked for.
 *
 * Distinct from `LogoWatermark`, which is a small corner image with a hover
 * label. This one draws text, takes no interaction, and reports no hit, so it
 * never intercepts a click meant for the chart under it.
 */

interface TextWatermarkOptions {
    /** The word itself. Kept short: it is read at a glance, not studied. */
    text: string;
    /**
     * Cap height in px at the plot's natural size, scaled down on a narrow chart
     * so a long word never runs past the edges. Default 64.
     */
    fontSize?: number;
    /** 0..1. Default 0.08, which reads as present without competing with bars. */
    opacity?: number;
    /** Default a neutral grey that works on either theme. */
    color?: string;
    font?: string;
    /**
     * Below the series by default. A mode marker that covered the candles would
     * be worse than no marker, since the candles are what the mode is about.
     */
    zOrder?: ZOrder;
    id?: string;
}
declare class TextWatermark implements IPrimitive {
    private _host;
    private _opts;
    constructor(opts: TextWatermarkOptions);
    zOrder(): ZOrder;
    attached(host: PrimitiveHost): void;
    detached(): void;
    autoscaleInfo(): null;
    /** Never hit: the mark must not eat a click aimed at the chart beneath it. */
    hitTest(): null;
    setOptions(patch: Partial<TextWatermarkOptions>): void;
    draw(ctx: CanvasRenderingContext2D, rc: PrimitiveRenderContext): void;
}

/**
 * The public option, event and payload types of the Chart class, with the two
 * small values published beside them (`compactVolume`, `PRICE_SCALE_MODES`).
 *
 * They sit in their own module so chart.ts can be split into internal pieces
 * without each piece importing the orchestrator for a type. chart.ts re-exports
 * every name declared here, so an import of './chart' or '../core/chart' keeps
 * the same names at the same path. `ChartClickEvent` and `PriceAxisState` are
 * declared in chart.ts instead: they carry deprecated members, and the
 * deprecation inventory (COMPATIBILITY.md and its test) names chart.ts as
 * their home.
 */

/** What a wheel zoom holds still. */
type ZoomAnchor = 'cursor' | 'right';
/** What a double-click on a pane does. See {@link ChartOptions.doubleClick}. */
type DoubleClickAction = 'reset' | 'maximize' | 'none';
/**
 * The `dblclick` event. `paneIndex` is the pane under the pointer. A listener
 * that acts on the double-click itself (opening a text editor for the selected
 * drawing, say) sets `handled` to true, and the chart's own action is skipped
 * for that press.
 */
interface DoubleClickEvent {
    paneIndex: number;
    x: number;
    y: number;
    handled: boolean;
}
/** Optional background text. Blank text follows the chart's symbol and interval. */
interface ChartWatermarkOptions extends Partial<TextWatermarkOptions> {
    visible?: boolean;
}
/** A pane's plot area in container media px, from `Chart.plotRect`. */
interface PlotRect {
    left: number;
    top: number;
    width: number;
    height: number;
}
/** Defensive branding snapshot emitted synchronously after setBranding as `branding:changed`. */
type BrandingChangedEvent = false | LogoWatermarkOptions;
/**
 * Which corporate-action / news markers the chart draws. Every type defaults to
 * on; an unlisted type is always drawn. Only filters the strip the chart owns
 * (`setEvents`), not an `EventMarkers` a host drives itself.
 */
interface ChartEventOptions {
    earnings?: boolean;
    dividend?: boolean;
    split?: boolean;
    news?: boolean;
}
/**
 * Chrome that lives on the axis strips rather than in the plot: a live clock in
 * the corner where the two axes meet, and a countdown to the current bar's close
 * inside the last-price tag.
 *
 * Both default to off. Neither is a thing a chart should start showing because
 * it upgraded, and a chart that sets none of this draws the axes it always drew.
 */
interface AxisChromeOptions {
    /**
     * Live clock in the corner between the price and time axes. `true` takes the
     * defaults; the object form is there for the one thing worth choosing, the
     * second row carrying the zone's offset from UTC.
     */
    sessionClock?: boolean | {
        showOffset?: boolean;
    };
    /** Second row in the last-price tag counting down to the bar's close. */
    barCountdown?: boolean;
    /**
     * Wall-clock UTC seconds. Both readings are times of day, so neither can use
     * `now`, which is a monotonic animation clock and not a calendar. Defaults to
     * the system clock; pass the feed's clock to keep a delayed or replayed chart
     * honest about what time its data thinks it is.
     */
    clock?: () => number;
}
/** Options for `Chart.exportSVG`. */
interface ExportSvgOptions {
    /**
     * Document width in media px. Absent means the live chart width. A different
     * size lays the chart out afresh for the export (the same bar spacing over a
     * wider or narrower plot, every scale re-measured for its new height) and
     * puts the live layout back before returning.
     */
    width?: number;
    /** Document height in media px. Absent means the live chart height. */
    height?: number;
    /**
     * Paint the theme background under the chart. Default true. Off, the
     * document has no ground of its own and takes the colour of whatever page
     * it is placed on, which is what an embedded figure usually wants.
     */
    background?: boolean;
    /**
     * The pixel ratio the paint runs at. Only 1 is accepted: SVG has no device
     * pixels, and a renderer asked for 2 would snap its hairlines to half-media-
     * pixel edges that scale as a blur. Named so the option reads the same as
     * the PNG path and a future scaled export has a place to land.
     */
    dpr?: 1;
}
/** Preferences for pointer panning and the view restored by reset. */
interface ChartNavigationOptions {
    /** Allow user plot translation, including touch, wheel and keys. Default: true. */
    panEnabled?: boolean;
    /** Allow user zoom and reset/fit actions. Programmatic setters remain available. Default: true. */
    zoomEnabled?: boolean;
    /** Mouse and pen plot drags. Touch gestures retain two-axis panning. Default: both. */
    mousePan: 'horizontal' | 'both';
    /** Latest bars to show initially and on reset. 0 fits all loaded bars (default). */
    defaultVisibleBars: number;
    /** Initial/reset spacing in CSS pixels. Positive values override the bar count; 0 disables it. */
    defaultBarSpacing?: number;
}
interface ChartOptions {
    document?: Document;
    pixelRatio?: () => number;
    /**
     * The frame scheduler, for deterministic tests. Supplied, it runs every
     * frame the chart paints, the one after a resize or a new pixel ratio
     * included; with the default one the chart paints those inside the
     * callback that reports them, so a resize never shows a cleared canvas.
     */
    raf?: {
        schedule: RafScheduler;
        cancel?: RafCanceller;
    };
    /** Full palette; pass `lightTheme` (the default), `darkTheme`, or a custom ChartTheme. */
    theme?: ChartTheme;
    priceAxisWidth?: number;
    timeAxisHeight?: number;
    /** Initial horizontal scale configuration, including spacing limits for wide profiles. */
    timeScale?: Partial<TimeScaleOptions>;
    /** Saved navigation preferences, also exposed in the Axes settings tab. */
    navigation?: Partial<ChartNavigationOptions>;
    /**
     * Where indicator legend rows start inside **one** pane, in media px. A host
     * that draws its own overlay in a pane's top-left corner (an OHLC readout, a
     * symbol line, a trade panel) needs to push these clear of it, or the rows
     * land underneath and their settings / close buttons become invisible and
     * unclickable.
     *
     * It follows the primary price pane, which is at the top until something
     * moves it below its studies: a readout of the price belongs to the price
     * pane, wherever it sits. Maximizing a study pane hides the price pane, so
     * the maximized pane, now in the same corner, inherits the offset. Every
     * other pane keeps the default corner, because a short study pane would
     * have its legend pushed off it entirely.
     *
     * Defaults to `{ top: 6, left: 8 }`.
     */
    legendOffset?: {
        top?: number;
        left?: number;
    };
    /**
     * Crosshair behaviour. 'normal' (default): the cross follows the pointer
     * exactly. 'magnet': the horizontal line snaps to the nearest O/H/L/C of the
     * bar under the cursor (price pane only).
     */
    crosshairMode?: CrosshairMode;
    /** Snap the vertical crosshair to the nearest primary bar's center. Default false; independent of the price magnet. */
    crosshairSnapToBar?: boolean;
    /** Fit the primary series' scale to that series only. Does not enable auto-fit. Default false. */
    priceOnlyAutoScale?: boolean;
    /** Collapse study legend rows into a count without hiding their plots or stopping calculations. Default false. */
    indicatorLegendCollapsed?: boolean;
    /**
     * Optional chrome on the axis strips: the corner clock and the bar-close
     * countdown. Both are off unless asked for, so a chart that omits this block
     * draws the axes it always drew.
     */
    axisChrome?: AxisChromeOptions;
    /**
     * Time source for kinetic animation (defaults to performance.now).
     * `WidgetOptions.now` is a different clock (the widget's wall clock, epoch
     * milliseconds) and is not passed to a widget's chart.
     */
    now?: () => number;
    /**
     * Ease a wheel zoom over a few frames instead of landing the whole step on
     * one. Default true, matching the inertial pan a flick already gets: a chart
     * that glides when panned and jumps when zoomed reads as two different
     * instruments. Off restores the single-frame step.
     */
    animZoom?: boolean;
    /** Ease automatic price ranges during navigation. Defaults to animZoom (true). */
    animAutoscale?: boolean;
    /**
     * What a wheel zoom holds still: the bar under the cursor, or the right edge
     * (the latest bar). Default `'cursor'`, which is what the chart has always
     * done. `'right'` keeps the most recent bar pinned while history stretches
     * away from it, which is what a live chart usually wants.
     */
    zoomAnchor?: ZoomAnchor;
    /**
     * What a double-click on a pane does. `'reset'` (the default, and what the
     * chart has always done) fits every loaded bar on screen and autoscales,
     * which also lands the viewport on the oldest bar and so wakes a history
     * loader. `'maximize'` gives the pane the whole stack, and a second
     * double-click puts the stack back, which is what a multi-pane terminal
     * usually wants from the gesture. `'none'` only emits the event. While a
     * tool is being placed a double-click finishes the shape whatever this says.
     */
    doubleClick?: DoubleClickAction;
    /**
     * Level of detail for a zoomed-out chart (§4.4). Default true: once bars are
     * narrower than the stick a candle is drawn with (about one CSS px; exactly
     * one at a whole pixel ratio), the bars sharing a device-pixel column are
     * drawn as one OHLC-preserving stick (open of the first, close of the last,
     * the column's high and low), and line and histogram series keep each
     * column's first, lowest, highest and last values. A frame then costs the
     * plot's width, not the number of bars in view. Above that spacing nothing
     * changes. Data, autoscale, indicators and the crosshair still see every
     * bar. `false` draws every bar at every zoom.
     *
     * The default time scale never zooms out past one bar per CSS px
     * (`timeScale.minBarSpacing: 1`), so it matters once a host lowers that floor.
     */
    conflate?: boolean;
    /**
     * Column width in sticks (default 1). Higher merges into wider columns, and
     * from spacings that many times wider, for a coarser, cheaper picture.
     */
    conflationFactor?: number;
    /**
     * Which backend paints the series. `'canvas2d'` (the default) is the 2D
     * path every chart has always drawn with. `'webgl2'` asks for the GPU
     * backend: it throws when the tier that registers it has not been imported
     * (a missing import is a mistake in the code), and on a device where that
     * tier finds no WebGL2 it falls back to `canvas2d` with one console warning
     * (a property of the machine the host should still hear about). `'auto'`
     * takes `webgl2` when it is registered and available on this device and
     * `canvas2d` otherwise, silently. Decided once, at construction; read what
     * was actually chosen from `rendererKind`. A GPU backend that loses its
     * context later moves the chart to `canvas2d` for the rest of the session
     * and emits 'renderer:fallback'.
     */
    renderer?: RendererChoice;
    /**
     * Build the backend directly, one call per pane, bypassing `renderer` and
     * the registry. For a host bringing its own backend, and for tests that
     * want to see what the pane asks a backend to paint. A factory that returns
     * null gets the 2D backend for that pane.
     */
    renderBackend?: RenderBackendFactory;
    /**
     * Grid lines: visibility (both default to true) plus per-axis colour, dash,
     * width and spacing. Unset colours/dashes fall through to the theme.
     */
    grid?: Partial<GridOptions>;
    /**
     * The settings dialog's Canvas block: grid, crosshair, scale text/lines and
     * plot margins. Every field is an override of the theme, so a later
     * `setTheme` still restyles anything the dialog did not touch.
     */
    canvas?: CanvasOptions;
    /** Per-field status-line switches applied to every pane legend on the chart. */
    statusLine?: LegendStatusLineOptions;
    /**
     * Square side of a legend action button in media px. Default 16, held to
     * 12..28. Applied to every pane legend, because the rows stack against it.
     */
    legendIconSize?: number;
    /** Accessible label for the chart container (screen readers). */
    ariaLabel?: string;
    /**
     * Keyboard shortcuts. Pass a configured `ShortcutManager`, options to build
     * one, or `false` to disable keyboard control. Defaults to the built-in keymap.
     */
    shortcuts?: ShortcutManager | Partial<ShortcutManagerOptions> | false;
    /**
     * Custom price formatter for every pane's axis tick labels, the last-price
     * tag, and price-line labels. e.g. `(p) => '$' + p.toFixed(2)`. When omitted,
     * a tick-size-aware `toFixed` is used. Change it later via `setPriceFormatter`.
     */
    priceFormatter?: (price: number) => string;
    /**
     * Default price-scale options applied to every pane (tick size `minMove`,
     * `mode: 'linear' | 'logarithmic' | 'percentage' | 'indexed-to-100'`,
     * `inverted`, and top/bottom margins). `minMove` is the instrument's, so it
     * lands only on panes that quote it, see `setPriceScaleOptions`.
     * Tune a single pane later via `chart.panes()[n].priceScale.setOptions(...)`.
     */
    priceScale?: Partial<PriceScaleOptions>;
    /**
     * Where an indicator gets another instrument's bars. The engine is handed
     * one symbol's history and owns no transport, so a study that compares
     * against a benchmark asks through this and the host answers from wherever
     * it keeps candles. Change it later with `setBarsProvider`. Without one an
     * indicator's `requestBars` rejects, and the study reports itself
     * unsupported rather than drawing something invented.
     */
    barsProvider?: IndicatorBarsProvider | IndicatorBarsProviderAccess;
    /**
     * Custom time-axis and crosshair label formatter (receives UTC seconds). When
     * omitted, labels follow `timezone`, which is IST unless set. e.g. for UTC:
     * `(s) => new Date(s * 1000).toISOString().slice(11, 16)`.
     */
    timeFormatter?: (utcSeconds: number, tickMark?: TickMarkType) => string;
    /**
     * IANA zone the time axis and crosshair label in, e.g. 'America/New_York' or
     * 'Europe/London'. Defaults to 'Asia/Kolkata': a caller who passes nothing
     * gets exactly the labels the chart produced before this option existed.
     *
     * An IANA name and not a fixed offset, because a zone that observes DST is a
     * different offset in July than in January and a fixed one is silently wrong
     * for half the year. Change it at runtime with `setTimezone` when the terminal
     * moves between an NSE symbol and a US one. An explicit `timeFormatter`
     * outranks this: a host that formats its own labels has settled the question.
     *
     * Throws if the runtime does not recognise the name.
     */
    timezone?: string;
    /**
     * Hover-revealed zoom / step controls above the time axis, as terminals show.
     * `true` by default: they stay invisible until the pointer nears the bottom
     * of the chart. Pass `false` to drop them, or an options object to restyle.
     */
    timeNavigator?: boolean | Partial<TimeNavigatorOptions>;
    /** OpenAlgo corner mark by default. Pass false to hide it or options for custom branding. */
    branding?: boolean | LogoWatermarkOptions;
    /** Background text, off by default. Blank text follows setDataContext. */
    watermark?: boolean | ChartWatermarkOptions;
    /**
     * Let the primary price pane leave the top of the stack: below its studies
     * through `movePane`, `setPrimaryPaneIndex`, a study pane's own up and down
     * controls, or a restored layout that saved it lower down. Default false,
     * which keeps the price pane pinned at slot 0 exactly as every earlier
     * release did: `movePane` refuses to move or displace it,
     * `setPrimaryPaneIndex` returns false, and `restoreState` refuses a layout
     * that puts it anywhere else, so an explicit pane 0 always means the price.
     *
     * Opt-in because a host that passes 0 to mean the price pane (a price or
     * order line, `coordinateToPrice(y, 0)` pricing a right-click order, a price
     * alert check, `panes()[0]`) would read a study's units the moment a user
     * moved a study above the candles. Before turning it on, drop those explicit
     * zeros or ask `primaryPaneIndex()`, and follow `paneMoved`. Decided once,
     * at construction; read it back with `movablePrimaryPane()`. `createWidget`
     * and `createChartGrid` hand it to their charts as given, off by default too.
     */
    movablePrimaryPane?: boolean;
}
/** Where `chart.addIndicator` puts a study, and how the host holds it. */
interface AddIndicatorOptions {
    /** The pane: an existing one, or `panes().length` for a new one. Default: the price pane for an overlay, a new pane otherwise. */
    paneIndex?: number;
    /** The scale the study's plots map to on that pane. */
    priceScaleId?: PriceScaleId;
    /** A scale per plot key, for a study whose plots belong on different scales. */
    plotPriceScaleIds?: Readonly<Record<string, PriceScaleId>>;
    /** What the user may do with the study; see `IndicatorPolicy`. */
    policy?: IndicatorPolicy;
    /** The id to give the study, to bring a removed one back under its own identity. */
    instanceId?: string;
    /** On a transformed chart, `'underlying'` computes on the host's bars instead of the elements drawn. */
    barSource?: IndicatorBarSource;
}
/** The chart options `chart.applyOptions` changes at runtime, without recreating the chart. */
interface ChartApplyOptions {
    theme?: ChartTheme;
    grid?: Partial<GridOptions>;
    canvas?: CanvasOptions;
    statusLine?: LegendStatusLineOptions;
    legendIconSize?: number;
    priceScale?: Partial<PriceScaleOptions>;
    priceFormatter?: ((price: number) => string) | null;
    timeFormatter?: ((utcSeconds: number, tickMark?: TickMarkType) => string) | undefined;
    timezone?: string;
    crosshairMode?: CrosshairMode;
    crosshairSnapToBar?: boolean;
}
interface AddSeriesOptions {
    /**
     * Target pane index. Omitted means the primary price pane wherever it sits
     * (slot 0 until something moves it, see `Chart.primaryPaneIndex`). Higher
     * panes are created on demand. An index that is not a whole number of 0 or
     * more throws a `RangeError` and adds nothing.
     */
    paneIndex?: number;
    /** Style overrides merged onto the chart type's defaults. */
    style?: SeriesStyle | undefined;
    /**
     * Which price axis this series maps to. 'right' (default) and 'left' each draw
     * an axis and autoscale independently; '' is a hidden overlay scale (no axis)
     * for a volume histogram inside the price pane.
     */
    priceScaleId?: PriceScaleId | undefined;
    /**
     * Value formatting applied to this series' price scale (axis + crosshair tag):
     * `price` (tick-size precision), `volume` (compact 1.2K / 3.4M / 5.6B),
     * `percent` (a `%` suffix at a fixed precision), or a `custom` formatter.
     *
     * `percent` suffixes the value as it stands and does **not** scale it: a study
     * that already returns 0..100 reads `62.24%`, and one that returns a 0..1
     * fraction reads `0.62%`. Multiplying here would put the axis and the plotted
     * value into disagreement, which is the one thing a formatter must never do.
     */
    priceFormat?: PriceFormat | undefined;
    /**
     * Have the chart apply a price-driven transform to the bars this series is
     * given, as `Chart.setSeriesTransform` does. The series draws with `type`,
     * so a host passes the transform's own renderer (`getSeriesTransform(type).renderer`)
     * unless it wants another. An unknown type or invalid option throws.
     */
    transform?: SeriesTransformSpec;
}
/** Compact volume/number formatter (1.2K / 3.4M / 5.6B). */
declare function compactVolume(v: number): string;
/**
 * Emitted on every crosshair move (and `null` fields on pointer-leave) so a host
 * can render an OHLC legend / tooltip. `bar` is the hovered bar of the primary
 * price series; `point` is container-relative media px for positioning a
 * floating tooltip. See `subscribeCrosshairMove`.
 */
interface CrosshairMoveEvent {
    /** Linked readouts have no physical pointer position and are not pointer gestures. */
    source?: 'linked';
    /** UTC seconds of the hovered bar, or null when off the data / pointer left. */
    time: number | null;
    /** Logical index under the cursor, or null. */
    index: number | null;
    /** Price under the cursor on the hovered pane, or null. */
    price: number | null;
    /** Hovered bar of the primary (first) price series, or null. */
    bar: Bar | null;
    /** Cursor position in container media px, or null on leave. */
    point: {
        x: number;
        y: number;
    } | null;
    /** Pane under the cursor, or null on leave. */
    paneIndex?: number | null;
    /**
     * True while a pointer is held. Placement mode swallows the pan path, so
     * this is the only way to tell a hover from a drag while it is still
     * happening. Absent on the all-null leave payload.
     */
    pressed?: boolean;
    /** Modifier keys held during the move. Absent on the leave payload. */
    modifiers?: PointerModifiers;
    /** Device behind the move. Absent on the leave payload. */
    pointerType?: PointerKind;
    /** Pressure as reported for the move (see `PointerSample`). Absent on the leave payload. */
    pressure?: number;
    /**
     * Every position the pointer passed through since the last move, in the
     * drag payload's space (container x, pane-local y), present only while
     * `pressed`. Placement mode never arms a drag, so a freehand stroke reads
     * its coalesced samples here.
     */
    samples?: PointerSample[];
}
/** Modifier keys held during a pointer gesture. */
interface PointerModifiers {
    shift: boolean;
    alt: boolean;
    ctrl: boolean;
    meta: boolean;
}
/** Device behind a pointer gesture. Anything the browser does not name reports as a mouse. */
type PointerKind = 'mouse' | 'touch' | 'pen';
/**
 * One position a pointer passed through during a drag, in the same media px
 * space as the payload's `point` (container x, pane-local y). `pressure` is
 * the pointer events convention: 0..1 from hardware that measures it, 0.5
 * while a button that cannot is held, 0 when nothing is known.
 */
interface PointerSample {
    x: number;
    y: number;
    pressure: number;
}
/**
 * What the engine reports about the physical pointer behind a gesture.
 * `crosshair:move`, `click`, `drag:start`, `drag` and `drag:end` all carry these keys.
 */
interface PointerInfo {
    modifiers: PointerModifiers;
    pointerType: PointerKind;
    pressure: number;
}
/** An event-strip click, including every member of a clustered marker. */
interface ChartEventClick extends EventMarkerDetails {
    /** Chart-container CSS pixels, including the vertical offset of an event pane. */
    point: {
        x: number;
        y: number;
    };
    paneIndex: number;
}
/** Payload of the `drag` event: a draggable primitive being moved. */
interface ChartDragEvent extends PointerInfo {
    id: string;
    price: number;
    time: number;
    paneIndex: number;
    /** Where the gesture was grabbed, so a delta starts at the press rather than the first move. */
    fromPrice: number;
    fromTime: number;
    /** Current pointer position: container media x, pane-local media y. */
    point: {
        x: number;
        y: number;
    };
    /**
     * Every position the pointer passed through since the previous `drag`,
     * oldest first, the last one at `point`. Browsers deliver moves at frame
     * rate and fold the positions between frames into the event; a freehand
     * stroke drawn from `point` alone is a polyline of frame-rate corners.
     */
    samples: PointerSample[];
}
/** Payload of `drag:start` (press) and `drag:end` (release) for a primitive drag. */
interface ChartDragEndEvent extends PointerInfo {
    id: string;
    price: number;
    time: number;
    paneIndex: number;
    /** Release position: container media x, pane-local media y. */
    point: {
        x: number;
        y: number;
    };
}
/**
 * What the pointer was over when the context menu was raised. The chart cannot
 * know which menu items an app wants, but it does know what was hit, which is
 * the part an app cannot work out for itself: a canvas gives it a pixel, not an
 * object. `primitive` is anything hit-testable that is not one of the named
 * kinds (a price line, an order pill, a marker).
 */
type ContextMenuTargetKind = 'drawing' | 'indicator' | 'legend' | 'primitive' | 'series' | 'price-scale' | 'time-scale' | 'empty';
interface ContextMenuTarget {
    kind: ContextMenuTargetKind;
    /** Hit-test id of the thing under the pointer, when there was one. */
    id: string | null;
    /** Indicator instance id, when `kind` is 'indicator'. */
    instanceId?: string;
    /** Exact plot key when a study's plotted series was hit rather than its legend. */
    plotKey?: string;
    /** Series type, when `kind` is 'series'. */
    seriesType?: SeriesType;
    /** Which axis strip was hit, when `kind` is 'price-scale'. */
    side?: 'right' | 'left';
    /**
     * Which of the pane's scales that strip acts on, when `kind` is
     * 'price-scale': the side's own scale, or the hidden overlay scale ('') when
     * the side carries no series of its own and the pane's values are all on the
     * overlay. It is the argument the `priceAxis*` calls take.
     */
    scaleId?: PriceScaleId;
}

/**
 * The setters that change what `getState` saves without an event of their
 * own, and so announce it with `layout:change`.
 */
type LayoutSetter = 'setPaneWeight' | 'setPriceAxisOptions' | 'setPriceAxisAutoFit' | 'setPriceAxisLockRatio' | 'setPriceScaleOptions' | 'setAutoScale' | 'setGridOptions' | 'setCanvasOptions' | 'setStatusLineOptions' | 'setWatermarkOptions' | 'setTradingSettings' | 'setAxisChromeOptions' | 'setEventOptions' | 'applyOptions' | 'setNavigationOptions';
/**
 * Payload of the `layout:change` event (`chart.on('layout:change', ...)`):
 * a setter in {@link LayoutSetter} has run and the saved layout may differ.
 * It fires once per outermost call, after the change is applied: the canvas
 * block setting the grid on its way is one event, named `setCanvasOptions`.
 * A call that names no pane or scale the chart has changes nothing and fires
 * nothing, and neither does a restore, which announces itself with
 * `state:restore:start` and `state:restore:end`. Compare what you read back
 * if a no-op matters: setting a value to what it already was still fires.
 */
interface LayoutChangeEvent {
    setter: LayoutSetter;
}
/** Payload of the `contextmenu` event (`chart.on('contextmenu', ...)`). */
interface ContextMenuEvent {
    paneIndex: number;
    /** Cursor position in container media px, for placing the menu. */
    point: {
        x: number;
        y: number;
    };
    /** Price under the pointer on that pane, or null off the plot. */
    price: number | null;
    /** UTC seconds under the pointer, or null when there is no data. */
    time: number | null;
    /** Logical bar index under the pointer, or null off the plot. */
    index: number | null;
    target: ContextMenuTarget;
    /** Suppress the browser's own menu. Call it to show your own. */
    preventDefault(): void;
}
/**
 * Payload of the 'renderer:fallback' event: the chart has moved every pane
 * from a GPU backend to `canvas2d` for the rest of the session, because the
 * device's context was lost or turned out unusable. The frame that noticed
 * was already painted through 2D by the backend itself, so nothing went
 * blank; this is the host's cue to update anything that shows the renderer.
 */
interface RendererFallbackEvent {
    from: RenderBackendKind;
    to: 'canvas2d';
    reason: RendererFallbackReason;
}

/**
 * Trading visualization API (ARCHITECTURE.md §9). A data-driven layer on top of
 * the chart: your app pushes exchange state (positions, orders, trades) and the
 * chart renders labelled price-line "pills" (with a cancel/close button and
 * drag-to-modify) plus trade-fill markers; user interaction is relayed back as
 * `trading:*` events for the app to send to the exchange. Original,
 * framework-free API.
 */

type PositionSide = 'long' | 'short';
type TradingOrderSide = 'buy' | 'sell';
type TradingOrderType = 'limit' | 'stop' | 'stop_limit';
type TradeMarkerVariant = 'chevron' | 'bubble' | 'count';
type TradingLineVariant = 'standard' | 'line-only';
type TradingLineStyle = 'solid' | 'dashed' | 'dotted';
interface TradingPosition {
    id: string;
    side: PositionSide;
    entryPrice: number;
    size: number;
    pnlText?: string;
    pnlPercent?: string;
    color?: string;
    readOnly?: boolean;
    variant?: TradingLineVariant;
}
interface TradingOrder {
    id: string;
    type: TradingOrderType;
    side: TradingOrderSide;
    price: number;
    size: number;
    parentId?: string;
    bracketRole?: 'tp' | 'sl';
    color?: string;
    lineStyle?: TradingLineStyle;
    lineWidth?: number;
    readOnly?: boolean;
    draggable?: boolean;
    variant?: TradingLineVariant;
}
interface TradingTrade {
    id: string;
    side: TradingOrderSide;
    price: number;
    size: number;
    /** Execution time in milliseconds. */
    timestamp: number;
    variant?: TradeMarkerVariant;
    color?: string;
    label?: string;
}
interface TradingSyncPayload {
    positions?: TradingPosition[];
    orders?: TradingOrder[];
    trades?: TradingTrade[];
}
/** Payload of `trading:order_modify`: an order line was dragged and released. */
interface TradingOrderModifyEvent {
    orderId: string;
    newPrice: number;
    previousPrice: number;
}
/** Payload of `trading:bracket_modify`: a take-profit or stop-loss leg was dragged and released. */
interface TradingBracketModifyEvent {
    parentId: string | undefined;
    bracketRole: 'tp' | 'sl';
    newPrice: number;
}
interface TradingColors {
    long: string;
    short: string;
    order: string;
    tp: string;
    sl: string;
    buy: string;
    sell: string;
}
interface TradingSettings {
    longColor?: string;
    shortColor?: string;
    orderColor?: string;
    tpColor?: string;
    slColor?: string;
    buyColor?: string;
    sellColor?: string;
}
/** What the controller needs from the chart (the Chart implements this). */
interface TradingHost {
    addPrimitive(p: IPrimitive): void;
    removePrimitive(p: IPrimitive): void;
    subscribeClick(cb: (externalId: string) => void): void;
    subscribeDrag(onDrag: (externalId: string, price: number) => void, onDragEnd?: (externalId: string, price: number) => void): void;
    /** Optional: route trading events onto the chart's unified `chart.on(...)` bus. */
    emit?(event: string, payload: unknown): void;
    /**
     * Optional: the instrument's tick schedule when the layer is built. The
     * layer is built on first use, often after the instrument was applied, and
     * applying one does not build it early: building it subscribes to drags,
     * which makes every `ns-resize` price line on the chart draggable.
     */
    tickSchedule?(): TickSchedule | null;
}
declare const DEFAULT_TRADING_COLORS: TradingColors;
/** Self-contained trade-fill markers (chevron / bubble / count) over the plot. */
declare class TradeMarkersPrimitive implements IPrimitive {
    private _trades;
    private _colors;
    private _host;
    constructor(colors: TradingColors);
    attached(host: PrimitiveHost): void;
    detached(): void;
    zOrder(): ZOrder;
    autoscaleInfo(): null;
    setTrades(trades: TradingTrade[]): void;
    setColors(colors: TradingColors): void;
    draw(ctx: CanvasRenderingContext2D, rc: PrimitiveRenderContext): void;
}
declare class TradingController {
    private readonly _host;
    private readonly _positions;
    private readonly _orders;
    private readonly _trades;
    private readonly _listeners;
    private readonly _dragPrev;
    private _colors;
    private _markers;
    private _ticks;
    constructor(host: TradingHost);
    on(event: string, cb: (payload: unknown) => void): () => void;
    off(event: string, cb: (payload: unknown) => void): void;
    private _emit;
    setSettings(settings: TradingSettings): void;
    getSettings(): TradingColors;
    /**
     * Snap dragged order and bracket lines, and the price their modify events
     * carry, to the instrument's ticks. Null, the default, passes the pointer's
     * price through unrounded, as it always has. The host's own validation stays
     * authoritative either way; this only stops a drag previewing a price the
     * instrument cannot trade at. `Instrument.applyTo` and
     * `chart.setTickSchedule` set it, so call this after either to override it.
     */
    setTickSchedule(schedule: TickSchedule | null): void;
    setPositions(positions: readonly TradingPosition[]): void;
    setOrders(orders: readonly TradingOrder[]): void;
    setTrades(trades: readonly TradingTrade[]): void;
    addTrade(trade: TradingTrade): void;
    upsertOrder(order: TradingOrder): void;
    removeOrder(id: string): void;
    syncState(payload: TradingSyncPayload): void;
    /**
     * Refresh a position's P&L pill in place, without recreating its line: the
     * pill reads `pnlText`, then `pnlPercent` in brackets. Either one left out
     * keeps what the pill showed. The host formats both, since only it knows
     * the account's currency and how it rounds.
     */
    updatePositionPnl(id: string, pnlText?: string, pnlPercent?: string): void;
    /**
     * The form that also takes the P&L as a number, which nothing ever read:
     * the pill shows `pnlText` and `pnlPercent` only.
     *
     * @deprecated Removed in 3.0.0. Leave the number out: `updatePositionPnl(id, pnlText, pnlPercent)` (since
     * 2.6.0) sets the same pill.
     */
    updatePositionPnl(id: string, unrealizedPnl: number, pnlText?: string, pnlPercent?: string): void;
    getPositions(): TradingPosition[];
    getOrders(): TradingOrder[];
    getTrades(): TradingTrade[];
    clear(): void;
    private _renderTrades;
    private _sync;
    private _sig;
    /** Info segment for a position: live P&L text (side/size live in badge/qty). */
    private _positionPill;
    private _positionOpts;
    private _orderOpts;
    private _onClick;
    private _onDrag;
    private _onDragEnd;
}

/**
 * Interactive value capture (ARCHITECTURE.md §7). A settings input that names a
 * price or a time is declarative and the host renders it, but the *value* can
 * come from pointing at the chart, and only the engine knows what is under the
 * cursor. So the host arms a pick ("the user is now choosing a price"), the next
 * click on the plot answers with one, and the pick disarms itself.
 *
 * Built on the `click` event the draw tier's placement mode already resolves
 * anchors from, rather than a second capture path: same pane resolution, same
 * on-demand autoscale, same payload.
 *
 * Placement mode is deliberately *not* armed while picking. A pick wants panning
 * left alone (scroll back to the bar you mean, then click it), and a drag emits
 * no click outside placement mode, so panning cannot answer the pick by
 * accident. It also keeps a pick from cancelling an active drawing tool.
 */

type PickKind = 'price' | 'time';
/**
 * What a `'point'` pick answers: the bar time and the price under one click.
 * One capture rather than a time pick and a price pick in turn, so a study
 * input pairing the two is never written half from one click and half from
 * another.
 */
interface PickPoint {
    time: number;
    price: number;
}
/** Cancel a capture, or inspect whether this invocation still owns it. */
interface PickHandle {
    (): void;
    readonly active: () => boolean;
}
/** Optional target for Chart.beginPick. Times remain absolute UTC seconds. */
interface PickOptions {
    paneIndex?: number;
    priceScaleId?: PriceScaleId;
}
/**
 * The slice of the chart a pick needs. Structural, so `Chart` satisfies it with
 * nothing to cast and this module never imports the core (which imports this).
 */
interface PickHost {
    on(event: string, cb: (payload: unknown) => void): () => void;
    emit(event: string, payload: unknown): void;
    readonly dataLayer: {
        timeToIndexFloat(time: number): number;
        indexToTime(index: number): number | undefined;
    };
}
/**
 * Arm the next plot click to resolve to a price, a bar time, or both as a
 * `'point'` ({@link PickPoint}), and hand it to `cb`. Returns a cancel
 * function; calling it (or arming another pick on the same chart) disarms
 * without calling back. The chart emits `pick:start`
 * (`{ kind }`) and `pick:end` (`{ kind, value }`, `value` null when cancelled)
 * so a host can show its own cursor or hint while the pick is live.
 *
 * A time is snapped to the bar the click landed on, because a time between two
 * bars matches no bar and anything anchored to it would never line up. Clicking
 * past the last bar keeps the projected time, which is what a pick in the empty
 * right-hand space means.
 */
declare function beginPick(host: PickHost, kind: 'point', cb: (value: PickPoint) => void): PickHandle;
declare function beginPick(host: PickHost, kind: PickKind, cb: (value: number) => void): PickHandle;

/**
 * Declarative chart-settings schema: the tabs and controls a host renders to
 * build the terminal's settings dialog, without hardcoding a list.
 *
 * WHY it reuses `IndicatorInput` rather than inventing a widget vocabulary: a
 * host that can already render the indicator settings form (built from
 * `indicatorStyleInputs`) can render this one with no new widget code. It adds
 * exactly one widget of its own, `colorPair`, because a bullish/bearish pair is
 * one row of the dialog and expressing it as two `color` inputs forces a host
 * into a section header plus two stacked rows: three times the height, for the
 * property a trader changes most.
 *
 * WHY read/write live on the same object as the input: the schema and the
 * accessors are one structure, so a control cannot drift from the option it
 * drives, and there is no second table to keep in step. That is also the rule
 * for what ships here: **every control maps to an option that actually changes
 * what is drawn or stored.** Controls this engine has no backing for are absent
 * rather than inert; a checkbox that does nothing is worse than one that is not
 * there.
 *
 * The tabs are our own five, not a reference terminal's seven. Alerts are not
 * here because they have a dialog and a saved document of their own, and
 * corporate events are not here because nothing in the engine sources them:
 * an empty tab is worse than an absent one.
 *
 * Keys are dotted paths (`symbol.upColor`, `canvas.grid.vertColor`), so a patch
 * is a flat `Record<string, value>` that survives JSON. They are a wire format
 * a host may have written down, so they stay put when the grouping above them
 * changes: a key names the option it writes, not the tab it is shown on.
 */

/**
 * Tabs of the settings dialog. Five groups, chosen so a trader finds a setting
 * without hunting: what the instrument is painted with, what the header reads
 * out, what the two axes do, what the surface around the data looks like, and
 * what the trade layer draws.
 */
type ChartSettingsTabId = 'price' | 'readout' | 'axes' | 'appearance' | 'trading';
/**
 * Two colours on one labelled row, with an optional switch in front of them:
 *
 *     [x] Borders   [up] [down]
 *
 * `key` identifies the row and is not itself a value. The value keys are
 * `up.key`, `down.key` and `enabled.key`, each an ordinary flat key of
 * `ChartSettingsValues`, so a host patches this row exactly the way it patches
 * a plain colour and nothing about read/apply has to know the widget exists.
 *
 * `enabled` is absent when the pair has no visibility flag behind it (a candle
 * body is always drawn), which is the difference between a row whose checkbox
 * does something and one whose checkbox would be a lie.
 *
 * It takes `visibleWhen` and `activeWhen`, and a condition that reads any of
 * its three value keys reads the pair. It takes no `inline`: the row already
 * holds a switch and two swatches, so it always stands on its own.
 */
interface ChartSettingsColorPairInput extends Omit<IndicatorInputPresentation, 'inline'> {
    key: string;
    type: 'colorPair';
    label: string;
    group?: string;
    /** Help text for the row, on the same terms as `IndicatorInput['tooltip']`. */
    tooltip?: string;
    enabled?: {
        key: string;
        default: boolean;
    };
    up: {
        key: string;
        label: string;
        default: string;
    };
    down: {
        key: string;
        label: string;
        default: string;
    };
}
/** Everything a host may be handed in `ChartSettingsTab.inputs`. */
type ChartSettingsInput = IndicatorInput | ChartSettingsColorPairInput;
interface ChartSettingsTab {
    id: ChartSettingsTabId;
    label: string;
    /** Controls in display order; `input.group` names the sub-heading. */
    inputs: ChartSettingsInput[];
}
type ChartSettingsValue = string | number | boolean;
/** Flat, JSON-safe snapshot of every control, keyed by the schema's dotted key. */
type ChartSettingsValues = Record<string, ChartSettingsValue>;
/**
 * The settings slice of a saved chart state. It is declared here rather than on
 * `ChartState` so this module owns its own persistence: `chart.getState()`
 * returns a `ChartState & ChartSettingsState`, which is still a `ChartState` to
 * every existing consumer, and `restoreState` reads it back.
 *
 * `events` stays in the saved state although the dialog no longer offers the
 * switches: the chart still owns `setEventOptions`, a host that feeds its own
 * corporate actions still sets them, and dropping the field would throw that
 * host's saved layout away.
 */
interface ChartSettingsState {
    navigation?: Partial<ChartNavigationOptions>;
    canvas?: CanvasOptions;
    statusLine?: LegendStatusLineOptions;
    watermark?: ChartWatermarkOptions;
    trading?: TradingSettings;
    events?: ChartEventOptions;
    /**
     * The axis-strip switches. Only the switches: `AxisChromeOptions.clock` is a
     * callback, which no JSON survives, and a host that supplied one supplies it
     * again when it builds the chart.
     */
    axisChrome?: Pick<AxisChromeOptions, 'sessionClock' | 'barCountdown'>;
}
/**
 * The settings dialog, described. Takes the chart because the Price tab depends
 * on the primary series type (a candle has borders, a line has a dash), because
 * the colour defaults come from the live theme, and because the timezone list
 * has to include whatever zone the chart is already in.
 */
declare function chartSettingsSchema(chart: Chart): ChartSettingsTab[];
/** Current value of every control in the schema, keyed the same way. */
declare function readChartSettings(chart: Chart): ChartSettingsValues;
/**
 * Apply a patch produced by the dialog. Only the keys present are written, so a
 * single changed control is a one-key object; unknown keys are ignored, which is
 * what lets a state saved by a newer build restore into an older one.
 */
declare function applyChartSettings(chart: Chart, patch: Readonly<Partial<ChartSettingsValues>>): void;

/** UTC seconds when this recorded candle becomes complete and available. */
type ReplayBarEndTime = (bar: Bar, index: number) => number;
/** Explicit availability avoids treating a coarse candle's open as its close. */
interface ReplayTiming {
    barEndTime: ReplayBarEndTime;
    /** Required with finer bars; their final prices also need a known availability time. */
    subBarEndTime?: ReplayBarEndTime;
}

/**
 * Market replay: walk a historical session forward bar by bar so a trader can
 * practise on it, and so every indicator redraws exactly as it stood at that
 * past moment.
 *
 * Headless, in the spirit of `DrawingController` (src/draw/controller.ts): it
 * owns the playhead and the transitions and ships no DOM, so the host renders
 * its own transport bar, scrub slider and clock from `state()` and the events.
 *
 * The mechanic is deliberately boring. Replay feeds the chart a **prefix** of
 * the full bar array through the ordinary `series.setData` path, and that is
 * what makes indicators free: for the primary series that path invalidates and
 * recomputes every study at once (`ChartSeries._setData`), and
 * `IndicatorInstance.recompute` re-reads the whole history from `sourceBars()`. Shorten that history and every indicator, level,
 * fill, marker and legend reconstructs itself as it was at that bar, with no
 * replay-aware code anywhere in the indicator tier.
 */

/** Schedules a repeating callback and returns its canceller. Inject in tests. */
type ReplayScheduler = (cb: () => void, intervalMs: number) => () => void;
/**
 * The slice of the time scale replay saves and restores. Declared structurally
 * rather than as `TimeScale` so nothing here depends on the scale's shape
 * beyond the two numbers that *are* the viewport.
 */
interface ReplayViewport {
    readonly barSpacing: number;
    readonly rightOffset: number;
    setBarSpacing(value: number): void;
    setRightOffset(value: number): void;
}
/**
 * The slice of the chart this controller needs. `Chart` satisfies it; declaring
 * it structurally keeps the controller testable against a stub and free of a
 * circular import back into core.
 */
interface ReplayChartHost {
    emit(event: string, payload: unknown): void;
    readonly timeScale: ReplayViewport;
    /**
     * Optional. When the chart can name its own primary price series, `series`
     * may be omitted and replay drives that one.
     */
    primarySeries?(): SeriesApi | null;
    /**
     * Optional. When the chart reports its own end, replay listens for it while
     * it owns the chart's data: a chart destroyed mid-session stops the playback
     * timer and is written to no more, and nothing is restored to it.
     */
    on?(event: 'destroy', callback: () => void): () => void;
}
/** Everything a transport bar and a clock need, in one object. */
interface ReplayState {
    /** 0-based newest bar; -1 before the first observation in timed replay. */
    index: number;
    /** Bars in the replay set (the scrub bar's maximum is `total - 1`). */
    total: number;
    playing: boolean;
    /** Multiplier over `barMs`: 2 plays two bars per `barMs`. */
    speed: number;
    /**
     * The bar at `index`, the replay clock's "now". Null when there is no data.
     *
     * Under intra-bar replay this is the **partial** bar as it stands this step,
     * not the completed one, which is what makes it the right thing to drive a
     * host's own forming volume bar or an OHLC readout from.
     */
    bar: Bar | null;
    /**
     * Which step of the forming bar this is, 0-based, and how many it takes. Both
     * are 0 and 1 without `subBars`, so a transport that shows them needs no
     * special case for the plain mode.
     */
    subIndex: number;
    subSteps: number;
    /**
     * True while the newest bar forms along a simulated path (`simulate`), because
     * no finer bar covers it. Its forming prices were never traded, so a host shows
     * that plainly; the bar closes on the real bar either way.
     */
    simulated: boolean;
}
/** How a bar without finer data forms under `ReplayOptions.simulate`. */
interface ReplaySimulation {
    /**
     * Steps such a bar takes, the last being the bar itself: a whole number from 2
     * to 240. The displayed interval over the finer one it stands in for (5 for a
     * 5-minute chart over 1-minute data) keeps the pace the same either way.
     */
    steps: number;
}
interface ReplayOptions {
    /** Explicit candle availability for time-aligned replay. Omitted preserves index-based replay. */
    timing?: ReplayTiming;
    /** Initial UTC availability time. Requires timing; otherwise startIndex selects a completed candle. */
    startTime?: number;
    /** False prepares and validates the snapshot without changing the chart. Default true. */
    autoStart?: boolean;
    /**
     * The series replay drives. The first one owns the timeline; any others (a
     * volume histogram, a comparison line) are truncated to the same instant by
     * time, because the shared DataLayer merges *every* series onto one axis and
     * a series left at full length would drag future timestamps back onto it.
     *
     * Optional only if the host chart implements `primarySeries()`.
     */
    series?: SeriesApi | readonly SeriesApi[];
    /**
     * The full session. Defaults to the primary series' current data. Never
     * mutated, and never handed to the chart as-is: each frame gets its own slice.
     */
    bars?: readonly Bar[];
    /**
     * Finer-grained bars the displayed ones are built from: the 1-minute session
     * under a 5-minute chart. Given these, the playhead advances a **sub-bar** at
     * a time and the newest bar forms in front of the user the way a live one
     * does, instead of appearing complete. A 5-minute bar over 1-minute data
     * takes five steps.
     *
     * They must cover the same session as `bars` and be sorted by time; sub-bars
     * outside any displayed bucket are ignored. The last step of a bucket emits
     * the displayed bar **verbatim** rather than the aggregate, so a bucket always
     * closes on exactly the number the chart would have shown without this option,
     * whatever the two feeds disagree about in between. Every step before it is
     * held inside that bar, the way a live candle grows: the displayed bar's open
     * from the first step, the finer extremes and close within its high and low,
     * and volume no more than its volume. A forming bar never shows a price its
     * closed bar denies.
     *
     * Only the first driven series forms partially. Followers are cut to completed
     * buckets, because the controller cannot know how to half-aggregate an
     * arbitrary one: a volume histogram is summed, not OHLC-merged. The partial
     * bar reaches the host as `ReplayState.bar` on every frame, so a host that
     * wants a growing volume bar writes it from there.
     */
    subBars?: readonly Bar[];
    /**
     * Form a displayed bar that no finer bar covers over simulated steps instead
     * of landing it whole: the 5-minute bars older than the 1-minute history a feed
     * keeps, or every bar when there is no finer feed at all. The path runs from
     * the open to the nearer extreme, the other extreme and the close, held inside
     * the bar like every forming bar (see `subBars`), and closes on the bar itself.
     * Real finer bars always win where they exist. `ReplayState.simulated` says
     * which steps are simulated. Off by default, so a bar without finer data takes
     * one step as before.
     */
    simulate?: ReplaySimulation;
    /** Bar to open at (0-based). Default 0. */
    startIndex?: number;
    /** Wall-clock milliseconds per bar at speed 1. Default 1000. */
    barMs?: number;
    /** Initial speed multiplier. Default 1. */
    speed?: number;
    /** Called on every playhead move, after the chart has been updated. */
    onFrame?: ((state: ReplayState) => void) | undefined;
    /** Playback clock. Default `performance.now`. */
    now?: () => number;
    /**
     * Playback timer. Default `setInterval`, with the interval held from 1 ms to
     * 2^31 - 1 ms, the longest a platform timer waits (a longer one fires at once).
     */
    scheduler?: ReplayScheduler;
}
declare class ReplayController {
    private readonly _chart;
    private readonly _series;
    private readonly _bars;
    /** What each driven series held before replay took over, for `stop()`. */
    private readonly _restore;
    private readonly _restoreConfirmation;
    private readonly _view;
    private readonly _onFrame;
    private readonly _now;
    private readonly _schedule;
    private readonly _barMs;
    private readonly _startIndex;
    private readonly _timeline;
    private readonly _startTime;
    private _time;
    private _pointIndex;
    private readonly _subBars;
    /**
     * For each displayed bar, where its sub-bars start in `_subBars` and how many
     * there are. Built once: the alternative is a binary search per step, on a
     * path that runs on every played frame.
     *
     * A bucket with no sub-bars takes one step and shows the displayed bar, so a
     * gap in the finer feed costs that bar its formation, not its existence.
     */
    private readonly _subStart;
    private readonly _subCount;
    private readonly _intra;
    /** Steps a bar without finer data takes under `simulate`; 0 when off. */
    private readonly _simSteps;
    private _speed;
    private _index;
    /** Step within the forming bar, 0-based. Always 0 without `subBars`. */
    private _sub;
    private _playing;
    /** True once replay owns the chart's data; false before start and after stop. */
    private _active;
    private _transition;
    private _cancel;
    private _interval;
    /** Clock reading the last advance was charged to; keeps playback drift-free. */
    private _lastAdvance;
    /** Stops listening for the chart's end; held only while replay owns its data. */
    private _release;
    /**
     * Constructing the controller **enters replay**: it snapshots the chart's data
     * and viewport and immediately shows `startIndex`. That is the gesture a
     * replay button makes, and it means the snapshot is taken before anything has
     * moved, so `stop()` can put the user back exactly where they were.
     */
    constructor(chart: ReplayChartHost, options?: ReplayOptions);
    /**
     * Map every sub-bar onto the displayed bar whose bucket it falls in.
     *
     * The displayed bars' own times are the bucket starts, so one forward pass
     * over both sorted arrays does it. A sub-bar before the first displayed bar
     * belongs to no bucket and is dropped rather than folded into bar 0, which
     * would open that bar at a price the chart never showed.
     */
    private _buildBuckets;
    /** How many steps the bar at `index` takes. At least one, always. */
    private _steps;
    /** Whether the bar at `index` forms along the simulated path. */
    private _simulated;
    /** The bar shown at step `sub` of `index`, for a step before its last. */
    private _formingAt;
    /**
     * Jump the playhead to a bar. Out-of-range values clamp to the session.
     *
     * A seek lands on a **complete** bar even under intra-bar replay: scrubbing to
     * a half-formed candle is not a position anyone asks for, and it would make
     * the same slider position mean different things on the way past.
     */
    seek(index: number): void;
    /** Project only observations available by these UTC seconds. Requires timing. */
    seekTime(time: number): void;
    /** Availability clock in timed mode; the displayed bar's timestamp in legacy mode. */
    time(): number | null;
    /** Observation timestamps for a shared clock. Requires timing; returns a copy. */
    timePoints(): readonly number[];
    /**
     * Move `n` steps forward. Stops dead at the end of the session.
     *
     * A step is one displayed bar normally, and one sub-bar under intra-bar
     * replay, so the transport button means "advance the chart by the smallest
     * amount it can show" in both modes.
     */
    step(n?: number): void;
    /** Move `n` steps back. Stops dead at the first bar. */
    stepBack(n?: number): void;
    /**
     * Move the playhead `delta` steps, carrying across bar boundaries.
     *
     * Written as a walk rather than arithmetic on a flat step count because
     * buckets hold different numbers of sub-bars: a session with a gap, or a
     * partial first bucket, has no constant steps-per-bar to divide by.
     */
    private _advance;
    /**
     * Start (or re-speed) playback. Calling it on the last bar plays nothing and
     * reports `replay:end` instead of arming a timer that could never advance.
     */
    play(options?: {
        speed?: number;
    }): void;
    /** Halt playback, leaving the playhead (and the chart) where it is. */
    pause(): void;
    /**
     * Leave replay: every driven series gets its pre-replay data back and the
     * viewport is restored to the pixel, so a user who exits does not lose their
     * place. Safe to call twice; a later `seek`/`step`/`play` re-enters replay
     * from `startIndex`.
     */
    stop(): void;
    state(): ReplayState;
    /**
     * Put the chart at `index`. Every transition goes through here (seek, step
     * and each played frame alike), so arriving at a bar leaves the chart in the
     * same state however the user got there.
     */
    private _apply;
    private _write;
    /**
     * One timer tick. The number of bars owed comes from the clock, not from the
     * tick count, so a coarse or throttled timer still plays at the requested
     * speed, and a host may drive this from rAF instead with no change here.
     */
    private readonly _tick;
    /**
     * The chart is gone: there is nothing to restore to and nothing to draw on,
     * so the clock stops and the controller leaves replay without writing. A
     * write in progress sees the transition move and goes no further.
     */
    private readonly _gone;
    /** True on the last step of the last bar, which is where playback stops. */
    private _atEnd;
    /** The session is over: stop the timer and tell the host once. */
    private _end;
    private _stopTimer;
}

/** Visual settings use the chart-settings wire format without its data or trading fields. */
type LinkAppearanceValues = Record<string, string | number | boolean>;
/** The host owns reading and applying settings so linking needs no chart class or DOM. */
interface LinkAppearanceAdapter {
    read(): Readonly<LinkAppearanceValues>;
    apply(values: Readonly<LinkAppearanceValues>): void;
}
/** Retain supported appearance fields, excluding instrument text, data, navigation and trades. */
declare function filterLinkAppearance(values: Readonly<Partial<LinkAppearanceValues>>): LinkAppearanceValues;

/**
 * The chart's event bus: every name `chart.on(...)` takes, the payload each
 * carries, and the registry behind `on`, `once`, `off` and the engine's emit.
 *
 * {@link ChartEventMap} is the one inventory of the bus. The engine emits each
 * name in it, except the three a host emits for a link group (`symbol`,
 * `interval`, `chartType`); tests/chart-event-map.test.ts holds every emit and
 * subscription in src to it with the compiler. A lazy tier adds its own names
 * by declaration merging (the draw tier's are in src/draw/events.ts), which is
 * also how a host types an event of its own:
 *
 * ```ts
 * declare module 'openalgo-charts' {
 *   interface ChartEventMap { 'myapp:signal': { price: number } }
 * }
 * ```
 *
 * Names. A new name is `namespace:action`, lower case, words joined by a
 * hyphen, in the present tense (`objects:change`, `draw:preview-clear`). The
 * names that predate the rule (the camelCase pane and indicator events, the
 * single words, `trading:*` in snake case, and the past tense of
 * `branding:changed`, `timezone:changed`, `alerts:changed` and
 * `alerts:restored`) keep their spelling: renaming one would break every
 * listener on it.
 *
 * Payloads. One shape per name. An event with nothing to say carries an empty
 * object ({@link EmptyEvent}), except `events:change`, which has always carried
 * `undefined`.
 */

/** Payload of an event with nothing to report but that it happened: an empty object. */
type EmptyEvent = Record<string, never>;
/** Payload of `hover`: the hit id under the pointer, or null when it left every primitive. */
interface ChartHoverEvent {
    id: string | null;
}
/** Payload of `drag:cancel`: a primitive drag ended without a commit. */
interface ChartDragCancelEvent {
    id: string;
    paneIndex: number;
    reason: 'pointercancel' | 'pinch' | 'escape';
}
/**
 * Payload of `pan` and `zoom`. `from` and `to` are UTC seconds, or null where
 * that edge falls outside the loaded data; the logical pair is the raw
 * fractional bar range.
 */
interface ChartViewportEvent {
    from: number | null;
    to: number | null;
    logicalFrom: number;
    logicalTo: number;
}
/** Payload of `resize`: the container's new size in CSS pixels. */
interface ChartResizeEvent {
    width: number;
    height: number;
}
/** Payload of `lazy-load`: the window that neared the oldest bar, in UTC seconds. */
interface LazyLoadEvent {
    from: number | null;
    to: number | null;
    direction: 'backward';
}
/** Payload of `paneAdded`, `paneRemoved` and `paneResized`. */
interface PaneEvent {
    paneIndex: number;
}
/** Payload of `paneMoved`: the two slots whose panes swapped. */
interface PaneMovedEvent {
    from: number;
    to: number;
}
/** Payload of `paneMaximized`: the maximized pane, or null when none is any more. */
interface PaneMaximizedEvent {
    paneIndex: number | null;
}
/** Payload of `paneCollapsed`. */
interface PaneCollapsedEvent {
    paneIndex: number;
    collapsed: boolean;
}
/** Payload of `priceAxisPlacementChanged`: where the scale draws now. It keeps its id. */
interface PriceAxisPlacementChangedEvent extends PriceAxisPlacement {
    paneIndex: number;
    scaleId: PriceScaleId;
}
/** Payload of `indicatorRemoved`, `indicatorSettings` and `indicatorSource`. */
interface IndicatorInstanceEvent {
    instanceId: string;
    indicatorId: string;
    paneIndex: number;
}
/** Payload of `indicator:data-status`. `id` is the instance id. */
interface IndicatorDataStatusEvent {
    id: string;
    indicatorId: string;
    status: Readonly<IndicatorDataStatus>;
}
/** Payload of `timezone:changed`: the IANA zone the axis and studies now use. */
interface TimezoneChangedEvent {
    timezone: string;
}
/** Payload of `pick:start`. */
interface PickStartEvent {
    kind: PickKind | 'point';
}
/** Payload of `pick:end`: the picked value, or null when the pick was cancelled. */
interface PickEndEvent extends PickStartEvent {
    value: number | PickPoint | null;
}
/**
 * Every event on the chart's bus, by name, with the payload a listener
 * receives. `chart.on`, `once` and `off` take these names and type the
 * listener from here; the draw tier adds the `draw:*` and `drawing:*` names.
 * Add a name of your own by declaration merging (see the module note).
 */
interface ChartEventMap {
    /** Once, on the microtask after the chart is built, so a listener added on the next line still hears it. */
    ready: EmptyEvent;
    /** Once, at the end of `destroy()`, with the chart already torn down. Every listener is dropped after it. */
    destroy: EmptyEvent;
    /** The container size changed, by the resize observer or an `applySize` that changed it. */
    resize: ChartResizeEvent;
    /** A GPU backend lost its context or proved unusable; every pane draws through canvas2d from now on. */
    'renderer:fallback': RendererFallbackEvent;
    /** The brand mark changed; the payload is its options, or false when it was removed. */
    'branding:changed': BrandingChangedEvent;
    /** The pointer moved over the plot, or left it (every field null). */
    'crosshair:move': CrosshairMoveEvent;
    /** Every crosshair position a readout should show: the pointer's, and a linked chart's (`source: 'linked'`). */
    'crosshair:readout': CrosshairMoveEvent;
    click: ChartClickEvent;
    /** Set `handled` to stop the chart's own double-click action. */
    dblclick: DoubleClickEvent;
    /** The pointer entered or left a hit-testable primitive. */
    hover: ChartHoverEvent;
    /** A press on a draggable primitive, before it moves. */
    'drag:start': ChartDragEndEvent;
    drag: ChartDragEvent;
    'drag:end': ChartDragEndEvent;
    'drag:cancel': ChartDragCancelEvent;
    /** A right click, axis strips included. Call `preventDefault` to show your own menu. */
    contextmenu: ContextMenuEvent;
    /** A timeline event marker was clicked. */
    'event:click': ChartEventClick;
    /** `beginPick` started. */
    'pick:start': PickStartEvent;
    /** A pick resolved or was cancelled. */
    'pick:end': PickEndEvent;
    /** The visible window moved without changing its span. */
    pan: ChartViewportEvent;
    /** The visible window's span changed. */
    zoom: ChartViewportEvent;
    /** The view neared the oldest bar and the history loader ran. */
    'lazy-load': LazyLoadEvent;
    paneAdded: PaneEvent;
    paneRemoved: PaneEvent;
    paneMoved: PaneMovedEvent;
    paneMaximized: PaneMaximizedEvent;
    paneCollapsed: PaneCollapsedEvent;
    /** A pane divider drag was released. */
    paneResized: PaneEvent;
    /**
     * `movePriceAxis` moved a pane's prices to the other strip.
     *
     * @deprecated Removed in 3.0.0 with `movePriceAxis`, the only call that emits it.
     *   Listen for `priceAxisPlacementChanged` (since 2.5.4).
     */
    priceAxisMoved: {
        paneIndex: number;
        from: 'right' | 'left';
        to: 'right' | 'left';
    };
    priceAxisPlacementChanged: PriceAxisPlacementChangedEvent;
    /** A setter that changes the saved layout, and has no event of its own, ran. */
    'layout:change': LayoutChangeEvent;
    /** The source, study or chart object inventory changed. Re-read it; the payload says nothing. */
    'objects:change': EmptyEvent;
    /** Chart-owned timeline events, or their visibility, changed. */
    'events:change': undefined;
    /** Appearance settings a link group mirrors were applied, as the values that changed. */
    'style:change': LinkAppearanceValues;
    'timezone:changed': TimezoneChangedEvent;
    /** `setDataContext` changed the instrument, interval or variant; the payload is the new context. */
    'data:context': Readonly<ChartDataContext> | undefined;
    /** The primary source was written: a live bar, a reset or a page of history. */
    'data:update': ChartDataUpdate;
    /** Studies were invalidated because the primary bars or the clock they read changed; a write fires it before `data:update`. */
    'data:range': EmptyEvent;
    /** `setBarsProvider` or `invalidateRequestedData` ran; studies that fetch ask again. */
    'data:requests': EmptyEvent;
    /** `restoreState` began. A listener may start a newer restore, which supersedes this one. */
    'state:restore:start': EmptyEvent;
    'state:restore:end': EmptyEvent;
    /** The drawings a restore carries, for the draw tier to load. Opaque to the base. */
    'drawings:restore': unknown;
    /** The alerts a restore carries, for an `AlertController` to load. */
    'alerts:restore': AlertsDocument;
    indicatorRemoved: IndicatorInstanceEvent;
    /** A study legend's settings button was pressed. The engine ships no form. */
    indicatorSettings: IndicatorInstanceEvent;
    /** A study legend's source button was pressed (a descriptor with `hasSource: true`). */
    indicatorSource: IndicatorInstanceEvent;
    /** A condition a study descriptor declared came true on a live bar. */
    'indicator:alert': IndicatorAlertPayload;
    'indicator:data-status': IndicatorDataStatusEvent;
    'alert:created': AlertChangeEvent;
    'alert:updated': AlertChangeEvent;
    'alert:removed': AlertRemovedEvent;
    'alert:expired': AlertChangeEvent;
    'alert:triggered': AlertTriggeredPayload;
    'alert:error': AlertErrorEvent;
    'alerts:changed': AlertsChangedEvent;
    'alerts:restored': AlertsRestoredEvent;
    /** Evaluation advanced what the alerts have consumed; save the chart state, deliver nothing. */
    'alerts:checkpoint': EmptyEvent;
    'replay:start': ReplayState;
    'replay:frame': ReplayState;
    'replay:play': ReplayState;
    'replay:pause': ReplayState;
    'replay:end': ReplayState;
    'replay:stop': ReplayState;
    'trading:order_modify': TradingOrderModifyEvent;
    'trading:order_cancel': {
        orderId: string;
    };
    'trading:order_click': {
        order: TradingOrder;
    };
    'trading:position_close': {
        positionId: string;
    };
    'trading:position_click': {
        position: TradingPosition;
    };
    'trading:bracket_modify': TradingBracketModifyEvent;
    /** The host switched instrument. The widget emits it; a link group and a drawing link follow it. */
    symbol: {
        symbol: string;
        exchange?: string;
    } | string;
    /** The host switched timeframe. A link group follows it. */
    interval: {
        interval: string;
    } | string;
    /** The host switched chart type. A link group follows it. */
    chartType: {
        chartType: string;
    } | string;
}

/**
 * The chart (ARCHITECTURE.md §3.3): it owns the shared DataLayer and time
 * scale, the panes, the invalidate mask and the render loop, and it is the
 * API a host calls. Most of the work lives in the chart-*.ts collaborators;
 * this file holds the state they share and the methods that carry the
 * documented contracts.
 */

/**
 * Payload of the `click` event (`chart.on('click', ...)`). Its `pressure` is
 * the pressure at the press, not the release, which always reads 0.
 */
interface ChartClickEvent extends PointerInfo {
    /** `externalId` of the hit primitive, or null on empty plot. */
    id: string | null;
    /** Price under the press on that pane, or null off the plot. */
    price: number | null;
    /** UTC seconds under the press, interpolated between bars and past the right edge. */
    time: number;
    paneIndex: number;
    /** Press position: container media x, pane-local media y. */
    point: {
        x: number;
        y: number;
    };
    /** Set on the release half of a press-drag-release while a host is placing a shape. */
    viaDrag?: boolean;
    /**
     * Shift at the click: the same state as `modifiers.shift`, in the flat form
     * the first click payloads carried.
     *
     * @deprecated Removed in 3.0.0. Read `modifiers.shift` (since 2.0.0), which carries the same state.
     */
    shiftKey: boolean;
    /** @deprecated Removed in 3.0.0. Read `modifiers.ctrl` (since 2.0.0), which carries the same state. */
    ctrlKey: boolean;
    /** @deprecated Removed in 3.0.0. Read `modifiers.meta` (since 2.0.0), which carries the same state. */
    metaKey: boolean;
}
/**
 * What a host needs to render a menu over one price axis: which items are on,
 * and which of them mean anything on this axis. See `Chart.priceAxisState`.
 */
interface PriceAxisState {
    paneIndex: number;
    scaleId: PriceScaleId;
    /** Current visible side; hidden scales report 'right'. Read priceAxisPlacement for hidden state. */
    side: 'right' | 'left';
    /** Some series on the pane maps to this scale. */
    active: boolean;
    /** Auto-fit: the range tracks the data rather than staying where it was put. */
    autoFit: boolean;
    inverted: boolean;
    mode: PriceScaleMode;
    /** False while the scale still sits on its 0..1 placeholder (nothing measured). */
    scaled: boolean;
    lockRatio: boolean;
    /**
     * Whether `movePriceAxis` would do anything: something to move, and a free side.
     *
     * @deprecated Removed in 3.0.0, with {@link Chart.movePriceAxis}, the only operation it describes. Placement
     * through {@link Chart.setPriceAxisPlacement} (since 2.5.4) needs no such check.
     */
    movable: boolean;
}
declare class Chart {
    private readonly _container;
    private readonly _doc;
    private readonly _pixelRatio;
    private _theme;
    private readonly _panes;
    /**
     * The primary price pane, held by identity. It is the pane the chart is
     * built with, and it can sit in any slot: `_panes` is the visual order, so
     * "the price pane" is wherever this is, never simply slot 0. See
     * `primaryPaneIndex`.
     */
    private _primaryPane;
    private readonly _loop;
    /** The frame scheduler, kept for the one-shot re-measure after construction. */
    private readonly _raf;
    /** Whether the host supplied `raf`, and so owns every frame, the resize ones included. */
    private readonly _rafInjected;
    private _remeasureHandle;
    private readonly _dataLayer;
    private readonly _timeScale;
    private _timeScaleMutationDepth;
    private readonly _priceAxisWidth;
    private readonly _timeAxisHeight;
    private _pending;
    /** While a frame steps the glides and the studies: what they invalidate is painted by that frame. */
    private _framing;
    private _scaleMutationDepth;
    /** Size and device-pixel-ratio observation; see chart-pixels.ts. */
    private readonly _pixels;
    /** The device pixel ratio the canvases were last sized at. */
    private _layoutRatio;
    private _width;
    private _height;
    private _hasFitContent;
    /** Layout setters running inside another one, or inside a restore: only the outermost announces. */
    private _layoutDepth;
    private _crosshairMode;
    private _crosshairSnapToBar;
    private _priceOnlyAutoScale;
    private _indicatorLegendCollapsed;
    private _shortcuts;
    private _trading;
    private _tickSchedule;
    /** Pointer, wheel and keyboard routing, hover, and the gesture state they keep; see chart-input.ts. */
    private readonly _input;
    private readonly _now;
    private readonly _conflate;
    private readonly _conflationFactor;
    /**
     * One backend per pane; see `ChartOptions.renderer` and `renderBackend`.
     * Replaced by the 2D factory after a session fallback, so a pane added
     * later matches the ones already on screen.
     */
    private _backendFactory;
    /**
     * The kind the first pane got. A factory may decline per pane, so this is
     * what the chart actually paints with rather than what was asked for.
     */
    private _rendererKind;
    /** What the `renderer` option asked for, so a decline can be reported once. */
    private readonly _requestedRenderer;
    private _gridVert;
    private _gridHorz;
    /** The Canvas option block: overrides of the theme, never a copy of it. */
    private readonly _canvas;
    /** Status-line switches pushed onto every pane legend, host-added ones included. */
    private readonly _statusLine;
    /** Legend action-button side in media px; undefined leaves the primitive's default. */
    private _legendIconSize;
    /** Axis-strip chrome switches. Empty is the shipped chart: neither drawn. */
    private readonly _axisChrome;
    /**
     * The corner clock's object form, kept across an off/on toggle. A switch that
     * turns the clock off must not also throw away the `showOffset` a host chose
     * for it: switching it back on would silently be a different clock, and
     * nothing on the switch could put the choice back.
     */
    private _sessionClockForm;
    /** Wall-clock UTC seconds for the two axis readings, injectable for tests. */
    private _wallClock;
    /**
     * Trade-layer colours held here rather than on the controller, so reading or
     * setting them never has to instantiate one. `chart.trading` hands them over
     * when the controller is finally created.
     */
    private readonly _tradingSettings;
    /** Chart-owned event strip (see `setEvents`), plus which types it draws. */
    private _events;
    private _eventMarkers;
    private _eventPane;
    /** Attaching, stacking and re-homing primitives, and the event strip's type switches; see chart-primitives.ts. */
    private readonly _primitives;
    private readonly _navigation;
    private _liveRegion;
    private _lastDragX;
    private _lastDragT;
    private _dragVelocity;
    /** Kinetic pan, the zoom glide and the autoscale easing they start; see chart-motion.ts. */
    private readonly _motion;
    private readonly _animZoom;
    private readonly _zoomAnchor;
    private readonly _doubleClick;
    /** `movablePrimaryPane`: without it the price pane stays pinned at slot 0. */
    private readonly _movablePrimaryPane;
    private readonly _firstDataId;
    /** Handle + record of the primary price series (see `primarySeries`). */
    private _primary;
    private readonly _seriesRecords;
    private readonly _seriesProvenance;
    private readonly _indicators;
    /**
     * Where the price source sits among the studies of its pane: undefined until
     * something places it (the source then stays where it was added, as it
     * always did), null at the back of the series band, or the instance id of
     * the study it paints directly above.
     */
    private _sourceAbove;
    /**
     * Each study legend row's own buttons, before its study's policy withholds
     * any, and the list the chart last gave the row. A row showing any other
     * list was set by the host since, and that list becomes its own.
     */
    private readonly _legendActions;
    /** Saving and restoring the chart state; see chart-state.ts. */
    private readonly _persistence;
    private readonly _indicatorRanges;
    private readonly _ownedScaleRanges;
    private readonly _seriesOwners;
    /** Making series and the data paths behind their handles; see chart-series.ts. */
    private readonly _series;
    private _dataContext;
    private _barsProvider;
    private _barsRequests;
    private _barsProviderRevision;
    private _requestedDataRevision;
    /** Adding, moving and recomputing studies, and the host they talk to; see chart-studies.ts. */
    private readonly _studies;
    private _indicatorsDirty;
    private readonly _indicatorRefreshes;
    private readonly _indicatorReservedIds;
    /** Opaque drawing-tier payload, round-tripped through get/restoreState. */
    private _drawingState;
    private _alertState;
    /** Making, removing, moving, maximizing and folding panes, and their layout; see chart-panes.ts. */
    private readonly _layout;
    /**
     * Panes folded to a header strip. Held by pane identity, like `_pricePanes`,
     * so the fold follows its pane through a move or a removal above it without
     * any index to patch.
     */
    private readonly _collapsed;
    /** Legend rows per pane, so new ones stack below existing ones. */
    private readonly _legends;
    private readonly _studyLegends;
    /** Legend row stacking and offsets, the study count toggle, and legend button presses; see chart-legends.ts. */
    private readonly _legendStack;
    /**
     * Pane holding the primary price series (only this pane gets magnet
     * snapping). By identity, for the reason `_primaryPane` is: a move changes
     * its slot and nothing else about it.
     */
    private _firstPane;
    private _historyLoader;
    private _loadingHistory;
    private _priceFormatter;
    private _priceScaleOptions;
    /**
     * The panes whose numbers are the instrument's price, which is what decides
     * whether a chart-wide `minMove` reaches them (see `_scalePatchFor`).
     *
     * Held by pane identity rather than by index, for the reason spelled out in
     * `_createSeries`: `removePane` splices the array and `movePane` swaps two
     * entries, so a pane's slot number is not the pane. Weak because a removed
     * pane is destroyed and nothing else keeps it alive.
     */
    private readonly _pricePanes;
    /** Which scales a price-scale setting reaches, and one axis' state; see chart-scales.ts. */
    private readonly _scales;
    private _timeFormatter;
    private _timezone;
    private _leftAxisWidth;
    private _rightAxisWidth;
    private _axisColumnWidth;
    private _emptyPriceAxis;
    private _timeNav;
    private _schedulingTimeNav;
    /** Pane the navigator is currently attached to, so it can follow the bottom. */
    private _timeNavPane;
    private _branding;
    /** Branding options, the background text, the option batch and the exports; see chart-appearance.ts. */
    private readonly _appearance;
    private readonly _readByCollaborators;
    constructor(container: HTMLElement, options?: ChartOptions);
    /** Register a callback fired when the user pans near the left (oldest) edge. */
    setHistoryLoader(loader: () => void): void;
    /** Call after a history-paging load resolves to re-enable the trigger. */
    historyLoadComplete(): void;
    get dataLayer(): DataLayer;
    /**
     * Lay the time axis past the last bar out in these trading hours, or drop
     * them with null, and repaint every pane, so a drawing already placed past
     * the last bar moves to the time it now means.
     *
     * This is the call for a host setting hours on its own: a `SessionCalendar`
     * or an `Instrument` it holds, or any object with `sessionFrom`.
     * `Instrument.applyTo` and `SessionCalendar.applyTo` come here too.
     * `chart.dataLayer.setSessionCalendar` sets the same hours and asks for no
     * frame, for a host about to load bars or move the view anyway, either of
     * which repaints.
     */
    setSessionCalendar(calendar: SessionCalendarSource | null): void;
    /**
     * The primary series' bars as drawn, without allocating a history copy on each live
     * update: a transformed series' elements, which its handle's `getData` does not return.
     */
    primaryBars(): readonly Bar[];
    get timeScale(): TimeScale;
    /** Restore a saved logical range (e.g. preserve the user's zoom across a data reload). */
    setVisibleLogicalRange(range: LogicalRange): void;
    /** The current visible logical range. */
    getVisibleLogicalRange(): LogicalRange;
    /** Fit all bars into view (no-arg convenience; bar count from the data). */
    fitContent(): void;
    /** Current navigation preferences, safe to save as JSON. */
    navigationOptions(): Readonly<ChartNavigationOptions>;
    /**
     * A new default count or spacing immediately restores that view without dropping history.
     * `getState` saves these preferences, so the change is announced with `layout:change`;
     * a pan or zoom switch also fires `objects:change`, as it always has.
     */
    setNavigationOptions(patch: Partial<ChartNavigationOptions>): void;
    private _fitDefaultView;
    /** The keyboard shortcut manager (null when shortcuts are disabled). */
    get shortcuts(): ShortcutManager | null;
    /**
     * The data-driven trading layer: push positions/orders/trades and the chart
     * renders pills + markers, emitting `trading:*` events on interaction. Created
     * on first access.
     */
    get trading(): TradingController;
    /**
     * Whether the trade layer exists yet. Reading `chart.trading` creates one,
     * which subscribes to clicks and drags and so makes every `ns-resize` price
     * line draggable, so anything that merely inspects the chart (a settings
     * dialog) asks this first.
     */
    hasTrading(): boolean;
    /**
     * Trade-layer colours, whether or not the controller has been created. Once
     * it exists it is the single answer (a host may set colours on it directly);
     * before that, the held patch is folded onto the defaults. The fold is needed
     * because the two shapes name a colour differently: the patch says
     * `longColor`, the resolved palette says `long`.
     */
    tradingSettings(): TradingColors;
    /**
     * Recolour the trade layer. Safe before it exists: the patch is held and
     * handed over the moment `chart.trading` builds the controller.
     */
    setTradingSettings(patch: TradingSettings): void;
    /**
     * Announce that a setter changed the saved layout, unless it ran inside
     * another one or inside a restore: the outer call is the one announced.
     */
    private _layoutChanged;
    /** Run `fn` with the layout setters it calls counted as part of the caller's change. */
    private _withinLayoutChange;
    /** Add a series and return its data handle. */
    addSeries(type: SeriesType, options?: AddSeriesOptions): SeriesApi;
    /** Live renderer type, or null for a foreign, removed or destroyed series handle. */
    seriesType(series: SeriesApi): SeriesType | null;
    /** Frozen style snapshot, including renderer defaults, or null for an unavailable handle. */
    seriesStyle(series: SeriesApi): Readonly<SeriesStyle> | null;
    /**
     * Assign a host-owned series to a scale on its current pane without replacing it.
     * Both scales retain their configuration. Explicit series formatting applies to
     * the target as it does when adding a series, including a shared target scale.
     * Returns false for invalid IDs, unchanged assignments, unavailable handles or
     * indicator-owned plots, whose pane-bound visuals must move with the whole study.
     */
    setSeriesPriceScale(series: SeriesApi, scaleId: PriceScaleId): boolean;
    /**
     * Change a live series' renderer without replacing its handle, data or attachments.
     * Explicit styles survive; inherited renderer defaults give way to the new type.
     * It never transforms data: a `point-figure` or `kagi` renderer draws the bars
     * it is given, which is how a host that prepares its own elements keeps
     * working, and `setSeriesTransform` is how the chart applies one itself.
     * Returns false for an unchanged type or a foreign, removed or destroyed handle.
     * An unregistered type throws before any state changes.
     */
    setSeriesType(series: SeriesApi, type: SeriesType): boolean;
    /**
     * Have the chart apply a price-driven transform (Heikin Ashi, Renko, range
     * bars, line break, point and figure or Kagi, once `openalgo-charts/transform`
     * is imported) to a series, or draw its bars as given again with null. The
     * series keeps taking the host's own bars through `setData`, `update` and
     * `prependData`, and `getData` hands those back; the chart draws the elements
     * and forms them again on every tick. Studies read the elements unless one is
     * set to the underlying bars (`IndicatorApi.setBarSource`). A new transform
     * type selects its renderer; `setSeriesType` afterwards keeps the transform.
     * Returns false for an unchanged choice or an unavailable handle. An unknown
     * type or an invalid option throws before anything changes.
     */
    setSeriesTransform(series: SeriesApi, spec: SeriesTransformSpec | null): boolean;
    /** The transform a series was given, or null for none and for a handle not live on this chart. */
    seriesTransform(series: SeriesApi): SeriesTransformSpec | null;
    /**
     * The primary price series: the first one added, and the one the magnet
     * crosshair, the OHLC legend, the market-replay controller and a settings
     * dialog's Symbol tab all describe. Null until a price series exists.
     */
    primarySeries(): SeriesApi | null;
    /**
     * Type and live style of the primary series, for a settings dialog: the type
     * decides which controls apply (a candle has borders, a line has a dash), and
     * the style is the object `applyOptions` patches.
     */
    primarySeriesInfo(): {
        type: SeriesType;
        style: Readonly<SeriesStyle>;
    } | null;
    /**
     * Add a horizontal price line (order/SL/TP/alert/level) to a pane. Omitting
     * the pane means the primary price pane, wherever it sits.
     */
    addPriceLine(opts: PriceLineOptions, paneIndex?: number): PriceLine;
    /** Add an earnings/dividend/split event-marker strip to a pane, the primary price pane by default. */
    addEventMarkers(paneIndex?: number, options?: Partial<EventMarkersOptions>): EventMarkers;
    /**
     * Hand the chart the corporate-action / news calendar and let it own the
     * strip. The difference from `addEventMarkers` is who filters: holding the
     * full list here is what lets `setEventOptions` (the settings dialog's Events
     * switches) turn a type off and back on without the host re-supplying data.
     */
    setEvents(events: readonly ChartEvent[], paneIndex?: number): void;
    /** The chart-owned strip, or null before events or strip options are supplied. */
    eventMarkers(): EventMarkers | null;
    /** Configure clustering without replacing event data or group visibility. */
    setEventMarkerOptions(options: Partial<EventMarkersOptions>): void;
    setEventGroups(groups: readonly EventGroup[]): void;
    setEventGroupVisible(id: string, visible: boolean): void;
    /** Turn event types on/off. Unlisted types stay visible. */
    setEventOptions(patch: ChartEventOptions): void;
    eventOptions(): ChartEventOptions;
    /**
     * Add a registered indicator. Built-in descriptors live in the lazy
     * `openalgo-charts/indicators` tier: import it (or register your own with
     * `registerIndicator`) before calling this.
     *
     * `'onchart'` indicators overlay the price pane; `'pane'` indicators get a new
     * pane of their own unless `paneIndex` says otherwise. The returned handle
     * recomputes automatically whenever the source data changes.
     *
     * ```ts
     * import 'openalgo-charts/indicators';
     * const macd = chart.addIndicator('macd', { fastPeriod: 8 });
     * macd.setSettings({ fastPeriod: 12 });
     * macd.remove();
     * ```
     *
     * `options.policy` restricts what the user may do with the study (see
     * `IndicatorPolicy`); the host keeps changing it with `{ force: true }`.
     *
     * `options.instanceId` gives the study that id instead of a new one, so a
     * host that brings a removed study back (an undo) brings back its identity:
     * the studies that read its output and the alerts that name it find it
     * again. An id a study on this chart holds now throws, since two studies
     * cannot answer to one id; the id of a removed study is free to take back.
     *
     * `options.barSource` `'underlying'` has a study on a transformed chart
     * compute on the host's bars rather than the elements drawn (see
     * `IndicatorApi.setBarSource`).
     */
    addIndicator(indicatorId: string, settings?: Readonly<IndicatorSettings>, options?: AddIndicatorOptions): IndicatorApi;
    /**
     * Every live indicator instance, in renderer stacking order.
     *
     * Flushes any pending recompute first. Indicator maths is deferred to the
     * frame, so a caller that updates a bar and reads a value back in the same
     * turn would otherwise see the previous tick's numbers.
     */
    indicators(): readonly IndicatorApi[];
    /**
     * Move an existing study to an existing pane or a new pane at panes().length.
     * A study whose policy is not `movable` stays unless `options.force` is set.
     */
    moveIndicator(instanceId: string, paneIndex: number, options?: IndicatorEditOptions): boolean;
    /**
     * Change study stacking order among studies on the same pane. A study whose
     * policy is not `movable` stays unless `options.force` is set.
     */
    reorderIndicator(instanceId: string, direction: -1 | 1, options?: IndicatorEditOptions): boolean;
    /** Whether a call may make a change a study's policy reserves for its host. */
    private _policyAllows;
    /**
     * The series band of a pane, back to front: `'source:primary'` where the
     * price source is on this pane, and `'indicator:<id>'` for each study that
     * lives on it and plots a series here, in the order they paint. The same
     * ids as the object inventory. Host series and a study's plots placed on
     * another pane paint in the band too, in their own slots, but are not
     * entries of it.
     */
    seriesStack(paneIndex?: number): string[];
    /**
     * Move a source or study directly above or below another entry of the same
     * pane's series band (see `seriesStack`). A study keeps its other bands'
     * layers in the new study order, so its fills, levels and markers follow.
     * The drawings placed above an entry travel with it. False, with nothing
     * changed, for an entry of another pane, a move that changes nothing, or a
     * study whose policy is not `movable` unless `options.force` is set.
     */
    moveInSeriesStack(id: string, target: string, where: 'above' | 'below', options?: IndicatorEditOptions): boolean;
    /**
     * Paint an attached primitive inside the series band of its pane, directly
     * above the source or study `above` names (an id from `seriesStack`), or
     * pass null to return it to its own z-order band. While that entry plots
     * no series on the primitive's pane it paints in its own band. A drawing
     * layer is the case this exists for. False for a primitive on no pane.
     */
    setPrimitiveStackAbove(primitive: IPrimitive, above: string | null): boolean;
    /**
     * Remove one indicator instance by its handle id. Returns true if it existed
     * and went; a study whose policy is not `removable` stays unless
     * `options.force` is set.
     */
    removeIndicator(instanceId: string, options?: IndicatorEditOptions): boolean;
    /** Optional instrument identity supplied by the host, never inferred from bars. */
    getDataContext(): Readonly<ChartDataContext> | undefined;
    /** Instrument capability from the host. A missing bar reading does not change it. */
    get hasOpenInterest(): boolean | undefined;
    /** Clear the previous source bars before changing context, then load the new source. */
    setDataContext(context: ChartDataContext | undefined): void;
    /**
     * Register, replace or remove (`null`) the provider indicators reach through
     * `requestBars`. Read at request time, so an indicator added before the
     * provider was set is served once one exists.
     */
    setBarsProvider(provider: IndicatorBarsProvider | IndicatorBarsProviderAccess | null): void;
    private _cancelBarsRequests;
    /** Announce changed requested data without replacing a provider or price bars. */
    invalidateRequestedData(): void;
    /** Whether the configured provider supplies explicit availability snapshots. */
    hasSnapshotProvider(): boolean;
    /** Whether a bars provider is registered, so a host can grey what needs one. */
    hasBarsProvider(): boolean;
    /** Replace chart-owned branding. Manually attached primitives are independent. */
    setBranding(options: boolean | LogoWatermarkOptions): void;
    /** Host branding options, excluded from saved chart state. */
    brandingOptions(): false | LogoWatermarkOptions;
    /** Patch background text preferences. Boolean input changes visibility only. */
    setWatermarkOptions(options: boolean | ChartWatermarkOptions): void;
    /** JSON-safe preferences. Automatic text remains blank in this snapshot. */
    watermarkOptions(): Readonly<ChartWatermarkOptions>;
    /**
     * The `IndicatorHost` a study instance talks to; see chart-studies.ts. Kept
     * here by name because the restore and tests build a host through it.
     */
    private _indicatorHost;
    /**
     * Subscribe to clicks on hit-testable primitives (markers, events, lines),
     * with the primitive's `externalId`. Every subscriber hears every click, the
     * trade layer's among them; the function returned unsubscribes this one.
     *
     * Before 2.6.0 each call replaced the previous callback, so a host that
     * subscribes again on the same chart calls the returned function first.
     */
    subscribeClick(cb: (externalId: string) => void): () => void;
    /**
     * Subscribe to crosshair movement for an OHLC legend / tooltip. The callback
     * fires with the hovered bar of the primary price series on every move, and
     * with all-null fields when the pointer leaves the plot. A linked crosshair
     * also updates the readout, with source 'linked' and no pointer coordinates.
     * Several may subscribe, as with `subscribeClick`; the function returned
     * unsubscribes this one.
     */
    subscribeCrosshairMove(cb: (e: CrosshairMoveEvent) => void): () => void;
    /**
     * Update the readout under a link group's separately drawn crosshair. The
     * physical pointer takes precedence. This never emits a pointer move event,
     * so hosts do not interpret it as drawing input or echo it to another group.
     */
    setLinkedCrosshairIndex(index: number | null): void;
    private _readoutIndex;
    /**
     * Subscribe to drags of draggable primitives (order / SL / TP lines, drawing
     * handles). Fires per move and on release.
     *
     * `time` is the UTC seconds under the cursor, interpolated between bars and
     * extrapolated past the right edge, so a two-axis drag (a trendline endpoint,
     * a projection) has a usable time even where the gapless axis has no bar.
     * Price-only consumers can simply ignore it.
     *
     * Several may subscribe, as with `subscribeClick`; the function returned
     * unsubscribes this pair. While any subscription holds, a price line whose
     * cursor is `ns-resize` drags instead of panning the chart.
     */
    subscribeDrag(onDrag: (externalId: string, price: number, time: number) => void, onDragEnd?: (externalId: string, price: number, time: number) => void): () => void;
    /**
     * Guarantee a pane's price scale has a real range before converting between y and price.
     * Autoscaling normally happens during paint, so every coordinate API (and
     * the price carried by click/drag events) used to answer with the default
     * 0..1 (or ±Infinity) until the first frame had run. Callers cannot be asked
     * to wait for a paint, so scale on demand.
     */
    private _ensureScaled;
    /** Container-relative x (media px) to UTC seconds on the (gapless) time axis. */
    private _xToTime;
    /** UTC seconds to container-relative x (media px). The inverse of `_xToTime`. */
    timeToCoordinate(time: number): number;
    /** Container-relative x (media px) to UTC seconds. */
    coordinateToTime(x: number): number;
    private readonly _bus;
    /** Subscribe to a chart event, typed by its {@link ChartEventMap} name. Returns an unsubscribe function. */
    on<K extends keyof ChartEventMap>(event: K, cb: (payload: ChartEventMap[K]) => void): () => void;
    /** @deprecated Removed in 3.0.0. Use a {@link ChartEventMap} name, or add yours to it by declaration merging (since 2.6.0). */
    on(event: string, cb: (payload: unknown) => void): () => void;
    /** Subscribe to the next occurrence of an event, then unsubscribe. Returns a function that cancels it first. */
    once<K extends keyof ChartEventMap>(event: K, cb: (payload: ChartEventMap[K]) => void): () => void;
    /** @deprecated Removed in 3.0.0. Use a {@link ChartEventMap} name, or add yours to it by declaration merging (since 2.6.0). */
    once(event: string, cb: (payload: unknown) => void): () => void;
    /** Remove one listener, or (when `cb` is omitted) every listener for an event. */
    off<K extends keyof ChartEventMap>(event: K, cb?: (payload: ChartEventMap[K]) => void): void;
    /** @deprecated Removed in 3.0.0. Use a {@link ChartEventMap} name, or add yours to it by declaration merging (since 2.6.0). */
    off(event: string, cb?: (payload: unknown) => void): void;
    /**
     * Put an event on the chart's bus, for every listener of that name.
     *
     * @deprecated Removed in 3.0.0. The bus carries the chart's own events. Announce an
     *   instrument with `setDataContext`, drive a link group with `LinkGroup.setSymbol`,
     *   `setInterval` and `setChartType` (all three since 2.5.10), and keep events of your own on your own emitter.
     */
    emit(event: string, payload: unknown): void;
    /** @internal The engine's own emit, checked against {@link ChartEventMap}. */
    _emit<K extends keyof ChartEventMap>(event: K, payload: ChartEventMap[K]): void;
    /** Keep internal intermediate ranges from repainting or interrupting a gesture. */
    private _mutateTimeScale;
    /** Emit a viewport event ('pan' | 'zoom') carrying the visible time + logical range. */
    private _emitViewport;
    /**
     * Emit a viewport event for a change whose kind is not known up front, and
     * only if the window actually moved.
     *
     * The gesture paths know exactly what happened (a wheel is a zoom, a drag is
     * a pan) and emit directly. The programmatic paths do not: restoring a saved
     * range, fitting content or pressing an arrow key can move the window, resize
     * it, or do nothing at all because the scale was already there or clamped.
     * The span is the discriminator a listener cares about, and the no-move check
     * matters because a linked grid would otherwise re-broadcast on every no-op.
     */
    private _emitViewportIfMoved;
    /**
     * Public: attach any primitive (indicators, profiles, custom overlays) to a
     * pane, the primary price pane when none is named, or to a chart anchor.
     */
    addPrimitive(primitive: IPrimitive, where?: number | PrimitivePlacement): void;
    /**
     * Map a price to a container-relative Y in media (CSS) px, for positioning DOM
     * overlays (order panels, tooltips) over a pane. Returns null if the pane
     * doesn't exist or is collapsed to its header strip, which plots no price.
     * The inverse is `coordinateToPrice`.
     */
    priceToCoordinate(price: number, paneIndex?: number): number | null;
    /**
     * Map a container-relative media-px Y back to a price on a pane (inverse of
     * priceToCoordinate). Null where that is, for the same panes. Both default
     * to the primary price pane wherever it sits.
     */
    coordinateToPrice(y: number, paneIndex?: number): number | null;
    /**
     * A pane's plot in container px: `left` and `top` from the container's
     * top-left corner, inside the price axis columns and above the time axis,
     * and the size a primitive on that pane paints into. What a host lays an
     * overlay against, and what a drawing pinned to the screen is a fraction of.
     * Null for a pane with no plot on screen: one collapsed to its header
     * strip, one hidden behind a maximized pane, or no pane at that index.
     *
     * The pane is scaled first, the way a price conversion scales it: a caller
     * that places something on the plot reads the pane's prices next, and a
     * pane no frame has painted yet (just made, or just moved) still holds its
     * placeholder range.
     */
    plotRect(paneIndex: number): PlotRect | null;
    /**
     * Grid lines at runtime: visibility of each axis, plus its colour, dash,
     * width and spacing. Omitted fields keep their current value. Repaints every
     * pane.
     */
    setGridOptions(opts: Partial<GridOptions>): void;
    /** Current grid options, visibility first (it is the one field always set). */
    gridOptions(): {
        vertLines: boolean;
        horzLines: boolean;
    } & Partial<GridOptions>;
    /**
     * The Canvas option block (grid, crosshair, scale text/lines, plot margins).
     * Each sub-block merges field by field, so setting one grid colour leaves the
     * rest of the grid alone.
     */
    setCanvasOptions(patch: CanvasOptions): void;
    /** The Canvas option block as it stands (theme fallbacks are not folded in). */
    canvasOptions(): CanvasOptions;
    /**
     * Price-scale options for every pane (mode, inverted, tick size, margins),
     * and the default new panes inherit.
     *
     * `scope` says how far a chart-wide setting reaches:
     *  - `'primary'` (default): each pane's right scale only. A mode change
     *    wants this: rebasing a volume overlay quotes percent change in lots.
     *  - `'axes'`: every scale that draws a ladder, so the left axis moves with
     *    the right. Plot margins want this.
     *  - `'all'`: the hidden overlay scales too. Almost nothing should: an
     *    overlay's margins are its creator's placement, see `Pane.axisScales`.
     *
     * `minMove` is the one field no scope carries onto a pane that does not quote
     * the instrument, whichever scope is asked for: see `_scalePatchFor`. Every
     * other field is a property of the axis and reaches exactly as far as `scope`
     * says.
     */
    setPriceScaleOptions(patch: Partial<PriceScaleOptions>, scope?: 'primary' | 'axes' | 'all'): void;
    /**
     * The chart-wide price-scale defaults: every field `setPriceScaleOptions`,
     * the `priceScale` construction option and the canvas margins have set, and
     * what a pane added later starts from. `priceScaleOptions()` reads the price
     * pane's own scale instead, which a change made to that one axis moves and
     * this does not, so the two together tell a chart-wide change from a
     * one-axis one. `minMove` here reaches only the panes that quote the
     * instrument. A detached copy; empty when nothing was ever set.
     */
    priceScaleDefaults(): Partial<PriceScaleOptions>;
    /** The primary pane's price-scale options (what the Scales tab reads), wherever it sits. */
    priceScaleOptions(): PriceScaleOptions;
    /**
     * Put every pane's price axis back under autoscale, or pin it where it is.
     * `PriceScale.setAutoScale` alone changes nothing on screen until something
     * else asks for a frame; this re-measures and repaints.
     */
    setAutoScale(on: boolean): void;
    /** Whether only the primary series contributes to autoscale on its current scale. */
    priceOnlyAutoScale(): boolean;
    /** Keep manual ranges and ratio locks while choosing which data auto-fit measures. */
    setPriceOnlyAutoScale(on: boolean): void;
    /**
     * State of one price axis, for a host rendering a menu over it: what is
     * currently on, and which items are worth offering. Null for a pane that does
     * not exist.
     *
     * `active` false is a scale no series maps to: the ladder on an empty chart,
     * or the side a menu was raised on before anything was plotted there. That is
     * a row to render disabled with its state visible, not one to leave out.
     */
    priceAxisState(paneIndex?: number, scaleId?: PriceScaleId): PriceAxisState | null;
    /** Detached visible placement. Hidden named scales retain their independent range. */
    priceAxisPlacement(paneIndex: number, scaleId: PriceScaleId): PriceAxisPlacement | null;
    /** Move or reorder a scale without changing its ID, sources, formatter or range. */
    setPriceAxisPlacement(paneIndex: number, scaleId: PriceScaleId, side: PriceAxisSide, order?: number): boolean;
    /** Active price columns in pane CSS coordinates, ordered nearest the plot on each side. */
    priceAxisLayout(paneIndex?: number): readonly PriceAxisSlot[];
    private _priceAxisPane;
    /**
     * Options for one pane's scale (mode, invert, tick size, margins) rather than
     * every pane's. The four modes are one field, so picking one drops the
     * previous by construction: a menu renders them as a single choice.
     */
    setPriceAxisOptions(paneIndex: number, scaleId: PriceScaleId, patch: Partial<PriceScaleOptions>): void;
    /**
     * Auto-fit one axis: its range tracks the data again, or stays where the user
     * left it. Turning it on releases any ratio lock on that axis, for the reason
     * given in `setAutoScale`.
     */
    setPriceAxisAutoFit(paneIndex: number, scaleId: PriceScaleId, on: boolean): void;
    /**
     * Pin one axis' price-per-bar ratio: zooming the time axis then rescales the
     * prices with it, so a trend drawn at 45 degrees stays at 45 degrees. The
     * axis goes manual, because auto-fit would re-fit the data every frame and
     * undo the ratio being held.
     *
     * Returns whether the axis is now in the state asked for. Locking fails on a
     * scale nothing has measured: there is no ratio to hold on an empty pane, or
     * on one whose series plot no values at all.
     */
    setPriceAxisLockRatio(paneIndex: number, scaleId: PriceScaleId, on: boolean): boolean;
    /**
     * Move a pane's price axis to the other strip, taking the series that map to
     * it and everything the axis was set to. Returns false when that side carries
     * nothing, or when the other side is already occupied: one strip draws one
     * axis (see `Pane.moveSeriesScale`), which is what `movable` reports.
     *
     * @deprecated Removed in 3.0.0. Use {@link Chart.setPriceAxisPlacement} (since 2.5.4), which moves a scale's
     * column and keeps its id; this method instead swaps the built-in side scales and reassigns their series and
     * studies.
     */
    movePriceAxis(paneIndex: number, from: 'right' | 'left', to: 'right' | 'left'): boolean;
    /**
     * Per-field status-line switches, applied to every pane legend on the chart:
     * the host's symbol row and the indicator rows alike, which is what makes one
     * switch mean the same thing everywhere. Merges field by field.
     */
    setStatusLineOptions(patch: LegendStatusLineOptions): void;
    statusLineOptions(): LegendStatusLineOptions;
    /** Whether study legend rows are replaced by their applied-instance count. */
    indicatorLegendCollapsed(): boolean;
    /** A display preference only; studies retain visibility, calculation and subscription state. */
    setIndicatorLegendCollapsed(on: boolean): void;
    /**
     * How large a legend's action buttons are drawn, in media px.
     *
     * Chart-wide rather than per legend: the rows stack against the height the
     * buttons need, so two sizes on one pane would stack against two different
     * heights and overlap. The primitive holds it to a range it can actually
     * draw.
     */
    setLegendIconSize(size: number): void;
    legendIconSize(): number | undefined;
    /**
     * Turn the axis-strip chrome on or off, and hand it a clock. Merges field by
     * field, so switching the countdown on leaves the corner clock alone.
     */
    setAxisChromeOptions(patch: AxisChromeOptions): void;
    /** The axis-chrome switches as they stand. */
    axisChromeOptions(): AxisChromeOptions;
    /** Crosshair behaviour ('normal' or 'magnet'). Set it via `applyOptions`. */
    crosshairMode(): CrosshairMode;
    /** Whether the vertical crosshair snaps to an existing primary bar's center. */
    crosshairSnapToBar(): boolean;
    /**
     * Whether the price pane may leave the top of the stack, see
     * `ChartOptions.movablePrimaryPane`. False unless the host opted in, and
     * while it is false the price pane stays at slot 0 for the life of the chart.
     */
    movablePrimaryPane(): boolean;
    /** The active palette. Swap it with `setTheme`. */
    theme(): ChartTheme;
    /**
     * Flatten every pane's base + overlay canvas into one opaque canvas (device
     * px). The chart renders as stacked layered canvases, so the browser's native
     * right-click "Save image" only captures the layer under the pointer (usually
     * the transparent crosshair overlay): use this to export the full chart.
     */
    takeScreenshot(): HTMLCanvasElement;
    /**
     * The chart as a standalone SVG document: every pane's base and overlay
     * paint, in the order the DOM stacks them, run once into a serialising
     * context at pixel ratio 1. Text stays text (selectable, searchable) and
     * lines stay lines, so the file scales without the blur a PNG picks up.
     *
     * Nothing transient is in it: no crosshair, no hover state, no drag. What is
     * in it is exactly what the renderers, primitives and drawing tools draw,
     * because it is the same code drawing. A primitive that paints through a
     * call with no vector form (a bitmap logo, a shadow) is simply thinner in
     * the export; see `SvgContext` for the list.
     *
     * Returns the string only. Saving it is the host's job, the way
     * `downloadScreenshot` is the host-facing half of `takeScreenshot`:
     * `new Blob([svg], { type: 'image/svg+xml' })` and an anchor is all it takes.
     */
    exportSVG(options?: ExportSvgOptions): string;
    /** Chart-anchored primitives, re-homed by `_rehomeAnchored`. */
    private readonly _anchored;
    /** Remove a primitive from whichever pane holds it. */
    removePrimitive(primitive: IPrimitive): void;
    /**
     * A host for the (lazy-loaded) trade layer to attach/detach its primitives
     * on a pane. Without an index it follows the primary price pane, resolved at
     * each attach, so an order line drawn after the price pane moved still lands
     * beside the candles.
     */
    tradeHost(paneIndex?: number): {
        addPrimitive(p: IPrimitive): void;
        removePrimitive(p: IPrimitive): void;
    };
    /**
     * The render backend the chart is painting series with right now: what the
     * first pane was given, which is the `renderer` option unless its factory
     * declined (no WebGL2 on this device) and the 2D backend stood in, and
     * `canvas2d` from the moment a GPU backend degrades ('renderer:fallback').
     */
    get rendererKind(): RenderBackendKind;
    /**
     * The name `rendererKind` shipped under; the same value.
     *
     * @deprecated Removed in 3.0.0. Use {@link Chart.rendererKind} (since 2.0.0), which reports the same backend.
     */
    get renderer(): RenderBackendKind;
    /** A backend for one more pane, with the 2D one standing in for a refusal. */
    private _newBackend;
    /**
     * Leave the GPU backend for good. Every pane is switched in the same call
     * so the chart never paints half its panes on each path, later panes get
     * the 2D factory, and the frame that noticed is repainted on the new
     * backends (the one just painted went through the old backend's own 2D
     * path, so nothing was blank in between).
     */
    private _fallbackToCanvas2d;
    /**
     * Set a custom price formatter for every pane's axis labels, last-price tag,
     * and price-line labels at runtime (e.g. switch to a currency format). Pass
     * null to restore the default tick-size-aware formatting.
     */
    setPriceFormatter(fn: ((price: number) => string) | null): void;
    /**
     * Set a custom time-axis + crosshair label formatter (UTC seconds -> string)
     * at runtime. Pass undefined to go back to the default labels, which follow
     * the chart's `timezone` (IST unless set).
     */
    setTimeFormatter(fn: ((utcSeconds: number, tickMark?: TickMarkType) => string) | undefined): void;
    /** The IANA zone the chart labels time in. */
    timezone(): string;
    /**
     * Change the zone the time axis and crosshair label in, without rebuilding the
     * chart: a terminal switching from an NSE symbol to a US one needs exactly
     * this. Throws on a name the runtime does not recognise, rather than quietly
     * labelling in the old zone, because a chart showing the wrong hours is the
     * kind of wrong nobody notices until it costs money.
     */
    setTimezone(zone: string): void;
    /**
     * Turn pointer gestures into anchor placement instead of panning. A host arms
     * this while a drawing tool is active: a press no longer scrolls the chart, and
     * a press-drag-release is reported as two `click` events (press point, then
     * release point, the latter tagged `viaDrag`) so a two-point shape can be drawn
     * in one gesture. `DrawingController` drives this for you.
     */
    setPlacementMode(active: boolean): void;
    /**
     * Arm the next plot click to answer with a price, a bar time, or both as a
     * `'point'`, handed to `cb`. Returns a cancel function; arming another pick
     * on this chart cancels the pending one. `pick:start` and `pick:end` bracket
     * it so a host can show its own cursor while the pick is live. A target's
     * pane limits where the click counts and its scale is the one a price, a
     * point's included, is read on. See `input/pick` for why this does not touch
     * placement mode.
     */
    beginPick(kind: 'point', cb: (value: PickPoint) => void, options?: PickOptions): PickHandle;
    beginPick(kind: PickKind, cb: (value: number) => void, options?: PickOptions): PickHandle;
    /** Swap the palette at runtime (dark/light toggle) without recreating the chart. */
    setTheme(theme: ChartTheme): void;
    /**
     * Apply a subset of chart options at runtime (theme, grid, formatters,
     * crosshair mode) without recreating the chart.
     */
    applyOptions(opts: ChartApplyOptions): void;
    panes(): readonly Pane[];
    /**
     * Capture the chart's serialisable state: viewport, grid, crosshair mode,
     * pane weights and price scales, indicator instances, and a `drawings` slot
     * the drawing tier fills. JSON-safe.
     *
     * Series **data** is not captured: the app owns that (it knows the symbol,
     * the timeframe, and the feed). Series *descriptors* are, so an app that
     * rebuilds its own series can re-apply their styling and placement.
     */
    getState(): ChartState & ChartSettingsState & {
        timezone: string;
    };
    /**
     * Re-apply a state captured by `getState`. Restores grid, crosshair mode,
     * pane weights and price scales, indicators, and the viewport: everything
     * the chart is the source of truth for.
     *
     * It does **not** recreate series: the chart has no way to know their data.
     * The returned report lists the series descriptors it saw so the caller can
     * rebuild them (`addSeries(s.type, { paneIndex: s.paneIndex, style: s.style })`)
     * and then feed them.
     *
     * Restore the viewport *after* your data lands: logical ranges index bars, so
     * a range applied to an empty chart means nothing. Call `restoreState` again
     * (or `setVisibleLogicalRange`) once the series are populated.
     */
    restoreState(state: unknown, options?: ChartRestoreOptions): RestoreReport;
    /**
     * The opaque `drawings` slot in the chart state. The base engine only
     * round-trips it; the drawing tier reads and writes it.
     */
    drawingState(): unknown;
    setDrawingState(value: unknown): void;
    /**
     * A price rounded to the tick the pane's own axis is written with, or, on
     * the price pane of an instrument with a tick schedule, to the tick of the
     * band the price falls in.
     *
     * A dragged alert's price comes from a pointer, and a pixel maps to a price
     * with a dozen decimals behind it: dropped where the axis reads 1255.90 it
     * was stored as 1255.8706204379562, a price the instrument cannot trade at
     * and a number nothing in the interface could show. The scale already knows
     * the tick, because it is the one the axis is written with, so this is the
     * chart's answer rather than something every host works out again.
     *
     * A scale holds one tick, and with a schedule that is the grid every band
     * lies on: 105.87 is on a 0.01 grid and still no price in a 0.25 band. So
     * the price pane, whose prices are the instrument's, asks the schedule.
     * Another pane is in a study's own units, which no schedule describes.
     *
     * A scale with no declared tick rounds nothing: there is no tick to round to
     * and inventing one would move a price somebody chose.
     */
    snapPrice(paneIndex: number, price: number): number;
    /**
     * The instrument's tick schedule, or null for a constant tick, the default.
     * `Instrument.applyTo` sets it from the instrument's `tickBands`.
     */
    tickSchedule(): TickSchedule | null;
    /**
     * Hand the chart the instrument's price-dependent ticks, for a host that
     * keeps its own instrument metadata rather than calling
     * `Instrument.applyTo`. Price alerts and anything else rounding through
     * {@link snapPrice} on the price pane then land in the band a price falls
     * in, and the trading layer's order and bracket drags take the same
     * schedule, now or when it is built. Null restores the constant tick. It
     * describes the loaded instrument, so it is not part of the saved state.
     */
    setTickSchedule(schedule: TickSchedule | null): void;
    /** Detached JSON state, also available when no alert controller is attached. */
    alertState(): AlertsDocument | undefined;
    /** Runtime snapshot. JSON safety of opaque payloads is checked when state is read. */
    setAlertState(document: AlertsDocument | undefined): void;
    private _reserveAlertStudyIds;
    invalidate(build: (mask: InvalidateMask) => void): void;
    applySize(width: number, height: number): void;
    /** Stays on the chart by name: its callers here and tests read the bottom pane through it. */
    private _bottomPaneIndex;
    /**
     * Set a pane's relative height weight. Panes share the chart height in
     * proportion to their weights, so only the ratio matters.
     */
    setPaneWeight(index: number, weight: number): void;
    paneWeight(index: number): number;
    /**
     * Remove a pane, everything drawn in it, and any indicator that lives there.
     * The primary price pane is never removable, in whatever slot it sits:
     * removing it would leave the chart with no price series to describe and
     * nothing for a series, a price line or an on-chart study to default to. A
     * study pane is removable in any slot, the top one included.
     *
     * Returns false when the index is out of range or names the primary pane,
     * and when it holds a study whose policy is not `removable` unless
     * `options.force` is set.
     */
    removePane(index: number, options?: IndicatorEditOptions): boolean;
    /**
     * Move a pane up or down one slot, swapping it with its neighbour.
     *
     * By default the primary price pane is pinned at the top, as it always was:
     * a move that would take it off slot 0, or put a study pane above it, is
     * refused, so a host that means the price pane when it passes 0 keeps
     * meaning it. With `movablePrimaryPane` on, any pane moves, the price pane
     * included, and any pane can displace it: the price pane is then an identity
     * rather than a slot (see `primaryPaneIndex`), and everything that means
     * "the price pane" follows it. Returns false for an unknown slot, a
     * direction other than -1 or 1, a move off either end, or a pinned price pane.
     */
    movePane(index: number, direction: -1 | 1): boolean;
    /**
     * Where the primary price pane sits in the stack, counted from the top. The
     * primary pane is the one the chart is built with. It holds the price series
     * a host adds without naming a pane and the on-chart studies, and it is what
     * every call that defaults to "the price pane" means: `addSeries`,
     * `addPriceLine`, `addEventMarkers`, `addPrimitive`, `tradeHost`, the
     * coordinate calls, comparisons, and a drawing magnet or price alert. It is
     * 0 until something moves it, and always 0 on a chart without
     * `movablePrimaryPane`; it is never removed and never collapsed.
     */
    primaryPaneIndex(): number;
    /**
     * Move the primary price pane to another slot, for example below its
     * studies. It goes one adjacent move at a time, so each step is an ordinary
     * `movePane` with its own `paneMoved` event: drawings, alerts and anything a
     * host keys by slot follow it the way they follow any other move. Weights,
     * scales, a fold or a maximize stay with the panes that own them. Returns
     * false for a slot that does not exist or the slot it already holds, and
     * always false on a chart built without `movablePrimaryPane`, where the price
     * pane stays at the top: that option is the one switch for every move.
     */
    setPrimaryPaneIndex(index: number): boolean;
    /** The primary pane's slot; 0 once the chart is torn down and there are no panes. */
    private _primaryIndex;
    /**
     * Expand one pane to fill the chart, hiding the others. Calling it again (or
     * on another pane) puts the stack back exactly as it was, since the stored
     * weights were never disturbed.
     */
    maximizePane(index: number): boolean;
    /** The maximized pane index, or null when none is. */
    maximizedPane(): number | null;
    /**
     * Fold a pane to a header strip, or open it again. A collapsed pane keeps
     * everything it holds: its series still take data, its studies still
     * recompute, and its drawings, scales and stored weight are untouched, so
     * opening it brings back exactly the height it had. The strip shows one
     * legend row, the pane's first study row with the control that opens it,
     * ahead of any row the host placed there, and on the bottom pane the time
     * axis; nothing else in it paints or answers the pointer, and no coordinate
     * conversion maps a price onto it.
     *
     * The primary price pane stays open in whatever slot it sits, top, middle or
     * bottom: it carries the price the chart exists to show, and a chart always
     * keeps one open pane for its furniture and the time navigator. Collapse is
     * a property of a pane rather than a slot, so a fold travels with its pane
     * through a move, and a study pane moved above the price pane, to slot 0,
     * folds like any other. Collapsing the maximized pane ends the maximize,
     * since a strip cannot fill the chart; maximizing a collapsed pane shows it
     * whole until the maximize ends. Returns false for the primary pane, an
     * unknown index, a value that is not a boolean, or no change.
     */
    setPaneCollapsed(index: number, collapsed: boolean): boolean;
    /** Whether a pane is collapsed to its header strip. The primary price pane never is. */
    paneCollapsed(index: number): boolean;
    /** Stays on the chart by name: the pointer release and tests route a legend press through it. */
    private _handleLegendAction;
    /** Stays on the chart by name: its callers here and tests read the pane boxes through it. */
    private _paneLayout;
    /**
     * Keyboard hints for the navigator tooltips, read from the live keymap so a
     * rebind shows up in the tooltip instead of a stale hardcoded string. The
     * one-bar step buttons have no default binding, so they get no hint.
     */
    private _navHints;
    /**
     * Keep the navigator on the lowest open pane: it belongs just above the time
     * axis, and adding, removing or collapsing a pane moves which one that is.
     * Its current pane is found by identity: a pane removed above it shifts the
     * slot it was added at, and removing it from that stale slot missed it and
     * attached it a second time.
     */
    private _syncTimeNavPane;
    /**
     * Push the pointer to the navigator and keep painting while it fades, so the
     * animation runs even when nothing else on the chart is changing.
     */
    private _feedTimeNav;
    /** How one pane paints and hit-tests right now: the bottom one carries the time axis. */
    private _renderContext;
    /**
     * The corner clock's options, or undefined when it is off. Built per frame so
     * a zone change reaches it without the pane holding a stale copy.
     */
    private _sessionClockOptions;
    /**
     * The countdown row's options, or undefined when it is off or there is
     * nothing to count. The interval is read back from the bars rather than
     * configured: the chart is never told its own timeframe, and a chart that
     * switched timeframe mid-session has to follow within a screen of bars.
     */
    private _barCountdownOptions;
    private readonly _onPixelRatio;
    private readonly _checkPixelRatio;
    /**
     * Run the pending frame now rather than on the next animation frame, for
     * the callbacks that clear a canvas once this frame's animation callbacks
     * have run: a resize observed before the browser paints, a new pixel ratio.
     * Waiting would show the cleared canvas for a frame.
     *
     * Not with a scheduler the host injected (`raf`): that host owns every frame
     * the chart paints, as with the kinetic glide, so the frame already asked
     * for runs when the host runs it.
     */
    private _paintNow;
    private _onFrame;
    private _paintFrame;
    private readonly _onPointerEnter;
    private readonly _onContextMenu;
    private readonly _onPointerDown;
    private readonly _onPointerMove;
    private readonly _onPointerUp;
    private readonly _onPointerUpNative;
    private readonly _onPointerCancel;
    private readonly _onLostPointerCapture;
    private readonly _onPointerLeave;
    private readonly _onWheel;
    private readonly _onDblClick;
    private readonly _onKeyDown;
    /** Stays on the chart by name: the navigator's buttons and tests run a command through it. */
    private _runShortcut;
    /** Stays on the chart by name, for the frame and for tests; the pointer handlers set it. */
    private get _cursor();
    /**
     * Restore the preferred visible bar count and re-enable
     * auto-scaling on every price axis (undoing any pan/zoom or manual axis drag).
     * Same as double-clicking the chart.
     */
    resetScale(): void;
    /**
     * Composite the full chart (all panes + overlays) and trigger a PNG download.
     * This is what the screenshot keyboard shortcut runs; call it from a toolbar
     * button for a reliable "save image": the browser's native right-click
     * "Save image as…" captures only the topmost (transparent overlay) canvas.
     */
    downloadScreenshot(filename?: string): void;
    /** Refresh the polite live-region summary screen readers announce. */
    private _updateAccessibleSummary;
    private _maybeLoadHistory;
    /** Stays on the chart by name: the pointer release and tests start a glide through it. */
    private _startKinetic;
    /**
     * True once `destroy()` has run. Anything holding a chart it did not create
     * (a link group, a controller, a host cache) needs to know the object is a
     * corpse before it calls into it: inferring it from a side effect such as an
     * empty pane list works only for as long as nothing else can empty one.
     */
    get isDestroyed(): boolean;
    private _destroyed;
    private _destroying;
    destroy(): void;
}
/** Create a chart inside the given container element. */
declare function createChart(container: HTMLElement, options?: ChartOptions): Chart;

/**
 * Vector export.
 *
 * `SvgContext` implements the subset of `CanvasRenderingContext2D` the
 * renderers, primitives and drawing tools call, and serialises every call to
 * SVG. The chart's ordinary paint, run once into one of these instead of the
 * pane canvases, comes out as a standalone document: text stays text, lines
 * stay lines, and the file scales without the blur a PNG picks up.
 *
 * The subset is the one the test recorder (`tests/helpers/fake-ctx.ts`) already
 * had to cover for the same code paths, plus what the drawing tools add
 * (curves, ellipses, rounded rectangles). The op set is the contract: anything
 * that draws to a pane through a method not here does not reach the export.
 *
 * What is out of reach for a serialiser, and what it does about it:
 *
 * - `setTransform`, `resetTransform`, `getTransform`: a whole-matrix reset has
 *   no place in a document built from nested groups. `translate`, `scale` and
 *   `rotate` are supported and are what the paint code uses.
 * - `createRadialGradient`, `createConicGradient`, `createPattern`,
 *   `getImageData`, `putImageData`, `createImageData`, `strokeText`,
 *   `drawFocusIfNeeded`, `isPointInPath`, `isPointInStroke`, and `fill`,
 *   `stroke` or `clip` handed a `Path2D`.
 * - `drawImage` of anything that is not an `<img>` with a `src` or a canvas
 *   that can hand over a data URL.
 *
 * Each of these throws under `strict` (the tests run that way, so a renderer
 * that grows a new call fails loudly) and otherwise records its name in
 * `unsupported` and paints nothing, which is the right production answer: a
 * missing logo is a better export than a thrown one. Shadow, composite and
 * smoothing properties are accepted and ignored, since the one primitive that
 * sets a shadow is decoration on a button.
 *
 * `measureText` is approximate: there is no font engine here, so widths come
 * from a per-character table scaled by the font size (see `charWidth`). The
 * paint code uses it to size tag boxes and cull overlapping axis labels, both
 * of which tolerate a few percent.
 */
/** Options for an `SvgContext`. */
interface SvgContextOptions {
    /**
     * Throw on any call the serialiser cannot express. Off, the call is recorded
     * in `unsupported` and skipped.
     */
    strict?: boolean;
    /**
     * What `clearRect` paints. A canvas clears to transparent, which a document
     * that is appended to cannot do; when the export is opaque the honest stand-in
     * is the background colour. Absent, `clearRect` paints nothing.
     */
    background?: string | undefined;
}
/** A gradient handle: set it as `fillStyle` the way a canvas one is set. */
declare class SvgLinearGradient {
    readonly id: string;
    readonly x0: number;
    readonly y0: number;
    readonly x1: number;
    readonly y1: number;
    readonly stops: {
        offset: number;
        color: string;
    }[];
    constructor(id: string, x0: number, y0: number, x1: number, y1: number);
    addColorStop(offset: number, color: string): void;
    toString(): string;
}
/**
 * A 2D context that writes SVG. Construct one at the document's media size,
 * hand it (cast) to whatever paints, and read `toString()`.
 */
declare class SvgContext {
    readonly width: number;
    readonly height: number;
    fillStyle: string | SvgLinearGradient;
    strokeStyle: string | SvgLinearGradient;
    lineWidth: number;
    lineCap: CanvasLineCap;
    lineJoin: CanvasLineJoin;
    miterLimit: number;
    globalAlpha: number;
    font: string;
    textAlign: CanvasTextAlign;
    textBaseline: CanvasTextBaseline;
    shadowColor: string;
    shadowBlur: number;
    shadowOffsetX: number;
    shadowOffsetY: number;
    globalCompositeOperation: string;
    imageSmoothingEnabled: boolean;
    /**
     * Deliberately undefined: a primitive that wants an offscreen canvas asks the
     * context for its element, and the honest answer is that there is none.
     */
    readonly canvas: undefined;
    /** Names of the calls that could not be exported, first occurrence only. */
    readonly unsupported: string[];
    private readonly _out;
    private readonly _frames;
    private readonly _gradients;
    private readonly _clips;
    private _rootOpen;
    private _path;
    /** Whether the current subpath has a current point (a lone `arc` needs to know). */
    private _hasPoint;
    private _dash;
    /** Where along the dash list a stroke starts, as a canvas reads it. */
    lineDashOffset: number;
    private _ids;
    private readonly _strict;
    private readonly _background;
    private _fontCache;
    constructor(width: number, height: number, options?: SvgContextOptions);
    /** This context typed as the canvas one the paint code is written against. */
    asCanvasContext(): CanvasRenderingContext2D;
    save(): void;
    restore(): void;
    /**
     * Open an explicit `<g>` with the given attributes, optionally translated
     * and clipped to a rectangle in the group's own coordinates. The chart wraps
     * each pane in one so the export carries the same per-pane offset and clip
     * the DOM stacking gives the canvases. Balanced by `popGroup`.
     */
    pushGroup(attrs: Record<string, string | number>, options?: {
        translate?: {
            x: number;
            y: number;
        };
        clip?: {
            x: number;
            y: number;
            width: number;
            height: number;
        };
    }): void;
    popGroup(): void;
    translate(x: number, y: number): void;
    scale(x: number, y: number): void;
    rotate(angle: number): void;
    setLineDash(segments: number[]): void;
    getLineDash(): number[];
    beginPath(): void;
    closePath(): void;
    moveTo(x: number, y: number): void;
    lineTo(x: number, y: number): void;
    quadraticCurveTo(cx: number, cy: number, x: number, y: number): void;
    bezierCurveTo(c1x: number, c1y: number, c2x: number, c2y: number, x: number, y: number): void;
    rect(x: number, y: number, w: number, h: number): void;
    /**
     * The radii argument takes the forms the paint code uses: one number, or an
     * array of up to four (top-left, top-right, bottom-right, bottom-left, the
     * CSS order). Radii are scaled down together when they would overlap, as the
     * canvas does, so a pill shorter than its radius stays a pill.
     */
    roundRect(x: number, y: number, w: number, h: number, radii?: number | number[]): void;
    arc(x: number, y: number, r: number, a0: number, a1: number, ccw?: boolean): void;
    ellipse(x: number, y: number, rx: number, ry: number, rotation: number, a0: number, a1: number, ccw?: boolean): void;
    fill(rule?: CanvasFillRule | Path2D, _rule?: CanvasFillRule): void;
    stroke(path?: Path2D): void;
    clip(rule?: CanvasFillRule | Path2D): void;
    fillRect(x: number, y: number, w: number, h: number): void;
    strokeRect(x: number, y: number, w: number, h: number): void;
    clearRect(x: number, y: number, w: number, h: number): void;
    fillText(text: string, x: number, y: number): void;
    measureText(text: string): TextMetrics;
    createLinearGradient(x0: number, y0: number, x1: number, y1: number): SvgLinearGradient;
    /**
     * An `<img>` becomes an `<image>` of its `src`; a canvas becomes one of its
     * data URL. The nine-argument crop is a nested `<svg>` whose viewBox is the
     * source rectangle. Anything else (an `ImageBitmap`, a video frame) has no
     * URL to point at and is skipped.
     */
    drawImage(source: CanvasImageSource, ...args: number[]): void;
    setTransform(): void;
    resetTransform(): void;
    getTransform(): undefined;
    createRadialGradient(): undefined;
    createConicGradient(): undefined;
    createPattern(): null;
    getImageData(): undefined;
    putImageData(): void;
    createImageData(): undefined;
    strokeText(): void;
    drawFocusIfNeeded(): void;
    isPointInPath(): boolean;
    isPointInStroke(): boolean;
    /**
     * The document. Groups still open (a paint that ended inside a `save`) are
     * closed here without being popped, so the string is the same however many
     * times it is read.
     */
    toString(): string;
    private _snapshot;
    private _apply;
    /** Close every frame above index `i` and frame `i` itself, oldest last. */
    private _closeTo;
    /** Open a `<g>` that the enclosing frame (or the root) will close. */
    private _openGroup;
    private _clipId;
    private _rect;
    /** The `fill`/`stroke` attribute value for a style, or null for nothing visible. */
    private _paint;
    private _strokeAttrs;
    private _opacity;
    private _font;
    private _imageHref;
    private _unsupported;
}

/**
 * Aligning a second instrument onto the primary series' bars.
 *
 * The x-axis is a gapless logical index over the times the shared DataLayer
 * holds (ARCHITECTURE.md §4.1, §5.3), so two instruments do not share a bar
 * index and cannot be laid side by side by position. They are matched by
 * timestamp, and the two directions of mismatch get opposite answers:
 *
 * - **A comparison bar with no primary bar is dropped.** The DataLayer merges
 *   *every* series' times into one index space, so a time only the comparison
 *   has would mint a new logical index: a column the primary instrument has no
 *   candle for, inserted mid-chart, shifting every bar after it. That happens
 *   for real (a different exchange's holiday calendar, a 24/7 instrument next
 *   to an NSE one, a feed that emits a stray print), and warping the primary's
 *   own axis to accommodate a comparison is never the right trade. The print is
 *   counted in `dropped` so a host can say so rather than silently losing it.
 *
 * - **A primary bar with no comparison bar becomes whitespace**, which is a NaN
 *   bar the line renderer breaks across, so the comparison shows a *gap*. The
 *   alternative, carrying the last known value forward, draws a flat segment
 *   through a session the instrument never traded and, worse, in percentage
 *   mode it anchors the move on the far side of the gap to a print that does
 *   not exist. Omitting the bar entirely is worse still: the renderer would
 *   join the two sides with one straight line across the holiday.
 *
 * Matching is on the exact timestamp. Bar-open times are bucketed by the candle
 * builder (§10.2) and stored as UTC seconds (§4.0), so two instruments on the
 * same interval agree to the second; anything that does not agree is a
 * different interval, which no tolerance window could rescue.
 */

/** What one alignment pass did, for a host that wants to report coverage. */
interface ComparisonAlignment {
    /** Items handed to the comparison series: one per primary bar. */
    bars: number;
    /** Primary bars the comparison also traded (a value is drawn). */
    matched: number;
    /** Primary bars with no comparison print (drawn as a gap). */
    gaps: number;
    /** Comparison prints discarded for having no primary bar at that time. */
    dropped: number;
}
/**
 * Project `comparison` onto `primary`'s bars: one item per primary bar, in the
 * primary's order, so the result occupies exactly the logical indices the chart
 * already has and adds none of its own.
 *
 * Neither input is mutated and neither has to be sorted: matching goes through
 * a time map, which also collapses a repeated timestamp to its last item, the
 * same rule the DataLayer applies when it merges (`sortedUniqueByTime`).
 */
declare function alignToPrimary(primary: readonly Bar[], comparison: readonly SeriesDataItem[]): {
    items: SeriesDataItem[];
    alignment: ComparisonAlignment;
};

/**
 * Multi-symbol comparison: put a second instrument on the primary one's pane
 * and read them together (NIFTY against BANKNIFTY, a stock against its index).
 *
 * Headless, in the spirit of `DrawingController` (src/draw/controller.ts) and
 * `ReplayController`: it owns the series, the alignment and the scales, and
 * ships no DOM, so the host draws its own symbol chips and legend rows from
 * `list()`.
 *
 * Three decisions carry the design:
 *
 * 1. **Each comparison owns a scale.** A free legacy overlay or left scale
 *    preserves the first source's placement; additional sources use named
 *    hidden scales so their different price units cannot affect one another.
 *
 * 2. **Comparability comes from the scale, not from the data.** The bars handed
 *    over are stored as the instrument's own prices, so the legend, the
 *    crosshair and any live update still speak in real prices. What makes the
 *    lines readable together is the pane mode: `percentage` and
 *    `indexed-to-100` give every scale its own baseline (its first visible
 *    close, or an explicit common timestamp), and `_mirror` gives it the same
 *    band of percent the primary's axis is showing. Without that mirror each
 *    scale would autoscale to its own data and a 1% mover would look exactly
 *    like a 10% mover, both filling the pane.
 *
 * 3. **Alignment is by timestamp** and lives in `./align`, which documents what
 *    happens in each direction of mismatch.
 *
 * Each comparison owns a scale and baseline. Further instruments use keyed
 * hidden scales so their absolute prices cannot change another source's units.
 */

/**
 * How the pane quotes prices while a comparison is on it. The two rebasing
 * modes are the reason the lines are comparable at all; `none` leaves the
 * pane's own mode alone, for a host that wants the raw overlay.
 */
type ComparisonMode = 'percentage' | 'indexed-to-100' | 'none';
/** Independent first-visible closes, or the first visible timestamp shared by all visible sources. */
type ComparisonBaseline = 'first-visible' | 'common';
interface ComparisonOptions {
    /** Instrument label, e.g. 'BANKNIFTY'. Carried on the handle for the host's UI. */
    symbol: string;
    /** The instrument's own bars. Aligned to the primary series, see `./align`. */
    bars: readonly SeriesDataItem[];
    /** Line colour shorthand; `style.color` wins if both are given. */
    color?: string;
    /** Style overrides merged onto the chart type's defaults. */
    style?: SeriesStyle;
    /** Renderer for the comparison. Default 'line'. */
    type?: SeriesType;
    /** Pane to draw on. Default: the price pane, wherever it sits (`Chart.primaryPaneIndex`). */
    paneIndex?: number;
}
interface ComparisonControllerOptions {
    /** Pane mode applied while any comparison is on it. Default 'percentage'. */
    mode?: ComparisonMode;
    /** Baseline policy. Default 'first-visible' preserves existing integrations. */
    baseline?: ComparisonBaseline;
}
/** What `addComparison` hands back: one instrument on the chart. */
interface ComparisonHandle {
    readonly symbol: string;
    /**
     * The series this comparison draws through, for style patches and markers.
     * Data set on it directly skips alignment (use `setBars`), and removing it
     * directly leaves the pane rebased with nothing on it (use `remove`).
     */
    readonly series: SeriesApi;
    /** The slot of the pane it draws on, read live: it follows that pane through a move. */
    readonly paneIndex: number;
    /** The hidden scale it maps to. Never the pane's own price axis. */
    priceScale(): PriceScale;
    /** How the last alignment against the primary's bars went. */
    alignment(): ComparisonAlignment;
    /** Eligible aligned bar in its own price units; null for gaps, suppressed baselines or forming replay candles. */
    barAt(time: number): Readonly<Bar> | null;
    /** Replace the instrument's bars (a longer history, a refreshed fetch). */
    setBars(bars: readonly SeriesDataItem[]): void;
    /** Take this instrument off the chart. Safe to call twice. */
    remove(): void;
    /** Every comparison on this chart, in the order they were added. */
    list(): readonly ComparisonHandle[];
}
/**
 * The slice of a pane the controller reads. Declared structurally, like
 * `ReplayViewport`, so nothing here depends on `Pane` beyond the two members
 * that decide where a comparison can go.
 */
interface ComparisonPane {
    readonly priceScale: PriceScale;
    series(): readonly {
        readonly scaleId: string;
        readonly style?: SeriesStyle;
    }[];
}
/**
 * The slice of the chart this controller needs. `Chart` satisfies it; declaring
 * it structurally keeps the controller testable against a stub.
 */
interface ComparisonChartHost {
    addSeries(type: SeriesType, options: AddSeriesOptions): SeriesApi;
    panes(): readonly ComparisonPane[];
    addPrimitive(primitive: IPrimitive, paneIndex?: number): void;
    removePrimitive(primitive: IPrimitive): void;
    primarySeries(): SeriesApi | null;
    /** Avoids allocating a primary history copy when the host provides it. */
    primaryBars?(): readonly Bar[];
    getVisibleLogicalRange?(): {
        from: number;
        to: number;
    };
    on?(event: string, callback: (payload: unknown) => void): () => void;
    /** The shared logical axis locates visible primary bars even when another series adds times. */
    readonly dataLayer: {
        readonly length: number;
        indexToTime?(index: number): number | undefined;
    };
    /** Slot of the price pane, the default target. Absent means slot 0. */
    primaryPaneIndex?(): number;
}
declare class ComparisonController {
    private readonly _chart;
    private _mode;
    private _baseline;
    private readonly _panes;
    /** Insertion-ordered, and the handle is the key so `remove` is a lookup. */
    private readonly _items;
    /** Axis length plus primary identity and boundaries avoid scanning history each frame. */
    private _alignedAt;
    /** Guards `realign` against re-entry through its own `setData` repaint. */
    private _realigning;
    private _nextScale;
    private _syncing;
    private _alignedPrimary;
    private _primaryLength;
    private _primaryFirst;
    private _primaryLast;
    private readonly _off;
    private _destroyed;
    constructor(chart: ComparisonChartHost, options?: ComparisonControllerOptions);
    /** Put an instrument on the chart alongside the primary series. */
    add(options: ComparisonOptions): ComparisonHandle;
    /** Take one instrument off. Returns false if it was already gone. */
    remove(handle: ComparisonHandle): boolean;
    /** Every comparison on the chart, in the order they were added. */
    list(): readonly ComparisonHandle[];
    /** Take them all off, putting every pane back the way it was found. */
    clear(): void;
    /**
     * Change the mode the panes are held in while comparisons are on them.
     * Panes that already have one switch immediately.
     */
    setMode(mode: ComparisonMode): void;
    get mode(): ComparisonMode;
    /** Select a shared timestamp without changing the instruments' stored price units. */
    setBaseline(baseline: ComparisonBaseline): void;
    get baseline(): ComparisonBaseline;
    /**
     * Shared baseline timestamp, or null for no overlap, no visible sources, or
     * independent baselines. Asks about the price pane unless a slot is named.
     */
    baselineTime(paneIndex?: number): number | null;
    /**
     * Re-project onto the primary's current bars. Chart handles data replacement
     * and replay automatically. A structural host without data events or primary
     * history identity can call this after replacing its timestamps.
     */
    realign(): void;
    /**
     * Bring the overlay scales in line with the primary axis, re-aligning first
     * if the shared time axis has moved under us (a live bar, history paged in,
     * a replay step). Runs once per base paint through the per-pane hook.
     *
     * Re-aligning here writes series data mid-frame, which is safe because the
     * hook is a `bottom` primitive: it runs after the pane has autoscaled and
     * before any series is drawn, so the frame that notices the change is also
     * the frame that draws it. The repaint that write asks for lands on the next
     * frame, or re-enters this one on a host that runs frames synchronously,
     * which is what `realign`'s guard is for.
     */
    sync(): void;
    /** Remove every comparison and forget the chart. */
    destroy(): void;
    private _dispose;
    private _needsAlignment;
    private _rememberAlignment;
    /**
     * Preserve the first comparison's legacy placement when the scale is free.
     * Additional comparisons need their own baselines, and an occupied volume
     * or left scale must keep its source and units.
     */
    private _scaleIdFor;
    private _openPane;
    private _closePane;
    private _applyMode;
    private _restoreScale;
    private _restoreMode;
    /**
     * Give the overlay the same band of percent the primary axis is showing, so
     * equal moves land on equal pixels and the divergence between two
     * instruments is the thing you see.
     *
     * Each scale has its own price baseline, and rebasing maps price to the
     * ratio `price / baseline`. So the
     * two scales agree exactly when their ranges hold the same ratios, which is
     * one multiplication: the primary's range times `baseline_overlay /
     * baseline_primary`. It holds for `indexed-to-100` and `percentage` alike,
     * since they are one ladder a hundred points apart.
     *
     * A null baseline means this scale is not rebasing (linear, logarithmic, or
     * a pane with nothing visible on it yet). Then there is no shared ladder to
     * join and the overlay keeps its own autoscale, which is what mode 'none'
     * asks for: two instruments each filling the pane, comparable in shape only.
     *
     * The overlay stays under autoscale throughout, even though the range it
     * measures is overwritten here every frame. That measurement is one pass over
     * bars the renderer is about to walk anyway, and paying for it buys the
     * failure mode we want: a chart that stops calling `sync` falls back to an
     * independently scaled overlay instead of freezing on a stale range.
     */
    private _mirror;
    private _primaryBars;
    private _record;
    private _visible;
    private _visiblePrimary;
    private _commonAnchor;
    private _setSuppressed;
    private _write;
    private _align;
    /**
     * Ask for a full repaint after a mode change. The pane hands a rebasing scale
     * its baseline during the autoscale pass, so a `Light` repaint (all a
     * primitive's `requestUpdate` raises) would paint the new mode before it has
     * anything to quote against. `applyOptions` is the series handle's own route
     * to a Full invalidation, and an empty patch changes no style. Several of
     * them coalesce into one frame, so asking every comparison costs nothing.
     */
    private _repaint;
}
/**
 * The controller for a chart, created on first use. Use it to change the mode
 * for the whole chart, to list what is on it, or to clear it.
 *
 * A chart has one, so a later call returns the same controller, with the `mode`
 * and `baseline` it passes applied through `setMode` and `setBaseline`. An
 * attach helper merges a repeated request this way; a controller that owns
 * state a second owner would fight over (alerts, a replay group) refuses one.
 */
declare function comparisonController(chart: ComparisonChartHost, options?: ComparisonControllerOptions): ComparisonController;
/**
 * Put an instrument on the chart next to the primary one:
 *
 * ```ts
 * const bn = addComparison(chart, { symbol: 'BANKNIFTY', bars, color: '#f0b90b' });
 * ```
 *
 * The pane switches to percentage while any comparison is on it and goes back
 * to the mode it had when the last one leaves.
 */
declare function addComparison(chart: ComparisonChartHost, options: ComparisonOptions): ComparisonHandle;

/**
 * The public option and column types of `exportChartDataCsv`, published from
 * chart-data-export.ts. A module of their own so the export's parts (snapshot,
 * alignment, formatting) can name them without importing the module that
 * imports them.
 */

/** Inclusive UTC-second bounds over installed primary rows. Omitted bounds are unbounded. */
interface ChartDataCsvRange {
    readonly from?: number;
    readonly to?: number;
}
/** Canonical identities stay available even when a callback changes visible headers. */
type ChartDataColumn = Readonly<{
    key: string;
    title: string;
} & ({
    kind: 'time' | 'timeLabel' | 'logicalIndex' | 'timeOrigin';
} | {
    kind: 'primary';
    field: 'open' | 'high' | 'low' | 'close' | 'volume' | 'oi';
} | {
    kind: 'indicator';
    instanceId: string;
    plotKey: string;
    field?: 'open' | 'high' | 'low' | 'close';
} | {
    kind: 'comparison';
    comparisonIndex: number;
    symbol: string;
    field: 'close';
})>;
/** Detached installed axis. Replay never exposes its unrevealed source history here. */
interface ChartDataProjectionContext {
    readonly axisTimes: readonly number[];
    readonly replay?: Readonly<{
        time: number;
        forming: boolean;
        asOf?: number;
    }>;
}
interface ChartDataCsvFormatters {
    /** Adds a time_label column without replacing raw UTC seconds. */
    time?: (utcSeconds: number) => string;
    /** Called only for finite primary, study and comparison values. */
    value?: (value: number, column: ChartDataColumn) => string;
    header?: (column: ChartDataColumn) => string;
}
/** Options for a numeric snapshot of the chart's currently loaded data. */
interface ChartDataCsvOptions {
    /** All studies by default, false for none, or instance IDs in the requested column order. Hidden studies remain eligible. */
    indicators?: boolean | readonly string[];
    /** Filter rows after full installed-history calculation, without changing study warmup. */
    range?: ChartDataCsvRange;
    /** Override the chart's registered comparisons, for example with an explicitly managed controller's list. */
    comparisons?: readonly Pick<ComparisonHandle, 'symbol' | 'barAt'>[];
    /** Source rows by default. Display uses effective study offsets on the shared axis. */
    alignment?: 'source' | 'display';
    /** Resolves positions outside the captured axis. Null explicitly leaves time unknown. */
    projectTime?: (logicalIndex: number, context: Readonly<ChartDataProjectionContext>) => number | null;
    formatters?: ChartDataCsvFormatters;
}

/**
 * CSV with UTC seconds, unrounded OHLC/volume/OI, study values and aligned comparison closes.
 * Reads only installed primary rows, so replay exposes only its revealed prefix.
 * Missing or nonfinite values are blank. Source alignment precedes visual offsets;
 * display alignment shifts selected plots, expanding candle plots into OHLC fields.
 * Primary and comparison cells remain on installed primary positions in both modes.
 * Formatter text is escaped and protected against spreadsheet formula execution.
 * Does not fetch data, include trading state or initiate a browser download.
 */
declare function exportChartDataCsv(chart: Chart, options?: ChartDataCsvOptions): string;

type ChartObjectKind = 'source' | 'indicator' | 'drawing' | 'profile' | 'group';
/**
 * Where a row paints on its pane, back to front: `below` the series (a drawing
 * sent behind them), in the `series` band (the price source, each study, and a
 * drawing placed directly above one of them), or `above` everything the series
 * band holds (drawings in front, the default).
 */
type ChartObjectBand = 'below' | 'series' | 'above';
interface ChartObjectCapabilities {
    readonly select: boolean;
    readonly visibility: boolean;
    readonly lock: boolean;
    readonly remove: boolean;
    readonly settings: boolean;
    readonly focus: boolean;
    readonly reorder?: boolean;
    readonly move?: boolean;
    /** Takes part in its pane's stack and can be moved in it with `place`. */
    readonly place?: boolean;
}
/** One immutable row in a chart's object inventory. */
interface ChartObjectSnapshot {
    readonly id: string;
    readonly sourceId: string;
    readonly kind: ChartObjectKind;
    readonly name: string;
    readonly paneIndex: number;
    readonly visible: boolean;
    readonly locked?: boolean;
    readonly selected: boolean;
    readonly groupId?: string;
    readonly dataStatus?: Readonly<IndicatorDataStatus>;
    readonly capabilities: ChartObjectCapabilities;
    /** The band it paints in, for a row that is part of its pane's stack (see `stack`). */
    readonly band?: ChartObjectBand;
    /** For a drawing in the series band: the id of the source or study row it paints directly above. */
    readonly stackAbove?: string;
    /**
     * True for a drawing whose interval range leaves out the chart's interval,
     * and for a group all of whose drawings it leaves out: kept and listed so it
     * can be found, but not on the chart. `visible` stays the user's switch.
     */
    readonly hiddenOnInterval?: boolean;
}
/** State supplied by a host for an explicitly managed profile or source. */
interface ChartObjectDefinition {
    kind: ChartObjectKind;
    name: string;
    /** Slot of the pane it belongs to; omitted means the price pane, wherever it sits. */
    paneIndex?: number;
    visible?: boolean;
    locked?: boolean;
    selected?: boolean;
    groupId?: string | undefined;
    dataStatus?: Readonly<IndicatorDataStatus> | undefined;
    /** Kept but not on the chart at its current interval; see `ChartObjectSnapshot.hiddenOnInterval`. */
    hiddenOnInterval?: boolean;
}
/** Actions are synchronous and offered only when their callback exists. */
interface ChartObjectProvider {
    id: string;
    get(): ChartObjectDefinition | null;
    subscribe?(listener: () => void): () => void;
    select?(): void;
    setVisible?(visible: boolean): void;
    setLocked?(locked: boolean): void;
    remove?(): void;
    openSettings?(): void;
    focus?(): void;
    reorder?(direction: -1 | 1): boolean;
    move?(paneIndex: number): boolean;
}
/** Structural contract keeps the base tier independent of drawing code. */
interface ChartObjectDrawing {
    id: string;
    tool: string;
    paneIndex: number;
    points: readonly {
        time: number;
        price: number;
    }[];
    visible?: boolean;
    locked?: boolean;
    zIndex?: number;
    /** The source or study it paints directly above, when it is placed in the series band. */
    stackAbove?: string;
    /**
     * The drawing tier's policy flags the inventory honours: `listed` false
     * leaves the drawing out, `selectable` false withholds select, and
     * `editable` false withholds hide, lock and remove.
     */
    policy?: {
        readonly selectable?: boolean;
        readonly editable?: boolean;
        readonly listed?: boolean;
    };
}
interface ChartObjectDrawingGroup {
    id: string;
    name: string;
    members: readonly string[];
}
interface ChartObjectDrawingSource {
    drawings(): readonly ChartObjectDrawing[];
    get(id: string): ChartObjectDrawing | undefined;
    selection(): readonly string[];
    select(id: string | readonly string[] | null, additive?: boolean): void;
    update(id: string, patch: {
        visible?: boolean;
        locked?: boolean;
    }): void;
    remove(id: string): boolean;
    /** Delete several as one undo step; a group row holding an unlisted drawing needs it to offer remove. */
    removeMany?(ids: readonly string[]): void;
    reorder?(id: string, direction: -1 | 1): boolean;
    /**
     * Move a drawing directly above or below another drawing or a series-band
     * entry of its pane, as one undo step. Without it drawings cannot be placed.
     */
    placeInStack?(id: string, target: {
        drawing: string;
    } | {
        entry: string;
    }, where: 'above' | 'below'): boolean;
    groups?(): readonly ChartObjectDrawingGroup[];
    createGroup?(name: string, ids: readonly string[]): ChartObjectDrawingGroup | null;
    renameGroup?(id: string, name: string): boolean;
    removeGroup?(id: string, removeDrawings?: boolean): boolean;
    updateMany?(patches: ReadonlyArray<{
        id: string;
        patch: {
            visible?: boolean;
            locked?: boolean;
        };
    }>): void;
    /**
     * The ids of the drawings the chart's interval leaves off the chart. Each
     * one's row is marked `hiddenOnInterval` and offers no focus, which would
     * bring into view a place where nothing is drawn. Asked once per refresh,
     * as a list, so a large inventory costs one pass rather than a lookup per
     * row. Without it every drawing is shown.
     */
    hiddenOnInterval?(): readonly string[];
}
interface ChartObjectsOptions {
    drawings?: ChartObjectDrawingSource;
    /** Opens the host's existing editor for a built-in object. */
    onSettings?(object: ChartObjectSnapshot): void;
    /**
     * A drawing row's name from its tool id, such as the draw registry's
     * display name. This tier cannot read that registry, so without it the
     * row is named from the id ("Trend line"). Since 2.5.10.
     */
    drawingName?(tool: string): string;
}
/** Shared object operations for widgets and custom terminals. No DOM is created. */
declare class ChartObjects {
    private readonly _chart;
    private readonly _options;
    private readonly _off;
    private readonly _providers;
    private readonly _listeners;
    private _entries;
    /** Each drawing row's place among the drawings of its slot: its z-index, then its list position. */
    private _order;
    /**
     * The draw order the rows were last published with. A restack changes no
     * row, only the order `stack` reads, and a panel listing that order still
     * has to hear of it.
     */
    private _stackKey;
    private _publishedStackKey;
    private _rows;
    private _selected;
    private _destroyed;
    private _refreshing;
    private _pending;
    constructor(chart: Chart, options?: ChartObjectsOptions);
    /** Current immutable inventory, also refreshed for hosts with unsignalled provider state. */
    list(): readonly ChartObjectSnapshot[];
    get(id: string): ChartObjectSnapshot | undefined;
    /** Delivers current state immediately, then only inventory changes. */
    subscribe(listener: (objects: readonly ChartObjectSnapshot[]) => void): () => void;
    /** Provider IDs receive a custom: prefix and cannot replace built-in objects. */
    register(provider: ChartObjectProvider): () => void;
    /** Re-read explicitly registered host state without installing a polling timer. */
    refresh(): void;
    select(id: string | null, additive?: boolean): boolean;
    /** Pane targets include one new pane after the current stack. */
    paneCount(): number;
    /** The price pane's slot among the targets, so a list can name it wherever it sits. */
    primaryPaneIndex(): number;
    canReorder(id: string, direction: -1 | 1): boolean;
    reorder(id: string, direction: -1 | 1): boolean;
    move(id: string, paneIndex: number): boolean;
    /**
     * A pane's stack in paint order, back to front: drawings behind the series,
     * then each series-band entry (the price source, each study) followed by
     * the drawings placed directly above it, then the drawings in front. Group
     * rows, profiles and other rows outside the stack are not in it.
     */
    stack(paneIndex: number): readonly ChartObjectSnapshot[];
    /** Whether `place` would move `id` and paint the result. */
    canPlace(id: string, targetId: string, where: 'above' | 'below'): boolean;
    /**
     * Move a row directly above or below another row of its pane's stack. A
     * drawing goes anywhere in its pane: next to a drawing it joins that
     * drawing's slot, above an entry it is placed on that entry, below one it
     * goes on top of the slot under it. A source or study moves between whole
     * slots, taking the drawings placed on it along, since nothing can paint
     * between an entry and a drawing placed on it: such a move, one out of the
     * series band, and one across panes are refused, not approximated.
     */
    place(id: string, targetId: string, where: 'above' | 'below'): boolean;
    private _stackOf;
    /** The move `place` would make, or null for one it refuses or that changes nothing. */
    private _plan;
    canGroup(): boolean;
    /** Accept inventory ids so callers never need to strip provider prefixes. */
    createGroup(name: string, ids: readonly string[]): string | null;
    renameGroup(id: string, name: string): boolean;
    ungroup(id: string): boolean;
    setVisible(id: string, on: boolean): boolean;
    setLocked(id: string, on: boolean): boolean;
    remove(id: string): boolean;
    openSettings(id: string): boolean;
    focus(id: string): boolean;
    private _act;
    private _read;
    private _focusDrawing;
    /** Detach this observer and its providers, leaving chart objects in place. */
    destroy(): void;
}

/**
 * The color as rgba() with the given alpha (parse failure returns the input).
 * The canvas form, published as the package's `withAlpha`. The widget's
 * `withAlpha` is a different function on purpose: it writes token values.
 */
declare function withAlpha(color: string, alpha: number): string;

/**
 * Re-exported, not reimplemented: one import path covers an indicator's colour
 * work, while the colour parsing stays in `pill.ts`. A copy of it elsewhere in
 * the base costs bytes and drifts out of step with it.
 */

declare function verticalGradient(ctx: CanvasRenderingContext2D, heightPx: number, topColor: string, bottomColor: string): CanvasGradient;
/**
 * Blend `low` to `high` in sRGB by where `value` sits in [min, max], clamped
 * outside. Heatmap plots and per-bar colouring call this once per bar, so it
 * allocates only the result string: no closure, no cache, no lookup table.
 *
 * The clamp is two comparisons rather than Math.min/Math.max because both are
 * false against a not-available value, which lands it on `low` instead of
 * poisoning the output with NaN. Canvas ignores an unparseable fillStyle and
 * silently keeps the previous one, so a bad string would bleed a neighbour's
 * colour across the bar rather than fail loudly.
 */
declare function fromGradient(value: number, min: number, max: number, low: string, high: string): string;

/**
 * Eased wheel zoom (ARCHITECTURE.md §3.2, §7). A wheel tick used to land its
 * whole zoom step on one frame, which reads as the chart jumping rather than
 * moving. This spreads the same step over a short exponential approach, the
 * counterpart to `KineticAnimation` (input/kinetic.ts) for the other half of the viewport.
 *
 *   applied(t) = base + (target − base) · (1 − e^(−t/tau))
 *
 * The work is done in LOG space and the caller exponentiates the per-frame
 * delta, because zoom composes multiplicatively: two 1.1x steps are 1.21x, not
 * 1.2x. Easing the multiplier directly would make the first half of a glide
 * cover more zoom than the second, and a glide interrupted midway would land on
 * a different scale than one left alone.
 *
 * Closed form, so it is deterministic and unit-testable with no Date or rAF
 * inside: the caller samples {@link ZoomGlide.appliedAt} each frame and zooms by
 * the delta since its previous sample.
 */
interface ZoomGlideOptions {
    /**
     * Time constant in ms. The glide covers ~63% of the distance remaining after
     * each rebase in one tau, and is cut off at `stopFraction`.
     */
    tau: number;
    /** Stop once this much of the remaining distance has been applied. */
    stopFraction: number;
    /**
     * Skip the animation for a step smaller than this, in log units. Below it the
     * glide is a handful of sub-pixel frames nobody can see, and scheduling them
     * costs more than the step is worth.
     */
    minLogStep: number;
}
declare const DEFAULT_ZOOM_GLIDE_OPTIONS: ZoomGlideOptions;
declare class ZoomGlide {
    /** Log-space zoom already applied at the last rebase: where the curve starts. */
    private _base;
    /** Log-space zoom to end on, measured from the glide's origin. */
    private _target;
    /** Elapsed-ms reading of the last rebase, so the curve restarts from there. */
    private _epoch;
    private readonly _tau;
    private readonly _duration;
    /** @param totalLogFactor Signed log-space zoom to cover, `Math.log(factor)`. */
    constructor(totalLogFactor: number, options?: Partial<ZoomGlideOptions>);
    /**
     * The fraction of a step to apply on the input event itself, before any
     * frame runs.
     *
     * A glide that starts at zero adds a frame of input latency to every wheel
     * tick, and leaves `barSpacing` reading its old value to anything that looks
     * synchronously after the event. Both are worse than the jump the easing was
     * meant to fix. The lead is exactly what the first frame would have applied
     * anyway (one 60fps frame of the curve), so the chart responds instantly and
     * the remainder still eases.
     */
    static leadFraction(options?: Partial<ZoomGlideOptions>): number;
    /** Whether a step this large is worth animating rather than applying at once. */
    static shouldAnimate(totalLogFactor: number, options?: Partial<ZoomGlideOptions>): boolean;
    /**
     * Fold another wheel tick into a glide already running.
     *
     * The target ACCUMULATES, so spinning the wheel fast covers the sum of its
     * notches rather than only the last one, and scrolling back the other way
     * genuinely reverses (five notches out after one notch in ends four notches
     * out, which is what the user asked for).
     *
     * The curve is rebased rather than merely retargeted: it restarts from what
     * has already been applied, at the current time. Without that, changing the
     * target changes `appliedAt` for times ALREADY sampled, so the next frame
     * computes a delta against a curve the caller never rode and the chart
     * jumps, most visibly when a reversal flips the target's sign. Rebasing does
     * not make a stream of ticks crawl, because each tick also pushes the target
     * further away: it is progress toward a FIXED target that resetting would
     * stall.
     *
     * @param appliedSoFar What the caller has already zoomed by, in log units.
     * @param elapsedMs    The clock reading this rebase happens at.
     */
    add(extraLogFactor: number, appliedSoFar: number, elapsedMs: number): void;
    /** Signed log-space zoom applied from the glide's origin up to `elapsedMs`. */
    appliedAt(elapsedMs: number): number;
    /** Where the glide ends: the accumulated total, from the origin. */
    get totalLogFactor(): number;
    get durationMs(): number;
    finished(elapsedMs: number): boolean;
}

/** Clamp `value` into the inclusive range [min, max]. */
declare function clamp(value: number, min: number, max: number): number;
/** Linear interpolation between `a` and `b` by fraction `t` (0..1). */
declare function lerp(a: number, b: number, t: number): number;
/**
 * Round `value` to the nearest multiple of `step` (the instrument tick size).
 * Used for snapping dragged order/SL/TP prices to a valid tick. Returns
 * `value` unchanged when `step <= 0`.
 */
declare function roundToTick(value: number, step: number): number;
/** A canvas size: media px, or device px for a backing buffer. */
interface Size {
    width: number;
    height: number;
}
/**
 * Pure: compute the integer device-pixel backing-buffer size for a canvas.
 * Here rather than in core/canvas.ts so a renderer can take it by path without
 * reaching back into the core.
 */
declare function bitmapSize(mediaWidth: number, mediaHeight: number, dpr: number): Size;
/** Pure: snap a media-space coordinate to a crisp device-pixel edge. */
declare function snapToDevicePixel(mediaCoord: number, dpr: number): number;

/**
 * Generate up to ~`maxTicks` nicely-rounded tick values spanning [min, max].
 * Returns ascending values aligned to the chosen step (may sit slightly inside
 * the range). Returns a single midpoint when the range is degenerate.
 */
declare function niceTicks(min: number, max: number, maxTicks?: number): number[];
/** Decimal places implied by a price step / tick size (for label formatting). */
declare function precisionForStep(step: number): number;

/**
 * Candlestick renderer (ARCHITECTURE.md §6). Pure geometry helpers are split
 * out for unit testing; drawing happens in the bitmap (device-px) scope.
 */

interface CandleStyle {
    upColor: string;
    downColor: string;
    borderUpColor: string;
    borderDownColor: string;
    wickUpColor: string;
    wickDownColor: string;
    borderVisible: boolean;
    wickVisible: boolean;
    /**
     * Paint the body at all. Off leaves the candle as its outline and wick, which
     * is what a settings dialog's "Body" switch means.
     *
     * Distinct from `hollow`, which empties only the UP candles and is the
     * hollow-candle chart type. This empties both, and it is the one switch here
     * that can legitimately leave nothing drawn: with borders off as well, a
     * candle is reduced to its wick. That is two deliberate switches rather than
     * an accident, so it is honoured rather than second-guessed.
     */
    bodyVisible?: boolean;
    /** Draw up-candle bodies as outlines only (hollow candles). */
    hollow?: boolean;
    /**
     * "Color bars based on previous close": a bar is up when it closed above the
     * bar before it rather than above its own open. Body, border and wick all
     * follow the same verdict, so the candle never disagrees with itself.
     */
    colorByPreviousClose?: boolean;
    /** Per-bar body-width scale 0..1 (volume candles); 1 = full width. */
    widthScale?: (bar: Bar) => number;
}
declare const DEFAULT_CANDLE_STYLE: CandleStyle;
/**
 * Pure: optimal candle body width in device px for a given bar spacing. Leaves
 * a ~1px gap between candles, keeps a minimum of 1px, and matches odd/even
 * parity with the wick so the body stays symmetric about the (1px) wick.
 */
declare function optimalBarWidth(barSpacing: number, dpr: number): number;
/**
 * Level-of-detail tier for one candle.
 *
 * - `full`: the body (and any border) is distinguishable from the wick, so all
 *   of it is drawn.
 * - `wick`: the body would repaint exactly the pixels the wick already covered,
 *   so it is skipped. This is not an approximation: `optimalBarWidth` matches
 *   the body's odd/even parity to the wick's, which collapses the two to the
 *   same width and the same `x` once bars get tight, and the wick's high-low
 *   span always contains the body's open-close span. Same rect, same colour,
 *   twice the work.
 */
type CandleTier = 'full' | 'wick';
/**
 * Pure: whether a candle's body still shows once it is this narrow.
 *
 * Only returns `wick` when the body is provably invisible under the wick, so a
 * chart drawn at either tier is pixel-identical. Anything that could make the
 * body read differently - a hollow candle's outline, a border wide enough to
 * survive, a body colour that differs from the wick's, a hidden wick with
 * nothing to hide behind - stays `full`.
 *
 * `bodyW` is the per-bar width, so a volume candle whose `widthScale` has
 * narrowed it is judged on what it will actually draw.
 */
declare function candleTier(bodyW: number, wickW: number, style: CandleStyle): CandleTier;
/** Where one candle lands, in device px. Every field is an integer. */
interface CandleGeometry {
    /** Bar centre. */
    cx: number;
    /** Left edge of the body. */
    bodyX: number;
    /** Body width after any per-bar scale, never below 1. */
    bodyW: number;
    /** Left edge of the wick. */
    wickX: number;
    /** Wick width: 1 device px per whole unit of `dpr`. */
    wickW: number;
}
/**
 * Pure: the device-pixel rectangle geometry of one candle. This is the single
 * source of the snapping and parity rules, so a second backend rasterising
 * candles with its own primitives lands on the same pixels as `drawCandles`.
 *
 * `widthScale` is the per-bar body scale a volume candle applies (1 = the
 * full `optimalBarWidth`). It is clamped to 0.05..1: a zero-volume bar still
 * has to show a sliver of body, and nothing may grow past the gap.
 */
declare function candleGeometry(x: number, barSpacing: number, dpr: number, widthScale?: number): CandleGeometry;

/**
 * The shipped render backend: series painted by their registry renderer on the
 * pane's own 2D context. This is the path every chart took before the port
 * existed, so it is the reference the other backends are held to, pixel for
 * pixel (tests/e2e/render-parity.spec.ts).
 */

declare class Canvas2dBackend implements IRenderBackend {
    readonly kind = "canvas2d";
    private _canvas;
    private _ctx;
    mount(canvas: HTMLCanvasElement, ctx2d: CanvasRenderingContext2D | null): void;
    /**
     * Nothing to do: the pane's `CanvasLayer` owns the backing store and sized it
     * already. Re-sizing it here would clear a bitmap that was just painted.
     */
    resize(_widthPx: number, _heightPx: number, _dpr: number): void;
    /** The same clear `CanvasLayer.clearBitmap` does, so the op stream is unchanged. */
    beginFrame(clear: boolean): void;
    drawSeries(entry: RendererEntry, items: readonly DrawItem[], priceToY: (price: number) => number, barSpacing: number, dpr: number, style: SeriesStyle, rc: SeriesRenderContext): void;
    /** Every call above drew straight to the canvas; there is nothing to flush. */
    endFrame(): void;
    overlay2d(): CanvasRenderingContext2D | null;
    destroy(): void;
}

/**
 * Histogram renderer (ARCHITECTURE.md §6), used for volume and for a study's
 * histogram plots. Bars are drawn from a base value (0) up to each bar's
 * close. Columns are drawn by `drawColumns` in bars.ts.
 */

interface HistogramStyle {
    color: string;
    /** Value the bars grow from (default 0). */
    base: number;
}
declare const DEFAULT_HISTOGRAM_STYLE: HistogramStyle;

/** True while replay owns this chart, including when its playback clock is paused. */
declare function isReplaying(chart: object): boolean;

type ReplayScope = 'focused' | 'all';
/** Chart implements this; headless hosts may supply the lifecycle hooks too. */
interface ReplayGroupChartHost extends ReplayChartHost {
    readonly isDestroyed?: boolean;
    on?(event: 'destroy', callback: () => void): () => void;
}
interface ReplayGroupMember {
    id: string;
    chart: ReplayGroupChartHost;
    /** Transport and clock options belong to the group. */
    options: Pick<ReplayOptions, 'series' | 'bars' | 'subBars' | 'onFrame'> & {
        timing: ReplayTiming;
    };
}
interface ReplayGroupState {
    active: boolean;
    destroyed: boolean;
    scope: ReplayScope;
    focusedId: string;
    /** UTC availability time, or null when the active histories have no observations. */
    time: number | null;
    /** Index in the union of active observation times; -1 before the first. */
    index: number;
    total: number;
    playing: boolean;
    speed: number;
    members: readonly {
        id: string;
        active: boolean;
        state: ReplayState;
    }[];
}
interface ReplayGroupOptions {
    /** Default focused. Changing toolbar focus alone does not redirect replay. */
    scope?: ReplayScope;
    /** Default the first member. */
    focusedId?: string;
    startTime?: number;
    /** Wall-clock milliseconds per observation at speed 1. Default 1000. */
    barMs?: number;
    speed?: number;
    now?: () => number;
    scheduler?: ReplayScheduler;
    /**
     * Runs after all active charts reach a frame or transport transition. One that
     * throws ends replay, as any failure inside a transport call does: the group
     * restores its charts, destroys itself and rethrows.
     */
    onChange?: (state: ReplayGroupState) => void;
}
/** One opt-in clock driving validated ReplayControllers by availability time. */
declare class ReplayGroup {
    private _members;
    private _activeIds;
    private _times;
    private _scope;
    private _focusedId;
    private _time;
    private _startTime;
    private _active;
    private _destroyed;
    private _playing;
    private _busy;
    private _closing;
    private _revision;
    private _speed;
    private readonly _barMs;
    private readonly _now;
    private readonly _scheduler;
    private readonly _onChange?;
    private _cancel;
    private _clockRevision;
    private _lastAdvance;
    private _pending;
    constructor(members: readonly ReplayGroupMember[], options?: ReplayGroupOptions);
    state(): ReplayGroupState;
    seekTime(time: number): void;
    /** Seek an observation index, clamped to the active timeline. */
    seek(index: number): void;
    step(n?: number): void;
    stepBack(n?: number): void;
    private _move;
    setScope(scope: ReplayScope, focusedId?: string): void;
    play(options?: {
        speed?: number;
    }): void;
    pause(): void;
    /** Restore data/viewports, retaining the captured session for later re-entry. */
    stop(): void;
    /** Restore surviving charts and release snapshots, listeners and ownership. */
    destroy(): void;
    private _validateScope;
    private _prepare;
    private _prepareEntering;
    private _install;
    private _wanted;
    private _rebuildTimes;
    private _check;
    private _apply;
    private _atEnd;
    private _tick;
    private _clearClock;
    private _emit;
    private _notify;
    private _execute;
    private _chartGone;
    private _cleanup;
}

/**
 * Time alignment for chart linking: the one thing that makes a grid of charts
 * behave as a workspace instead of four charts that happen to move together.
 *
 * **The x-axis is a gapless logical index over each chart's own bars, so
 * logical index N is a different instant on every chart.** Two linked charts
 * normally hold different bar arrays: different symbols, different intervals,
 * different history depth, different holidays. Copying a logical index or a
 * logical range straight across looks perfect on two charts of the same symbol
 * and interval, and is wrong everywhere else, so nothing in this module ever
 * moves an index across a chart boundary. Every value is converted index ->
 * time on the sender and time -> index on the receiver, against that chart's
 * own data.
 *
 * The two conversions live here, apart from the group, because they are pure
 * and are the part worth reading twice.
 */

/**
 * The slice of `DataLayer` alignment reads. Declared structurally, in the
 * spirit of `ComparisonChartHost`, so the conversions can be tested against a
 * literal instead of a live chart.
 */
interface LinkDataLayer {
    /** Number of logical indices (distinct bar times). */
    readonly length: number;
    indexToTime(index: number): number | undefined;
    timeToIndex(time: number): number | undefined;
    indexToTimeFloat(index: number): number;
    timeToIndexFloat(time: number): number;
}
/**
 * What a follower does when the leader's instant falls between its bars.
 *
 * - `nearest` (default): show the crosshair on the last bar that had OPENED by
 *   the leader's instant. A hole inside the follower's history is a missing bar,
 *   not a missing instant, and snapping to a real bar reads as "this one lines
 *   up" where a line floating between two candles belongs to neither.
 *
 *   Deliberately not the closest bar by timestamp. A bar is stamped at its open
 *   and lasts until the next one opens, so past the halfway point of a daily bar
 *   the arithmetically nearest stamp is tomorrow's, and the follower would
 *   highlight a candle that did not exist yet at the leader's instant.
 * - `hide`: draw nothing unless the follower has a bar at exactly that time.
 *   For a workspace where a linked crosshair is a data claim.
 *
 * Both policies refuse to answer **outside** the follower's coverage: an
 * instant before its first bar or after its last one is not a gap in its data,
 * it is a period the chart does not cover at all, and pinning the crosshair to
 * the first or last bar there would assert an alignment that does not exist.
 */
type LinkMissingPolicy = 'nearest' | 'hide';
/**
 * Leader instant -> follower logical index, or null for "draw no crosshair".
 *
 * Returns an integer: a linked crosshair always sits on one of the follower's
 * own bars, never interpolated between them.
 */
declare function followerIndex(follower: LinkDataLayer, time: number, whenMissing?: LinkMissingPolicy): number | null;
/**
 * Leader visible range -> the follower range showing the same wall-clock window.
 *
 * Both ends go through the fractional conversions, so the window survives a
 * different bar interval (a daily follower under an hourly leader ends up with
 * a fraction of a bar visible, which is exactly right) and survives an end
 * landing in the empty margin to the right of the last bar, where `indexToTime`
 * has nothing to return.
 *
 * Returns null when the answer would be meaningless rather than guessing:
 * either chart empty, a single-bar follower (every time maps to index 0, so the
 * span collapses), or a non-finite endpoint.
 *
 * A follower whose history does not overlap the window at all is *not* refused.
 * `timeToIndexFloat` extrapolates past either edge (left of the first bar at
 * the first gap, right of the last on the follower's own future bar times), so
 * it scrolls into its own empty margin and shows nothing, which is the truth.
 * Clamping it back onto its last bars would show the user a different period
 * than the leader.
 */
declare function followerRange(leader: LinkDataLayer, follower: LinkDataLayer, range: LogicalRange): LogicalRange | null;

/**
 * The drawings channel of a link group, split across two tiers on purpose.
 *
 * The group, here in the base tier, decides **who** shares drawings with whom:
 * the members of one group, while its `drawings` switch is on. The draw tier's
 * `DrawingLinkGroup` decides **which** drawings cross and moves them. Keeping
 * the second half out of this file is what keeps drawing code out of the base
 * bundle, and keeping the first half here is what lets one group, and one
 * saved set of switches, carry every channel a grid links on.
 */
/**
 * One member's side of drawing sharing, supplied by the host. Usually two
 * lines over a draw-tier `DrawingLinkGroup` kept for the group:
 *
 * ```ts
 * group.add(chart, { drawings: {
 *   join: () => sharing.add(chart, controller),
 *   leave: () => sharing.remove(chart),
 * } });
 * ```
 *
 * The group calls `join` when the member is in it and the channel is on, and
 * `leave` when either stops being true (the switch goes off, the member is
 * removed, replaces its adapter or is destroyed, or the group is destroyed).
 * Calls alternate: never two joins without a leave between them.
 *
 * Which drawings cross, as the draw tier's `DrawingLinkGroup` decides it:
 *
 * - **Only between charts showing the same instrument**, symbol and exchange.
 *   A level drawn on one stock means nothing on another, so a chart on another
 *   instrument neither sends nor receives, whatever its group.
 * - **Only price-pane drawings in data space.** A drawing pinned to the
 *   viewport is a place on this chart's screen and stays on it.
 * - **Drawn while the channel is on, to the charts present.** A drawing
 *   crosses to the charts on its instrument when it is drawn, and its edits
 *   and its deletion follow it there. A drawing that has never been shared
 *   stays private to its chart until the host shares it
 *   (`DrawingLinkGroup.share`), so switching the channel on never sprays one
 *   chart's levels across a desk; a chart that arrives on the instrument later
 *   gets a drawing shared without it the same way, when the host shares it again.
 * - **An instrument change keeps drawings with their instrument.** Drawings are
 *   kept per instrument (`InstrumentDrawings`), so a chart leaving an instrument
 *   takes its copies of the shared drawings into that instrument's saved
 *   document and shows the incoming instrument's own. When it comes back they
 *   reconnect, and take the state the charts still on it hold now, a deletion
 *   included. With symbol linking on, the whole group moves together and keeps
 *   sharing on the new instrument.
 * - **Leaving keeps what arrived, and re-linking reconnects it.** Switching the
 *   channel off, or taking a chart out of the group, stops the traffic; every
 *   copy already made stays on the chart that holds it. A copy still carries its
 *   link, so when its chart shares again (the switch goes back on, or it
 *   rejoins) it reconnects and takes the state the charts already sharing hold,
 *   as on returning to an instrument: their edits, and a deletion made while
 *   they were sharing. An edit made to a copy while unlinked therefore gives way
 *   on re-linking. Members join in the order they were added, so when the switch
 *   goes on for the whole group the first of them that holds the drawing sets
 *   its state.
 */
interface LinkDrawingsAdapter {
    /** Start sharing this chart's drawings with the rest of the group. */
    join(): void;
    /** Stop sharing. Copies already made stay on every chart that holds one. */
    leave(): void;
}

/**
 * Chart linking groups: a grid of charts that behaves as one workspace.
 *
 * Headless, in the spirit of `ComparisonController` and `ReplayController`. It
 * ships no DOM, so the host draws its own link badge / colour chips and decides
 * which charts belong to which group.
 *
 * Each channel switches independently because a user routinely
 * wants one without the others (mirror the cursor across four timeframes but
 * keep each zoom; or slave every chart's symbol but let each keep its own
 * window):
 *
 * - **crosshair**: hovering one chart marks the same instant on the others.
 * - **viewport**: panning or zooming one moves the others to the same window.
 * - **symbol**: changing the instrument on one changes it on the others.
 * - **interval**: changing a timeframe asks each following host to adopt it.
 * - **chartType**: changing the chart style (candles, bars, a line) asks each
 *   following host to adopt it, the same way. Colours, scales and the status
 *   line are appearance, not chart type: a user linking candles to candles
 *   may well want each chart's own colours.
 * - **appearance**: visual settings pass through a structural host adapter.
 * - **drawings**: the group decides which charts share drawings, and the draw
 *   tier moves them, only between charts on the same instrument. See `./drawings`.
 *
 * Four decisions carry the design.
 *
 * 1. **Everything crosses a chart boundary as a time, never as a logical
 *    index.** See `./align`. This is the whole difficulty of the feature: the
 *    naive version works perfectly on two charts of the same symbol and
 *    interval, which is how it would ship broken.
 *
 * 2. **A follower's crosshair is a primitive, not the chart's own crosshair.**
 *    The engine's crosshair belongs to the pointer inside that chart; a linked
 *    one is a second, weaker mark that must not fight it. See `./crosshair`.
 *
 * 3. **One re-entrancy guard covers every channel.** A syncs B, B echoes
 *    back to A, and the pair either oscillates forever or blows the stack. Any
 *    member event that arrives *while the group is broadcasting* is an echo of
 *    that broadcast by definition (a human cannot pan two charts in one call
 *    stack), so it is dropped. The guard is deliberately group-wide rather than
 *    per channel: a symbol change that reloads data can move a viewport, and
 *    that second-order echo is the same bug wearing a different hat.
 *
 * 4. **The group never keeps a destroyed chart alive.** `Chart.destroy` sets
 *    `isDestroyed` and emits `'destroy'`, so a member is released the moment it
 *    dies rather than at the next channel event. A `LinkChart` that is not a
 *    `Chart` may report neither, so members are also probed by pane count
 *    before every use (the price pane can never be removed by any other route) and
 *    dropped on the spot, which matters because `addPrimitive` on a destroyed
 *    chart would resurrect a pane.
 *
 * The host owns instrument selection and loading. It tells the group a selection changed (by
 * emitting `'symbol'` on the chart's own event bus, or by calling `setSymbol`),
 * and it supplies the per-member `onSymbol` callback that actually loads the
 * new instrument's bars. A member without `onSymbol` broadcasts symbol changes
 * but never receives them, which is how a host pins one chart of a grid.
 */

/**
 * The slice of the chart a link group drives. `Chart` satisfies it; declaring
 * it structurally keeps the group testable against a stub, which is the only
 * practical way to prove the feedback guard (the engine's own programmatic
 * setters do not echo today, and the guard exists for the day they do and for
 * the host callbacks that echo right now).
 */
interface LinkChart {
    on(event: string, cb: (payload: unknown) => void): () => void;
    getVisibleLogicalRange(): LogicalRange;
    setVisibleLogicalRange(range: LogicalRange): void;
    readonly dataLayer: LinkDataLayer;
    /**
     * Set by `Chart`. Optional so the group stays testable against a stub and
     * usable by a host wrapping something that is not a `Chart`, but when it is
     * there it is the answer: an empty pane list is only ever an inference.
     */
    readonly isDestroyed?: boolean;
    /** Only the count is read: the fallback destruction probe. See `alive`. */
    panes(): readonly unknown[];
    addPrimitive(primitive: IPrimitive, paneIndex?: number): void;
    removePrimitive(primitive: IPrimitive): void;
    /** Optional readout support alongside the group's vertical marker. */
    setLinkedCrosshairIndex?(index: number | null): void;
}
interface LinkOptions {
    /** Mirror the hovered instant onto every other member. Default true. */
    crosshair?: boolean;
    /** Mirror pan and zoom as a wall-clock window. Default true. */
    viewport?: boolean;
    /** Mirror the instrument, via each member's `onSymbol`. Default false. */
    symbol?: boolean;
    /** Mirror the timeframe, via each member's `onInterval`. Default false. */
    interval?: boolean;
    /** Mirror the chart type (candles, bars, a line), via each member's `onChartType`. Default false. */
    chartType?: boolean;
    /** Copy visual chart settings through each member's adapter. Default false. */
    appearance?: boolean;
    /**
     * Share drawings through each member's `drawings` adapter. Default false.
     * Only charts on the same instrument share, and only what is drawn while it
     * is on: see `LinkDrawingsAdapter` for the whole rule.
     */
    drawings?: boolean;
    /** What a follower does with an instant it has no bar for. Default 'nearest'. */
    whenMissing?: LinkMissingPolicy;
}
interface LinkMemberOptions {
    /** Visual settings only; no instrument, timeframe, study or trading state is copied. */
    appearance?: LinkAppearanceAdapter;
    /** The instrument this chart is showing right now, if the host tracks one. */
    symbol?: string;
    /**
     * Load `symbol` into this chart. The group calls it; the host does the work
     * (fetch bars, `series.setData`). Omit to make the member symbol-read-only:
     * it still broadcasts its own changes, it just never follows anyone else's.
     */
    onSymbol?: (symbol: string, chart: LinkChart) => void;
    /** The interval this chart is showing, if the host tracks one. */
    interval?: string;
    /** Apply the interval synchronously; return false to refuse an unsupported token. */
    onInterval?: (interval: string, chart: LinkChart) => boolean | void;
    /** The chart type this chart is showing (a registered chart type id), if the host tracks one. */
    chartType?: string;
    /**
     * Show `chartType` on this chart. Return false to refuse one it cannot draw,
     * which leaves the member on its own type and asks again on the next change.
     * Omit to keep this chart's type out of the group's reach.
     */
    onChartType?: (chartType: string, chart: LinkChart) => boolean | void;
    /** This chart's side of drawing sharing. Without one it never shares drawings. */
    drawings?: LinkDrawingsAdapter;
}
/** Every option resolved, as `options()` reports them. */
type ResolvedLinkOptions = Required<LinkOptions>;
declare class LinkGroup {
    private readonly _members;
    private _options;
    /** True while the group is applying a change to followers. See decision 3. */
    private _broadcasting;
    private _symbol;
    /** The latest selection on each token channel, kept even while that channel is off. */
    private readonly _tokens;
    private _destroyed;
    constructor(options?: LinkOptions);
    options(): ResolvedLinkOptions;
    /**
     * Flip switches at runtime. Turning `crosshair` off clears the linked lines
     * immediately rather than leaving the last one frozen on every follower;
     * turning `symbol` on makes the group agree on the instrument it already
     * knows, because a switch that only takes effect on the *next* change would
     * leave a linked grid visibly unlinked.
     *
     * `interval` and `chartType` converge the same way, on the latest selection.
     * Turning `drawings` on joins every member that has an adapter, and turning
     * it off makes each leave.
     *
     * `viewport` has no equivalent convergence: nothing in the group says which
     * member's window the others should have adopted, so it takes effect on the
     * next pan or zoom.
     */
    setOptions(patch: LinkOptions): void;
    members(): readonly LinkChart[];
    has(chart: LinkChart): boolean;
    /** The instrument the group has agreed on, or null if nobody declared one. */
    symbol(): string | null;
    /** Latest interval selected by a member, even while interval linking is off. */
    interval(): string | null;
    /** Latest chart type selected by a member, even while chart type linking is off. */
    chartType(): string | null;
    /**
     * The member's own logical index its linked crosshair is marking, or null
     * when it is showing none (it is the chart being hovered, the leader's
     * instant falls outside its coverage, or crosshair sync is off).
     *
     * A host wanting the linked bar's OHLC reads it back with
     * `chart.dataLayer.indexToTime(...)`, the same way a native crosshair readout
     * does.
     */
    crosshairIndex(chart: LinkChart): number | null;
    /**
     * Put a chart in the group. Adding one twice updates its member options
     * instead of double-subscribing.
     *
     * A member joining a group that already has a symbol adopts it (when symbol
     * sync is on and it can follow), because joining a linked workspace is
     * exactly the moment a user expects the new chart to fall in line. The same
     * holds for the interval and the chart type, and a member joining while
     * drawings are shared joins the sharing.
     */
    add(chart: LinkChart, member?: LinkMemberOptions): void;
    /** Take a chart out of the group. Safe to call twice, and after `destroy`. */
    remove(chart: LinkChart): void;
    /**
     * The host reporting an instrument change on one member: the imperative twin
     * of emitting `'symbol'` on that chart's event bus. Records it and, when
     * symbol sync is on, loads it into every other member that can follow.
     */
    setSymbol(chart: LinkChart, symbol: string): void;
    /** Report a host-owned interval selection; followers opt in through `onInterval`. */
    setInterval(chart: LinkChart, interval: string): void;
    /**
     * Report a host-owned chart type selection: the imperative twin of emitting
     * `'chartType'` on that chart's event bus. Followers opt in through `onChartType`.
     */
    setChartType(chart: LinkChart, chartType: string): void;
    /** Report an appearance edit. `style:change` does this automatically for settings patches. */
    syncAppearance(chart: LinkChart): void;
    /** Unlink everything: no listeners, no linked crosshairs, no references. */
    destroy(): void;
    private _onCrosshair;
    private _onViewport;
    private _onSymbolEvent;
    private _applySymbol;
    /** Push the group's agreed symbol onto everyone who is not already on it. */
    private _convergeSymbol;
    private _report;
    private _applyToken;
    private _followToken;
    private _convergeToken;
    /** Join or leave drawing sharing, each at most once in a row. See `LinkDrawingsAdapter`. */
    private _share;
    /**
     * Run `apply` on every member except the originator, exactly once, with the
     * echo guard held. `onLeader` (optional) runs on the originator itself.
     *
     * Re-entrant calls return without doing anything, which is what breaks the
     * A -> B -> A loop: the inner call is always an echo of the outer one.
     */
    private _broadcast;
    /** Drop destroyed members, releasing the group's reference to them. */
    private _prune;
    private _find;
    private _release;
    /**
     * Point this member's linked crosshair at one of its own bars (or clear it),
     * keeping one primitive per pane so the line spans price, volume and
     * indicator panes the way the native global crosshair does.
     */
    private _setCrosshair;
    private _detachCrosshairs;
}
/** Create a link group. `group.add(chart)` puts a chart in it. */
declare function createLinkGroup(options?: LinkOptions): LinkGroup;

/**
 * The crosshair a linked chart shows for someone else's cursor.
 *
 * The engine's own crosshair is driven by the pointer that is inside that
 * chart, and there is exactly one of them per chart. A follower needs a second,
 * independent one, so it is a primitive rather than anything in the core: it
 * sits on the `top` z-order, which is the layer `Pane.paintTop` repaints for a
 * cursor move, so a linked crosshair costs the same overlay repaint the native
 * one does.
 *
 * Two deliberate differences from the native crosshair:
 *
 * - **Vertical line only.** The horizontal line marks a price, and the price
 *   under a cursor on another instrument is not a price on this one. On a grid
 *   of four symbols a mirrored price line would be a straight lie four times
 *   over. The vertical line marks an instant, and an instant is shared.
 * - **Drawn at reduced opacity**, in the follower's own crosshair colour, so it
 *   reads as a reflection of a cursor somewhere else rather than a second
 *   cursor in this chart.
 */

/** Opacity of a linked crosshair relative to the pane's own crosshair colour. */
declare const LINK_CROSSHAIR_ALPHA = 0.55;
declare class LinkCrosshair implements IPrimitive {
    private _index;
    private _host;
    attached(host: PrimitiveHost): void;
    detached(): void;
    zOrder(): ZOrder;
    /** The follower's own logical index to draw at; null draws nothing. */
    index(): number | null;
    /** Move (or clear) the line. Repaints only when something actually changed. */
    setIndex(index: number | null): void;
    draw(ctx: CanvasRenderingContext2D, rc: PrimitiveRenderContext): void;
}

/**
 * Live candle aggregation (ARCHITECTURE.md §10.2). The WS feed does not deliver
 * interval candles: LTP mode gives a tick price (+ last-traded-qty), Quote mode
 * gives a *cumulative day* volume. This builder buckets ticks into interval OHLC
 * with explicit volume, session-reset, and late-tick policies. Pure and
 * deterministic (no Date/rAF) so it is fully unit-testable.
 */

type VolumeMode = 'ltq-sum' | 'day-delta';
type LateTickPolicy = 'foldIntoBar' | 'dropOlderThanPrevBar';
interface CandleBuilderOptions {
    intervalSec: number;
    /** 'ltq-sum' accumulates last-traded-qty; 'day-delta' diffs cumulative day volume. */
    volumeMode: VolumeMode;
    lateTickPolicy: LateTickPolicy;
    /**
     * UTC-seconds of a known session open. Buckets align to it so e.g. 5-minute
     * bars start at 09:15, not at an arbitrary epoch floor. Defaults to 0 (epoch).
     */
    sessionAnchorSec: number;
}
declare const DEFAULT_CANDLE_BUILDER_OPTIONS: CandleBuilderOptions;
interface Tick {
    time: UTCSeconds;
    price: number;
    /** Last-traded quantity (LTP mode). */
    ltq?: number;
    /** Cumulative day volume (Quote mode). */
    cumDayVolume?: number;
    /**
     * Open interest as at this tick, where the feed carries it. Unlike the two
     * quantities above it is not accumulated into the bar: it replaces, because
     * it is a level and not a flow (see `Bar.oi`). A feed that does not report it
     * leaves the built bar without one, which is the honest result rather than a
     * zero that reads as "nobody is holding this".
     */
    oi?: number;
}
interface CandleUpdate {
    bar: Bar;
    /** True when this tick started a new interval bar (append vs mutate-in-place). */
    isNew: boolean;
    /**
     * True while the bar's open, high, low and volume cover only the ticks this
     * builder has seen. A bar is provisional when it was opened from a tick
     * without the builder having streamed the bar before it: a cold start, or a
     * seed from an older bucket, both of which mean the trades between the
     * bucket's true open and the first tick seen were missed. The close is still
     * the latest price. An authoritative bar for the same bucket, from history,
     * corrects the rest through `reconcile`.
     */
    provisional?: boolean;
}
declare class CandleBuilder {
    private readonly _opts;
    private _current;
    private _cumAtBarStart;
    private _lastCum;
    private _hasCum;
    /** The current bar has received at least one tick, so a rollover from it sees the true open. */
    private _streamed;
    private _provisional;
    constructor(options?: Partial<CandleBuilderOptions>);
    /** Seed with the last historical bar so the first live tick continues it. */
    seed(lastBar: Bar, cumDayVolumeSoFar?: number): void;
    current(): Bar | null;
    /** Whether the current bar is provisional; see `CandleUpdate.provisional`. */
    isProvisional(): boolean;
    /**
     * Adopt an authoritative bar for the current bucket, typically the broker's
     * history for the bar that is still forming.
     *
     * A provisional bar takes the authoritative open, because its own is the
     * first tick this builder happened to see. Every bar takes the union of the
     * extremes, since both sides observed real prices, and the larger volume,
     * since volume inside a bar only grows. The close stays with the ticks: a
     * history snapshot was taken before the request went out, and a price that
     * jumps backwards on every repair is the one thing a live chart must not do.
     *
     * Returns the reconciled bar, or null when the builder holds no bar for that
     * bucket, in which case the caller decides whether to `seed` instead.
     */
    reconcile(authoritative: Bar): Bar | null;
    /** Bucket-start (bar-open) time for a tick, aligned to the session anchor. */
    bucketStart(time: UTCSeconds): UTCSeconds;
    /**
     * Feed one tick. Returns the affected bar (mutated current or a fresh one),
     * or `null` if the tick was dropped by the late-tick policy.
     */
    onTick(tick: Tick): CandleUpdate | null;
    private _foldInto;
    private _volumeForNewBar;
    private _volumeForSameBar;
}

/**
 * Free-standing shapes an indicator paints in its pane (ARCHITECTURE.md §8):
 * lines, boxes, labels and polylines anchored to time and price.
 *
 * One primitive holds the whole list rather than one primitive per shape. A
 * descriptor rebuilds its shapes on every recompute, so per-shape primitives
 * would mean attaching and detaching a dozen objects on every live tick, and
 * the pane would re-sort its z-order for each of them.
 *
 * Anchors are **times**, never logical indices: paging history in at the left
 * edge shifts every index, and a trendline pinned to an index would slide off
 * its pivots the moment it did.
 */

declare class IndicatorDrawings implements IPrimitive {
    private _items;
    private _host;
    private _visible;
    private readonly _priceScale;
    /**
     * The labels and boxes that carry an id or a tooltip, where the last frame
     * put them. Only those are hit-testable: a bare trendline stays under the
     * pointer as pure ink, so hovering a busy study does not light up a cursor
     * on every ray it drew.
     */
    private _hits;
    /**
     * @param priceScale The scale prices are measured on, read on every frame
     * from that frame's context. Without it, or when it returns nothing, the
     * layer uses the scale its pane binds it to. Shapes quoted in the pane's own
     * price units, the way a study's price-pane shapes are, return
     * `rc.readoutPriceScale`: they then go wherever the candles' axis goes, and
     * the layer never holds an axis of its own or pins one in place.
     */
    constructor(priceScale?: (rc: PrimitiveRenderContext) => PriceScale | null | undefined);
    attached(host: PrimitiveHost): void;
    detached(): void;
    /** Over the series, under the crosshair: these are annotations on the data. */
    zOrder(): ZOrder;
    /**
     * A shape is drawn where the descriptor put it and never argues with the
     * scale. A projection reaching far above the data would otherwise squash the
     * study it annotates into a band a few pixels tall.
     */
    autoscaleInfo(): null;
    setItems(items: readonly IndicatorDrawing[]): void;
    setVisible(on: boolean): void;
    draw(ctx: CanvasRenderingContext2D, rc: PrimitiveRenderContext): void;
    /** Keep a label's or box's rect when it has something to say on hover or click. */
    private _recordHit;
    /**
     * The hovered shape's tooltip, as a plate clear of the shape: above it when
     * there is room, below it otherwise. Drawn last so it sits over everything
     * else in the layer, and only while the chart reports the pointer on it, so
     * it costs nothing on a frame with nothing hovered.
     */
    private _drawTooltip;
    hitTest(x: number, y: number): PrimitiveHit | null;
}

/**
 * Per-bar pane shading: one full-height column behind the data, in whatever
 * colour the descriptor gave that bar.
 *
 * A regime study answers "which state is the market in", and that is a property
 * of the whole bar rather than of a price. As a column it reads at a glance and
 * costs the price scale nothing, where a plot would need a value to sit at and
 * would drag the pane's autoscale around with it.
 *
 * What makes it affordable is the work skipped: everything outside the visible
 * range is dropped before anything is painted, and adjacent bars sharing a
 * colour become one rect instead of one rect each. A year of two-state shading
 * is a handful of fills, not one per bar per frame.
 */

declare class IndicatorBackground implements IPrimitive {
    private _colors;
    /** Time of `_colors[0]`'s bar. See `draw` for why this is a time, not an index. */
    private _anchor;
    private _host;
    private _visible;
    attached(host: PrimitiveHost): void;
    detached(): void;
    /** The same layer the bands use: behind the series, so the candles stay crisp. */
    zOrder(): ZOrder;
    /** Shading has no price of its own and must never widen the pane's range. */
    autoscaleInfo(): null;
    setColors(colors: readonly (string | null)[], bars: readonly Bar[]): void;
    setVisible(on: boolean): void;
    draw(ctx: CanvasRenderingContext2D, rc: PrimitiveRenderContext): void;
}

/**
 * Dims everything from a bar onward, and draws the line where the cut falls.
 *
 * Two jobs, one shape. Picking a replay start means answering "from here, what
 * happens next?", and the honest way to ask it is to hide the answer: the bars
 * to the right of the cursor are greyed while the user chooses, so a start bar
 * is picked on what was known at the time rather than on the shape of what
 * followed. Once replay is running the same veil marks the part of the session
 * that has not been reached yet.
 *
 * The shade is drawn over the series rather than under it, because covering the
 * future is the entire point: a translucent wash still shows the shape faintly,
 * which is what tells a user there is more session there to walk into.
 */

interface ReplayShadeOptions {
    /**
     * The last bar left uncovered. Everything after it is dimmed. Null draws
     * nothing at all, which is how a host keeps one instance around across
     * entering and leaving the mode.
     */
    index: number | null;
    /** Wash colour over the covered bars. Default a dark, low-alpha neutral. */
    color?: string;
    /** The rule at the cut. Default a muted blue, the colour of a chosen thing. */
    lineColor?: string;
    lineWidth?: number;
    /** Draw the divider. Default true. */
    lineVisible?: boolean;
    zOrder?: ZOrder;
    id?: string;
}
declare class ReplayShade implements IPrimitive {
    private _opts;
    private _host;
    constructor(opts: ReplayShadeOptions);
    attached(host: PrimitiveHost): void;
    detached(): void;
    zOrder(): ZOrder;
    /** Takes no part in autoscale: it covers bars, it is not one. */
    autoscaleInfo(): null;
    /** Never hit, so the click that picks a bar reaches the chart. */
    hitTest(): null;
    /** Moving or clearing the cut repaints at once: a host sets it on a click, not only on a pointer move. */
    setOptions(patch: Partial<ReplayShadeOptions>): void;
    draw(ctx: CanvasRenderingContext2D, rc: PrimitiveRenderContext): void;
}

/**
 * Shades the hours outside the regular session: pre-open, post-close and
 * extended hours, read from a session calendar.
 *
 * On an extended-hours chart the bars before the opening and after the close
 * look exactly like the session's own, and a move made on thin pre-open volume
 * is easy to read as one the whole market made. A faint wash behind those bars
 * says which is which without a legend.
 *
 * The phases come from the calendar as spans, not from a question per bar: the
 * spans of the visible time are laid out once and kept while a pan stays inside
 * them, and each span becomes one rect found by a binary search over the bar
 * times. The cost per frame follows the sessions in view, not the bars. With a
 * gapless axis a span shows only where bars fall in it, so a regular-hours feed
 * shades nothing, and daily or longer bars, which have no part of the day to
 * show, are skipped.
 */

interface SessionShadeOptions {
    /**
     * The hours to shade by. Absent or null, the chart's own calendar is read
     * each frame (`chart.setSessionCalendar`, `Instrument.applyTo`), so the
     * shade follows the instrument and clears when the chart drops its hours.
     */
    calendar?: SessionPhaseSource | null;
    /** Wash behind pre-open bars; null leaves them unshaded. Default: a faint tint of the theme's line colour. */
    preColor?: string | null;
    /** Wash behind post-close bars; null leaves them unshaded. Default: a faint amber. */
    postColor?: string | null;
    /** Wash behind extended-hours bars; null leaves them unshaded. Default: a faint tint of the theme's axis text. */
    extendedColor?: string | null;
    /** Draw at all. Default true. */
    visible?: boolean;
}
/** What `attachSessionShading` returns: the shade's controls, and its removal. */
interface SessionShading {
    /** Change some options; the rest keep their values. */
    setOptions(patch: SessionShadeOptions): void;
    /** The options in force. */
    options(): SessionShadeOptions;
    /** Take the shade off the chart. Calling it again does nothing. */
    destroy(): void;
}
declare class SessionShade implements IPrimitive {
    private _opts;
    private _host;
    /** The spans last laid out, the source they came from and the time they cover. */
    private _cache;
    constructor(options?: SessionShadeOptions);
    attached(host: PrimitiveHost): void;
    detached(): void;
    /** Behind the series, like any background: the candles stay crisp over it. */
    zOrder(): ZOrder;
    /** A wash has no price and never widens the range. */
    autoscaleInfo(): null;
    /** Never hit, so every click reaches the bars and drawings under it. */
    hitTest(): null;
    hitBounds(): null;
    options(): SessionShadeOptions;
    setOptions(patch: SessionShadeOptions): void;
    draw(ctx: CanvasRenderingContext2D, rc: PrimitiveRenderContext): void;
    /**
     * The spans over `[from, to)`, laid out again only when the view leaves the
     * time the last answer covered. That answer reaches a view's width, and at
     * least a day, past each side, so a pan reuses it.
     */
    private _spans;
}
/**
 * Shade a chart's pre-open, post-close and extended hours. Nothing is shaded
 * until a host calls this. The shade sits behind the series in the price pane,
 * follows that pane when it moves, and fills the pane maximized over it. A
 * chart carries one: a second call returns the same shading with the new
 * options merged in.
 */
declare function attachSessionShading(chart: Chart, options?: SessionShadeOptions): SessionShading;

/**
 * Inline Buy/Sell button panel (ARCHITECTURE.md §9). An on-chart trade
 * panel docked inside the plot: a SELL button (bid), a quantity chip, and a BUY
 * button (ask), drawn on the overlay canvas so it stays fixed while the chart
 * pans/zooms. Clicks hit-test to `${id}:sell` / `${id}:buy` / `${id}:qty`, which
 * the chart routes through `subscribeClick`: the app places the order. Prices
 * update cheaply on every tick (`setPrices` / `setMark`).
 */

interface BuySellButtonsOptions {
    /** Stable id prefix; hit-tests emit `${id}:sell` / `${id}:buy` / `${id}:qty`. Default `trade`. */
    id?: string;
    /** Corner to dock to. Default `top-left`. */
    position?: WatermarkPosition;
    /**
     * Gap from the plot edges in media px. A number applies to both axes; pass
     * `{ x, y }` to offset independently (e.g. clear an OHLC legend at the top).
     */
    margin?: number | {
        x: number;
        y: number;
    };
    /** Quantity shown in the centre chip (e.g. lots or absolute qty). */
    qty?: string | number;
    /** Buy button color (defaults to the theme `buy`). */
    buyColor?: string;
    /** Sell button color (defaults to the theme `sell`). */
    sellColor?: string;
    /** Button labels. Default `BUY` / `SELL`. */
    buyLabel?: string;
    sellLabel?: string;
    /** Show the price line above each label. Default true. */
    showPrices?: boolean;
    /**
     * Uniform size multiplier for the whole panel: box, gaps and type. Default 1.
     * A dense trading layout wants these smaller so they do not crowd the pane's
     * legend rows. Clamped to 0.6..1.5; below that the labels stop being legible.
     */
    scale?: number;
}
declare class BuySellButtons implements IPrimitive {
    private readonly _id;
    private readonly _position;
    private readonly _mx;
    private readonly _my;
    private readonly _scale;
    private readonly _btnW;
    private readonly _qtyW;
    private readonly _h;
    private readonly _buyLabel;
    private readonly _sellLabel;
    private readonly _showPrices;
    private _qty;
    private _buyColor?;
    private _sellColor?;
    private _bid;
    private _ask;
    private _host;
    private _sellRect;
    private _buyRect;
    private _qtyRect;
    constructor(options?: BuySellButtonsOptions);
    attached(host: PrimitiveHost): void;
    detached(): void;
    zOrder(): ZOrder;
    autoscaleInfo(): null;
    /** Distinct bid/ask (shows a spread on the two buttons). */
    setPrices(bid: number, ask: number): void;
    /** Single mark price shown on both buttons (when there is no bid/ask). */
    setMark(price: number): void;
    setQty(qty: string | number): void;
    setColors(buyColor?: string, sellColor?: string): void;
    private _origin;
    draw(ctx: CanvasRenderingContext2D, rc: PrimitiveRenderContext): void;
    private _fmt;
    private _drawButton;
    private _drawChip;
    /** Trace a rect rounded on the left or right pair of corners only. */
    private _roundedSide;
    hitTest(x: number, y: number, _rc: PrimitiveRenderContext): PrimitiveHit | null;
}

/**
 * Exponential moving average (ARCHITECTURE.md §8: an indicator that validates
 * the series/extensibility model). Pure; the result is plotted as a `line`
 * series, demonstrating derived data on the shared time axis.
 */

/** EMA over a numeric series. Seeds from the first value; k = 2/(period+1). */
declare function ema(values: readonly number[], period: number): number[];
/**
 * EMA of bar closes as plottable bars (close = ema; O/H/L = ema), ready to feed
 * a `line` series via `series.setData(...)`.
 */
declare function emaSeries(bars: readonly Bar[], period: number): Bar[];

/**
 * Relative Strength Index using Wilder smoothing.
 *
 * Uses Wilder smoothing (RMA): the first average gain/loss is the simple mean of
 * a complete finite window of `period` deltas, then each subsequent average
 * carries `(prev*(period-1) + current)/period`. Missing deltas preserve seeded
 * state but emit NaN. Each leg retries an unavailable seed independently.
 */

/** Wilder RSI over a numeric series. Returns NaN for the first `period` slots. */
declare function rsi(values: readonly number[], period?: number): number[];
/**
 * RSI of bar closes as plottable bars (close = rsi). Warmup bars carry NaN so a
 * `line` series breaks cleanly before the first value (the renderer skips
 * non-finite points) instead of dropping to zero.
 */
declare function rsiSeries(bars: readonly Bar[], period?: number): Bar[];

/**
 * Average True Range: Wilder's ATR, matching `openalgo.atr(high, low, close, 14)`.
 * Shared by the Supertrend indicator. True range of the first bar is high-low.
 */
/** True range series. tr[0] = high[0]-low[0]; thereafter the classic 3-way max. */
declare function trueRange(high: readonly number[], low: readonly number[], close: readonly number[]): number[];
/**
 * Wilder ATR. The first value is the SMA of the first `period` consecutive
 * finite true ranges, so on complete data it lands at index period-1. A missing
 * or overflowing true range leaves its own slot NaN and keeps the average, and
 * the next finite one carries on from it: a gap costs only the bars it covers.
 */
declare function atr(high: readonly number[], low: readonly number[], close: readonly number[], period?: number): number[];

/**
 * Supertrend: matching `openalgo.supertrend(high, low, close, period=10, multiplier=3.0)`,
 * which returns (supertrend value, direction). Direction follows the OpenAlgo
 * convention: -1 = uptrend (line is support below price), +1 = downtrend (line
 * is resistance above price). ATR uses Wilder smoothing (see ./atr).
 */

interface SupertrendPoint {
    /** The Supertrend band value, or NaN during ATR warmup and on a bar with no ATR or no close. */
    value: number;
    /** -1 = uptrend (bullish), +1 = downtrend (bearish). */
    direction: -1 | 1;
}
/**
 * Supertrend value + direction per bar. Warmup bars carry value=NaN, and so does
 * a bar with no ATR or no close. Such a bar leaves the bands, the direction and
 * the last accepted close as they stood, and the next complete bar carries on
 * from them.
 */
declare function supertrend(bars: readonly Bar[], period?: number, multiplier?: number): SupertrendPoint[];
/**
 * Supertrend as two plottable `line` series for direction coloring: `up`
 * (uptrend / support, typically green) carries the value only while direction is
 * -1; `down` (downtrend / resistance, typically red) only while +1. The inactive
 * series carries NaN, so each renders as separate segments that swap at flips
 * (the line renderer breaks across non-finite points).
 */
declare function supertrendSeries(bars: readonly Bar[], period?: number, multiplier?: number): {
    up: Bar[];
    down: Bar[];
};

/**
 * OHLC-preserving conflation (ARCHITECTURE.md §4.4): what a zoomed-out series
 * pass draws instead of one mark per bar.
 *
 * Once bars are narrower than the stick a candle is drawn with, several of them
 * land in the same device-pixel column, and drawing each one paints the same
 * pixels over and over: 200,000 bars on a 1,000 px plot put two hundred marks
 * in every column where one would do. The level of detail at the end of this
 * file merges what shares a column into one OHLC-preserving stick (open =
 * first, close = last, high = max, low = min, volume = sum; never a lossy
 * average), so a frame costs the plot's width rather than the history's
 * length. It is on by default (`conflate`) and inert above its threshold: a
 * chart zoomed in far enough that every bar has a column of its own paints
 * exactly what it always did.
 *
 * The group helpers (`conflationGroupSize`, `conflateBars`, `conflateItems`,
 * `mergeBars`) merge fixed-size groups, for a host that downsamples bars
 * itself; the pane merges by column.
 */

/**
 * How many source bars to merge per drawn bar. Returns 1 (no conflation) while
 * each bar is at least `minPx` wide; otherwise ceil(minPx / barWidthPx) scaled
 * by `factor` (higher = more aggressive smoothing).
 */
declare function conflationGroupSize(barSpacing: number, dpr: number, minPx?: number, factor?: number): number;
/** Merge a single group of bars into one OHLC-preserving bar (uses the first bar's time). */
declare function mergeBars(group: readonly Bar[]): Bar;
/** Conflate a bar series into groups of `groupSize` (identity when groupSize ≤ 1). */
declare function conflateBars(bars: readonly Bar[], groupSize: number): Bar[];
/** Conflate already-projected draw items: merge bars and place x at the group centre. */
declare function conflateItems<T extends {
    x: number;
    bar: Bar;
}>(items: readonly T[], groupSize: number): {
    x: number;
    bar: Bar;
}[];

/**
 * The chart's side of a series transform (`Chart.setSeriesTransform`):
 * turning one on or off, and forming a series' elements again on each tick.
 *
 * `registerSeriesTransform` installs it, and before anything is registered no
 * series can hold a transform, so a chart-only import, which registers
 * nothing, carries none of this. chart-series.ts keeps the map of runs, the
 * routing through it and the refusal of a spec when nothing is registered.
 */

/** Register a transform under the id a `SeriesTransformSpec` names it by. */
declare function registerSeriesTransform(type: string, definition: SeriesTransformDefinition): void;

/** Cancels a subscription. */
type UnsubscribeFn = () => void;
interface BarsRequest {
    symbol: string;
    exchange: string;
    /** Interval token, e.g. "1m", "5m", "1h", "D". */
    interval: string;
    from?: UTCSeconds;
    to?: UTCSeconds;
    /** Fetch authoritative history instead of a cached snapshot, when supported. */
    noCache?: boolean;
    /** Cancel this consumer's request. Existing feeds may ignore cancellation. */
    signal?: AbortSignal | undefined;
    /**
     * Deadline in milliseconds, including response-body reading. The shared
     * request pool holds it to 2^31 - 1 ms, the longest a timer waits.
     */
    timeoutMs?: number;
    /** Preferred number of bars; a date-range feed may return a different count. */
    countBack?: number;
    /**
     * Which of the provider's series: a session, an adjustment, a currency or a
     * unit. Absent is the provider's default. A feed serves only what it
     * declares through `dataVariants`, and never derives one series from another.
     */
    variant?: DataVariant;
}
/** An optional provider page before an exclusive UTC-second anchor. */
interface BarsPageRequest extends BarsRequest {
    before: UTCSeconds;
    countBack: number;
}
/** Providers can distinguish an empty date window from exhausted history. */
interface BarsPage {
    bars: Bar[];
    hasMore?: boolean;
    /** Exclusive anchor for the next page, including pages without observations. */
    nextBefore?: UTCSeconds;
}
/** Optional context for continuing history and recovering an interrupted stream. */
interface BarSubscriptionOptions {
    /** Last historical time-bucketed bar, used as the live builder's starting point. */
    seedFrom?: Bar;
    /** Cumulative day volume at the seed snapshot, if the host knows it. */
    cumDayVolumeSoFar?: number;
    /**
     * The stream reconnected and may have missed data. Refresh authoritative
     * history, then resubscribe with its last bar as the seed. Older bars must
     * not be delivered through onBar, whose consumers commonly accept only tails.
     */
    onResync?: () => void;
}
/** What a live push knows about the bar beyond its values. */
interface LiveBarMeta {
    /**
     * The bar's open, high, low and volume cover only the ticks its builder saw:
     * it opened the bucket from a tick without having streamed the bar before
     * it. See `CandleUpdate.provisional`. A consumer holding history for the
     * same bucket keeps that open and widens the extremes rather than replacing
     * them.
     */
    provisional?: boolean;
}
/**
 * Broker-agnostic market-data source. The chart depends only on this.
 * `subscribeBars` is optional: a history-only feed (e.g. `OpenAlgoDataFeed`) omits
 * it, while a live feed (`OpenAlgoLiveDataFeed`, or your own) implements it.
 */
interface DataFeed {
    getBars(req: BarsRequest): Promise<Bar[]>;
    getBarsPage?(req: BarsPageRequest): Promise<BarsPage>;
    /**
     * The variants this provider serves for an instrument and interval. Without
     * it the feed serves only its default series, and a request for any other
     * variant is reported unsupported rather than sent.
     */
    dataVariants?(query: DataVariantQuery): DataVariantCapabilities | Promise<DataVariantCapabilities>;
    /** Read a closed-bar snapshot without initiating a network request. */
    getCachedBars?(req: BarsRequest): Promise<Bar[] | undefined>;
    subscribeBars?(req: BarsRequest, onBar: (bar: Bar, meta?: LiveBarMeta) => void, opts?: BarSubscriptionOptions): UnsubscribeFn;
    /**
     * `opts.depthLevel` requests a book depth (broker-dependent: 5/20/30/50).
     * Named on the interface so a caller holding a `DataFeed` can ask for one;
     * an implementation is free to ignore it and send the broker's default.
     */
    subscribeDepth?(req: BarsRequest, onDepth: (depth: MarketDepth) => void, opts?: {
        depthLevel?: number;
    }): UnsubscribeFn;
    /**
     * Instruments matching what the user typed, closest first. A widget given
     * this feed and no `symbolSearch` callback looks symbols up through it.
     */
    searchSymbols?(request: SymbolSearchRequest): Promise<SymbolMatch[]>;
}
/** One result of a symbol lookup. Each contract carries its own symbol and exchange. */
interface SymbolMatch {
    symbol: string;
    exchange?: string;
    name?: string;
    assetClass?: string;
    iconUrl?: string;
    contractGroup?: {
        label?: string;
        contracts: readonly SymbolMatch[];
    };
}
interface SymbolSearchRequest {
    /** What the user typed. */
    query: string;
    /** Cancels this lookup once a newer query replaces it. */
    signal?: AbortSignal | undefined;
}
interface DepthLevel {
    price: number;
    qty: number;
    orders?: number;
}
/** Variable-depth book; `bids`/`asks` length = whatever the broker streams (5..200). */
interface MarketDepth {
    /** Exchange event timestamp in UTC seconds, when supplied by the feed. */
    timeSec?: UTCSeconds;
    bids: DepthLevel[];
    asks: DepthLevel[];
    ltp: number;
    ltq?: number;
}
/**
 * An instrument as the host's feed names it. Both parts are opaque: they are
 * compared exactly and never case-folded or parsed, so one symbol on two
 * venues is two instruments.
 */
interface InstrumentKey {
    symbol: string;
    exchange: string;
}
/**
 * A quote as its provider reports it. It is the provider's own observation,
 * never a bar's close relabelled: a host without a quote source shows no
 * price rather than the last candle. Absent fields are unknown, never zero.
 */
interface QuoteSnapshot extends InstrumentKey {
    /** Last traded price. */
    last: number;
    /** Reference close for the change figures, usually the previous session's. */
    previousClose?: number;
    bid?: number;
    ask?: number;
    /** Session volume. */
    volume?: number;
    /** Exchange time of the last trade, UTC seconds. */
    time?: UTCSeconds;
    /** The provider knows this quote is delayed rather than real time. */
    delayed?: boolean;
}
interface QuoteRequest {
    instruments: readonly InstrumentKey[];
    /** Cancels this snapshot when the rows asking for it leave the screen. */
    signal?: AbortSignal;
}
/**
 * A quote stream as its provider reports it. Quotes held while the stream is
 * not `live` may have missed updates, so a consumer shows them as stale.
 */
type QuoteStreamStatus = 'connecting' | 'live' | 'reconnecting' | 'disconnected';
interface QuoteStreamHandlers {
    onQuote(quote: QuoteSnapshot): void;
    onStatus?(status: QuoteStreamStatus, error?: Error): void;
}
/**
 * Optional quote source for watchlist rows. Separate from `DataFeed`, which
 * serves bars: a quote comes only from here.
 */
interface QuoteFeed {
    /** A snapshot per instrument. An instrument missing from the answer is unknown. */
    getQuotes(request: QuoteRequest): Promise<QuoteSnapshot[]>;
    /**
     * Push quotes for these instruments until the returned function is called.
     * Omit it for a snapshot-only source. A consumer may call this once per
     * instrument so it can release exactly the rows that scroll away; a
     * provider multiplexes them over its own connection.
     */
    subscribeQuotes?(instruments: readonly InstrumentKey[], handlers: QuoteStreamHandlers): UnsubscribeFn;
}
/** One page of an instrument's news, newest first. */
interface NewsRequest extends InstrumentKey {
    /** The provider's opaque cursor from the previous page; absent for the newest page. */
    cursor?: string;
    /** Preferred page size. A provider may return fewer. */
    limit: number;
    signal?: AbortSignal;
}
/** Every text field is plain text: a consumer never interprets it as markup. */
interface NewsItem {
    /** Stable provider identity, used to drop an item repeated across pages. */
    id: string;
    headline: string;
    /** The publisher's name. */
    source?: string;
    /** Publication time, UTC seconds. */
    time: UTCSeconds;
    summary?: string;
    /** The full article. A consumer opens only http and https links. */
    url?: string;
}
interface NewsPage {
    items: NewsItem[];
    /** Cursor for the next older page; absent or null when there is none. */
    nextCursor?: string | null;
}
/** Optional symbol news source. Coverage and rights remain the provider's. */
interface NewsFeed {
    getNews(request: NewsRequest): Promise<NewsPage>;
}
/** The side of an order, one declaration for the base and the trade tier. */
type OrderSide = 'BUY' | 'SELL';
/** How an order is priced, one declaration for the base and the trade tier. */
type OrderType = 'MARKET' | 'LIMIT' | 'SL' | 'SL-M';
/**
 * The order `TradeFeed.placeOrder` takes.
 *
 * @deprecated Removed in 3.0.0. Only `TradeFeed` takes it. Use `PlaceRequest` from
 * `openalgo-charts/trade` (since 1.0.0), the order `OrderEngine` and `OrderFeed.place` take.
 */
interface PlaceOrder {
    symbol: string;
    exchange: string;
    side: OrderSide;
    type: OrderType;
    qty: number;
    price?: number;
    triggerPrice?: number;
    /** Idempotency token so a retried place never double-fills. */
    clientToken?: string;
}
/**
 * A broker surface of place, modify and cancel plus order and position
 * subscriptions. Nothing in the library takes or implements it: the trading
 * layer never calls one.
 *
 * @deprecated Removed in 3.0.0. Implement `OrderFeed` from `openalgo-charts/trade`
 * (since 1.0.0), which `OrderEngine` writes through and `OpenAlgoTradeFeed`
 * implements, and hand the book to the chart with `chart.trading.syncState`.
 */
interface TradeFeed {
    placeOrder(o: PlaceOrder): Promise<{
        orderId: string;
    }>;
    modifyOrder(orderId: string, patch: Partial<PlaceOrder>): Promise<void>;
    cancelOrder(orderId: string): Promise<void>;
    subscribeOrders(cb: (orders: unknown[]) => void): UnsubscribeFn;
    subscribePositions(cb: (positions: unknown[]) => void): UnsubscribeFn;
}

type TradingOperation = 'place' | 'modify' | 'cancel';
/** Optional restrictions on existing write paths; omissions preserve legacy support. */
interface TradingCapabilities {
    readonly place?: boolean | 'unknown';
    readonly modify?: boolean | 'unknown';
    readonly cancel?: boolean | 'unknown';
    /** Accepted types for new orders. An empty list disables placement. */
    readonly orderTypes?: readonly OrderType[];
    /** Accepted modes for new orders; does not change the broker's actual mode. */
    readonly modes?: readonly ('live' | 'analyzer')[];
}
interface TradingCapabilityRequest {
    readonly operation: TradingOperation;
    readonly symbol?: string;
    readonly exchange?: string | undefined;
    readonly orderId?: string | undefined;
    readonly type?: OrderType | undefined;
    readonly mode?: 'live' | 'analyzer' | undefined;
}
/** A configured provider returning undefined declares that support is unavailable. */
type TradingCapabilitySource = TradingCapabilities | ((request: Readonly<TradingCapabilityRequest>) => TradingCapabilities | undefined);
type TradingCapabilityResult = {
    supported: true;
} | {
    supported: false;
    reason: string;
};
/** Shared by host controls and the write boundary. Never grants broker authority. */
declare function checkTradingCapability(source: TradingCapabilitySource | undefined, request: TradingCapabilityRequest): TradingCapabilityResult;
/** Raised only before delivery, so an unsupported write does not become ambiguous. */
declare class TradingCapabilityError extends Error {
    readonly preflight: true;
    readonly operation: TradingOperation;
    constructor(operation: TradingOperation, reason: string);
}
declare function assertTradingCapability(source: TradingCapabilitySource | undefined, request: TradingCapabilityRequest): void;

/** Limits apply to each pool, whose identity belongs to one data feed. */
interface HistoryRequestPoolOptions {
    maxConcurrent?: number;
    timeoutMs?: number;
}
/**
 * Shares in-flight historical requests while each consumer owns its lifetime.
 * Higher priorities run first among queued work; active work is not preempted.
 */
declare class HistoryRequestPool {
    private readonly _feed;
    private readonly _max;
    private readonly _timeout;
    private readonly _jobs;
    private _active;
    private _draining;
    constructor(feed: DataFeed, options?: HistoryRequestPoolOptions);
    getBars(req: BarsRequest, priority?: number): Promise<Bar[]>;
    getBarsPage(req: BarsPageRequest, priority?: number): Promise<BarsPage>;
    private _request;
    private _remove;
    private _finish;
    private _drain;
}
/** Reuse one pool only for consumers of this exact feed instance. */
declare function sharedHistoryRequests(feed: DataFeed): HistoryRequestPool;

/** `unsupported`: the provider does not declare the requested variant, so nothing was fetched. */
type DataLoadingStatus = 'idle' | 'loading' | 'ready' | 'empty' | 'refreshing' | 'stale' | 'error' | 'unsupported';
type HistoryLoadingStatus = 'idle' | 'loading' | 'error' | 'exhausted' | 'limited';
type DataUpdateReason = 'load' | 'cache' | 'live' | 'refresh' | 'prepend' | 'resume' | 'state';
/** Display bars are held still while paused; bars() retains the live store. */
interface DataLoadingSnapshot {
    readonly request: Readonly<BarsRequest> | null;
    readonly bars: readonly Bar[];
    readonly status: DataLoadingStatus;
    readonly historyStatus: HistoryLoadingStatus;
    readonly hasMore: boolean | null;
    readonly error?: Error;
    readonly historyError?: Error;
    readonly reason: DataUpdateReason;
    readonly paused: boolean;
    /** With status `unsupported`: the first field of the request's variant the provider does not serve. */
    readonly unsupported?: DataVariantDimension;
}
interface DataLoadingOptions {
    requestPool?: HistoryRequestPool;
    timeoutMs?: number;
    pageSize?: number;
    /** Date-window fallback for feeds without getBarsPage. Defaults to the load window. */
    pageWindowSec?: number;
    /** Empty windows inspected per gesture, without claiming permanent exhaustion. */
    maxEmptyPages?: number;
    maxBars?: number;
    /** Optional history repair cadence. Zero disables polling. */
    pollIntervalMs?: number;
    /**
     * Repair the tail when a bar closes, driven by the stream rather than the
     * clock. A pushed bar that opens a new bucket means the bar before it has
     * just closed, so one refresh runs `delayMs` later (default 2500 ms), long
     * enough for a broker's history to have published the close. Nothing fires
     * while the stream is quiet, so a closed market costs no requests. When the
     * reply still stops short of the bar that closed, the repair retries
     * `retries` times (default 2), `retryDelayMs` apart (default 5000 ms).
     * Off by default.
     */
    refreshOnBarClose?: boolean | {
        delayMs?: number;
        retries?: number;
        retryDelayMs?: number;
    };
    /**
     * Refresh at once when a pushed bar skips one or more whole buckets, which
     * is what a stream leaves behind after a dropped socket, a hidden tab or a
     * sleeping machine. Needs a fixed-length interval. Off by default.
     */
    refreshOnGap?: boolean;
    /**
     * How many bars back from the tail a refresh re-fetches. Unset re-fetches
     * the whole load window on every refresh, which is what polling always did;
     * a small number turns each repair into a tail request. A gap repair widens
     * the window to cover the gap. Needs a fixed-length interval; any other
     * interval keeps the whole window.
     */
    refreshWindowBars?: number;
    /** UTC seconds. */
    now?: () => number;
}
/**
 * Owns history, paging and live updates for one current instrument. Consumers
 * decide how to paint snapshots; no canvas, DOM or broker execution is owned here.
 */
declare class DataLoadingController {
    private readonly _feed;
    private readonly _pool;
    private readonly _options;
    private readonly _listeners;
    private _state;
    private _bars;
    private _scope;
    private _refreshAbort;
    private _pageAbort;
    private _loadWork;
    private _pageWork;
    private _unsubscribe;
    private _stream;
    private _generation;
    private _refreshId;
    private _before;
    private _buffer;
    private _poll;
    private _visible;
    private _destroyed;
    /**
     * Whether the provider has said it serves the current request's variant, or
     * the request names none. Until it has, nothing fetches, refreshes or pages
     * that variant: a declaration that failed is not a declaration.
     */
    private _declared;
    /** Seconds per bar of the current request, null when the interval is not fixed-length. */
    private _seconds;
    /** Bucket whose open is only the first tick a builder saw, until history covers it. */
    private _provisionalTime;
    private _repairTimer;
    private _repairRetries;
    private _pendingRepair;
    private readonly _barClose;
    constructor(feed: DataFeed, options?: DataLoadingOptions);
    getState(): DataLoadingSnapshot;
    bars(): readonly Bar[];
    subscribe(listener: (snapshot: DataLoadingSnapshot) => void): UnsubscribeFn;
    load(req: BarsRequest): Promise<readonly Bar[]>;
    private _load;
    refresh(): Promise<readonly Bar[]>;
    private _refresh;
    /** The span a refresh asks history for: the request's own width ending now, or a tail window. */
    private _refreshWindow;
    /**
     * Put a refresh's answer in place of the window it covers, keep what the
     * stream built past it, and fold in what arrived meanwhile. Returns the
     * newest bar time the answer held.
     */
    private _mergeRefresh;
    /**
     * Supply bars from an existing host subscription instead of subscribing twice.
     *
     * `meta.provisional` marks a bar whose open is only the first tick a builder
     * saw, see `CandleUpdate.provisional`. Pushed onto a bar history already
     * holds for that bucket, it keeps that bar's open and widens the extremes
     * instead of replacing them, so a repair that found the true open is not
     * undone by the next tick.
     */
    pushBar(value: Bar, meta?: LiveBarMeta): void;
    /** The stream opened a bucket: the one before it closed, and any between were skipped. */
    private _onNewBucket;
    private _scheduleRepair;
    /** Run a repair now, or queue one behind the refresh already in flight rather than aborting it. */
    private _requestRepair;
    private _clearRepairTimer;
    /**
     * One older page. `until` (UTC seconds) widens a date feed's window to reach
     * it in one request, up to `maxBars` bars of a fixed interval; a paged feed
     * keeps its ordinary window.
     */
    loadMore(until?: number): Promise<readonly Bar[]>;
    private _loadMore;
    setPaused(paused: boolean): void;
    setVisible(visible: boolean): void;
    destroy(): void;
    private _current;
    private _replaceWindow;
    private _limit;
    private _publish;
    private _startStream;
    private _release;
    private _clearPoll;
    private _schedulePoll;
    private _cancel;
}

/**
 * OpenAlgo REST adapter (ARCHITECTURE.md §10.0). The chart depends only on the
 * `DataFeed` interface; this is the only file that knows OpenAlgo's REST shape.
 *
 * History endpoint: POST `${baseUrl}/api/v1/history`.
 * Search endpoint: POST `${baseUrl}/api/v1/search`.
 * The request fields and interval mapping are pinned by offline adapter
 * fixtures. Response timestamps accept epoch seconds, epoch milliseconds,
 * offset-qualified timestamps and unqualified IST date/time strings.
 */

interface OpenAlgoConfig {
    baseUrl: string;
    apiKey: string;
    /** Injectable fetch (defaults to global fetch); lets the adapter be tested offline. */
    fetchImpl?: typeof fetch;
    /**
     * Instrument capability from host metadata. Explicit false omits the API's
     * placeholder OI column before it can become a misleading cash reading.
     * Missing/unknown capability preserves finite observations, including zero.
     */
    hasOpenInterest?(request: Readonly<BarsRequest>): boolean | undefined;
}
interface HistoryRow {
    timestamp?: number | string;
    time?: number | string;
    open: number;
    high: number;
    low: number;
    close: number;
    volume?: number;
    /** Present on a derivatives response; the platform always sends the column. */
    oi?: number;
}
interface HistoryResponse {
    status?: string;
    message?: string;
    data?: HistoryRow[];
}
/** Pure: coerce a row timestamp (epoch s / epoch ms / IST string) to UTC seconds. */
declare function rowTimeToUtcSeconds(value: number | string): number;
/** Pure: map an OpenAlgo history response into sorted internal bars. */
declare function mapHistoryResponse(json: HistoryResponse, hasOpenInterest?: boolean): Bar[];
declare class OpenAlgoDataFeed implements DataFeed {
    private readonly _config;
    private readonly _fetch;
    constructor(config: OpenAlgoConfig);
    getBars(req: BarsRequest): Promise<Bar[]>;
    /** Instruments matching `request.query` on every exchange, in the order the platform ranks them. */
    searchSymbols(request: SymbolSearchRequest): Promise<SymbolMatch[]>;
}

/**
 * OpenAlgo WebSocket adapter (ARCHITECTURE.md §10). Speaks the documented
 * OpenAlgo WS proxy protocol (default port 8765, or wss://host/ws in production):
 *
 *   1. authenticate: { action:'authenticate', api_key }
 *      server answers { type:'auth', status:'success' } (or a refusal)
 *   2. subscribe   : { action:'subscribe', symbol, exchange, mode }   mode 1=LTP 2=Quote 3=Depth
 *                    (Depth adds depth_level, e.g. 5/20/30/50)
 *   3. server pushes { type:'market_data', symbol, exchange, mode, data:{...} }
 *   4. heartbeat   : OpenAlgo's proxy pings at the protocol level, which a
 *      browser answers without telling JavaScript, so the client keeps its own
 *      watchdog and asks { action:'ping' } when the stream goes quiet. An
 *      application-level 'ping' frame, which some builds send, is ponged.
 *
 * Maps inbound LTP / Quote / Depth into typed callbacks the chart consumes
 * (candle builder, last price, DOM ladder). The socket is injectable so the
 * adapter is unit-testable with a fake socket and no network.
 *
 * Step 1 is a gate, not a formality. Nothing but the auth frame leaves the
 * socket until the server acknowledges: a proxy that drops pre-auth frames
 * otherwise leaves a chart that believes it is subscribed and is permanently
 * silent, which under a Buy button is the worst state this library can be in.
 */

type WsMode = 'LTP' | 'Quote' | 'Depth';
/**
 * Socket lifecycle reported by `onState`.
 *
 * 'open' means authenticated and carrying traffic, not merely TCP-connected:
 * the window between transport open and the auth acknowledgement still reads
 * as 'connecting', because nothing can be sent in it.
 */
type WsState = 'connecting' | 'open' | 'closed' | 'error' | 'reconnecting';
/** A non-market-data control frame (auth / subscribe ack, or a server error). */
interface WsControlMessage {
    type?: string;
    status?: string;
    message?: string;
    [k: string]: unknown;
}
/**
 * A warning the client raised about its own connection, delivered on the same
 * control channel. `type` is deliberately a value no server frame uses, so a
 * host can never mistake one for the other.
 */
interface WsClientWarning extends WsControlMessage {
    type: 'client_warning';
    /**
     * AUTH_TIMEOUT | AUTH_FAILED | AUTH_UNACKNOWLEDGED | RECONNECT_ABANDONED |
     * HEARTBEAT_DEAD | STREAM_RESYNC | SEQUENCE_GAP
     */
    code: string;
    message: string;
}
/** Minimal socket surface (the browser WebSocket satisfies this). */
interface SocketLike {
    send(data: string): void;
    close(): void;
    onopen: (() => void) | null;
    onclose: (() => void) | null;
    onerror?: (() => void) | null;
    onmessage: ((ev: {
        data: string;
    }) => void) | null;
    /** 1 === OPEN (browser WebSocket.OPEN). Used to gate sends. */
    readyState?: number;
}
type SocketFactory = (url: string) => SocketLike;
interface OpenAlgoWsConfig {
    url: string;
    apiKey: string;
    socketFactory?: SocketFactory | undefined;
    /**
     * Auto-reconnect after an unexpected close: re-authenticate and resubscribe
     * every active subscription, with jittered exponential backoff. Enabled by
     * default; `close()` is treated as intentional and never reconnects.
     */
    reconnect?: {
        enabled?: boolean;
        baseDelayMs?: number;
        maxDelayMs?: number;
        maxAttempts?: number;
        /** Full jitter on each delay (default true). Off gives the old lockstep timing. */
        jitter?: boolean;
        /** Injectable [0,1) source, so a test pins the delay instead of guessing it. */
        random?: () => number;
    } | undefined;
    /** Handshake gating. Data frames wait for the server's answer to `authenticate`. */
    auth?: {
        /**
         * Require a positive acknowledgement (default true). Set false only for a
         * proxy build known to answer nothing: the connection then counts as
         * authenticated on transport open, and one AUTH_UNACKNOWLEDGED warning is
         * raised per connection so the missing guarantee is visible rather than
         * silently assumed.
         */
        requireAck?: boolean;
        /** Wait for the acknowledgement this long before failing the connection (default 5000). */
        ackTimeoutMs?: number;
    } | undefined;
    /**
     * Liveness watchdog. After `timeoutMs` with no inbound frame the client asks
     * the far end a direct question and gives it `probeMs` to answer; only
     * silence to that counts as death, and the socket is then reconnected
     * (defaults 45000 and 5000; a `timeoutMs` of 0 disables the watchdog).
     */
    heartbeat?: {
        timeoutMs?: number;
        probeMs?: number;
    } | undefined;
}
interface LtpEvent {
    symbol: string;
    exchange: string;
    ltp: number;
    ltq?: number;
    /** Cumulative day volume (Quote mode): feeds the candle builder's day-delta mode. */
    volume?: number;
    timeSec: number;
}
/**
 * Real-time order lifecycle event from the `subscribe_orders` stream: fills,
 * partial fills, rejections, cancellations, pushed by the broker (or by the
 * sandbox engine in analyze mode).
 */
interface OrderUpdateEvent {
    orderId: string;
    symbol: string;
    exchange: string;
    action: 'BUY' | 'SELL';
    quantity: number;
    price: number;
    /** Undefined when the broker reports 0 (plain LIMIT/MARKET). */
    triggerPrice?: number;
    pricetype: string;
    product: string;
    /** Lowercase OpenAlgo status: open | trigger pending | complete | rejected | cancelled | ... */
    status: string;
    filledQuantity: number;
    pendingQuantity: number;
    averagePrice: number;
    /** Broker RMS/OMS text when rejected. */
    rejectionReason: string;
    /** 'live' for broker events, 'analyze' for sandbox events. */
    mode: string;
}
/**
 * Pure: build a subscribe message, `{ action, symbol, exchange, mode }`, where
 * `mode` is the numeric OpenAlgo data mode. Depth subscriptions may request a
 * book depth (broker-dependent: 5/20/30/50).
 *
 * The wire key the proxy reads is `depth`; 1.x sent `depth_level`, which the
 * proxy never read, so every request above the default was silently served
 * at five levels. Both keys go out: `depth` for the proxy, `depth_level` for
 * any consumer that learned the old name from this library. A wire key has no
 * declaration to tag, so COMPATIBILITY.md lists `depth_level` as deprecated:
 * 3.0.0 stops sending it.
 */
declare function formatSubscribe(mode: WsMode, symbol: string, exchange: string, depthLevel?: number): string;
declare function formatUnsubscribe(mode: WsMode, symbol: string, exchange: string): string;
/**
 * Pure: split the documented `SYMBOL.EXCHANGE` topic.
 *
 * Only that two-part form is accepted. Guessing at other separators would
 * misattribute a tick to the wrong instrument, and a wrong price on the right
 * chart is worse than no price at all.
 */
declare function parseTopic(topic: unknown): {
    symbol: string;
    exchange: string;
} | null;
/**
 * Pure: classify the server's answer to the handshake as 'ok', 'failed', or
 * null for a frame that is not about authentication at all.
 *
 * Absence of an error is never an acknowledgement, so 'ok' is only ever
 * returned for a frame that says so. An error frame counts as a refusal
 * because the handshake is the only thing outstanding when it arrives, and it
 * is recognised by `status: 'error'` as well as by `type: 'error'`: OpenAlgo's
 * proxy answers a bad key with `{ status:'error', code, message }` and no type
 * at all, so keying only on the type left the one refusal that matters
 * unclassified and retried on a timer until the key got rate limited.
 */
declare function classifyAuthAck(raw: unknown): 'ok' | 'failed' | null;
/**
 * Pure: the per-topic sequence number on an inbound frame, when the server
 * sends one.
 *
 * OpenAlgo's documented market_data frame carries no sequence, so the gap
 * detection built on this is inert against a stock proxy, and that is
 * deliberate: a client cannot see a silent drop on an unsequenced stream, and
 * claiming gap detection the wire does not support would be a lie in the UI.
 * A proxy that does number its frames gets the warnings for free.
 */
declare function readSequence(raw: unknown): number | undefined;
/**
 * Full-jitter backoff: attempt n waits somewhere in [ceiling/2, ceiling).
 *
 * Deterministic backoff has every client in a fleet reconnecting in the same
 * millisecond after a proxy restart, which knocks the recovering server back
 * down. `random` is a parameter so a test gets an exact number.
 */
declare function backoffDelayMs(attempt: number, opts: {
    baseDelayMs: number;
    maxDelayMs: number;
    jitter?: boolean;
}, random?: () => number): number;
/**
 * Pure: classify + normalise an inbound message into an LTP or Depth event.
 * Payload fields live under `data` per the protocol, but the parser also
 * tolerates a flat shape for resilience across broker adapters.
 *
 * The proxy puts identity on the envelope. Existing nested broker identity
 * takes precedence, with `topic` retained as the final fallback.
 */
declare function parseMessage(raw: unknown): {
    kind: 'ltp';
    event: LtpEvent;
} | {
    kind: 'depth';
    symbol: string;
    exchange: string;
    depth: MarketDepth;
} | null;
declare class OpenAlgoWsFeed {
    private readonly _config;
    private readonly _factory;
    private _sock;
    private _phase;
    /**
     * Guards every socket callback. A socket we have abandoned keeps its handlers
     * (a host may still hold the reference and fire them) but a stale one must
     * not schedule a second reconnect for an event we already handled.
     */
    private _epoch;
    private readonly _ltpCbs;
    private readonly _depthCbs;
    private readonly _stateCbs;
    private readonly _controlCbs;
    private readonly _orderCbs;
    private _ordersSubscribed;
    /**
     * Desired subscription state, keyed by mode:symbol:exchange, replayed in full
     * once each connection authenticates. This replaces the old frame queue on
     * purpose: a queue can hold two frames for one key (one queued, one replayed)
     * and grows without bound during a long outage, while desired state is
     * inherently deduplicated and bounded by the number of subscriptions.
     */
    private readonly _subs;
    /** Last sequence seen per topic. Only ever populated by a server that numbers frames. */
    private readonly _seq;
    private _userClosed;
    /** The server refused the key. Retrying that on a timer gets it blocked, so we do not. */
    private _fatal;
    private _everAuthed;
    private _attempts;
    private _reconnectTimer;
    private _authTimer;
    private _liveTimer;
    private readonly _rc;
    private readonly _auth;
    private readonly _hbTimeoutMs;
    private readonly _hbProbeMs;
    constructor(config: OpenAlgoWsConfig);
    /**
     * Open the socket. Also the deliberate way back from a refused key or an
     * earlier `close()`: both are user-intent states, and only user intent clears
     * them, so this is the one way out of the fatal state back to connecting.
     */
    connect(): void;
    /** True once the handshake is acknowledged: the only time data frames go out. */
    isReady(): boolean;
    private _openSocket;
    /** Subscribe to socket lifecycle (connecting / open / closed / error). */
    onState(cb: (state: WsState) => void): () => void;
    /** Subscribe to control frames: auth / subscribe acks, server errors, client warnings. */
    onControl(cb: (msg: WsControlMessage) => void): () => void;
    private _emitState;
    private _emitControl;
    private _warn;
    /**
     * Transport open. Send the handshake and nothing else: the subscription
     * replay waits for the acknowledgement, because a proxy that discards
     * pre-auth frames would swallow it and leave the chart silent.
     */
    private _onOpen;
    private _onAuthenticated;
    private _onAuthFailed;
    /** A failure we detected ourselves: drop the socket and back off as if it had closed. */
    private _failConnection;
    private _onClose;
    /** Schedule a reconnect with jittered exponential backoff unless the user closed us. */
    private _maybeReconnect;
    /**
     * Send a data frame, or hold it back until the handshake is answered.
     *
     * Held frames are not queued: the caller's intent is already recorded in
     * `_subs` / `_ordersSubscribed`, and that is what gets replayed.
     */
    private _sendGated;
    /**
     * Restart the liveness watchdog. A TCP connection routinely stays open after
     * the far end is gone and `onclose` may never fire; left alone that shows a
     * connected badge over a frozen price. Any inbound frame counts, including a
     * ping or a control ack.
     */
    private _armLiveness;
    /**
     * Silence is not death, so ask before concluding one.
     *
     * OpenAlgo's proxy keeps the connection alive with protocol-level WebSocket
     * pings, which a browser answers itself and never surfaces to JavaScript, and
     * it broadcasts nothing else on a schedule. A quiet symbol out of hours
     * therefore produces no inbound frame at all, and a bare timeout would hang
     * up on a perfectly healthy socket and resubscribe every `timeoutMs` all
     * night. The proxy answers `{ action:'ping' }` with a pong, and a build that
     * does not know the action answers an error, which proves the far end is
     * there just as well: any reply at all rearms the watchdog through
     * `_dispatch`. Only silence to a direct question is death.
     */
    private _probeLiveness;
    private _clearTimer;
    /** Abandon the current socket: stale callbacks are disarmed by the epoch bump. */
    private _dropSocket;
    onLtp(cb: (e: LtpEvent) => void): () => void;
    onDepth(cb: (symbol: string, exchange: string, depth: MarketDepth) => void): () => void;
    subscribe(mode: WsMode, symbol: string, exchange: string, depthLevel?: number): void;
    unsubscribe(mode: WsMode, symbol: string, exchange: string): void;
    /** Subscribe to real-time order updates (fills / cancels / rejections). Account-level; replayed on reconnect. */
    onOrderUpdate(cb: (e: OrderUpdateEvent) => void): () => void;
    subscribeOrders(): void;
    unsubscribeOrders(): void;
    /**
     * Close intentionally. The instance is reusable afterwards: `connect()` opens
     * a fresh session.
     *
     * Bookkeeping is cleared with the socket because after this call nothing is
     * subscribed, and state that says otherwise is a lie a later reconnect would
     * act on. Event callbacks survive: the caller holds their unsubscribe
     * functions and never asked to drop them.
     */
    close(): void;
    /**
     * Gap check for a numbered stream. Returns false for a duplicate, which is
     * normal right after a resubscribe. Inert when the server sends no sequence,
     * which is the documented OpenAlgo case.
     */
    private _sequenceOk;
    private _dispatch;
}

/**
 * Trade-layer data model (ARCHITECTURE.md §9). Broker-agnostic shapes a broker
 * adapter produces; the chart depends only on these, not on OpenAlgo's REST.
 */

/** Lifecycle states (§9.5): the trade controller reconciles them from book snapshots, and the order engine moves them as it writes. */
type OrderStatus = 'pending' | 'working' | 'partial' | 'filled' | 'cancelled' | 'rejected';
type OrderRole = 'entry' | 'sl' | 'tp';
interface Order {
    id: string;
    symbol: string;
    side: OrderSide;
    type: OrderType;
    qty: number;
    filledQty: number;
    price: number;
    triggerPrice?: number | undefined;
    status: OrderStatus;
    /** Links SL/TP child orders to their position/entry. */
    parentId?: string;
    role?: OrderRole | undefined;
}
interface Position {
    symbol: string;
    /** Net signed quantity: positive = long, negative = short, 0 = flat. */
    netQty: number;
    avgPrice: number;
    /** The account holding it, when the provider says. `AccountManager.positions()` drops a row naming another account. */
    accountId?: string;
}

/**
 * Declarations for the trading operations that came after place, modify and
 * cancel: account reads, order preview, durations, leverage, and the native
 * position commands.
 *
 * The rule here is the opposite of the original capability flags. Those treat
 * an omission as "the legacy route still works", because a provider written
 * before they existed must keep placing orders. Nothing written before this
 * file can carry an account, a duration or a close, so an omission here means
 * unsupported. A host that assumed otherwise would drop the field on the wire
 * and send the order somewhere the user did not choose, or dress an opposite
 * order up as a native close.
 */

/** Time in force for a new order. `GTD` needs an expiry. */
type OrderDuration = 'DAY' | 'IOC' | 'FOK' | 'GTC' | 'GTD';
type TradingFeature = 'accounts' | 'executions' | 'orderHistory' | 'preview' | 'duration' | 'leverage' | 'close' | 'partialClose' | 'reverse' | 'brackets';
/** What a provider declares. Omitted, false and `'unknown'` all refuse. */
interface TradingFeatures {
    /** Account list, snapshots, positions and selection. */
    readonly accounts?: boolean | 'unknown';
    readonly executions?: boolean | 'unknown';
    readonly orderHistory?: boolean | 'unknown';
    readonly preview?: boolean | 'unknown';
    /** Durations accepted on new orders. Omitted: no explicit duration may be sent. */
    readonly durations?: readonly OrderDuration[];
    readonly leverage?: boolean | 'unknown';
    readonly close?: boolean | 'unknown';
    readonly partialClose?: boolean | 'unknown';
    readonly reverse?: boolean | 'unknown';
    /** Brackets whose stop and target legs the provider itself links. */
    readonly brackets?: boolean | 'unknown';
}
interface TradingFeatureRequest {
    readonly feature: TradingFeature;
    readonly symbol?: string | undefined;
    readonly exchange?: string | undefined;
    readonly account?: string | undefined;
    readonly mode?: 'live' | 'analyzer' | undefined;
    readonly type?: OrderType | undefined;
    readonly duration?: OrderDuration | undefined;
}
/** A configured provider returning undefined, throwing or answering late declares nothing. */
type TradingFeatureSource = TradingFeatures | ((request: Readonly<TradingFeatureRequest>) => TradingFeatures | undefined);

/**
 * Pre-trade validation (ARCHITECTURE.md §9.5). Pure guards run client-side so a
 * bad order is blocked and explained rather than round-tripping to a broker
 * rejection. Tick-size rounding, price-band, quantity and freeze-quantity checks.
 *
 * ADVISORY ONLY. The broker RMS is authoritative for every rule here. A check
 * that a user can bypass by editing the call in devtools is a UX affordance, not
 * a risk control: it exists to explain a mistake before the round trip, never to
 * be the thing that stops a bad order. Anything that must actually hold has to
 * live in the feed (see the OpenAlgo compatibility note: the production terminal
 * calls `trade.place` directly and never constructs `OrderEngine`).
 */

interface PriceBand {
    lower: number;
    upper: number;
}
interface OrderConstraints {
    tickSize: number;
    /**
     * Price-dependent ticks. When set it decides every snap and `tickSize` is
     * only read by older code, so give it the schedule's `minMove`. Absent or
     * null, the constant `tickSize` snaps exactly as it always has.
     */
    tickSchedule?: TickSchedule | null;
    priceBand?: PriceBand;
    /** Max quantity per single order (exchange freeze limit). */
    freezeQty?: number;
    /**
     * Contract lot size. When set, quantity must be a whole multiple of it: 100 on
     * a 75-lot future is rejected here rather than by the exchange.
     */
    lotSize?: number;
    /**
     * Set true only for a venue that genuinely trades fractions, such as crypto.
     * Absent means whole quantities, which is every Indian equity and F&O contract
     * and the reason fractional quantities are rejected by default: a fractional
     * quantity reaching those brokers is a defect, not a preference. A `lotSize`
     * grid still applies when this is true.
     */
    allowFractionalQty?: boolean;
}

/**
 * The order engine's public contract (order-engine.ts): the requests a host
 * sends, the receipts and results it gets back, the feed the engine drives,
 * the intents an order moves through, the engine's options, and the two
 * checks that tell what kind of failure a write met. Its own module so the
 * engine stays under the line limit; order-engine.ts re-exports all of it.
 */

interface PlaceRequest {
    symbol: string;
    exchange?: string | undefined;
    side: OrderSide;
    type: OrderType;
    qty: number;
    price?: number | undefined;
    triggerPrice?: number | undefined;
    /** Product: CNC (delivery), NRML (F&O carry), MIS (intraday). Required by OpenAlgo. */
    product?: 'CNC' | 'NRML' | 'MIS' | undefined;
    /** Idempotency token; a retry with the same token is never double-sent. */
    clientToken?: string | undefined;
    /** The account the order is for. Needs the `accounts` feature; never dropped on the way to the wire. */
    account?: string;
    /** Time in force. Omitted leaves the provider's own default. Needs the provider to list it. */
    duration?: OrderDuration;
    /** When a `GTD` order lapses, UTC seconds. Only with `GTD`. */
    expiresAt?: number;
    /** Margin multiplier to request. Needs the `leverage` feature. */
    leverage?: number;
}
/** What a provider says an order would cost before it is placed. Absent fields were not reported. */
interface OrderPreview {
    readonly accountId?: string;
    readonly estimatedPrice?: number;
    readonly estimatedValue?: number;
    readonly marginRequired?: number;
    readonly marginAvailableAfter?: number;
    readonly fees?: number;
    readonly currency?: string;
    readonly warnings?: readonly string[];
    /** Set when the provider would refuse the order, and why. */
    readonly rejectReason?: string;
    /** UTC seconds. */
    readonly asOf?: number;
}
/** Identifies the position a command acts on. */
interface PositionCommandRequest {
    symbol: string;
    exchange?: string;
    product?: 'CNC' | 'NRML' | 'MIS';
    account?: string;
    /** Idempotency token for this command alone. */
    clientToken?: string;
}
interface ClosePositionRequest extends PositionCommandRequest {
    /** How much to close. Omitted closes the whole position; a quantity is a partial close. */
    qty?: number;
}
type ReversePositionRequest = PositionCommandRequest;
/** An entry whose stop and target legs the provider places and links itself. */
interface BracketOrderRequest extends PlaceRequest {
    /** Trigger of the protective stop leg. */
    stopLoss: number;
    /** Limit price of the target leg. */
    takeProfit: number;
}
/** The broker's handle on a position command. `commandId` is what its order stream reports on. */
interface CommandReceipt {
    commandId: string;
    orderIds?: readonly string[];
}
interface BracketReceipt {
    orderId: string;
    stopLossId?: string;
    takeProfitId?: string;
}
/** Fields a modify may change. Whole-order feeds fill the rest from their cache. */
interface ModifyPatch {
    price?: number;
    triggerPrice?: number;
    qty?: number;
}
/**
 * The minimal broker write interface the `OrderEngine` drives: place / modify /
 * cancel. `OpenAlgoTradeFeed` implements this. Distinct from the base-tier
 * `TradeFeed` (a higher-level place + subscribe shape in `openalgo-charts`):
 * implement `OrderFeed` for the engine's write path.
 */
interface OrderFeed {
    /** Optional support declaration; a configured provider can report unavailable metadata. */
    readonly capabilities?: TradingCapabilitySource | undefined;
    /** Declares preview, durations, leverage, accounts and the position commands. Omitted declares none. */
    readonly features?: TradingFeatureSource | undefined;
    place(req: PlaceRequest & {
        mode: TradeMode;
    }): Promise<{
        orderId: string;
    }>;
    modify(orderId: string, patch: ModifyPatch): Promise<void>;
    cancel(orderId: string): Promise<void>;
    /** Read-only: what the order would cost. Must not place anything. */
    previewOrder?(req: PlaceRequest & {
        mode: TradeMode;
    }): Promise<OrderPreview>;
    closePosition?(req: ClosePositionRequest & {
        mode: TradeMode;
    }): Promise<CommandReceipt>;
    reversePosition?(req: ReversePositionRequest & {
        mode: TradeMode;
    }): Promise<CommandReceipt>;
    /**
     * `legClientTokens` are the client tokens the engine gives the stop and
     * target legs. A provider that echoes them on the legs' rows lets a bracket
     * whose answer was lost be reconciled leg by leg.
     */
    placeBracket?(req: BracketOrderRequest & {
        mode: TradeMode;
        legClientTokens: {
            stopLoss: string;
            takeProfit: string;
        };
    }): Promise<BracketReceipt>;
}
type TradeMode = 'live' | 'analyzer';

/**
 * OpenAlgo trade adapter (ARCHITECTURE.md §10.0). Implements the order engine's
 * OrderFeed over OpenAlgo REST (`/api/v1/placeorder`, `/modifyorder`,
 * `/cancelorder`) plus orderbook/positionbook fetches for reconciliation.
 *
 * Payloads match the local OpenAlgo docs: `placeorder` requires
 * strategy/symbol/action/exchange/pricetype/product/quantity; `modifyorder`
 * additionally requires the full order context, so we cache each order's context
 * (from place + the order book) and merge the patch on modify. Book responses
 * return string quantities/prices, which are decoded to numbers. Fetch is
 * injectable for offline tests; verify field names against your OpenAlgo build.
 *
 * Two guarantees live HERE rather than in `OrderEngine`, because the largest
 * consumer drives this class directly and never constructs the engine:
 *
 * - **Reading the book fails closed.** An unmappable enum or an unreadable
 *   number never becomes a tradable order. See `decodeOrder`.
 * - **The analyzer/live mode is checked, not claimed.** See `getServerMode`.
 */

/** How hard `place` checks the server's mode against the caller's. */
type ModeCheck = 'off' | 'auto' | 'always';
interface OpenAlgoTradeConfig {
    baseUrl: string;
    apiKey: string;
    /** Optional host/broker restrictions; omitted preserves the existing OpenAlgo path. */
    capabilities?: TradingCapabilitySource;
    /**
     * Instrument constraints for the advisory pre-trade check on `place`, looked
     * up per order. Return undefined when the instrument is unknown; the
     * universal checks below still apply.
     *
     * Optional because most of the value needs no configuration: a quantity that
     * is NaN, negative or fractional is wrong for every Indian instrument and is
     * rejected without knowing anything about the symbol. Supply this to add the
     * freeze limit and the lot grid, which do need instrument data.
     */
    constraints?: (symbol: string, exchange: string) => OrderConstraints | undefined;
    /**
     * Turn off the feed-level duplicate guard. Default false, i.e. the guard is
     * on. Only set true if a layer above already owns idempotency AND you have
     * read why that is usually the wrong call: see `_claimToken`.
     */
    disableIdempotency?: boolean;
    /** Strategy label sent with orders (OpenAlgo groups by strategy). */
    strategy?: string;
    /** Default product when a request doesn't specify one. */
    defaultProduct?: 'CNC' | 'NRML' | 'MIS';
    fetchImpl?: typeof fetch;
    /**
     * Mode guard on `place`. Default `'auto'`, which is described in full on
     * `_assertMode`. `'always'` checks both directions and refuses when the
     * server cannot be asked; `'off'` restores the pre-1.6 behaviour of trusting
     * the caller's `mode` outright.
     */
    verifyMode?: ModeCheck;
    /** How long a server-mode reading stays usable, in ms. Default 5000. */
    modeCacheMs?: number;
    /**
     * Called after an order book fetch that had to quarantine rows. Fail-closed
     * parsing drops nothing silently: a host that shows the book must be able to
     * tell the trader that part of it could not be read.
     */
    onDecodeIssue?: (snapshot: OrderBookSnapshot) => void;
    /** Clock, injectable so the mode cache is testable without real timers. */
    now?: () => number;
}
declare class OpenAlgoTradeFeed implements OrderFeed {
    private readonly _config;
    private readonly _fetch;
    private readonly _strategy;
    private readonly _defaultProduct;
    private readonly _ctx;
    private readonly _modifies;
    private _bookRead;
    /**
     * Client order ids this feed has put on the wire, and whether the outcome is
     * known. A claimed-but-unresolved entry blocks a repeat. See `_claimToken`.
     */
    private readonly _claimed;
    private readonly _verifyMode;
    private readonly _modeCacheMs;
    private readonly _now;
    private _mode?;
    private _modeAt;
    constructor(config: OpenAlgoTradeConfig);
    private _post;
    /**
     * Read the analyzer mode the SERVER is in.
     *
     * Analyzer mode is a server-side global in OpenAlgo: `get_analyze_mode()`
     * decides where every order goes, and it is flipped at
     * `POST /api/v1/analyzer/toggle`. It is not a per-order flag, there is no
     * mode field on the placeorder payload, and one added there would be read by
     * nothing. So this client never asserts a mode. It asks for the server's,
     * and `place` refuses when the answer disagrees with the caller's belief.
     *
     * Answers younger than `maxAgeMs` are reused, so a host that polls the books
     * (whose sandbox responses carry the mode already) never pays for this call.
     */
    getServerMode(maxAgeMs?: number): Promise<TradeMode>;
    /** The last server-mode reading, if one is still within `maxAgeMs`. */
    serverMode(maxAgeMs?: number): TradeMode | undefined;
    private _freshMode;
    /**
     * Refuse to send when the server's mode contradicts the caller's.
     *
     * The default is asymmetric because the two mistakes do not cost the same.
     * Believing you are on the sandbox while the server is live spends real
     * money, so `analyzer` demands a positive answer and probes for one when the
     * cache is cold. Believing you are live while the server is sandboxed is bad
     * but costs nothing, so `live` uses only what the cache already knows and
     * never adds a round trip to a real order. `verifyMode: 'always'` probes in
     * both directions.
     *
     * A caller that passes no mode at all is not checked: it has claimed nothing,
     * so there is nothing to contradict.
     */
    private _assertMode;
    /**
     * Advisory pre-trade quantity check, applied to EVERY order type.
     *
     * Advisory is the operative word: a user with devtools can call the broker
     * directly, so this cannot be a risk control. It is here because the broker's
     * RMS rejection arrives after a round trip and reads like a server error,
     * while this reads like the mistake it is, immediately.
     *
     * It lives in the feed rather than only in `OrderEngine` because the engine is
     * skippable. OpenAlgo's own terminal calls `place` directly and never
     * constructs an engine, so a guarantee reachable only through the engine is
     * not a guarantee for the largest consumer of this library.
     */
    private _assertQuantity;
    /**
     * Refuse a client order id this feed has already put on the wire.
     *
     * The costs are not symmetric, which is the whole argument. Holding a claim
     * too long costs one deliberate click after a banner. Releasing it too early
     * costs a doubled live position that nobody asked for, discovered later, quite
     * possibly on a leveraged intraday product.
     *
     * So a claim is released ONLY when the request provably never left. A non-ok
     * HTTP response, a timeout, an aborted socket: all of those mean the response
     * did not arrive, and say nothing whatsoever about whether the order did. Those
     * stay claimed and are reported as ambiguous.
     *
     * Enforcement note, because the boundary matters: this stops THIS FEED sending
     * the same id twice. It cannot make the broker deduplicate. Two browser tabs
     * are two feeds and two ledgers, and OpenAlgo's placeorder carries no client
     * order id field, so end-to-end idempotency is not available on this wire.
     * What is available, and what this delivers, is that a double-clicked button
     * or a retried promise does not become two orders.
     */
    private _claimToken;
    get capabilities(): TradingCapabilitySource | undefined;
    /**
     * Place an order through OpenAlgo's placeorder route.
     *
     * An order without `exchange` goes to `NSE`, and one without `product` takes
     * the feed's `defaultProduct`. Both defaults are documented and hosts place
     * through them, so they stay through 2.x. `modify()` is stricter on purpose:
     * it never guesses an exchange or product for an order the book does not
     * describe, since a guess there moves a live order somewhere its owner did
     * not put it. A host that trades on any other exchange passes `exchange`
     * with every order.
     *
     * Every refusal before the request is sent is a pre-flight error, so the
     * caller can correct and retry knowing nothing reached the broker.
     */
    place(request: PlaceRequest & {
        mode: TradeMode;
    }): Promise<{
        orderId: string;
    }>;
    /**
     * Whether a client order id is known to have reached the broker.
     *
     * `'ambiguous'` is the one worth handling: the request left and the outcome is
     * unknown, so the order may be live. A host should say so and offer the order
     * book, not a retry button.
     */
    tokenState(token: string): 'unknown' | 'inflight' | 'sent' | 'ambiguous';
    /**
     * Release a claim after the host has established what really happened, for
     * instance by finding no such order in a freshly fetched book.
     *
     * Deliberately manual. Nothing in this library can safely decide on its own
     * that an ambiguous order did not reach the exchange.
     */
    releaseToken(token: string): void;
    modify(orderId: string, changes: {
        price?: number;
        triggerPrice?: number;
        qty?: number;
    }): Promise<void>;
    private _modify;
    cancel(orderId: string): Promise<void>;
    /**
     * Fetch the order book, keeping the rows that could not be read.
     *
     * `getOrders` returns the first half of this. A host that renders the book
     * needs the second half too: "2 rows from the broker could not be read" is a
     * thing the trader must see, and a quietly shorter list is not.
     */
    getOrderBook(): Promise<OrderBookSnapshot>;
    /** Fetch the order book for reconciliation (maps to broker-agnostic orders). */
    getOrders(): Promise<DecodedOrder[]>;
    /** Fetch the position book for reconciliation. */
    getPositions(): Promise<Position[]>;
}
interface RawOrder {
    orderid?: string;
    symbol?: string;
    exchange?: string;
    action?: string;
    pricetype?: string;
    product?: string;
    quantity?: number | string;
    filled_quantity?: number | string;
    price?: number | string;
    trigger_price?: number | string;
    order_status?: string;
}
interface RawPosition {
    symbol?: string;
    quantity?: number | string;
    average_price?: number | string;
}
type OrderDecodeCode = 'MISSING' | 'NON_FINITE' | 'RANGE' | 'UNKNOWN_ENUM';
interface OrderDecodeIssue {
    /** Where it went wrong, e.g. `orders[3].action`. */
    path: string;
    code: OrderDecodeCode;
    /** What a readable value looks like, in words the trader can act on. */
    expected: string;
    /** The offending value, stringified and clipped. Never the whole payload. */
    got: string;
}
/** A book row that had no honest order form, kept for diagnosis. */
interface QuarantinedRow {
    issue: OrderDecodeIssue;
    raw: RawOrder;
}
interface OrderBookSnapshot {
    /** Rows that decoded, including any whose status is `unknown`. */
    orders: DecodedOrder[];
    quarantined: QuarantinedRow[];
}
/**
 * An order read from the book.
 *
 * Identical to `Order` except that `status` may be `unknown`: a broker status
 * this adapter cannot map is reported as unknown rather than asserted to be
 * working. Unknown is not terminal, so a later snapshot can still rescue the
 * row, and every consumer that gates on `status === 'working'` treats it as
 * untradable, which is the point.
 *
 * The member is widened here rather than in `OrderStatus` because widening the
 * shared union is a change to `src/trade/types.ts`, deferred to the v2 contract
 * work that owns that file.
 */
interface DecodedOrder extends Omit<Order, 'status'> {
    status: OrderStatus | 'unknown';
    /** The broker's own status text, kept verbatim whenever `status` is unknown. */
    rawStatus?: string;
}
type OrderDecodeResult = {
    ok: true;
    order: DecodedOrder;
} | {
    ok: false;
    issue: OrderDecodeIssue;
};
/**
 * Map a lowercase OpenAlgo order_status to the chart's status.
 *
 * An unrecognised status reads `unknown`. It used to read `working`, which
 * asserted that an order nobody could classify was live in the book and
 * draggable on the chart. OpenAlgo's own broker mappings emit the literal
 * string `unknown` when a broker sends a state they do not recognise, so this
 * default was reachable in production, not only from a malformed payload.
 */
declare function mapOrderStatus(s: string): OrderStatus | 'unknown';
/**
 * Decode one order book row, failing closed.
 *
 * The split follows what the field is for. Identity-bearing fields (id, symbol,
 * side, order type, quantity, prices) either read cleanly or the row is
 * quarantined: there is no substitute value that is safe to trade against, and
 * the old `action === 'SELL' ? 'SELL' : 'BUY'` turned a typo, a null and an
 * unrecognised broker enum alike into a BUY.
 *
 * Status is descriptive, so an unmappable one degrades to `unknown` and the row
 * is KEPT. Hiding an order that exists is worse than showing one nobody can
 * classify, which is the opposite trade-off from the identity fields and is
 * deliberate.
 */
declare function decodeOrder(r: RawOrder, path?: string): OrderDecodeResult;
/**
 * Total row mapper, kept for the published surface.
 *
 * Prefer `decodeOrder`, which says WHY a row could not be read instead of
 * handing back a placeholder. A row that fails an identity check has no honest
 * `Order` form, because `OrderSide` has no unknown member and so the
 * placeholder cannot say "side unreadable". It marks the whole record instead,
 * with `status: 'unknown'`, which is inert in every consumer: they all gate on
 * `status === 'working'` before drawing or acting on a row. The broker's own
 * word is passed through rather than replaced, because showing `FOO` is honest
 * and showing `BUY` is exactly the fail-open this replaced.
 *
 * `getOrders` does not use this: it quarantines such rows outright.
 *
 * @deprecated Removed in 3.0.0. Use {@link decodeOrder} (since 1.6.0), which
 * returns the reason a row could not be read, or
 * {@link OpenAlgoTradeFeed.getOrderBook}, which sets such rows aside as
 * `quarantined`.
 */
declare function mapOrder(r: RawOrder): DecodedOrder;
declare function mapPosition(r: RawPosition): Position;

/**
 * Composed OpenAlgo live data feed. Implements the full
 * `DataFeed` contract by combining history (REST), live ticks (WS), and a
 * per-subscription aggregator, so `subscribeBars()` actually delivers live
 * interval bars instead of being a no-op trap.
 */

interface OpenAlgoLiveConfig extends OpenAlgoConfig {
    /** WS proxy URL, e.g. ws://127.0.0.1:8765. */
    wsUrl: string;
    /**
     * Volume accounting for the live candle builder (default 'ltq-sum').
     *
     * 'day-delta' also selects the wire mode: a cumulative day volume only exists
     * on the Quote stream, so this feed subscribes Quote rather than LTP for it.
     */
    volumeMode?: VolumeMode;
    /**
     * Zone calendar intervals bucket in (default IST). Should be the chart's
     * configured timezone: a monthly bar opens at local midnight on the first,
     * and that instant differs by exchange.
     */
    timezone?: string;
    /**
     * Default book depth for `subscribeDepth`, broker-dependent (5/20/30/50).
     * Omitted leaves it to the broker, which is what this feed always did.
     * A per-call `opts.depthLevel` overrides it.
     */
    depthLevel?: number;
    /** Inject a custom socket (tests, React Native, non-browser runtimes). */
    socketFactory?: SocketFactory;
    /**
     * Socket policy, passed straight to `OpenAlgoWsFeed`. Present because the
     * handshake gate, the liveness watchdog and the backoff all have escape
     * hatches that were otherwise unreachable through this composed feed: a proxy
     * build that answers nothing needs `auth: { requireAck: false }`, and a host
     * with no way to say so would just see a chart that never receives a tick.
     */
    reconnect?: OpenAlgoWsConfig['reconnect'];
    auth?: OpenAlgoWsConfig['auth'];
    heartbeat?: OpenAlgoWsConfig['heartbeat'];
}
/**
 * Seconds per bar for a fixed-length interval code, e.g. `D` -> 86400,
 * `1m` -> 60, `1h` -> 3600.
 *
 * Throws `UnknownIntervalError` for a code nothing recognises, and a plain
 * error for a calendar or count-driven code, which has no seconds to give.
 * It used to answer 60 for both, which drew minute bars under whatever label
 * the caller thought it had asked for.
 *
 * Prefer `resolveInterval()` and `bucketStartOf()`: they cover every registered
 * code, not just the ones that happen to have a fixed length.
 */
declare function intervalToSeconds(interval: string): number;
declare class OpenAlgoLiveDataFeed implements DataFeed {
    private readonly _rest;
    private readonly _ws;
    private readonly _volumeMode;
    private readonly _timezone;
    private readonly _depthLevel;
    /** Wire mode ticks arrive on, decided once by `volumeMode`. */
    private readonly _tickMode;
    /** Reference counts per `mode:symbol:exchange`, so one consumer leaving cannot cut the rest off. */
    private readonly _subs;
    constructor(config: OpenAlgoLiveConfig);
    /**
     * Take a share in the remote subscription for this stream, subscribing only
     * if nobody held one, and hand back a release that unsubscribes remotely only
     * when the last holder lets go.
     *
     * Depth is negotiated upward: a consumer needing 20 levels re-subscribes
     * at 20 rather than living with the 5 an earlier one asked for. It is never
     * negotiated back down while consumers remain, because shrinking a book under
     * a consumer that is still reading it is the failure this method exists to
     * prevent, and the extra levels cost only bandwidth. The high-water mark is
     * per entry, so it resets when the last holder leaves.
     */
    private _acquire;
    getBars(req: BarsRequest): Promise<Bar[]>;
    searchSymbols(request: SymbolSearchRequest): Promise<SymbolMatch[]>;
    /**
     * Live bars: WS tick -> aggregator -> onBar (mutated/append bar). The tick
     * stream is LTP, or Quote when `volumeMode` is 'day-delta' and the bar
     * therefore needs a cumulative day volume.
     *
     * Fixed intervals go through `CandleBuilder`, which carries the late-tick
     * policy; calendar, tick-count and volume intervals go through
     * `TickBarAggregator`, which is the one that knows those boundaries.
     *
     * Pass `opts.seedFrom` (the last history bar) to continue that bar's bucket
     * seamlessly instead of starting a fresh one, and `opts.cumDayVolumeSoFar` so
     * a `day-delta` builder diffs against the right baseline. Seeding applies to
     * time-bucketed intervals: a count-driven bar cannot resume a historical one.
     */
    subscribeBars(req: BarsRequest, onBar: (bar: Bar, meta?: LiveBarMeta) => void, opts?: BarSubscriptionOptions): UnsubscribeFn;
    /** A broker may omit the tick timestamp; never bucket at the epoch, use now. */
    private static _tickTime;
    private _candleReader;
    private _aggregatorReader;
    /**
     * Live book. `opts.depthLevel` requests a book depth (broker-dependent:
     * 5/20/30/50), falling back to the feed's configured default and then to
     * whatever the broker sends unasked, which is what this method always did.
     * The socket has accepted a depth level all along; only the composed feed had
     * no way to name one.
     */
    subscribeDepth(req: BarsRequest, onDepth: (depth: MarketDepth) => void, opts?: {
        depthLevel?: number;
    }): UnsubscribeFn;
    close(): void;
}

/** Schedules a repeating callback and returns an unsubscribe. Inject in tests. */
type FeedScheduler = (cb: () => void, intervalMs: number) => UnsubscribeFn;
/**
 * Deterministic, in-memory data feed for tests and demos. No network.
 * Generates a reproducible synthetic OHLC walk from a fixed seed so that
 * pixel-diff and unit tests are stable across runs (no Math.random / Date.now).
 * It serves only its default series: a request naming another data variant is
 * refused with a `DataVariantUnsupportedError`.
 */
declare class FakeDataFeed implements DataFeed {
    private readonly intervalSec;
    private readonly _schedule;
    constructor(intervalSec?: number, scheduler?: FeedScheduler);
    getBars(req: BarsRequest): Promise<Bar[]>;
    /**
     * Emit deterministic synthetic bars on the scheduler (default: a 1s
     * setInterval). This actually streams, so feature detection
     * (`if (feed.subscribeBars)`) is honest. Pass a manual scheduler to drive it
     * by hand in tests, and `opts.tickMs` to change the cadence.
     */
    subscribeBars(req: BarsRequest, onBar: (bar: Bar) => void, opts?: BarSubscriptionOptions & {
        tickMs?: number;
    }): UnsubscribeFn;
}
/** Pure, seeded synthetic bar generator (deterministic: no global randomness). */
declare function generateBars(startTime: number, count: number, intervalSec: number): Bar[];

/** Fixed-length bars: `seconds` each, aligned to `anchorSec` (default epoch). */
interface IntervalBucketing {
    mode: 'interval';
    seconds: number;
    anchorSec?: number;
}
/** A bar closes after `count` trades. */
interface TickCountBucketing {
    mode: 'ticks';
    count: number;
}
/** A bar closes after `perBar` traded quantity. */
interface VolumeBucketing {
    mode: 'volume';
    perBar: number;
}
/** Calendar periods, which are not a fixed number of seconds. */
type CalendarUnit = 'month' | 'quarter' | 'year';
/**
 * A bar spans `count` calendar units, opening at local midnight on the first
 * day of the period in `timezone`.
 *
 * `timezone` on the entry pins the code to one exchange's calendar; leaving it
 * off lets the bucket follow whatever zone the caller resolves in, which is the
 * chart's configured zone.
 */
interface CalendarBucketing {
    mode: 'calendar';
    unit: CalendarUnit;
    /** Periods per bar (default 1): `{ unit: 'month', count: 6 }` is a half-year. */
    count?: number;
    timezone?: string;
}
/** Every way this engine knows how to close a bar. */
type Bucketing = IntervalBucketing | TickCountBucketing | VolumeBucketing | CalendarBucketing;
/**
 * The three time-agnostic modes `TickBarAggregator` shipped with. Kept as a
 * distinct name because `FootprintAggregator` handles exactly these three.
 */
type TickTimeframe = IntervalBucketing | TickCountBucketing | VolumeBucketing;
/** A registered interval code and the rule that buckets it. */
interface IntervalDescriptor {
    /** The code a host passes as `BarsRequest.interval`, e.g. `5m`, `MN`, `T500`. */
    code: string;
    bucketing: Bucketing;
}
/** Thrown when a code matches neither a registered entry nor a built-in token. */
declare class UnknownIntervalError extends Error {
    readonly code: string;
    constructor(code: string);
}
/**
 * Register an interval code. Returns a disposer, so a host that adds codes for
 * one instrument can take them away again.
 *
 * A registered code shadows a built-in token of the same name: a host whose
 * `W` means "calendar week in the exchange's zone" rather than "604800 seconds
 * from the epoch" is entitled to say so.
 */
declare function registerInterval(descriptor: IntervalDescriptor): () => void;
/** Remove a registered code. Returns false if nothing was registered under it. */
declare function unregisterInterval(code: string): boolean;
/** Every registered code, in registration order. Built-in tokens are not listed. */
declare function registeredIntervals(): readonly IntervalDescriptor[];
/** Resolve a code, or null when nothing recognises it. */
declare function tryResolveInterval(code: string): IntervalDescriptor | null;
/** True when `code` resolves. For a host validating an interval picker's input. */
declare function isKnownInterval(code: string): boolean;
/**
 * Resolve a code, throwing `UnknownIntervalError` if nothing recognises it.
 *
 * Loud on purpose. The previous behaviour was a silent fall back to 60 seconds,
 * so a typo drew minute bars under a label claiming something else, and nothing
 * anywhere said so. A chart that refuses to open is a bug report; a chart
 * showing the wrong timeframe is a wrong trade.
 */
declare function resolveInterval(code: string): IntervalDescriptor;
/** True when bars close on the clock or the calendar rather than on trade flow. */
declare function isTimeBucketed(b: Bucketing): boolean;
/**
 * UTC seconds at which the bar containing `timeSec` opened.
 *
 * `zone` is the fallback for calendar buckets that did not pin their own, and
 * should be the chart's configured timezone. Count-driven bars have no time
 * bucket at all, so their bar simply opens at the tick that started it.
 */
declare function bucketStartOf(b: Bucketing, timeSec: number, zone?: string): number;
/**
 * UTC seconds at which the next bar opens, or null for count-driven bars, whose
 * close depends on trade flow rather than on the clock.
 *
 * The gap between this and `bucketStartOf` is the bar's real length: 28, 29, 30
 * or 31 days for a month, and an hour short of that across a spring forward.
 */
declare function nextBucketStart(b: Bucketing, timeSec: number, zone?: string): number | null;
/**
 * How a bar's length reads to a human: a count, and the unit it counts.
 *
 * `M` is months, so a quarter reads as 3 and a year as 12, keeping the token
 * grammar's rule that lower-case `m` is minutes and upper-case `M` is a month.
 * `tick` counts trades. `other` is a bucket with neither a clock length nor a
 * trade count, which today means volume, and is where a later count-driven
 * mode can land without silently changing what an existing unit means.
 */
interface IntervalParts {
    multiplier: number;
    unit: 's' | 'm' | 'h' | 'D' | 'W' | 'M' | 'tick' | 'other';
}
/**
 * Split a code into a count and a unit, or null when nothing recognises it.
 * For an indicator that knows its own interval and wants to reason about it,
 * rather than hard-coding the handful of codes its author happened to test on.
 *
 * Read off the bucketing rule, not off the code's spelling, so it answers for
 * a registered code the built-in grammar never parsed, and it answers
 * canonically: `120m` and `2h` are the same bar and both read as 2 h. The
 * coarsest unit that divides the length wins, which is also why a 90-second
 * bar reads as 90 s rather than as one and a half minutes.
 */
declare function intervalParts(code: string): IntervalParts | null;
/**
 * True when the bar is not a whole number of minutes long, so its labels need
 * second precision. `1s`, `15s` and `90s` all qualify; `5m` does not.
 */
declare function isSecondsInterval(code: string): boolean;
/**
 * True for a fixed-length bar shorter than a day: the ones that want a session
 * reset, such as a VWAP anchored to the day's open or an opening range.
 *
 * A calendar period is false because it is coarser, and a count-driven bar is
 * false because it has no clock length to compare, not because it is known to
 * be long. Ask `isTickInterval` or `intervalParts` for those.
 */
declare function isIntradayInterval(code: string): boolean;
/**
 * True for a bar exactly one day long, whether spelled `D`, `24h` or `1440m`.
 *
 * Deliberately not "daily or coarser": weekly and monthly answer false, so a
 * caller can branch on intraday, daily and coarser as three independent
 * questions instead of an ordered ladder where the wrong order swallows a case.
 */
declare function isDailyInterval(code: string): boolean;
/**
 * True for a bar that closes after N trades. Volume bars close on quantity, not
 * on trade count, so they answer false; `intervalParts` separates the two.
 */
declare function isTickInterval(code: string): boolean;

/**
 * Tick aggregation (ARCHITECTURE.md 10.2). Aggregates raw trade ticks into
 * bars on a chosen bucketing: clock interval, calendar period, tick count, or
 * traded volume. Incremental and deterministic, so it works live and is
 * unit-testable.
 *
 * This is the foundation for tick / volume timeframes and (with classified
 * bid/ask ticks) for the footprint aggregator. Tick-count and volume bars need
 * real trade ticks: OHLCV alone can't produce them.
 *
 * The bucketing rules themselves live in `./intervals`, which is also where a
 * host registers its own interval codes; this class just applies one.
 */

interface AggTick {
    time: number;
    price: number;
    qty: number;
    /**
     * Open interest as at this tick, where the feed carries it. Carried onto the
     * bar as the latest reading rather than accumulated: see `Bar.oi`.
     */
    oi?: number;
}
interface BarUpdate {
    bar: Bar;
    isNew: boolean;
}
interface TickBarOptions {
    /**
     * Zone a calendar bucket resolves in when the bucketing did not pin its own.
     * Pass the chart's configured timezone: a month boundary is local midnight on
     * the first, and which instant that is depends on where the exchange is.
     */
    timezone?: string | undefined;
}
declare class TickBarAggregator {
    private readonly _tf;
    private readonly _zone;
    private _cur;
    private _count;
    constructor(tf: Bucketing, options?: TickBarOptions);
    current(): Bar | null;
    /**
     * Continue a historical bar instead of opening a fresh one on the first tick.
     * Time-bucketed modes only: a count-driven bar cannot be resumed, because the
     * trades that filled it were never counted here.
     */
    seed(lastBar: Bar): void;
    onTick(tick: AggTick): BarUpdate;
}

/**
 * Warm-load bar cache: a wrapper around ANY `DataFeed`, so a custom feed gets
 * the same behaviour as `OpenAlgoDataFeed`.
 *
 *     const feed = withBarCache(new OpenAlgoDataFeed(cfg), { ttlMs: 60_000 });
 *
 * The design decisions, all of which are load-bearing:
 *
 * **Key.** `symbol | exchange | interval`, then the data variant when the
 * request names one (extended hours, raw prices): each is a separate provider
 * series, and the default variant adds nothing, so keys written before
 * variants existed still match. The requested range is deliberately
 * NOT part of the key: one entry per series holds the widest set fetched so
 * far, and a narrower request is served by slicing it. Keying on the range
 * would miss on every pan and on every "same chart, one bar later" reload,
 * which is exactly the traffic warm-load is meant to remove.
 *
 * **Freshness.** Two independent gates, both of which must pass:
 *   1. `ttlMs` bounds absolute age.
 *   2. Nothing new can have closed. An entry is complete through `to`; the next
 *      bar closes at `to + 1 + intervalSec`, measured on the feed's own bar
 *      grid rather than on UTC midnight, so a daily Indian bar opening at 03:45
 *      UTC is judged against its own session, not against the wrong boundary.
 * The effect is what you want from a warm cache: a closed session stays usable
 * for the whole TTL, while a 1m chart is only reused inside the current minute.
 *
 * **The forming bar is never cached.** A bar whose close time is still in the
 * future keeps moving, and serving yesterday's snapshot of it to a live chart
 * is worse than not caching at all: the chart would paint a frozen candle and
 * have no way to know. So the trailing forming bar is dropped on store, and
 * coverage ends at the last CLOSED bar. A cache hit can therefore be short by
 * at most one bar, the one a live subscription re-supplies immediately, and is
 * never wrong about a bar it does return.
 *
 * **Bounds.** Capped on both entry count (`max`) and total cached bars
 * (`maxBars`), LRU-evicted. Entries alone do not bound memory (one intraday
 * series can be 100k bars); bar count is the honest proxy for bytes that can be
 * measured without serialising. Byte counting would mean stringifying every
 * entry on every write, which costs more than the cache saves.
 *
 * **Storage.** A bounded in-memory copy is always available. A host may inject
 * `storage` to persist through localStorage, IndexedDB, or another store, but
 * the engine will not choose one itself. Durable reads, writes, and deletes are
 * best effort so storage denial or quota exhaustion cannot block market data.
 * Store methods may return a promise. `CachedBars` is plain JSON so it
 * round-trips through `JSON.stringify` unchanged.
 *
 * **Opt-out.** `getBars({ ..., noCache: true })` always hits the network (and
 * refreshes the entry); `invalidate()` and `clear()` drop entries by hand.
 */

type MaybePromise<T> = T | Promise<T>;
/** One cached series. Plain JSON: safe to persist as-is. */
interface CachedBars {
    /** Persisted schema version. Missing means the compatible legacy schema. */
    version?: number;
    /** Closed bars only, ascending by time. */
    bars: Bar[];
    /** Coverage start: the `from` of the request that filled this entry. */
    from: UTCSeconds;
    /** Coverage end (inclusive): the last instant this entry is complete to. */
    to: UTCSeconds;
    /** Wall clock (ms) at store time, for TTL. */
    storedAt: number;
    /**
     * When the bar AFTER this entry's coverage closes, so freshness needs no
     * re-resolve. Null is impossible here: an entry whose bars have no knowable
     * close is never stored in the first place.
     */
    nextClose: UTCSeconds;
}
/**
 * Pluggable durable store. Sync or async operations are awaited and isolated
 * from the network result. The cache always keeps its own bounded memory copy.
 */
interface BarCacheStore {
    get(key: string): MaybePromise<CachedBars | undefined>;
    set(key: string, value: CachedBars): MaybePromise<void>;
    delete(key: string): MaybePromise<void>;
}
interface BarCacheOptions {
    /** Absolute age bound, ms. Default 5 minutes. */
    ttlMs?: number;
    /** Maximum entries before LRU eviction. Default 24. */
    max?: number;
    /** Maximum total cached bars before LRU eviction. Default 250_000. */
    maxBars?: number;
    /** Optional durable store. Bounded in-memory retention is always enabled. */
    storage?: BarCacheStore;
    /** Injectable clock (ms), for tests and for hosts with a server clock. */
    now?: () => number;
    /**
     * Override how the cache decides when a bar closes. Return null for "unknown",
     * which makes the cache refuse to store the series rather than guess. Defaults
     * to {@link barCloseSec}, which asks the interval registry.
     */
    barCloses?: (interval: string, barStartSec: UTCSeconds) => number | null;
    /** IANA zone for calendar intervals. Defaults to the engine default. */
    timezone?: string;
}
/** A `BarsRequest` that can force a fresh fetch. */
interface CachedBarsRequest extends BarsRequest {
    /** Skip the cached entry, fetch, and replace it with the fresh result. */
    noCache?: boolean;
}
/** Counts what this instance is tracking, not what a persistent store holds. */
interface BarCacheStats {
    entries: number;
    bars: number;
    hits: number;
    misses: number;
    evictions: number;
}
/** Current persisted entry schema. Legacy entries without this field remain readable. */
declare const BAR_CACHE_VERSION = 1;
/**
 * The instant a bar starting at `barStartSec` closes, or null when that cannot
 * be known from the interval alone.
 *
 * This asks the interval registry rather than parsing the token itself. A second
 * parser here was the bug: it matched a fixed set of letter codes and returned
 * 60 seconds for anything else, so a host that registered its own code got a
 * cache that believed a new bar closed every minute. A tick-count series keyed
 * that way is served stale for up to a minute at a time, and a registered
 * calendar code was approximated at 30 days.
 *
 * Null means "no fixed close", and it is returned for two genuinely different
 * situations that both demand the same conservative answer:
 *
 *  - **tick and volume bars**, which close on trade flow. A 500-tick bar may run
 *    for a second or an hour, so nothing about elapsed time says whether the
 *    last bar is complete.
 *  - **an unregistered code.** Guessing 60 seconds is how the old parser turned
 *    a typo into a silently wrong cache.
 *
 * The caller must treat null as "cannot cache and cannot serve past coverage",
 * which is the safe direction: it refetches rather than serving something stale.
 */
declare function barCloseSec(interval: string, barStartSec: UTCSeconds, zone?: string): number | null;
declare function barCacheKey(req: BarsRequest): string;
declare class BarCache implements DataFeed {
    /** The wrapped feed, for callers that need something this wrapper does not forward. */
    readonly source: DataFeed;
    private readonly _backing;
    private readonly _memory;
    /** Keys whose durable value may still exist after a failed write or delete. */
    private readonly _tombstones;
    private readonly _writes;
    private readonly _ttlMs;
    private readonly _max;
    private readonly _maxBars;
    private readonly _now;
    private readonly _barCloses;
    private readonly _index;
    private _tick;
    private _hits;
    private _misses;
    private _evictions;
    constructor(feed: DataFeed, options?: BarCacheOptions);
    subscribeBars?: (req: BarsRequest, onBar: (bar: Bar, meta?: LiveBarMeta) => void, ...rest: unknown[]) => UnsubscribeFn;
    subscribeDepth?: (req: BarsRequest, onDepth: (depth: MarketDepth) => void, ...rest: unknown[]) => UnsubscribeFn;
    getBarsPage?: (req: BarsPageRequest) => Promise<BarsPage>;
    dataVariants?: (query: DataVariantQuery) => DataVariantCapabilities | Promise<DataVariantCapabilities>;
    searchSymbols?: (request: SymbolSearchRequest) => Promise<SymbolMatch[]>;
    getBars(req: CachedBarsRequest): Promise<Bar[]>;
    /** Read any valid, TTL-retained closed overlap without initiating a source request. */
    getCachedBars(req: BarsRequest): Promise<Bar[] | undefined>;
    /**
     * Drop one series, or (with no argument) everything this cache knows of.
     * "Knows of" is literal with an injected persistent store: recency and size
     * are tracked in memory, so keys written by an earlier session are dropped
     * when they are next read and found expired, not by `clear()`. A store that
     * outlives the process is responsible for its own overall quota.
     */
    invalidate(req?: Pick<BarsRequest, 'symbol' | 'exchange' | 'interval' | 'variant'>): Promise<void>;
    clear(): Promise<void>;
    stats(): BarCacheStats;
    private _lookup;
    private _slice;
    private _put;
    private _drop;
    private _evict;
    private _remember;
    private _readEntry;
    private _validEntry;
    private _validBar;
    private _cloneEntry;
    private _writeBacking;
    private _deleteBacking;
    private _restoreLatestBacking;
}
/**
 * Wrap any `DataFeed` in a warm-load bar cache.
 *
 *     const feed = withBarCache(new OpenAlgoDataFeed(cfg), { ttlMs, max, storage });
 *
 * A hit ends at the last closed bar; a live subscription supplies the forming
 * one. A host with none must ask again with `noCache: true` when a hit ends
 * before a bar it has already shown, or its chart steps back to an older close.
 */
declare function withBarCache(feed: DataFeed, options?: BarCacheOptions): BarCache;

/**
 * Time conversions (ARCHITECTURE.md §4.0). Internal time is always UTC seconds.
 * Feed adapters convert broker formats here, at the edge:
 *   - REST history: IST date/time strings
 *   - WS feed: epoch milliseconds
 * India observes no DST, so IST is a fixed UTC+5:30 offset.
 */
/** IST offset in seconds (UTC+5:30). */
declare const IST_OFFSET_SECONDS: number;
/** Epoch milliseconds to UTC seconds. */
declare function epochMsToUtcSeconds(ms: number): number;
/**
 * Parse an IST wall-clock date/time string to UTC seconds. Accepts
 * `YYYY-MM-DD`, `YYYY-MM-DD HH:MM[:SS]`, and the `T`-separated ISO variant.
 * Parsing is explicit (never relies on the host machine's locale/timezone).
 */
declare function istStringToUtcSeconds(input: string): number;
interface IstParts {
    year: number;
    month: number;
    day: number;
    hour: number;
    minute: number;
    second: number;
    /** 0 = Sunday .. 6 = Saturday, in IST. */
    weekday: number;
}
/** UTC seconds to IST calendar parts (for axis labels / tick decisions). */
declare function utcSecondsToIstParts(utcSeconds: number): IstParts;
/** Format UTC seconds as an IST `HH:MM` clock label. */
declare function formatIstTime(utcSeconds: number): string;
/** Format UTC seconds as an IST `HH:MM:SS` clock label (sub-minute / tick timeframes). */
declare function formatIstTimeSeconds(utcSeconds: number): string;
/** Format UTC seconds as an IST `YYYY-MM-DD` date (for OpenAlgo history requests). */
declare function utcSecondsToIstDateString(utcSeconds: number): string;
/** Format UTC seconds as an IST `DD Mon` date label. */
declare function formatIstDate(utcSeconds: number): string;
/**
 * Crosshair time-tag label in IST, matching the reference style:
 * `Wed 21 May '26`, with ` HH:MM` appended for intraday (non-midnight) bars.
 */
declare function formatIstCrosshairLabel(utcSeconds: number): string;
/** True if the two UTC-second instants fall on different IST calendar days. */
declare function isNewIstDay(prevUtcSeconds: number, utcSeconds: number): boolean;
/**
 * The shipped default zone. A caller who passes no timezone gets exactly what
 * v1.2.0 produced; openalgo-charts is a general library that happens to ship an
 * Indian default, not an Indian library.
 */
declare const DEFAULT_TIMEZONE = "Asia/Kolkata";
/**
 * Calendar parts of an instant in some IANA zone. Structurally identical to
 * `IstParts`, which is now the special case rather than the assumption.
 */
type ZonedParts = IstParts;
/** Calendar periods the indicators and profiles anchor to. */
type ZonedPeriod = 'day' | 'week' | 'month' | 'quarter' | 'year';
/** True when the runtime recognises `zone` as an IANA zone name. */
declare function isValidTimezone(zone: string): boolean;
/** UTC seconds to calendar parts in `zone` (for axis labels / tick decisions). */
declare function utcSecondsToZonedParts(utcSeconds: number, zone?: string): ZonedParts;
/** Whole days since 1970-01-01, counted in `zone`. Allocation-free on a repeat. */
declare function zonedDayIndex(utcSeconds: number, zone?: string): number;
/**
 * Week number since the epoch, counted in `zone`, weeks starting Monday.
 * 1970-01-01 was a Thursday, hence the +3 before the divide.
 */
declare function zonedWeekIndex(utcSeconds: number, zone?: string): number;
/**
 * The zone's offset from UTC at this instant, in seconds. DST-correct: ask it
 * for a July instant and a January one in the same northern zone and the two
 * answers differ.
 */
declare function zoneOffsetSeconds(utcSeconds: number, zone?: string): number;
/** True if the two instants fall on different calendar days in `zone`. */
declare function isNewZonedDay(prevUtcSeconds: number, utcSeconds: number, zone?: string): boolean;
/** True if the two instants fall in different Monday-start weeks in `zone`. */
declare function isNewZonedWeek(prevUtcSeconds: number, utcSeconds: number, zone?: string): boolean;
/** True if the two instants fall in different calendar months in `zone`. */
declare function isNewZonedMonth(prevUtcSeconds: number, utcSeconds: number, zone?: string): boolean;
/** True if the two instants fall in different calendar quarters in `zone`. */
declare function isNewZonedQuarter(prevUtcSeconds: number, utcSeconds: number, zone?: string): boolean;
/** True if the two instants fall in different calendar years in `zone`. */
declare function isNewZonedYear(prevUtcSeconds: number, utcSeconds: number, zone?: string): boolean;
/** The boundary test for one period, for a caller that carries the period as data. */
declare function isNewZonedPeriod(prevUtcSeconds: number, utcSeconds: number, period: ZonedPeriod, zone?: string): boolean;
/**
 * Parse a wall-clock date/time string in `zone` to UTC seconds. Accepts the same
 * shapes as `istStringToUtcSeconds`: `YYYY-MM-DD`, `YYYY-MM-DD HH:MM[:SS]`, and
 * the `T`-separated ISO variant. Never consults the host machine's own zone.
 */
declare function zonedStringToUtcSeconds(input: string, zone?: string): number;
/**
 * Wall-clock components in `zone` to UTC seconds.
 *
 * Two passes because the offset we need is the one in force at the *answer*,
 * not at the guess: on a DST changeover day the first pass can land an hour out,
 * and the second pass corrects it. A wall time that a spring-forward skipped has
 * no exact answer; this settles on the instant one offset-step later, which is
 * the same choice `Date` makes.
 */
declare function zonedWallClockToUtcSeconds(year: number, month: number, day: number, hour?: number, minute?: number, second?: number, zone?: string): number;
/** UTC seconds of the local midnight opening this instant's day in `zone`. */
declare function startOfZonedDay(utcSeconds: number, zone?: string): number;
/** UTC seconds of the local midnight opening this instant's Monday in `zone`. */
declare function startOfZonedWeek(utcSeconds: number, zone?: string): number;
/** UTC seconds of the local midnight opening this instant's month in `zone`. */
declare function startOfZonedMonth(utcSeconds: number, zone?: string): number;
/** Format UTC seconds as an `HH:MM` clock label in `zone`. */
declare function formatZonedTime(utcSeconds: number, zone?: string): string;
/** Format UTC seconds as an `HH:MM:SS` clock label in `zone` (sub-minute timeframes). */
declare function formatZonedTimeSeconds(utcSeconds: number, zone?: string): string;
/** Format UTC seconds as a `DD Mon` date label in `zone`. */
declare function formatZonedDate(utcSeconds: number, zone?: string): string;
/** Format UTC seconds as a `YYYY-MM-DD` date in `zone`. */
declare function utcSecondsToZonedDateString(utcSeconds: number, zone?: string): string;
/** Crosshair time-tag label in `zone`, in the same style as the IST form. */
declare function formatZonedCrosshairLabel(utcSeconds: number, zone?: string): string;
/**
 * Bar indices that open a new trading session, read back from the timestamps.
 *
 * An exchange's overnight break is the widest recurring gap in an intraday
 * series, and it is the only thing in the bars themselves that says where one
 * trading day ends. Reading it back beats assuming a timezone: the same code
 * has to serve an exchange in Mumbai and one in New York, and a fixed midnight
 * lands mid-session for one of them. Getting it wrong splices the tail of one
 * session onto the head of the next across the overnight gap, which inflates
 * that period's high-low range and throws anything measured from it a long way
 * off.
 *
 * Returns null when the series shows no readable session break: a market that
 * never closes, bars already a day or coarser, or a feed whose only gaps are
 * weekends. The caller then falls back to a calendar rule, which is the right
 * answer in exactly those cases.
 */
declare function sessionStartIndices(times: readonly number[]): number[] | null;
/**
 * Per-bar flags marking the first bar of each trading session.
 *
 * Falls back to the calendar day in `zone` when the series has no readable
 * session break, which is the only answer available for daily bars and a
 * defensible one for a market that never closes. The zone defaults to IST, so a
 * caller that passes nothing gets exactly the old behaviour.
 */
declare function sessionStartFlags(times: readonly number[], zone?: string): boolean[];
/**
 * Per-bar flags marking the first bar of each calendar period, where `isNew`
 * decides what "period" means for two instants.
 *
 * The test runs on session opens rather than on every bar, so a session that
 * straddles the boundary is not cut in half: the last ninety minutes of a New
 * York Friday fall on a Saturday in IST, and testing bar to bar would start the
 * next week partway through Friday's session. With no readable sessions the
 * test runs bar to bar, which is the same thing when each bar is its own
 * session.
 */
declare function calendarPeriodFlags(times: readonly number[], isNew: (prevUtcSeconds: number, utcSeconds: number) => boolean): boolean[];
/** A trading window in local wall-clock terms, as parsed from a spec string. */
interface SessionSpec {
    /** Minutes from midnight, inclusive. */
    start: number;
    /** Minutes from midnight, exclusive. */
    end: number;
    /** Days the window opens on, 1..7 with 1 = Sunday. Absent means every day. */
    days?: readonly number[];
}
/**
 * Parse `"0915-1015"`, optionally with a day filter after a colon:
 * `"0930-1600:23456"` is Monday to Friday. Whitespace around the parts is
 * ignored. An end at or before the start is a window that runs past midnight.
 *
 * Returns null rather than throwing, because the spec is normally a string a
 * user typed into a settings field and a half-typed one arrives on every
 * keystroke.
 */
declare function parseSessionSpec(spec: string): SessionSpec | null;
/** True if this instant falls inside `spec` as read in `zone`. */
declare function inSessionAt(utcSeconds: number, spec: SessionSpec, zone?: string): boolean;
/**
 * Per-bar flags marking the bars inside `spec`. An unparseable spec string
 * marks nothing: a chart that keeps drawing beats one that dies on a stray
 * character, and the caller can check `parseSessionSpec` itself to tell an
 * empty window from a bad one.
 */
declare function sessionFlags(times: readonly number[], spec: string | SessionSpec, zone?: string): boolean[];

/** Headless, chart-owned trader alerts. Hosts subscribe to alert:triggered for delivery. */
declare class AlertController {
    private readonly _chart;
    private readonly _records;
    /** The alert whose line the pointer is over, for a key that acts on it. */
    private _hovered;
    private readonly _off;
    private readonly _now;
    private readonly _drawings;
    /** Whether a spent alert keeps its line; the host's choice, default 'show'. */
    private readonly _spentLines;
    private readonly _visuals;
    private _timer;
    private _timerAt;
    private _paused;
    private _restoring;
    private _replay;
    private _destroyed;
    private _revision;
    private _drag;
    constructor(_chart: AlertChartHost, options?: AlertControllerOptions);
    private _dragAvailability;
    /**
     * The alert the pointer is over, or none.
     *
     * Offered so a host can bind a key to it. The controller does not bind keys
     * itself: it owns no input and a chart embedded without the widget shell has
     * its own idea of what a keystroke means.
     */
    hovered(): string | undefined;
    private _dragPrice;
    private _startDrag;
    private _cancelDrag;
    /** Market evaluation keeps the committed source until a moved, owned gesture ends. */
    private _onDrag;
    add(input: AlertInput): Alert;
    update(id: string, patch: AlertPatch): Alert | undefined;
    remove(id: string): boolean;
    private _remove;
    list(): Alert[];
    toJSON(): AlertsDocument;
    /** Validate the complete replacement before touching current alerts or their visuals. */
    fromJSON(input: unknown): void;
    private _saveState;
    /** Availability is independent of lifecycle state; disabled records can still have valid anchors. */
    availability(id: string): AlertAvailability;
    enable(id: string): Alert | undefined;
    disable(id: string): Alert | undefined;
    /** Explicit host pause is independent of replay and never delivers a backlog. */
    setPaused(paused: boolean): void;
    destroy(): void;
    /**
     * Whether a drawing alert has lost its drawing. Only an alert on the
     * instrument the chart shows can tell: a host that keeps drawings per
     * instrument has another instrument's drawings off the chart, and an id in
     * one instrument's document may name an unrelated drawing in another's. So
     * an alert set on another instrument keeps its drawing reference, waits
     * without evaluating, and is judged when its instrument is back.
     */
    private _drawingGone;
    /** The slot the chart keeps its price pane in now; it moves, so it is never kept. */
    private _pricePane;
    private _assertAlive;
    private _seed;
    private _seedAll;
    private _onObjects;
    private _syncVisual;
    private _onData;
    /**
     * The first element of this update a record has not judged. An update that
     * still has the record's tail at its index, at its time or later (a forming
     * Kagi vertex or range bar is dated again as bars arrive), appended the
     * elements after it: the tail is judged again only when it changed, then
     * each newer element. Fewer bars, or a tail that moved because history
     * changed, leave nothing to attribute, and the update is judged as one to
     * the newest element against the tail the record last saw.
     */
    private _firstNew;
    /**
     * Judge `bars[index]` for one record, as the single update that brought it
     * would have: an element newer than the record's tail closes the one before
     * it, and a touch is measured from that tail. Returns whether a checkpoint
     * moved.
     */
    private _judge;
    private _plot;
    private _reading;
    private _closedMatch;
    private _touchMatch;
    private _barMatch;
    private _drawingValue;
    private _drawingTouch;
    private _trigger;
    private _expireDue;
    private _clearTimer;
    private _scheduleExpiry;
}

/** Shared editor fields in source units. Drawing bands own their bounds. */
declare function alertSettingsSchema(source: AlertSource, condition?: AlertCondition): readonly IndicatorInput[];

/** Validate and detach a version-1 document, or migrate a legacy bare list. */
declare function parseAlertsDocument(input: unknown): AlertsDocument;

/** Register runtime code by stable id; persisted alerts store only that id. */
declare function registerBarCondition(condition: BarCondition): void;
declare function unregisterBarCondition(id: string): boolean;
declare function getBarCondition(id: string): Readonly<BarCondition> | undefined;
declare function registeredBarConditions(): readonly Readonly<BarCondition>[];

export { ALT_PRESET, type AddIndicatorOptions, type AddSeriesOptions, type AggTick, type Alert, type AlertAvailability, type AlertChangeEvent, type AlertChartHost, type AlertCondition, AlertController, type AlertControllerOptions, type AlertDrawingInfo, type AlertDrawingLevel, type AlertDrawingProvider, type AlertDrawingValue, type AlertErrorEvent, type AlertEventPayload, type AlertInput, type AlertPatch, type AlertPolicy, type AlertRemovedEvent, type AlertRepeat, type AlertScope, type AlertSource, type AlertState, type AlertTriggeredPayload, type AlertsChangedEvent, type AlertsDocument, type AlertsRestoredEvent, type AxisChromeOptions, type AxisStyle, BAR_CACHE_VERSION, BUILTIN_COMMANDS, type Bar, BarCache, type BarCacheOptions, type BarCacheStats, type BarCacheStore, type BarCondition, type BarConditionAlertSource, type BarConditionContext, type BarConfirmationOptions, type BarSubscriptionOptions, type BarUpdate, type BarsPage, type BarsPageRequest, type BarsRequest, type BrandingChangedEvent, type Bucketing, BuySellButtons, type BuySellButtonsOptions, CHART_STATE_VERSION, type CachedBars, type CachedBarsRequest, type CalendarBucketing, type CalendarUnit, CandleBuilder, type CandleBuilderOptions, type CandleGeometry, type CandleStyle, type CandleTier, type CandleUpdate, Canvas2dBackend, type CanvasLineStyle, type CanvasOptions, Chart, type ChartApplyOptions, type ChartClickEvent, type ChartDataColumn, type ChartDataContext, type ChartDataCsvFormatters, type ChartDataCsvOptions, type ChartDataCsvRange, type ChartDataProjectionContext, type ChartDataUpdate, type ChartDragCancelEvent, type ChartDragEndEvent, type ChartDragEvent, type ChartEvent, type ChartEventClick, type ChartEventDetails, type ChartEventMap, type ChartEventOptions, type ChartHoverEvent, type ChartNavigationOptions, type ChartObjectBand, type ChartObjectCapabilities, type ChartObjectDefinition, type ChartObjectDrawing, type ChartObjectDrawingGroup, type ChartObjectDrawingSource, type ChartObjectKind, type ChartObjectProvider, type ChartObjectSnapshot, ChartObjects, type ChartObjectsOptions, type ChartOptions, type ChartResizeEvent, type ChartRestoreOptions, type ChartSettingsColorPairInput, type ChartSettingsInput, type ChartSettingsState, type ChartSettingsTab, type ChartSettingsTabId, type ChartSettingsValue, type ChartSettingsValues, type ChartState, ChartTable, type ChartTableOptions, type ChartTheme, type ChartViewportEvent, type ChartWatermarkOptions, type ComparisonAlignment, type ComparisonBaseline, type ComparisonChartHost, ComparisonController, type ComparisonControllerOptions, type ComparisonHandle, type ComparisonMode, type ComparisonOptions, type ComparisonPane, type ContextMenuEvent, type ContextMenuTarget, type ContextMenuTargetKind, type CrosshairMoveEvent, type CrosshairOptions, type CrosshairStyle, type CustomShortcut, DEFAULT_CANDLE_BUILDER_OPTIONS, DEFAULT_CANDLE_STYLE, DEFAULT_CHART_TABLE_OPTIONS, DEFAULT_HISTOGRAM_STYLE, DEFAULT_KEYMAP, DEFAULT_PRICE_SCALE_OPTIONS, DEFAULT_THEME, DEFAULT_TIMEZONE, DEFAULT_TIME_NAVIGATOR_OPTIONS, DEFAULT_TIME_SCALE_OPTIONS, DEFAULT_TRADING_COLORS, DEFAULT_ZOOM_GLIDE_OPTIONS, type DataAdjustment, type DataContextTarget, type DataFeed, DataLayer, DataLoadingController, type DataLoadingOptions, type DataLoadingSnapshot, type DataLoadingStatus, type DataSession, type DataUpdateReason, type DataVariant, type DataVariantCapabilities, type DataVariantDimension, type DataVariantQuery, type DecodedOrder, type DepthLevel, type DoubleClickAction, type DoubleClickEvent, type DrawAnchor, type DrawItem, type DrawingAlertSource, type EmptyEvent, type EventDetailBlock, type EventDetailField, type EventDetailInline, type EventDetailSpan, type EventGroup, type EventMarkerDetails, EventMarkers, type EventMarkersOptions, type ExportSvgOptions, FakeDataFeed, type FeedScheduler, type FillGradient, type FillPoint, type GridAxisStyle, type GridOptions, type GridStyle, type HistogramStyle, type HistoryLoadingStatus, HistoryRequestPool, type HistoryRequestPoolOptions, INDICATOR_LINE_STYLES, INDICATOR_PLOT_STYLES, INDICATOR_SOURCES, type IPrimitive, type IRenderBackend, IST_OFFSET_SECONDS, type IndexedBar, type IndicatorAlertContext, type IndicatorAlertFrequency, type IndicatorAlertPayload, type IndicatorAlertSource, type IndicatorAlertSpec, type IndicatorApi, type IndicatorAttachContext, IndicatorBackground, type IndicatorBackgroundSpec, type IndicatorBarSource, type IndicatorBarsProvider, type IndicatorBarsProviderAccess, type IndicatorBarsRequest, type IndicatorCalcContext, type IndicatorDataChange, type IndicatorDataStatus, type IndicatorDataStatusEvent, type IndicatorDescriptor, type IndicatorDrawing, IndicatorDrawings, type IndicatorEditOptions, type IndicatorExecutionContext, IndicatorFill, type IndicatorFillOptions, type IndicatorFillSpec, type IndicatorHost, type IndicatorInput, type IndicatorInputCondition, type IndicatorInputConditionValue, IndicatorInputError, type IndicatorInputPresentation, type IndicatorInstanceEvent, type IndicatorLevel, type IndicatorLevelContext, type IndicatorLineStyle, type IndicatorMarker, type IndicatorOutputTarget, type IndicatorPlot, type IndicatorPolicy, type IndicatorRequestState, type IndicatorSettings, type IndicatorSnapshotRequest, type IndicatorSource, type IndicatorState, type IndicatorStore, type IndicatorStudyOutput, type IndicatorStudySource, type IndicatorTableSpec, type IndicatorValues, Instrument, type InstrumentCalendar, type InstrumentKey, type InstrumentMetadata, type InstrumentSession, type IntervalBucketing, type IntervalDescriptor, type IntervalParts, InvalidationLevel, type IstParts, type KeymapEntry, LINK_CROSSHAIR_ALPHA, type LateTickPolicy, type LayoutChangeEvent, type LayoutSetter, type LazyLoadEvent, type LegendField, type LegendStatusData, type LegendStatusLineOptions, type LegendStatusSource, type LegendTitleMode, type LegendValue, type LinePoint, type LinkAppearanceAdapter, type LinkAppearanceValues, type LinkChart, LinkCrosshair, type LinkDataLayer, type LinkDrawingsAdapter, LinkGroup, type LinkMemberOptions, type LinkMissingPolicy, type LinkOptions, type LiveBarMeta, type LogicalRange, LogoWatermark, type LogoWatermarkOptions, type LtpEvent, type MarkerPosition, type MarkerShape, type MarkerSize, type MarketDepth, type MarketPhase, type MarketPhaseFn, type MarketStatus, type MaybePromise, type ModeCheck, type NewsFeed, type NewsItem, type NewsPage, type NewsRequest, type OpenAlgoConfig, OpenAlgoDataFeed, type OpenAlgoLiveConfig, OpenAlgoLiveDataFeed, type OpenAlgoTradeConfig, OpenAlgoTradeFeed, type OpenAlgoWsConfig, OpenAlgoWsFeed, type OrderBookSnapshot, type OrderDecodeCode, type OrderDecodeIssue, type OrderDecodeResult, type OrderSide, type OrderType, type OrderUpdateEvent, type OriginalTime, PRICE_LEVEL_KINDS, PRICE_SCALE_MODES, Pane, type PaneCollapsedEvent, type PaneEvent, type PaneInvalidation, PaneLegend, type PaneLegendAction, type PaneLegendOptions, type PaneMaximizedEvent, type PaneMovedEvent, type PaneState, type PickEndEvent, type PickHandle, type PickHost, type PickKind, type PickOptions, type PickPoint, type PickStartEvent, type PlaceOrder, type PlotBarColor, type PlotMarginOptions, type PlotRect, type PointerInfo, type PointerKind, type PointerModifiers, type PointerSample, type PositionSide, type PriceAlertSource, type PriceAxisPlacement, type PriceAxisPlacementChangedEvent, type PriceAxisSide, type PriceAxisSlot, type PriceAxisState, type PriceFormat, type PriceLevelInput, type PriceLevelKind, type PriceLevelQuote, type PriceLevelStyle, type PriceLevelValues, PriceLevels, type PriceLevelsOptions, PriceLine, type PriceLineOptions, type PriceRange, PriceScale, type PriceScaleId, type PriceScaleMode, type PriceScaleOptions, type PriceScaleState, type PrimitiveAnchor, type PrimitiveHit, type PrimitiveHost, type PrimitivePlacement, type PrimitiveRenderContext, type QuarantinedRow, type QuoteFeed, type QuoteRequest, type QuoteSnapshot, type QuoteStreamHandlers, type QuoteStreamStatus, type RawOrder, type RenderBackendFactory, type RenderBackendKind, type RenderDevice, type RendererChoice, type RendererEntry, type RendererFallbackEvent, type RendererFallbackReason, type ReplayBarEndTime, type ReplayChartHost, ReplayController, ReplayGroup, type ReplayGroupChartHost, type ReplayGroupMember, type ReplayGroupOptions, type ReplayGroupState, type ReplayOptions, type ReplayScheduler, type ReplayScope, ReplayShade, type ReplayShadeOptions, type ReplaySimulation, type ReplayState, type ReplayTiming, type ReplayViewport, type RequestedBarsSnapshot, type ResolvedLinkOptions, type RestoreReport, SCALE_FONT_MAX, SCALE_FONT_MIN, type ScaleCanvasOptions, type SeriesApi, type SeriesDataItem, type SeriesDataState, type SeriesId, type SeriesMarker, SeriesMarkers, type SeriesRenderContext, type SeriesState, type SeriesStyle, type SeriesTransformDefinition, type SeriesTransformRun, type SeriesTransformSpec, type SeriesType, type SeriesUpdateOptions, SessionCalendar, type SessionCalendarSource, type SessionCalendarSpec, type SessionPhase, type SessionPhaseSource, type SessionPhaseSpan, SessionShade, type SessionShadeOptions, type SessionShading, type SessionSpec, type ShortcutListItem, ShortcutManager, type ShortcutManagerOptions, type ShortcutPreset, type ShortcutScope, type ShortcutTriggerEvent, type Size, type SocketFactory, type SocketLike, type SupertrendPoint, SvgContext, type SvgContextOptions, SvgLinearGradient, type SymbolMatch, type SymbolSearchRequest, type TableCell, type TablePosition, TextWatermark, type TextWatermarkOptions, type Tick, type TickBand, TickBarAggregator, type TickBarOptions, type TickCountBucketing, type TickMarkType, TickSchedule, type TickTimeframe, TimeNavigator, type TimeNavigatorAction, type TimeNavigatorOptions, TimeScale, type TimeScaleOp, type TimeScaleOptions, type TimezoneChangedEvent, type TradeFeed, type TradeMarkerVariant, TradeMarkersPrimitive, type TradingBracketModifyEvent, type TradingCapabilities, TradingCapabilityError, type TradingCapabilityRequest, type TradingCapabilityResult, type TradingCapabilitySource, type TradingColors, TradingController, type TradingHost, type TradingLineStyle, type TradingLineVariant, type TradingOperation, type TradingOrder, type TradingOrderModifyEvent, type TradingOrderSide, type TradingOrderType, type TradingPosition, type TradingSettings, type TradingSyncPayload, type TradingTrade, type UTCSeconds, UnknownIntervalError, type UnsubscribeFn, VERSION, type VolumeBucketing, type VolumeMode, type WatermarkPosition, type Whitespace, type WsClientWarning, type WsControlMessage, type WsMode, type WsState, type ZOrder, type ZonedParts, type ZonedPeriod, type ZoomAnchor, ZoomGlide, type ZoomGlideOptions, addComparison, alertSettingsSchema, alignToPrimary, applyChartSettings, assertTradingCapability, atr, attachSessionShading, autoscaleRange, backendDegradation, backoffDelayMs, barCacheKey, barCloseSec, beginPick, bestHit, bitmapSize, bucketStartOf, calendarMarketPhase, calendarPeriodFlags, candleGeometry, candleTier, chartSettingsSchema, checkTradingCapability, clamp, classifyAuthAck, compactVolume, comparisonController, computePriceLevels, conflateBars, conflateItems, conflationGroupSize, createChart, createLinkGroup, createRenderBackend, darkTheme, dashPattern, dataVariantError, dataVariantKey, decodeOrder, drawLabel, drawShape, effectiveMarkerPx, ema, emaSeries, epochMsToUtcSeconds, eventToCombo, exportChartDataCsv, filterLinkAppearance, followerIndex, followerRange, formatCombo, formatIstCrosshairLabel, formatIstDate, formatIstTime, formatIstTimeSeconds, formatSubscribe, formatUnsubscribe, formatZonedCrosshairLabel, formatZonedDate, formatZonedTime, formatZonedTimeSeconds, fromGradient, generateBars, getBarCondition, getChartType, getIndicator, getSeriesTransform, hasIndicator, inSessionAt, indicatorDefaults, indicatorStyleInputs, intervalParts, intervalToSeconds, isDailyInterval, isIntradayInterval, isKnownInterval, isNewIstDay, isNewZonedDay, isNewZonedMonth, isNewZonedPeriod, isNewZonedQuarter, isNewZonedWeek, isNewZonedYear, isRebasing, isReplaying, isReservedCombo, isSecondsInterval, isTickInterval, isTimeBucketed, isValidCombo, isValidTimezone, isWhitespace, istStringToUtcSeconds, lastPriceLevelFromSeriesStyle, lerp, lightTheme, mapHistoryResponse, mapOrder, mapOrderStatus, mapPosition, markerSizePx, marketStatusAt, mergeBars, nextBucketStart, niceTicks, normalizeCombo, normalizeDataVariant, optimalBarWidth, parseAlertsDocument, parseCombo, parseIndicatorPolicy, parseMessage, parsePaneState, parseSessionSpec, parseTopic, plotStyleKeys, precisionForStep, publishDataContext, readChartSettings, readSequence, registerBarCondition, registerChartType, registerIndicator, registerInterval, registerRenderBackend, registerSeriesTransform, registeredBarConditions, registeredChartTypes, registeredIndicators, registeredIntervals, registeredRenderBackends, registeredSeriesTransforms, resolveCrosshairStyle, resolveGridStyle, resolveInterval, resolvePlotMargins, resolveRenderBackend, resolveScaleStyle, roundToTick, rowTimeToUtcSeconds, rsi, rsiSeries, seriesStyleForLastPriceLevel, sessionFlags, sessionStartFlags, sessionStartIndices, sharedHistoryRequests, snapToDevicePixel, sourceValue, sourceValues, startOfZonedDay, startOfZonedMonth, startOfZonedWeek, supertrend, supertrendSeries, tableOrigin, toBar, trueRange, tryResolveInterval, unregisterBarCondition, unregisterInterval, unregisterRenderBackend, unsupportedDataVariant, utcSecondsToIstDateString, utcSecondsToIstParts, utcSecondsToZonedDateString, utcSecondsToZonedParts, version, verticalGradient, watermarkRect, withAlpha, withBarCache, zoneOffsetSeconds, zonedDayIndex, zonedStringToUtcSeconds, zonedWallClockToUtcSeconds, zonedWeekIndex };
