/* ==========================================================================
 * honba-kao ⇄ openalgo-charts widget bridge
 *
 * Uses the widget tier (createWidget) which provides the full OpenAlgo
 * terminal experience: topbar, drawing rail, dialogs, indicator picker,
 * objects panel, settings, snapshot, layouts, and 502 indicators.
 *
 * The Rust side talks to window.honbaCharts.* (see openalgo_chart.rs).
 * ========================================================================== */

import { createWidget, injectWidgetStyles, WIDGET_COMPONENT_CSS, WIDGET_DIALOGS, registerWidgetDialog, registerWidgetDialogs, themeMode, openMenu, mountIndicatorPicker as mountBuiltinIndicatorPicker } from './openalgo-charts.widget.mjs';
import { createChart, BuySellButtons, darkTheme, lightTheme, intervalParts, TickBarAggregator, getChartType, registeredSeriesTransforms, registerInterval, isKnownInterval } from './openalgo-charts.mjs';
import './openalgo-charts.draw.mjs';
import { DrawingController, InstrumentDrawings, instrumentDrawingsKey, registerBuiltinDrawingTools, webStorageDrawingStore, chromeIconSvg } from './openalgo-charts.draw.mjs';
/* Transform tier: Heikin Ashi, Renko, range bars, line break, point & figure
   and Kagi. Without it the native "Chart type" menu lists only the core
   renderers. Importing registers them as a side effect; the explicit call
   below guards against the side effect being tree-shaken. */
import { registerTransformChartTypes } from './openalgo-charts.transform.mjs';
import { BUILTIN_INDICATORS } from './openalgo-charts.indicators.mjs';

/* Register drawing tools and transform chart types once (both idempotent). */
registerBuiltinDrawingTools();
registerTransformChartTypes();

/* Inject widget CSS (topbar, rail, dialogs, toasts, etc.). */
if (typeof document !== 'undefined') {
    injectWidgetStyles(document, WIDGET_COMPONENT_CSS);
}

/* Register built-in dialogs (settings, indicator picker, objects, etc.). */
registerWidgetDialogs(WIDGET_DIALOGS);

/* We also allow a custom indicator picker to be mounted (replaces built-in). */
let customIndicatorPickerMount = null;
export function registerCustomIndicatorPicker(mountFn) {
    customIndicatorPickerMount = mountFn;
    registerWidgetDialog('indicatorPicker', mountFn);
}

const widgets = new Map();
let nextId = 1;

/** Read a CSS custom property off the active skin. */
function cssVar(name, fallback) {
    const host = document.body || document.documentElement;
    const value = getComputedStyle(host).getPropertyValue(name);
    return (value && value.trim()) || fallback;
}

function isDarkMode() {
    return (document.documentElement.dataset.mode || 'dark') === 'dark';
}

/**
 * Compose the openalgo `ChartTheme` from the active skin.
 */
function themeFromSkin() {
    const base = isDarkMode() ? darkTheme : lightTheme;

    return {
        ...base,
        background: cssVar('--chart-background', base.background),
        grid: cssVar('--chart-grid', base.grid),
        axisText: cssVar('--chart-axis-text', base.axisText),
        axisLine: cssVar('--chart-axis-line', base.axisLine),
        paneSeparator: cssVar('--chart-pane-separator', base.paneSeparator),
        crosshair: cssVar('--chart-crosshair', base.crosshair),

        upColor: cssVar('--pnl-gain', base.upColor),
        downColor: cssVar('--pnl-loss', base.downColor),
        wickUpColor: cssVar('--pnl-gain', base.wickUpColor),
        wickDownColor: cssVar('--pnl-loss', base.wickDownColor),

        lineColor: cssVar('--primary-8', base.lineColor),
        areaTopColor: cssVar('--chart-area-top', base.areaTopColor),
        areaBottomColor: cssVar('--chart-area-bottom', base.areaBottomColor),

        baselineTopLine: cssVar('--pnl-gain', base.baselineTopLine),
        baselineTopFill: cssVar('--chart-area-top', base.baselineTopFill),
        baselineBottomLine: cssVar('--pnl-loss', base.baselineBottomLine),
        baselineBottomFill: cssVar('--chart-area-bottom', base.baselineBottomFill),

        lastPriceUp: cssVar('--pnl-gain', base.lastPriceUp),
        lastPriceDown: cssVar('--pnl-loss', base.lastPriceDown),
        lastPriceText: cssVar('--on-accent', base.lastPriceText),

        buy: cssVar('--pnl-gain', base.buy),
        sell: cssVar('--pnl-loss', base.sell),
        profit: cssVar('--pnl-gain', base.profit),
        loss: cssVar('--pnl-loss', base.loss),
    };
}

/** A style value may be a literal colour or the name of a CSS token. */
function resolveColor(value) {
    if (typeof value !== 'string' || value.length === 0) return undefined;
    if (value.startsWith('--')) return cssVar(value, undefined);
    return value;
}

/** Translate the small style bag Rust sends into series-type-specific fields. */
function seriesStyle(kind, style, theme) {
    const override = resolveColor(style && style.color);

    switch (kind) {
        case 'candlestick':
        case 'bar':
        case 'hollow-candle':
        case 'volume-candle':
        case 'high-low':
            return {
                upColor: theme.upColor,
                downColor: theme.downColor,
                borderUpColor: override || theme.upColor,
                borderDownColor: override || theme.downColor,
                wickUpColor: override || theme.wickUpColor,
                wickDownColor: override || theme.wickDownColor,
            };
        case 'area':
        case 'baseline':
        case 'hlc-area':
            return {
                lineColor: override || theme.lineColor,
                topColor: theme.areaTopColor,
                bottomColor: theme.areaBottomColor,
            };
        default:
            return {
                color: override || theme.lineColor,
                lineWidth: (style && style.lineWidth) || 2,
            };
    }
}

/* Volume overlay colour (matches labs workbench). */
const VOLUME_COLOR = 'rgba(41, 98, 255, 0.28)';

function volumeRows(bars) {
    return bars.map((bar) => ({
        time: bar.time,
        open: 0,
        high: bar.volume ?? 0,
        low: 0,
        close: bar.volume ?? 0,
    }));
}

/* ---------- Buy/Sell bubble (cursor freeze + top render) ---------- */

function getBuySellBubbleBounds(tradeBtns, chart, paneIndex) {
    const tb = tradeBtns;
    const sell = tb && tb._sellRect;
    const buy = tb && tb._buyRect;
    if (sell && buy) {
        return {
            left: sell.x,
            top: sell.y,
            right: buy.x + buy.w,
            bottom: sell.y + sell.h,
        };
    }
    const rect = chart && typeof chart.plotRect === 'function' ? chart.plotRect(paneIndex) : null;
    if (rect && tb && typeof tb._origin === 'function' && typeof tb._h === 'number') {
        const o = tb._origin(rect.width, rect.height);
        return {
            left: o.x,
            top: o.y,
            right: o.x + o.w,
            bottom: o.y + tb._h,
        };
    }
    return null;
}

function intersectSegmentWithBox(x0, y0, x1, y1, box) {
    const dx = x1 - x0;
    const dy = y1 - y0;
    let tMin = 0;
    let tMax = 1;
    const clip = (p, q) => {
        if (p === 0) return q >= 0;
        const t = q / p;
        if (p < 0) {
            if (t > tMax) return false;
            if (t > tMin) tMin = t;
        } else {
            if (t < tMin) return false;
            if (t < tMax) tMax = t;
        }
        return true;
    };
    if (
        clip(-dx, x0 - box.left) &&
        clip(dx, box.right - x0) &&
        clip(-dy, y0 - box.top) &&
        clip(dy, box.bottom - y0)
    ) {
        return { x: x0 + dx * tMin, y: y0 + dy * tMin };
    }
    return { x: x0, y: y0 };
}

