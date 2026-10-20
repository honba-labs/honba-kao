/* ==========================================================================
 * honba-kao ⇄ openalgo-charts widget bridge
 *
 * Uses the widget tier (createWidget) which provides the full OpenAlgo
 * terminal experience: topbar, drawing rail, dialogs, indicator picker,
 * objects panel, settings, snapshot, layouts, and 502 indicators.
 *
 * The Rust side talks to window.honbaCharts.* (see openalgo_chart.rs).
 * ========================================================================== */

import { createWidget, injectWidgetStyles, WIDGET_COMPONENT_CSS, WIDGET_DIALOGS, registerWidgetDialog, registerWidgetDialogs, themeMode, mountIndicatorPicker as mountBuiltinIndicatorPicker } from './openalgo-charts.widget.mjs';
import { createChart, BuySellButtons, darkTheme, lightTheme, intervalParts, TickBarAggregator, getChartType, registeredSeriesTransforms } from './openalgo-charts.mjs';
import './openalgo-charts.draw.mjs';
import { DrawingController, InstrumentDrawings, instrumentDrawingsKey, registerBuiltinDrawingTools, webStorageDrawingStore } from './openalgo-charts.draw.mjs';
import { BUILTIN_INDICATORS } from './openalgo-charts.indicators.mjs';

/* Register drawing tools once (idempotent). */
registerBuiltinDrawingTools();

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

// honba-fe's workbench set (openalgo-chart.tsx INTERVAL_GROUPS) plus 2d/3d.
// No month pill: this library build has no built-in month interval ("1M"
// throws UnknownIntervalError at createWidget unless registerInterval()'d).
const TOPBAR_INTERVALS = ['1m', '3m', '5m', '15m', '30m', '45m', '1h', '2h', '3h', '4h', '1d', '2d', '3d', '1w'];

/** Library interval code for a honba-kao timeframe label (see mount()). */
function canonicalInterval(code) {
    const raw = String(code).trim();
    return /^\d*M$/.test(raw) ? raw : raw.toLowerCase();
}

/** The widget chart type for a pane mode id (`ChartMode::as_id`): any
 *  registered price-series renderer or series transform, else candlestick. */
function initialChartTypeFor(mode) {
    if (typeof mode !== 'string' || mode === '') return 'candlestick';
    try { if (getChartType(mode).isPriceSeries) return mode; } catch { /* unknown renderer */ }
    try { if (registeredSeriesTransforms().includes(mode)) return mode; } catch { /* ignore */ }
    return 'candlestick';
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
            detail: { chartId: id, symbol: entry.symbol, interval: widget.interval(), chartType: widget.chartType() },
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
            time: Math.floor(Date.parse(bar.ts_event.iso) / 1000),
            open: bar.open,
            high: bar.high,
            low: bar.low,
            close: bar.close,
            volume: bar.volume,
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