function attachBuySellBubbleCursorFreeze(chart, tradeBtns) {
    const tb = tradeBtns;
    if (!chart || !tb) return;
    if (typeof tb.hitTest === 'function') {
        const origHitTest = tb.hitTest.bind(tb);
        tb.hitTest = (plotX, localY, rc) => {
            const hit = origHitTest(plotX, localY, rc);
            if (hit) return hit;
            const primaryPane = typeof chart.primaryPaneIndex === 'function' ? chart.primaryPaneIndex() : 0;
            const bounds = getBuySellBubbleBounds(tb, chart, primaryPane);
            if (bounds && plotX >= bounds.left && plotX <= bounds.right && localY >= bounds.top && localY <= bounds.bottom) {
                return { externalId: 'trade:qty', zOrder: 'top', distance: 0, cursor: 'pointer' };
            }
            return null;
        };
    }

    const input = chart._input;
    if (!input || typeof input._updateCursor !== 'function') return;

    let wasInsideBubble = false;
    let lastOutsidePoint = null;

    const isPointInBubble = (pane, x, localY) => {
        const leftAxisWidth = chart._leftAxisWidth ?? 0;
        const plotX = x - leftAxisWidth;
        const primaryPane = typeof chart.primaryPaneIndex === 'function' ? chart.primaryPaneIndex() : 0;
        if (pane !== primaryPane) {
            return { inside: false, bounds: null, plotX, leftAxisWidth };
        }
        const bounds = getBuySellBubbleBounds(tb, chart, pane);
        const inside = bounds !== null && plotX >= bounds.left && plotX <= bounds.right && localY >= bounds.top && localY <= bounds.bottom;
        return { inside, bounds, plotX, leftAxisWidth };
    };

    const origUpdateCursor = input._updateCursor.bind(input);
    input._updateCursor = function (pane, x, localY, y, event) {
        const { inside, bounds, plotX, leftAxisWidth } = isPointInBubble(pane, x, localY);

        if (inside && bounds) {
            if (!wasInsideBubble && lastOutsidePoint !== null && lastOutsidePoint.pane === pane) {
                const entry = intersectSegmentWithBox(lastOutsidePoint.plotX, lastOutsidePoint.localY, plotX, localY, bounds);
                const paneTop = y - localY;
                origUpdateCursor(pane, leftAxisWidth + entry.x, entry.y, paneTop + entry.y, event);
            }
            wasInsideBubble = true;
            const hit = this._hitAt ? this._hitAt(pane, x, localY) : null;
            if (this._setHover) this._setHover(hit);
            if (this._host && this._host._feedTimeNav) this._host._feedTimeNav(null);
            return;
        }

        wasInsideBubble = false;
        lastOutsidePoint = { pane, plotX, localY };
        origUpdateCursor(pane, x, localY, y, event);
    };

    if (typeof input._onPointerDown === 'function') {
        const origOnPointerDown = input._onPointerDown.bind(input);
        input._onPointerDown = function (event) {
            origOnPointerDown(event);
            const pt = this._localPoint ? this._localPoint(event) : null;
            if (pt) {
                const { inside } = isPointInBubble(pt.pane, pt.x, pt.localY);
                if (inside) this._dragging = false;
            }
        };
    }

    if (typeof input._onPointerLeave === 'function') {
        const origOnPointerLeave = input._onPointerLeave.bind(input);
        input._onPointerLeave = function () {
            wasInsideBubble = false;
            lastOutsidePoint = null;
            origOnPointerLeave();
        };
    }
}

function ensureBuySellBubbleOnTop(chart, tradeBtns) {
    const panes = chart && chart._panes;
    if (!Array.isArray(panes)) return;

    for (const pane of panes) {
        if (!pane || pane.__honbaBubbleTopPatched || typeof pane.paintTop !== 'function') continue;
        pane.__honbaBubbleTopPatched = true;
        const origPaintTop = pane.paintTop.bind(pane);
        pane.paintTop = function (cursor, rc, targetCtx) {
            origPaintTop(cursor, rc, targetCtx);
            try {
                const live = typeof this._live === 'function' ? this._live(rc) : this._primitives;
                if (!Array.isArray(live) || !live.includes(tradeBtns)) return;
                const ctx = targetCtx ?? this.top?.ctx;
                if (!ctx) return;
                const layout = typeof this._layout === 'function' ? this._layout(rc) : { plotLeft: 0 };
                const dpr = (rc && rc.dpr) ?? 1;
                ctx.save();
                if (layout.plotLeft > 0) ctx.translate(Math.round(layout.plotLeft * dpr), 0);
                const primCtx = typeof this._primitiveContext === 'function' ? this._primitiveContext(rc) : rc;
                const boundCtx = typeof this._boundPrimitiveContext === 'function'
                    ? this._boundPrimitiveContext(tradeBtns, primCtx, rc)
                    : primCtx;
                tradeBtns.draw(ctx, boundCtx);
                ctx.restore();
            } catch {
                /* ignore */
            }
        };
    }
}

/** Attach the click → DOM event dispatch for the bubble's hit zones. */
function attachBuySellClicks(entry) {
    entry.chart.subscribeClick((primitiveId) => {
        const id = String(primitiveId);
        const action = id.endsWith(':buy') ? 'buy'
            : id.endsWith(':sell') ? 'sell'
            : id.endsWith(':qty') ? 'qty'
            : null;
        if (!action) return;
        window.dispatchEvent(
            new CustomEvent('honba-order-click', {
                detail: { action, symbol: entry.symbol, qty: (entry.buySell && entry.buySell._qty) ?? 25 },
            }),
        );
    });
}

/** The in-chart corner bubble (`trade` primitive), labs' openalgo parity. */
function addBuySell(id, qty, symbol) {
    const entry = widgets.get(id);
    if (!entry) return false;
    entry.symbol = (symbol || '').toUpperCase();
    try {
        const btns = new BuySellButtons({
            id: 'trade',
            position: 'top-left',
            margin: { x: 14, y: 44 },
            qty,
            scale: 0.72,
            buyColor: entry.theme.buy,
            sellColor: entry.theme.sell,
        });
        entry.chart.addPrimitive(btns, 0);
        attachBuySellBubbleCursorFreeze(entry.chart, btns);
        ensureBuySellBubbleOnTop(entry.chart, btns);
        attachBuySellClicks(entry);
        entry.buySell = btns;
        return true;
    } catch (error) {
        console.warn('honbaCharts: BuySellButtons unavailable', error);
        return false;
    }
}

function setQuantity(id, qty) {
    const entry = widgets.get(id);
    if (!entry) return false;
    try {
        if (entry.buySell) {
            entry.buySell.setQty(qty);
            return true;
        }
        return false;
    } catch {
        return false;
    }
}

/* ---------- Drawing toolkit ---------- */

const CURSOR_ALIAS = ['', 'cursor', 'none', 'cross'];

function setDrawingTool(id, tool) {
    const entry = widgets.get(id);
    if (!entry || !entry.draw) return false;
    try {
        entry.draw.setTool(CURSOR_ALIAS.includes(tool) ? null : tool);
        return true;
    } catch (error) {
        console.warn('honbaCharts: setDrawingTool refused', tool, error);
        return false;
    }
}

function drawingTool(id) {
    const entry = widgets.get(id);
    if (!entry || !entry.draw) return 'cursor';
    try {
        return entry.draw.activeTool() ?? 'cursor';
    } catch {
        return 'cursor';
    }
}

function undoDrawing(id) {
    const entry = widgets.get(id);
    if (!entry || !entry.draw) return false;
    try {
        return entry.draw.undo();
    } catch {
        return false;
    }
}

function redoDrawing(id) {
    const entry = widgets.get(id);
    if (!entry || !entry.draw) return false;
    try {
        return entry.draw.redo();
    } catch {
        return false;
    }
}

function clearDrawings(id) {
    const entry = widgets.get(id);
    if (!entry || !entry.draw) return false;
    try {
        const ids = entry.draw.drawings().map((drawing) => drawing.id);
        if (ids.length > 0) entry.draw.removeMany(ids, { force: true });
        return true;
    } catch (error) {
        console.warn('honbaCharts: clearDrawings failed', error);
        return false;
    }
}

/** Publish the pane's instrument so drawings persist per symbol. */
function setSymbolContext(id, symbol) {
    const entry = widgets.get(id);
    if (!entry) return false;
    const name = (symbol || '').trim().toUpperCase();
    if (name === '') return false;
    try {
        entry.symbol = name;
        entry.chart.setDataContext({ symbol: name, exchange: 'NSE' });
        return true;
    } catch (error) {
        console.warn('honbaCharts: setDataContext failed', error);
        return false;
    }
}

/* ---------- Theme ---------- */

function refreshThemeAll() {
    if (widgets.size === 0) return;
    const theme = themeFromSkin();

    for (const entry of widgets.values()) {
        entry.theme = theme;
        if (entry.widget) {
            entry.widget.setTheme(theme);
        } else {
            /* Static (non-widget) chart: re-style its host-pushed series. */
            entry.chart.setTheme(theme);
            for (const series of entry.series) {
                try { series.api.applyOptions(seriesStyle(series.kind, series.style, theme)); } catch { /* ignore */ }
            }
        }
        if (entry.buySell) {
            try {
                entry.buySell.setColors(theme.buy, theme.sell);
            } catch {
                /* ignore */
            }
        }
    }
}

/* ---------- Indicator API ---------- */

function catalogEntry(definition) {
    return {
        id: definition.id,
        name: definition.name,
        category: definition.category || 'Other',
        placement: definition.placement === 'pane' ? 'pane' : 'onchart',
        inputs: definition.inputs || [],
    };
}

function indicatorCatalog() {
    return JSON.stringify(BUILTIN_INDICATORS.map(catalogEntry));
}

function chartIndicators(id) {
    const entry = widgets.get(id);
    if (!entry) return '[]';
    try {
        return JSON.stringify(entry.widget.chart.indicators().map((instance) => ({
            instanceId: instance.id,
            indicatorId: instance.indicatorId ?? instance.id,
            name: instance.name ?? instance.indicatorId ?? instance.id,
        })));
    } catch {
        return '[]';
    }
}

function addIndicator(id, indicatorId, settingsJson, instanceId) {
    const entry = widgets.get(id);
    if (!entry) return null;

    try {
        let settings;
        if (settingsJson) {
            const parsed = JSON.parse(settingsJson);
            if (parsed && Object.keys(parsed).length > 0) settings = parsed;
        }
        const options = instanceId ? { instanceId } : {};
        entry.widget.chart.addIndicator(indicatorId, settings, options);

        const instances = entry.widget.chart.indicators();
        const live = instances
            .filter((instance) => (instance.indicatorId ?? instance.id) === indicatorId)
            .find((instance) => !instanceId || instance.id === instanceId) ?? instances.at(-1);
        if (!live) return null;
        return JSON.stringify({ instanceId: live.id, indicatorId, name: live.name ?? indicatorId });
    } catch {
        return null;
    }
}

function removeIndicator(id, instanceId) {
    const entry = widgets.get(id);
    if (!entry) return false;
    try {
        const live = entry.widget.chart.indicators().find((instance) => instance.id === instanceId);
        if (!live) return false;
        live.remove();
        return true;
    } catch {
        return false;
    }
}

/* ---------- TradingView-style IST time axis ---------- */

const SHORT_MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
const SHORT_WEEKDAYS = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
const IST_OFFSET_SEC = 19800;

function pad2(n) { return n < 10 ? `0${n}` : String(n); }

function getDateParts(utcSec, tz) {
    if (!tz || tz === 'Asia/Kolkata' || tz === 'Asia/Calcutta') {
        const d = new Date((utcSec + IST_OFFSET_SEC) * 1000);
        return { year: d.getUTCFullYear(), month: d.getUTCMonth() + 1, day: d.getUTCDate(), hour: d.getUTCHours(), minute: d.getUTCMinutes(), second: d.getUTCSeconds(), weekday: d.getUTCDay() };
    }
    const d = new Date(utcSec * 1000);
    return { year: d.getFullYear(), month: d.getMonth() + 1, day: d.getDate(), hour: d.getHours(), minute: d.getMinutes(), second: d.getSeconds(), weekday: d.getDay() };
}

function paintTradingViewTimeAxis(ctx, pane, rc, tz) {
    const ts = rc && rc.timeScale;
    const dl = rc && rc.dataLayer;
    if (!ts || !dl || typeof ts.visibleRange !== 'function') return;

    const layout = typeof pane._layout === 'function' ? pane._layout(rc) : null;
    if (!layout || layout.plotWidth <= 0) return;

    const dpr = rc.dpr ?? 1;
    const vr = ts.visibleRange();
    const fromIdx = Math.max(0, Math.floor(vr.from));
    const toIdx = Math.min(dl.baseIndex ?? 0, Math.ceil(vr.to));
    if (toIdx < fromIdx) return;

    const barSpacing = Math.max(1, ts.barSpacing || 6);
    const step = Math.max(1, Math.round(72 / barSpacing));
    const axisTopY = Math.round(layout.plotHeight * dpr);
    const plotWidthPx = Math.round(layout.plotWidth * dpr);

    let minBarStepSec = Number.POSITIVE_INFINITY;
    let prevBarSec = dl.indexToTime(fromIdx);
    for (let i = fromIdx + 1; i <= toIdx; i++) {
        const sec = dl.indexToTime(i);
        if (sec !== undefined && prevBarSec !== undefined && sec > prevBarSec) {
            minBarStepSec = Math.min(minBarStepSec, sec - prevBarSec);
        }
        if (sec !== undefined) prevBarSec = sec;
    }
    const isIntraday = Number.isFinite(minBarStepSec) && minBarStepSec < 86400;
    const showSeconds = Number.isFinite(minBarStepSec * step) && minBarStepSec * step < 60;

    const fontSize = Math.max(10, Math.min(14, Math.round((rc.canvasOptions && rc.canvasOptions.scales && rc.canvasOptions.scales.fontSize) ?? (rc.theme && rc.theme.axisFontSize) ?? 11)));
    const textColor = (rc.canvasOptions && rc.canvasOptions.scales && rc.canvasOptions.scales.textColor) ?? (rc.theme && rc.theme.axisText) ?? '#8b91a7';
    const lineColor = (rc.canvasOptions && rc.canvasOptions.scales && rc.canvasOptions.scales.lineColor) ?? (rc.theme && rc.theme.axisLine) ?? '#2a3046';
    const regularFont = `${fontSize * dpr}px system-ui, sans-serif`;
    const boldFont = `600 ${fontSize * dpr}px system-ui, sans-serif`;

    ctx.save();
    if (layout.plotLeft > 0) ctx.translate(Math.round(layout.plotLeft * dpr), 0);
    ctx.fillStyle = textColor;
    ctx.strokeStyle = lineColor;
    ctx.font = regularFont;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'top';
    ctx.lineWidth = 1;

    ctx.beginPath();
    ctx.moveTo(0, axisTopY + 0.5);
    ctx.lineTo(plotWidthPx, axisTopY + 0.5);
    ctx.stroke();

    const candidates = [];
    const addedIndices = new Set();

    const pushTick = (idx, text, priority) => {
        if (addedIndices.has(idx)) return;
        const x = Math.round(ts.indexToX(idx) * dpr);
        if (x < 12 * dpr || x > plotWidthPx - 12 * dpr) return;
        ctx.font = priority >= 2 ? boldFont : regularFont;
        const half = ctx.measureText(text).width / 2;
        candidates.push({ idx, x, text, half, priority });
        addedIndices.add(idx);
    };

    if (!isIntraday) {
        const monthGroups = new Map();
        let prevParts = null;
        const prevStartSec = fromIdx > 0 ? dl.indexToTime(fromIdx - 1) : undefined;
        if (prevStartSec !== undefined) prevParts = getDateParts(prevStartSec, tz);

        for (let i = fromIdx; i <= toIdx; i++) {
            const sec = dl.indexToTime(i);
            if (sec === undefined) continue;
            const p = getDateParts(sec, tz);
            const mKey = `${p.year}-${p.month}`;
            let grp = monthGroups.get(mKey);
            if (!grp) { grp = { firstIdx: i, indices: [], parts: p }; monthGroups.set(mKey, grp); }
            grp.indices.push(i);

            if (prevParts && (p.year !== prevParts.year || p.month !== prevParts.month)) {
                if (p.year !== prevParts.year && p.month === 1) pushTick(i, String(p.year), 3);
                else pushTick(i, SHORT_MONTHS[p.month - 1], 2);
            } else if (i === fromIdx && !prevParts) {
                pushTick(i, SHORT_MONTHS[p.month - 1], 2);
            }
            prevParts = p;
        }

        const targetSubTickPx = 76 * dpr;
        for (const grp of monthGroups.values()) {
            const idxs = grp.indices;
            if (idxs.length < 2) continue;
            const startX = ts.indexToX(idxs[0]) * dpr;
            const endX = ts.indexToX(idxs[idxs.length - 1]) * dpr;
            const spanPx = endX - startX;
            const segments = Math.floor(spanPx / targetSubTickPx);
            if (segments >= 2) {
                for (let s = 1; s < segments; s++) {
                    const pos = Math.round((s * (idxs.length - 1)) / segments);
                    const barIdx = idxs[pos];
                    const sec = dl.indexToTime(barIdx);
                    if (sec === undefined) continue;
                    const p = getDateParts(sec, tz);
                    pushTick(barIdx, String(p.day), 0);
                }
            }
        }
    } else {
        const dayGroups = new Map();
        let prevParts = null;
        const prevStartSec = fromIdx > 0 ? dl.indexToTime(fromIdx - 1) : undefined;
        if (prevStartSec !== undefined) prevParts = getDateParts(prevStartSec, tz);

        for (let i = fromIdx; i <= toIdx; i++) {
            const sec = dl.indexToTime(i);
            if (sec === undefined) continue;
            const p = getDateParts(sec, tz);
            const dKey = `${p.year}-${p.month}-${p.day}`;
            let grp = dayGroups.get(dKey);
            if (!grp) { grp = []; dayGroups.set(dKey, grp); }
            grp.push(i);

            if (prevParts) {
                if (p.year !== prevParts.year || p.month !== prevParts.month) pushTick(i, SHORT_MONTHS[p.month - 1], 2);
                else if (p.day !== prevParts.day) pushTick(i, String(p.day), 1);
            } else if (i === fromIdx) {
                pushTick(i, `${p.day} ${SHORT_MONTHS[p.month - 1]}`, 2);
            }
            prevParts = p;
        }

        const targetTimeTickPx = 80 * dpr;
        for (const idxs of dayGroups.values()) {
            if (idxs.length < 2) continue;
            const startX = ts.indexToX(idxs[0]) * dpr;
            const endX = ts.indexToX(idxs[idxs.length - 1]) * dpr;
            const spanPx = endX - startX;
            const segments = Math.floor(spanPx / targetTimeTickPx);
            if (segments >= 2) {
                for (let s = 1; s < segments; s++) {
                    const pos = Math.round((s * (idxs.length - 1)) / segments);
                    const barIdx = idxs[pos];
                    const sec = dl.indexToTime(barIdx);
                    if (sec === undefined) continue;
                    const p = getDateParts(sec, tz);
                    const timeLabel = showSeconds ? `${pad2(p.hour)}:${pad2(p.minute)}:${pad2(p.second)}` : `${pad2(p.hour)}:${pad2(p.minute)}`;
                    pushTick(barIdx, timeLabel, 0);
                }
            }
        }
    }

    const byPriority = [...candidates].sort((a, b) => b.priority - a.priority || a.x - b.x);
    const minGap = 14 * dpr;
    const accepted = [];

    for (const cand of byPriority) {
        const fits = accepted.every((other) => cand.x + cand.half + minGap <= other.x - other.half || cand.x - cand.half - minGap >= other.x + other.half);
        if (fits) accepted.push(cand);
    }

    accepted.sort((a, b) => a.x - b.x);

    for (const tk of accepted) {
        ctx.font = tk.priority >= 2 ? boldFont : regularFont;
        ctx.fillText(tk.text, tk.x, axisTopY + 4 * dpr);
    }

    ctx.restore();
}

function attachTradingViewTimeFormatter(chart) {
    if (!chart) return;

    if (typeof chart.setTimeFormatter === 'function') {
        chart.setTimeFormatter((utcSeconds) => {
            const tz = typeof chart.timezone === 'function' ? chart.timezone() : 'Asia/Kolkata';
            const parts = getDateParts(utcSeconds, tz);
            let label = `${SHORT_WEEKDAYS[parts.weekday]} ${pad2(parts.day)} ${SHORT_MONTHS[parts.month - 1]} '${String(parts.year).slice(-2)}`;
            if (parts.hour !== 0 || parts.minute !== 0 || parts.second !== 0) {
                label += ` ${pad2(parts.hour)}:${pad2(parts.minute)}`;
                if (parts.second !== 0) label += `:${pad2(parts.second)}`;
            }
            return label;
        });
    }

    const panes = chart._panes;
    if (!Array.isArray(panes)) return;

    for (const pane of panes) {
        if (!pane || pane.__honbaTimeAxisPatched || typeof pane.paintBase !== 'function') continue;
        pane.__honbaTimeAxisPatched = true;
        const origPaintBase = pane.paintBase.bind(pane);
        pane.paintBase = function (rc, targetCtx) {
            const wantTimeAxis = Boolean(rc && rc.showTimeAxis);
            const patchedRc = wantTimeAxis ? { ...rc, showTimeAxis: false } : rc;
            origPaintBase(patchedRc, targetCtx);
            if (wantTimeAxis) {
                try {
                    const ctx = targetCtx ?? (this._backend && this._backend.overlay2d ? this._backend.overlay2d() : null) ?? this.base?.ctx;
                    if (ctx) {
                        const tz = rc.timezone ?? (typeof chart.timezone === 'function' ? chart.timezone() : 'Asia/Kolkata');
                        paintTradingViewTimeAxis(ctx, this, rc, tz);
                    }
                } catch { /* ignore */ }
            }
        };
    }
}

/* ---------- Widget mount / destroy ---------- */

/* Calendar months are not a built-in token in this library build ("1M"
   throws UnknownIntervalError at createWidget). They are registered as
   calendar buckets under `<n>mo` codes, NOT `<n>M`: the registry matches
   codes case-insensitively, so a registered "1M" shadows the built-in "1m"
   and turns one-minute charts monthly. intervalParts('1mo') reads unit 'M',
   which the feed maps to honba-ya's `mo`. Rust keeps saying "1M"; see
   canonicalInterval / honbaInterval. Idempotent across module re-imports. */
/** The `<count>mo` calendar-month code, registered on first use (a custom
 *  "2M" picked in the widget comes back from Rust as "2M" on every later
 *  remount, so registering only 1/3/6 here would fail those mounts). Null
 *  when the code cannot be registered. */
function ensureMonthInterval(count) {
    const code = `${count}mo`;
    try {
        if (!isKnownInterval(code)) registerInterval({ code, bucketing: { mode: 'calendar', unit: 'month', count } });
        return code;
    } catch (error) {
        console.warn('honbaCharts: could not register month interval', code, error);
        return null;
    }
}
for (const count of [1, 3, 6]) ensureMonthInterval(count);

/* The four grouped interval menus that replace the native flat pill row
   (the native top bar has no grouped or custom-interval control; see
   mountIntervalGroups). `root` is the group's label when the chart's
   interval is in another group. */
const INTERVAL_GROUPS = [
    { id: 'minutes', title: 'Minutes', root: '15m', units: ['s', 'm'], items: ['1m', '3m', '5m', '15m', '30m', '45m'], hint: '7m, 90s, or minutes as a number' },
    { id: 'hours', title: 'Hours', root: '1h', units: ['h'], items: ['1h', '2h', '3h', '4h'], hint: '6h, 12h' },
    { id: 'days', title: 'Days', root: '1d', units: ['D'], items: ['1d', '3d', '5d'], hint: '2d, 10d' },
    { id: 'weeks', title: 'Weeks & months', root: '1w', units: ['W', 'M'], items: ['1w', '3w', '1mo', '3mo', '6mo'], hint: '2w, 2M' },
];

/* Every grouped code, handed to the widget as its interval list so its own
   keyboard / state paths know them (the flat pill row itself is hidden). */
const TOPBAR_INTERVALS = INTERVAL_GROUPS.flatMap((group) => group.items);

/** Library interval code for a honba-kao timeframe label (see mount()):
 *  lower-case, with honba's month labels ("1M") mapped onto the registered
 *  calendar-month codes ("1mo"). */
function canonicalInterval(code) {
    const raw = String(code).trim();
    const month = /^(\d*)M$/.exec(raw);
    if (!month) return raw.toLowerCase();
    const count = Math.max(1, Number(month[1] || 1));
    return ensureMonthInterval(count) ?? `${count}mo`;
}

/** The reverse, for what goes back to Rust: "1mo" -> "1M" (Timeframe::from_id). */
function honbaInterval(code) {
    const month = /^(\d+)mo$/.exec(String(code));
    return month ? `${month[1]}M` : code;
}

/** The widget chart type for a pane mode id (`ChartMode::as_id`): any
 *  registered price-series renderer or series transform, else candlestick. */
function initialChartTypeFor(mode) {
    if (typeof mode !== 'string' || mode === '') return 'candlestick';
    try { if (getChartType(mode).isPriceSeries) return mode; } catch { /* unknown renderer */ }
    try { if (registeredSeriesTransforms().includes(mode)) return mode; } catch { /* ignore */ }
    return 'candlestick';
}

/** Short label for an interval code: `15m`, `1H`, `1D`, `1W`, `3M`. */
function intervalChipLabel(code) {
    const parts = intervalParts(code);
    if (!parts) return code;
    const unit = { s: 's', m: 'm', h: 'H', D: 'D', W: 'W', M: 'M' }[parts.unit];
    return unit ? `${parts.multiplier}${unit}` : code;
}

/** Long label for a menu row: `15 minutes`, `1 hour`, `3 months`. */
function intervalRowLabel(code) {
    const parts = intervalParts(code);
    if (!parts) return code;
    const names = { s: 'second', m: 'minute', h: 'hour', D: 'day', W: 'week', M: 'month' };
    const name = names[parts.unit];
    if (!name) return code;
    return `${parts.multiplier} ${name}${parts.multiplier === 1 ? '' : 's'}`;
}

/** The group an interval belongs to, by its unit (null for tick/volume bars). */
function intervalGroupOf(code) {
    const parts = intervalParts(code);
    return parts ? INTERVAL_GROUPS.find((group) => group.units.includes(parts.unit)) ?? null : null;
}

/**
 * Parse what a user typed into a "Custom…" row into a library interval code,
 * or null. Accepts `7m`, `2H`, `3d`, `2w`, `2M` (capital M = months, any
 * other unit case-insensitive), `90s`, and a bare number of minutes.
 */
function parseCustomInterval(text) {
    const raw = String(text ?? '').trim();
    const match = /^(\d+)\s*([a-zA-Z]*)$/.exec(raw);
    if (!match) return null;
    const count = Number(match[1]);
    if (!Number.isInteger(count) || count < 1) return null;
    const unit = match[2];
    let code;
    let unitSeconds;
    if (unit === '') { code = `${count}m`; unitSeconds = 60; }
    else if (unit === 'M' || /^mo(n(th)?s?)?$/i.test(unit)) { code = `${count}mo`; unitSeconds = 2_592_000; }
    else if (/^(s|m|h|d|w)$/i.test(unit)) {
        code = `${count}${unit.toLowerCase()}`;
        unitSeconds = { s: 1, m: 60, h: 3_600, d: 86_400, w: 604_800 }[unit.toLowerCase()];
    }
    else return null;
    /* The pane store's Timeframe::parse_custom only holds 1 s ..= 2 years
       (months counted as 30 days); a longer code would come back to Rust as
       an unparseable id and silently fall back to 1D in the saved pane. */
    if (count * unitSeconds > 63_072_000) return null;
    if (code.endsWith('mo') && ensureMonthInterval(count) === null) return null;
    return isKnownInterval(code) ? code : null;
}

/**
 * Icon-only Go to / Objects / Alerts in the native top bar. The library builds
 * these as text-only buttons and has no compact option, so the label text is
 * swapped for the library's own chrome glyph; the aria-label (the accessible
 * name) is kept. Objects and Alerts get a title as their hover label; Go to
 * already has the library's own tooltip (which strips `title`), so a title
 * there would show a second, native tooltip on top of it.
 */
function iconizeTopbarButtons(widget) {
    const topbar = widget.root.querySelector('.oac-topbar');
    if (!topbar) return;
    for (const [selector, icon, hasLibraryTip] of [['.oac-topbar__goto', 'calendar', true], ['.oac-topbar__objects', 'panels', false], ['.oac-topbar__alerts', 'bell', false]]) {
        const button = topbar.querySelector(selector);
        if (!button) continue;
        const label = button.getAttribute('aria-label') || button.textContent.trim();
        const glyph = widget.root.ownerDocument.createElement('span');
        glyph.className = 'oac-glyph oac-glyph--chrome';
        glyph.setAttribute('aria-hidden', 'true');
        glyph.innerHTML = chromeIconSvg(icon);
        button.replaceChildren(glyph);
        button.classList.add('oac-btn--icon');
        if (label) {
            button.setAttribute('aria-label', label);
            if (!hasLibraryTip && !button.title) button.title = label;
        }
    }
}

/**
 * A drawer for the rail's secondary controls (magnet, keep-armed, lock, hide,
 * delete-all, undo/redo: the library's `.oac-rail__ctl` block). A small
 * toggle goes in above the block; CSS shows the toggle and folds the block
 * away only when the chart host is short (same container query as the
 * short-pane rail fix in workbench.css), so on roomy panes the rail is the
 * library's own. Opening the drawer unfolds the block in place and scrolls
 * it into view; the drawing-tool groups above stay where they are.
 */
function mountRailDrawer(widget) {
    const rail = widget.root.querySelector('.oac-rail');
    if (!rail || !rail.querySelector('.oac-rail__ctl')) return;
    const doc = widget.root.ownerDocument;
    const toggle = doc.createElement('button');
    toggle.type = 'button';
    toggle.className = 'oac-rail__btn honba-rail-drawer';
    toggle.setAttribute('aria-expanded', 'false');
    toggle.setAttribute('aria-label', 'More drawing controls');
    toggle.title = 'More drawing controls';
    toggle.innerHTML = chromeIconSvg('more');
    toggle.addEventListener('click', () => {
        const open = rail.classList.toggle('honba-rail--drawer-open');
        toggle.setAttribute('aria-expanded', String(open));
        const ctl = rail.querySelector('.oac-rail__ctl');
        if (open && ctl) ctl.scrollIntoView({ block: 'nearest' });
    });
    /* The library rebuilds the rail's contents wholesale (`restorePrefs`, e.g.
       on loading a saved layout): that drops the toggle and builds a new
       `.oac-rail__ctl`, which the short-pane CSS would then keep hidden with
       no way to open it. Put the toggle back in front of the current block
       whenever the rail's children change. */
    const place = () => {
        const ctl = rail.querySelector('.oac-rail__ctl');
        if (ctl && toggle.nextElementSibling !== ctl) ctl.parentNode.insertBefore(toggle, ctl);
    };
    place();
    new MutationObserver(place).observe(rail, { childList: true });
}

/**
 * Replace the native top bar's flat interval pills with four grouped menus
 * (15m / 1H / 1D / 1W), each listing its siblings plus a "Custom…" entry.
 * Everything else in the native top bar stays as the library renders it.
 * Selection goes through `widget.setInterval`, so the widget's own
 * 'interval' event (and the honba-chart-spec store sync) fire as before.
 * Menus are the library's own `openMenu`; the custom input is a panel on
 * the widget's overlay stack, so Escape / outside-press / focus behave like
 * every other popover in the widget.
 */
function mountIntervalGroups(widget, alive) {
    const topbar = widget.root.querySelector('.oac-topbar');
    const pills = topbar && topbar.querySelector('.oac-pills');
    if (!topbar || !pills) return;
    const doc = widget.root.ownerDocument;
    const ctx = widget.context;

    const bar = doc.createElement('div');
    bar.className = 'oac-pills honba-tf';
    bar.setAttribute('role', 'group');
    bar.setAttribute('aria-label', 'Interval');
    pills.parentNode.insertBefore(bar, pills);

    const setInterval = (code) => {
        if (!alive()) return;
        try { widget.setInterval(code); } catch (error) { console.warn('honbaCharts: interval refused', code, error); }
    };

    const openCustom = (button, group) => {
        const panel = doc.createElement('div');
        panel.className = 'oac-pop honba-tf-custom';
        const label = doc.createElement('label');
        label.textContent = `Custom interval (${group.hint})`;
        const row = doc.createElement('div');
        row.className = 'honba-tf-custom__row';
        const input = doc.createElement('input');
        input.type = 'text';
        input.autocomplete = 'off';
        input.spellcheck = false;
        input.placeholder = group.hint;
        input.setAttribute('aria-label', 'Custom interval');
        const ok = doc.createElement('button');
        ok.type = 'button';
        ok.className = 'oac-btn';
        ok.textContent = 'Apply';
        const error = doc.createElement('p');
        error.className = 'honba-tf-custom__error';
        error.hidden = true;
        row.append(input, ok);
        label.appendChild(row);
        panel.append(label, error);

        let close = () => {};
        const commit = () => {
            const code = parseCustomInterval(input.value);
            if (!code) {
                error.textContent = 'Use a number and a unit: s, m, h, d, w or M (months), up to 2 years';
                error.hidden = false;
                input.focus();
                return;
            }
            close();
            setInterval(code);
        };
        input.addEventListener('keydown', (event) => {
            if (event.key === 'Enter') { event.preventDefault(); commit(); }
        });
        ok.addEventListener('click', commit);
        close = ctx.openOverlay(panel, { anchor: button, placement: 'below', dismissOnOutside: true, initialFocus: input });
    };

    const chips = INTERVAL_GROUPS.map((group) => {
        const button = doc.createElement('button');
        button.type = 'button';
        button.dataset.group = group.id;
        button.setAttribute('aria-haspopup', 'menu');
        const text = doc.createElement('span');
        const chev = doc.createElement('span');
        chev.className = 'honba-tf__chev';
        chev.setAttribute('aria-hidden', 'true');
        chev.textContent = '▾';
        button.append(text, chev);
        button.addEventListener('click', () => {
            const current = widget.interval();
            const rows = group.items
                .filter((code) => isKnownInterval(code))
                .map((code) => ({ label: intervalRowLabel(code), key: intervalChipLabel(code), on: code === current, onSelect: () => setInterval(code) }));
            if (!group.items.includes(current) && intervalGroupOf(current) === group) {
                rows.push({ label: intervalRowLabel(current), key: intervalChipLabel(current), on: true, onSelect: () => {} });
            }
            rows.push({ label: 'Custom…', onSelect: () => openCustom(button, group) });
            openMenu(ctx, button, [group.title, ...rows], { ariaLabel: `${group.title} intervals` });
        });
        bar.appendChild(button);
        return { group, button, text };
    });

    const refresh = () => {
        const current = widget.interval();
        const owner = intervalGroupOf(current);
        for (const { group, button, text } of chips) {
            const active = owner === group;
            text.textContent = intervalChipLabel(active ? current : group.root);
            button.setAttribute('aria-pressed', String(active));
            button.title = active ? `Interval: ${intervalRowLabel(current)}` : `${group.title} intervals`;
        }
    };
    refresh();
    widget.on('interval', refresh);
}

async function mount(element, options = {}) {
    const theme = themeFromSkin();
    const initialChartType = initialChartTypeFor(options.chartMode);
    // Canonical interval code: the library's codes are lower-case ("1d",
    // "1w") except the calendar month ("1M", which must not collapse into
    // "1m" = one minute). Rust hands over display labels ("1D", "1W"); the
    // top bar matches its pills by exact code, so a raw "1D" showed up as a
    // second "D" pill beside the canonical "1d" one.
    const normInterval = canonicalInterval(options.timeframe || '1d');
    const initialMeta = resolveSymbolMetadata(options.symbol);

    const widget = createWidget(element, {
        feed: honbaFeed,
        symbol: options.symbol?.toUpperCase(),
        exchange: initialMeta.exchange || 'NSE',
        interval: normInterval,
        theme,
        chartType: initialChartType,
        // The library's own drawing rail: every registered tool (~100, grouped
        // with flyouts), plus magnet / keep-armed / favourites. honba-kao's old
        // custom rail only exposed nine of them.
        rail: true,
        // Drawings persist per instrument in localStorage, through the
        // widget's own InstrumentDrawings (same key prefix as before).
        drawingStore: webStorageDrawingStore(window.localStorage, 'honba:charts:drawings:'),
        // The library's native top bar: symbol search, intervals, chart type,
        // indicators, objects, settings, capture (PNG/SVG/CSV/clipboard),
        // alerts, go-to-date and theme. Replaces honba-kao's hand-rolled bar.
        topbar: true,
        // The pills the native top bar shows (TOPBAR_INTERVALS). Without this
        // the library shows its short default list only.
        intervals: TOPBAR_INTERVALS,
        // No "Chart by OpenAlgo" link in the top bar (nor the OpenAlgo corner
        // mark): the documented ChartOptions.branding switch. The top bar's
        // link is derived from these branding options and drops when false.
        branding: false,
        // honba-kao is a desktop terminal: a narrow pane (2x2 grid, ticket
        // open) must keep the desktop rail and top bar instead of switching to
        // the phone chrome, which floats its controls over the chart.
        mobile: 'never',
        statusline: true,
        panels: false,
        bottombar: false,
    });

    await widget.ready;

    /* Attach TradingView time axis to the inner chart. */
    try { attachTradingViewTimeFormatter(widget.chart); } catch { /* ignore */ }

    /* Drawing toolkit: the widget's own controller (the one its rail arms),
       persisted per symbol via `drawingStore` above. A second controller here
       would split the rail's drawings from the bridge's undo/redo/clear. */
    try {
        widget.chart.on('draw:tool', (what) => {
            window.dispatchEvent(new CustomEvent('honba-drawing-tool', {
                detail: { chartId: id, tool: (what && what.tool) ?? null },
            }));
        });
    } catch (error) {
        console.warn('honbaCharts: drawing toolkit unavailable', error);
    }

    const id = nextId++;
    const entry = { widget, chart: widget.chart, draw: widget.draw, drawingsDoc: widget.instrumentDrawings, series: [], bars: [], volume: null, buySell: null, symbol: options.symbol?.toUpperCase() || '', theme };
    widgets.set(id, entry);

    /* The native top bar changes symbol / interval / chart type inside the
       widget; publish each change so the Rust pane store (order ticket, saved
       pane config) follows it. Rust skips the remount when the store catches
       up to what the widget already shows. */
    const publishSpec = () => {
        if (!widgets.has(id)) return;
        entry.symbol = (widget.symbol() || '').toUpperCase();
        window.dispatchEvent(new CustomEvent('honba-chart-spec', {
            detail: { chartId: id, symbol: entry.symbol, interval: honbaInterval(widget.interval()), chartType: widget.chartType() },
        }));
    };
    /* One theme: the chart follows honba-kao's app theme (refreshThemeAll on
       every skin change). The native bar's own theme button is hidden in CSS;
       if a shortcut flips the widget anyway, put the app theme back. */
    try {
        widget.on('theme', () => {
            if (!widgets.has(id)) return;
            /* Compare against the mode the widget itself derives from the app
               theme (background luminance), not `data-mode`: `setTheme` emits
               'theme' synchronously, so if the two ever disagreed (a custom
               skin) this handler would re-enter itself without end. */
            const appTheme = themeFromSkin();
            if (widget.theme() !== themeMode(appTheme)) {
                entry.theme = appTheme;
                widget.setTheme(appTheme);
            }
        });
    } catch (error) {
        console.warn('honbaCharts: theme event unavailable', error);
    }

    /* Grouped interval menus in place of the native pill row. */
    try {
        mountIntervalGroups(widget, () => widgets.has(id));
    } catch (error) {
        console.warn('honbaCharts: interval menus unavailable', error);
    }

    /* Icon-only Go to / Objects / Alerts; secondary rail controls in a drawer. */
    try { iconizeTopbarButtons(widget); } catch (error) { console.warn('honbaCharts: top bar icons unavailable', error); }
    try { mountRailDrawer(widget); } catch (error) { console.warn('honbaCharts: rail drawer unavailable', error); }

    try {
        widget.on('symbol', publishSpec);
        widget.on('interval', publishSpec);
        widget.on('layout', (what) => { if (what && what.chartType) publishSpec(); });
    } catch (error) {
        console.warn('honbaCharts: widget events unavailable', error);
    }

    /* Volume overlay in primary pane. */
    try {
        const volSeries = widget.chart.addSeries('histogram', {
            paneIndex: 0, priceScaleId: '', style: { color: VOLUME_COLOR }, priceFormat: { type: 'volume' },
        });
        volSeries.priceScale().setOptions({ marginTop: 0.82, marginBottom: 0 });
        volSeries.setData(volumeRows(entry.bars || []));
        entry.volume = { api: volSeries };
    } catch { /* ignore */ }

    /* Buy/Sell bubble. */
    try {
        const btns = new BuySellButtons({
            id: 'trade', position: 'top-left', margin: { x: 14, y: 44 },
            qty: options.quantity ?? 25, scale: 0.72,
            buyColor: theme.buy, sellColor: theme.sell,
        });
        widget.chart.addPrimitive(btns, 0);
        attachBuySellBubbleCursorFreeze(widget.chart, btns);
        ensureBuySellBubbleOnTop(widget.chart, btns);
        attachBuySellClicks(entry);
        entry.buySell = btns;
    } catch (e) { console.warn('honbaCharts: BuySellButtons unavailable', e); }

    /* Symbol context for drawing persistence. */
    if (options.symbol) {
        try {
            entry.symbol = options.symbol.toUpperCase();
            widget.chart.setDataContext({ symbol: entry.symbol, exchange: 'NSE' });
        } catch { /* ignore */ }
    }

    return id;
}

function mountChart(element, symbol, timeframe, chartMode, quantity) {
    const options = {};
    if (typeof symbol === 'string' && symbol) options.symbol = symbol;
    if (typeof timeframe === 'string' && timeframe) options.timeframe = timeframe;
    if (typeof chartMode === 'string' && chartMode) options.chartMode = chartMode;
    if (typeof quantity === 'number' && Number.isFinite(quantity)) options.quantity = quantity;
    return mount(element, options);
}

/**
 * A plain engine chart (no widget chrome, no feed) for host-supplied series:
 * the dashboard's equity curve and comparison panels push their own bars via
 * `setSeries`. Synchronous; shares the handle space with widget mounts.
 */
function mountStatic(element) {
    const theme = themeFromSkin();
    const chart = createChart(element, { theme });
    try { attachTradingViewTimeFormatter(chart); } catch { /* ignore */ }
    const id = nextId++;
    widgets.set(id, { widget: null, chart, draw: null, drawingsDoc: null, series: [], bars: [], volume: null, buySell: null, symbol: '', theme });
    return id;
}

function setSeries(id, kind, styleJson, barsJson) {
    const entry = widgets.get(id);
    if (!entry) return false;

    const style = styleJson ? JSON.parse(styleJson) : {};
    const bars = barsJson ? JSON.parse(barsJson) : [];
    const resolved = seriesStyle(kind, style, entry.theme);

    const series = entry.chart.addSeries(kind, { style: resolved });
    series.setData(bars);
    entry.series.push({ api: series, kind, style });
    entry.bars = bars.slice();

    if (entry.volume) {
        try { entry.volume.api.setData(volumeRows(bars)); } catch { /* ignore */ }
    }

    requestAnimationFrame(() => { try { entry.chart.fitContent(); } catch { } });
    return true;
}

function updateLast(id, barJson) {
    const entry = widgets.get(id);
    if (!entry) return false;
    let bar; try { bar = JSON.parse(barJson); } catch { return false; }
    if (!bar || typeof bar.time !== 'number') return false;

    const primary = entry.series[0] && entry.series[0].api;
    if (!primary) return false;

    const last = entry.bars[entry.bars.length - 1];
    if (last && last.time === bar.time) { entry.bars[entry.bars.length - 1] = { ...bar }; }
    else { entry.bars.push({ ...bar }); }

    try { primary.update(bar); } catch { return false; }
    if (entry.volume) {
        try { entry.volume.api.update({ time: bar.time, open: 0, high: bar.volume ?? 0, low: 0, close: bar.volume ?? 0 }); } catch { }
    }
    return true;
}

function addVolumeOverlay(id) {
    const entry = widgets.get(id);
    if (!entry || entry.volume) return false;
    try {
        const api = entry.chart.addSeries('histogram', { paneIndex: 0, priceScaleId: '', style: { color: VOLUME_COLOR }, priceFormat: { type: 'volume' } });
        api.priceScale().setOptions({ marginTop: 0.82, marginBottom: 0 });
        api.setData(volumeRows(entry.bars));
        entry.volume = { api };
        return true;
    } catch (error) { console.warn('honbaCharts: volume overlay unavailable', error); return false; }
}

function destroy(id) {
    const entry = widgets.get(id);
    if (!entry) return false;
    try {
        if (entry.widget) entry.widget.destroy();
        else entry.chart.destroy();
    } catch { }
    widgets.delete(id);
    return true;
}

/* ---------- Symbol metadata (for exchange inference) ---------- */

const SYMBOL_METADATA = {
    NIFTY: { exchange: 'NSE' }, BANKNIFTY: { exchange: 'NSE' }, FINNIFTY: { exchange: 'NSE' },
    MIDCPNIFTY: { exchange: 'NSE' }, SENSEX: { exchange: 'BSE' }, BANKEX: { exchange: 'BSE' },
    RELIANCE: { exchange: 'NSE' }, INFY: { exchange: 'NSE' }, HDFCBANK: { exchange: 'NSE' }, TCS: { exchange: 'NSE' },
    ICICIBANK: { exchange: 'NSE' }, SBIN: { exchange: 'NSE' }, ITC: { exchange: 'NSE' }, LT: { exchange: 'NSE' },
    AXISBANK: { exchange: 'NSE' }, KOTAKBANK: { exchange: 'NSE' }, BHARTIARTL: { exchange: 'NSE' }, HINDUNILVR: { exchange: 'NSE' },
    BAJFINANCE: { exchange: 'NSE' }, ASIANPAINT: { exchange: 'NSE' }, MARUTI: { exchange: 'NSE' }, SUNPHARMA: { exchange: 'NSE' },
    TITAN: { exchange: 'NSE' }, ULTRACEMCO: { exchange: 'NSE' }, NESTLEIND: { exchange: 'NSE' }, POWERGRID: { exchange: 'NSE' },
    NTPC: { exchange: 'NSE' }, TECHM: { exchange: 'NSE' }, WIPRO: { exchange: 'NSE' }, ADANIENT: { exchange: 'NSE' },
    ADANIPORTS: { exchange: 'NSE' }, COALINDIA: { exchange: 'NSE' }, BPCL: { exchange: 'NSE' }, ONGC: { exchange: 'NSE' },
    TATAMOTORS: { exchange: 'NSE' }, TATASTEEL: { exchange: 'NSE' }, JSWSTEEL: { exchange: 'NSE' }, HINDALCO: { exchange: 'NSE' },
    GRASIM: { exchange: 'NSE' }, CIPLA: { exchange: 'NSE' }, DRREDDY: { exchange: 'NSE' }, APOLLOHOSP: { exchange: 'NSE' },
    DIVISLAB: { exchange: 'NSE' }, BRITANNIA: { exchange: 'NSE' }, EICHERMOT: { exchange: 'NSE' }, HEROMOTOCO: { exchange: 'NSE' },
    BAJAJ_AUTO: { exchange: 'NSE' }, TVSMOTOR: { exchange: 'NSE' }, 'M&M': { exchange: 'NSE' }, BAJAJFINSV: { exchange: 'NSE' },
    SHRIRAMFIN: { exchange: 'NSE' }, INDUSINDBK: { exchange: 'NSE' }, FEDERALBNK: { exchange: 'NSE' }, IDFCFIRSTB: { exchange: 'NSE' },
    BANDHANBNK: { exchange: 'NSE' }, PNB: { exchange: 'NSE' }, BANKBARODA: { exchange: 'NSE' }, CANBK: { exchange: 'NSE' },
    UNIONBANK: { exchange: 'NSE' }, INDIANB: { exchange: 'NSE' }, CENTRALBK: { exchange: 'NSE' }, UCOBANK: { exchange: 'NSE' },
};

function resolveSymbolMetadata(symbol) {
    return SYMBOL_METADATA[symbol?.toUpperCase()] || { exchange: 'NSE' };
}

/* ---------- Live feed: honba-ya-api-rest (real bars + WS ticks) ---------- */

/** honba-ya tf unit for each `intervalParts().unit` the widget can ask for. */
const TF_UNIT_FOR_INTERVAL_UNIT = { s: 's', m: 'm', h: 'h', D: 'd', W: 'w', M: 'mo' };

/** Fixed bucket length (seconds) for each `intervalParts().unit`, used only to
 *  size the live `TickBarAggregator`. 'tick'/'other' have no fixed length. */
const AGGREGATOR_SECONDS_FOR_INTERVAL_UNIT = { s: 1, m: 60, h: 3600, D: 86400, W: 604800, M: 2592000 };

/** Build the honba-ya `tf=<n><unit>` token for a widget interval code (e.g. "1m", "1D", "1M"). */
function honbaTimeframeToken(interval) {
    const parts = intervalParts(interval);
    if (!parts) throw new Error(`honbaFeed: unrecognized interval "${interval}"`);
    const tfUnit = TF_UNIT_FOR_INTERVAL_UNIT[parts.unit];
    if (!tfUnit) throw new Error(`honbaFeed: interval "${interval}" (unit "${parts.unit}") has no honba-ya timeframe equivalent`);
    return `${parts.multiplier}${tfUnit}`;
}

/** Fixed-length seconds to aggregate live ticks into for a widget interval code, or null when the
 *  interval isn't a fixed-length bucket (tick/volume bars) and can't be built from price ticks alone. */
function aggregationSecondsForInterval(interval) {
    const parts = intervalParts(interval);
    if (!parts) return null;
    return AGGREGATOR_SECONDS_FOR_INTERVAL_UNIT[parts.unit] ? parts.multiplier * AGGREGATOR_SECONDS_FOR_INTERVAL_UNIT[parts.unit] : null;
}

/* ---- Shared live-tick WebSocket, ref-counted across all subscribeBars() callers ---- */

let liveSocket = null;
let liveSocketOpen = null;
/** symbol (upper, no exchange) -> Set of tick callbacks currently interested in it. */
const liveSymbolListeners = new Map();

function liveWsUrl() {
    const proto = window.location.protocol === 'https:' ? 'wss' : 'ws';
    return `${proto}://${window.location.hostname}:8000/ws`;
}

function sendLiveSymbolScope() {
    if (!liveSocket || liveSocket.readyState !== WebSocket.OPEN) return;
    try { liveSocket.send(JSON.stringify({ symbols: Array.from(liveSymbolListeners.keys()) })); } catch { /* ignore */ }
}

function ensureLiveSocket() {
    if (liveSocket) return liveSocketOpen;
    const ws = new WebSocket(liveWsUrl());
    liveSocket = ws;
    liveSocketOpen = new Promise((resolve) => {
        ws.addEventListener('open', () => { sendLiveSymbolScope(); resolve(ws); });
    });
    ws.addEventListener('message', (event) => {
        let tick;
        try { tick = JSON.parse(event.data); } catch { return; }
        if (!tick || typeof tick.symbol !== 'string') return;
        const listeners = liveSymbolListeners.get(tick.symbol.toUpperCase());
        if (!listeners) return;
        for (const listener of listeners) {
            try { listener(tick); } catch (err) { console.warn('honbaFeed: live tick listener failed', err); }
        }
    });
    ws.addEventListener('close', () => {
        if (liveSocket === ws) { liveSocket = null; liveSocketOpen = null; }
    });
    ws.addEventListener('error', (err) => { console.warn('honbaFeed: live socket error', err); });
    return liveSocketOpen;
}

/** Subscribe `onTick` to raw price ticks for `symbol`. Returns an unsubscribe function. */
function subscribeLiveTicks(symbol, onTick) {
    const key = symbol.toUpperCase();
    let listeners = liveSymbolListeners.get(key);
    if (!listeners) { listeners = new Set(); liveSymbolListeners.set(key, listeners); }
    listeners.add(onTick);
    ensureLiveSocket().then(sendLiveSymbolScope);

    return () => {
        const current = liveSymbolListeners.get(key);
        if (current) {
            current.delete(onTick);
            if (current.size === 0) { liveSymbolListeners.delete(key); sendLiveSymbolScope(); }
        }
        if (liveSymbolListeners.size === 0 && liveSocket) {
            try { liveSocket.close(); } catch { /* ignore */ }
            liveSocket = null;
            liveSocketOpen = null;
        }
    };
}

const honbaFeed = {
    async getBars({ symbol, exchange, interval, from, to, signal }) {
        const meta = resolveSymbolMetadata(symbol);
        const ex = exchange || meta.exchange || 'NSE';
        const tf = honbaTimeframeToken(interval);
        const url = `/api/bars/${encodeURIComponent(symbol)}.${ex}?tf=${encodeURIComponent(tf)}`
            + (from ? `&from=${encodeURIComponent(new Date(from * 1000).toISOString())}` : '')
            + (to ? `&to=${encodeURIComponent(new Date(to * 1000).toISOString())}` : '');
        console.debug('honbaFeed.getBars: fetching', url);

        const res = await fetch(url, { signal });
        let envelope;
        try {
            envelope = await res.json();
        } catch (err) {
            throw new Error(`honbaFeed: couldn't parse bars response for ${symbol}.${ex} (HTTP ${res.status}): ${err.message}`);
        }
        if (!res.ok || envelope == null || envelope.data == null || envelope.error) {
            const message = envelope?.error?.message || `honbaFeed: HTTP ${res.status} fetching bars for ${symbol}.${ex}`;
            throw new Error(message);
        }

        const rawBars = envelope.data.bars || [];
        return rawBars.map((bar) => ({
            time: Math.floor(Date.parse(bar.timestamp) / 1000),
            open: Number(bar.open),
            high: Number(bar.high),
            low: Number(bar.low),
            close: Number(bar.close),
            volume: Number(bar.volume),
        }));
    },

    subscribeBars(req, onBar, opts) {
        const { symbol, interval } = req;
        const seconds = aggregationSecondsForInterval(interval);
        if (!seconds) {
            console.warn(`honbaFeed.subscribeBars: interval "${interval}" has no fixed-length bucket to aggregate live ticks into; live updates disabled`);
            return () => { /* no-op */ };
        }

        const aggregator = new TickBarAggregator({ mode: 'interval', seconds }, { timezone: 'Asia/Kolkata' });
        if (opts?.seedFrom) {
            try { aggregator.seed(opts.seedFrom); } catch (err) { console.warn('honbaFeed.subscribeBars: failed to seed aggregator', err); }
        }

        return subscribeLiveTicks(symbol, (tick) => {
            if (typeof tick.price !== 'number' || typeof tick.timestamp !== 'number') return;
            const { bar } = aggregator.onTick({ time: Math.floor(tick.timestamp / 1000), price: tick.price, qty: 0 });
            onBar(bar, { provisional: true });
        });
    },

    // DataFeed contract: `searchSymbols({ query, signal })` returning a Promise
    // (the native top bar's symbol box calls it). A bare string is still
    // accepted for older callers.
    async searchSymbols(request) {
        const query = String((typeof request === 'string' ? request : request?.query) ?? '').toUpperCase();
        return Object.entries(SYMBOL_METADATA)
            .filter(([sym, meta]) => sym.includes(query))
            .slice(0, 20)
            .map(([symbol, meta]) => ({ symbol, exchange: meta.exchange, name: symbol, type: 'equity' }));
    },
};

/* ---------- Public API ---------- */

window.honbaCharts = {
    version: '2.6.7',
    mount: mountChart,
    mountStatic,
    setSeries,
    updateLast,
    addVolumeOverlay,
    addBuySell,
    setQuantity,
    setDrawingTool,
    drawingTool,
    undoDrawing,
    redoDrawing,
    clearDrawings,
    setSymbolContext,
    refreshThemeAll,
    destroy,
    indicatorCatalog,
    chartIndicators,
    addIndicator,
    removeIndicator,
    registerCustomIndicatorPicker,
};

window.dispatchEvent(new Event('honba-charts-ready'));