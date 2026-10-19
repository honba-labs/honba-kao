/* ==========================================================================
 * honba-kao ⇄ openalgo-charts bridge
 *
 * openalgo-charts ships a dependency-free ESM bundle, so the app loads it
 * directly (vendored, no CDN, no bundler) and this thin module is the only
 * JavaScript between the Leptos/WASM components and the chart engine.
 *
 * Responsibilities:
 *   1. Build a full openalgo `ChartTheme` from the active CSS skin, so all
 *      eight themes restyle the chart without touching TypeScript.
 *   2. Own the chart instances (Leptos hands out numeric handles).
 *   3. Re-theme every live chart when the app theme changes.
 *   4. Drive the TradingView-shaped extras the workbench wants on top of the
 *      plain engine: a live in-flight bar (`updateLast`), the volume
 *      histogram overlay, the corner Buy/Sell bubble, and the compact
 *      IST time axis.
 *
 * API surface used from Rust (see components/charts/openalgo_chart.rs):
 *   window.honbaCharts.mount(element)               -> id
 *   window.honbaCharts.setSeries(id, type, style, bars)
 *   window.honbaCharts.updateLast(id, bar)          // live feed tick
 *   window.honbaCharts.addVolumeOverlay(id)
 *   window.honbaCharts.addBuySell(id, qty, symbol)  // 'trade' bubble
 *   window.honbaCharts.setQuantity(id, qty)         // ticket follow-up
 *   window.honbaCharts.setSymbolContext(id, symbol) // drawings persist key
 *   window.honbaCharts.setDrawingTool(id, tool)     // '' = cursor
 *   window.honbaCharts.drawingTool(id)              -> string
 *   window.honbaCharts.undoDrawing / redoDrawing(id)
 *   window.honbaCharts.clearDrawings(id)
 *   window.honbaCharts.refreshThemeAll()
 *   window.honbaCharts.destroy(id)
 *   window.honbaCharts.indicatorCatalog()           -> JSON
 *   window.honbaCharts.chartIndicators(id)          -> JSON
 *   window.honbaCharts.addIndicator(id, indId, settingsJson, instanceId)
 *   window.honbaCharts.removeIndicator(id, instanceId)
 * ========================================================================== */

import { createChart, BuySellButtons, darkTheme, lightTheme } from './openalgo-charts.mjs';
/* Side-effect tiers: 112 built-in indicators and the drawing toolkit. */
import './openalgo-charts.draw.mjs';
import {
  DrawingController,
  InstrumentDrawings,
  instrumentDrawingsKey,
  registerBuiltinDrawingTools,
  webStorageDrawingStore,
} from './openalgo-charts.draw.mjs';
import { BUILTIN_INDICATORS } from './openalgo-charts.indicators.mjs';

/* The draw tier ships tool implementations that must be registered once
   (the function is internally idempotent). */
registerBuiltinDrawingTools();

const charts = new Map();
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
 *
 * The base palette supplies every field the engine requires (including the
 * baseline/last-price/area defaults we do not override), and the CSS tokens
 * override the ones that should follow the app theme.
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

function mount(element) {
  const theme = themeFromSkin();
  const chart = createChart(element, { theme });

  /* The TradingView-shaped IST time axis rides the engine's own paint pass;
     internals are guarded, so a bundle drift only costs the feature. */
  try {
    attachTradingViewTimeFormatter(chart);
  } catch {
    /* engine internals drifted — the stock axis is fine */
  }

  const id = nextId++;
  charts.set(id, { chart, series: [], bars: [], volume: null, buySell: null, symbol: '', theme });

  /* The drawing toolkit: one controller per chart, drawings persisted per
     symbol in localStorage and re-loaded on every fresh mount. Tool changes
     broadcast to the app so the drawing rail's pressed state follows the
     engine (Esc after placing, cancel, …). */
  try {
    const draw = new DrawingController(chart, { historyLimit: 50 });
    entry.drawings = draw;
    entry.drawingsDoc = new InstrumentDrawings(chart, draw, {
      store: webStorageDrawingStore(window.localStorage, 'honba:charts:drawings:'),
      key: instrumentDrawingsKey,
    });
    chart.on('draw:tool', (what) => {
      window.dispatchEvent(
        new CustomEvent('honba-drawing-tool', {
          detail: { chartId: id, tool: (what && what.tool) ?? null },
        }),
      );
    });
  } catch (error) {
    console.warn('honbaCharts: drawing toolkit unavailable', error);
  }

  return id;
}

function setSeries(id, kind, styleJson, barsJson) {
  const entry = charts.get(id);
  if (!entry) return false;

  const style = styleJson ? JSON.parse(styleJson) : {};
  const bars = barsJson ? JSON.parse(barsJson) : [];
  const resolved = seriesStyle(kind, style, entry.theme);

  const series = entry.chart.addSeries(kind, { style: resolved });
  series.setData(bars);
  entry.series.push({ api: series, kind, style });
  entry.bars = bars.slice();

  /* Fit once the element has a laid-out size (ResizeObserver will track it
     from then on, so this only matters for the first paint). */
  requestAnimationFrame(() => {
    try {
      entry.chart.fitContent();
    } catch {
      /* chart destroyed before the frame landed */
    }
  });

  return true;
}

/* -- Live feed ------------------------------------------------------------ */

/**
 * Update (or append) the in-flight bar: the primary series gets `update`,
 * the volume overlay (when mounted) follows, and the bridge-side bar
 * history stays aligned for a later overlay mount.
 */
function updateLast(id, barJson) {
  const entry = charts.get(id);
  if (!entry) return false;
  let bar;
  try {
    bar = JSON.parse(barJson);
  } catch {
    return false;
  }
  if (!bar || typeof bar.time !== 'number') return false;

  const primary = entry.series[0] && entry.series[0].api;
  if (!primary) return false;

  const last = entry.bars[entry.bars.length - 1];
  if (last && last.time === bar.time) {
    entry.bars[entry.bars.length - 1] = { ...bar };
  } else {
    entry.bars.push({ ...bar });
  }

  try {
    primary.update(bar);
  } catch {
    return false;
  }
  if (entry.volume) {
    try {
      entry.volume.api.update({
        time: bar.time,
        open: 0,
        high: bar.volume ?? 0,
        low: 0,
        close: bar.volume ?? 0,
      });
    } catch {
      /* overlay mid-rescale — the next tick fixes it */
    }
  }
  return true;
}

/* -- Volume overlay ------------------------------------------------------- */

/** The single translucent blue labs' workbench paints volume with. */
const VOLUME_COLOR = 'rgba(41, 98, 255, 0.28)';

/**
 * Mount the volume histogram in the primary pane's spare headroom (the
 * price scale keeps 18% for it, like the openalgo terminal). Bars mirror
 * the primary series' volume via `_histogramFrom`.
 */
function addVolumeOverlay(id) {
  const entry = charts.get(id);
  if (!entry || entry.volume) return false;
  try {
    const api = entry.chart.addSeries('histogram', {
      paneIndex: 0,
      priceScaleId: '',
      style: { color: VOLUME_COLOR },
      priceFormat: { type: 'volume' },
    });
    api.priceScale().setOptions({ marginTop: 0.82, marginBottom: 0 });
    api.setData(volumeRows(entry.bars));
    entry.volume = { api };
    return true;
  } catch (error) {
    console.warn('honbaCharts: volume overlay unavailable', error);
    return false;
  }
}

function volumeRows(bars) {
  return bars.map((bar) => ({
    time: bar.time,
    open: 0,
    high: bar.volume ?? 0,
    low: 0,
    close: bar.volume ?? 0,
  }));
}

/* -- Buy/Sell bubble ------------------------------------------------------ */

/** The bubble's two hit zones, in plot coordinates. */
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

/**
 * Freezes chart crosshair & OHLC statusline readout when the pointer moves
 * inside the Buy/Sell bubble's boundary, and resumes tracking as soon as
 * the pointer moves outside of the bubble's boundary.
 */
function attachBuySellBubbleCursorFreeze(chart, tradeBtns) {
  const tb = tradeBtns;
  if (!chart || !tb) return;
  if (typeof tb.hitTest === 'function') {
    const origHitTest = tb.hitTest.bind(tb);
    tb.hitTest = (plotX, localY, rc) => {
      const hit = origHitTest(plotX, localY, rc);
      if (hit) return hit;
      const primaryPane =
        typeof chart.primaryPaneIndex === 'function' ? chart.primaryPaneIndex() : 0;
      const bounds = getBuySellBubbleBounds(tb, chart, primaryPane);
      if (bounds && plotX >= bounds.left && plotX <= bounds.right &&
          localY >= bounds.top && localY <= bounds.bottom) {
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
    const primaryPane =
      typeof chart.primaryPaneIndex === 'function' ? chart.primaryPaneIndex() : 0;
    if (pane !== primaryPane) {
      return { inside: false, bounds: null, plotX, leftAxisWidth };
    }
    const bounds = getBuySellBubbleBounds(tb, chart, pane);
    const inside =
      bounds !== null && plotX >= bounds.left && plotX <= bounds.right &&
      localY >= bounds.top && localY <= bounds.bottom;
    return { inside, bounds, plotX, leftAxisWidth };
  };

  const origUpdateCursor = input._updateCursor.bind(input);
  input._updateCursor = function (pane, x, localY, y, event) {
    const { inside, bounds, plotX, leftAxisWidth } = isPointInBubble(pane, x, localY);

    if (inside && bounds) {
      if (!wasInsideBubble && lastOutsidePoint !== null && lastOutsidePoint.pane === pane) {
        const entry = intersectSegmentWithBox(
          lastOutsidePoint.plotX, lastOutsidePoint.localY, plotX, localY, bounds,
        );
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

/**
 * Ensures the Buy/Sell bubble renders on top of all dotted lines (crosshair
 * & last-price line) by repainting it at the end of the pane's top overlay
 * pass.
 */
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
        const layout =
          typeof this._layout === 'function' ? this._layout(rc) : { plotLeft: 0 };
        const dpr = (rc && rc.dpr) ?? 1;
        ctx.save();
        if (layout.plotLeft > 0) ctx.translate(Math.round(layout.plotLeft * dpr), 0);
        const primCtx =
          typeof this._primitiveContext === 'function'
            ? this._primitiveContext(rc)
            : rc;
        const boundCtx =
          typeof this._boundPrimitiveContext === 'function'
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
  const entry = charts.get(id);
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

/** Keep the bubble's qty in step with the order ticket. */
function setQuantity(id, qty) {
  const entry = charts.get(id);
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

/* -- Drawing toolkit ------------------------------------------------------ */

const CURSOR_ALIAS = ['', 'cursor', 'none', 'cross'];

/** Arm (or clear) the active drawing tool on one chart. */
function setDrawingTool(id, tool) {
  const entry = charts.get(id);
  if (!entry || !entry.drawings) return false;
  try {
    entry.drawings.setTool(CURSOR_ALIAS.includes(tool) ? null : tool);
    return true;
  } catch (error) {
    console.warn('honbaCharts: setDrawingTool refused', tool, error);
    return false;
  }
}

/** The active drawing tool id, or 'cursor' when none is armed. */
function drawingTool(id) {
  const entry = charts.get(id);
  if (!entry || !entry.drawings) return 'cursor';
  try {
    return entry.drawings.activeTool() ?? 'cursor';
  } catch {
    return 'cursor';
  }
}

function undoDrawing(id) {
  const entry = charts.get(id);
  if (!entry || !entry.drawings) return false;
  try {
    return entry.drawings.undo();
  } catch {
    return false;
  }
}

function redoDrawing(id) {
  const entry = charts.get(id);
  if (!entry || !entry.drawings) return false;
  try {
    return entry.drawings.redo();
  } catch {
    return false;
  }
}

/** Remove every drawing on one chart (history keeps it undoable). */
function clearDrawings(id) {
  const entry = charts.get(id);
  if (!entry || !entry.drawings) return false;
  try {
    const ids = entry.drawings.drawings().map((drawing) => drawing.id);
    if (ids.length > 0) entry.drawings.removeMany(ids, { force: true });
    return true;
  } catch (error) {
    console.warn('honbaCharts: clearDrawings failed', error);
    return false;
  }
}

/**
 * Publish the pane's instrument so drawings persist per symbol: the engine's
 * `InstrumentDrawings` listens for `data:context` and swaps its localStorage
 * document on every key change (`exchange:symbol`).
 */
function setSymbolContext(id, symbol) {
  const entry = charts.get(id);
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

function refreshThemeAll() {
  if (charts.size === 0) return;
  const theme = themeFromSkin();

  for (const entry of charts.values()) {
    entry.theme = theme;
    entry.chart.setTheme(theme);
    for (const series of entry.series) {
      series.api.applyOptions(seriesStyle(series.kind, series.style, theme));
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

function destroy(id) {
  const entry = charts.get(id);
  if (!entry) return false;
  entry.chart.destroy();
  charts.delete(id);
  return true;
}

/* -- Indicator API ------------------------------------------------------- */

/** One catalog row for Rust: inputs mirror the engine's setting definitions. */
function catalogEntry(definition) {
  return {
    id: definition.id,
    name: definition.name,
    category: definition.category || 'Other',
    placement: definition.placement === 'pane' ? 'pane' : 'onchart',
    inputs: definition.inputs || [],
  };
}

/** The full registry of built-in indicators as JSON. */
function indicatorCatalog() {
  return JSON.stringify(BUILTIN_INDICATORS.map(catalogEntry));
}

/** Live indicator instances on one chart as JSON. */
function chartIndicators(id) {
  const entry = charts.get(id);
  if (!entry) return '[]';
  try {
    return JSON.stringify(entry.chart.indicators().map((instance) => ({
      instanceId: instance.id,
      indicatorId: instance.indicatorId ?? instance.id,
      name: instance.name ?? instance.indicatorId ?? instance.id,
    })));
  } catch {
    return '[]';
  }
}

/**
 * Add one built-in indicator to a chart.
 * Returns the live instance JSON (or null when the engine refuses the id).
 */
function addIndicator(id, indicatorId, settingsJson, instanceId) {
  const entry = charts.get(id);
  if (!entry) return null;

  try {
    let settings;
    if (settingsJson) {
      const parsed = JSON.parse(settingsJson);
      if (parsed && Object.keys(parsed).length > 0) settings = parsed;
    }
    const options = instanceId ? { instanceId } : {};
    entry.chart.addIndicator(indicatorId, settings, options);

    const instances = entry.chart.indicators();
    const live = instances
      .filter((instance) => (instance.indicatorId ?? instance.id) === indicatorId)
      .find((instance) => !instanceId || instance.id === instanceId) ?? instances.at(-1);
    if (!live) return null;
    return JSON.stringify({ instanceId: live.id, indicatorId, name: live.name ?? indicatorId });
  } catch {
    return null;
  }
}

/** Remove one live indicator instance (idempotent). */
function removeIndicator(id, instanceId) {
  const entry = charts.get(id);
  if (!entry) return false;
  try {
    const live = entry.chart.indicators().find((instance) => instance.id === instanceId);
    if (!live) return false;
    live.remove();
    return true;
  } catch {
    return false;
  }
}

/* -- TradingView-shaped IST time axis ------------------------------------- */

const SHORT_MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
const SHORT_WEEKDAYS = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
const IST_OFFSET_SEC = 19800; // UTC+5:30

function pad2(n) {
  return n < 10 ? `0${n}` : String(n);
}

function getDateParts(utcSec, tz) {
  if (!tz || tz === 'Asia/Kolkata' || tz === 'Asia/Calcutta') {
    const d = new Date((utcSec + IST_OFFSET_SEC) * 1000);
    return {
      year: d.getUTCFullYear(),
      month: d.getUTCMonth() + 1,
      day: d.getUTCDate(),
      hour: d.getUTCHours(),
      minute: d.getUTCMinutes(),
      second: d.getUTCSeconds(),
      weekday: d.getUTCDay(),
    };
  }
  const d = new Date(utcSec * 1000);
  return {
    year: d.getFullYear(),
    month: d.getMonth() + 1,
    day: d.getDate(),
    hour: d.getHours(),
    minute: d.getMinutes(),
    second: d.getSeconds(),
    weekday: d.getDay(),
  };
}

/**
 * TradingView-style compact timeline formatter:
 * Shows the month name only on the first and last visible ticks of each month
 * (e.g. '02Jul', then '05', '07', ... until the month ends with '31Jul'),
 * saving horizontal real estate and increasing visibility.
 */
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

  // Detect whether bars are intraday or daily+
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
  const showSeconds =
    Number.isFinite(minBarStepSec * step) && minBarStepSec * step < 60;

  const fontSize = Math.max(
    10,
    Math.min(
      14,
      Math.round((rc.canvasOptions && rc.canvasOptions.scales && rc.canvasOptions.scales.fontSize) ??
        (rc.theme && rc.theme.axisFontSize) ?? 11),
    ),
  );
  const textColor =
    (rc.canvasOptions && rc.canvasOptions.scales && rc.canvasOptions.scales.textColor) ??
    (rc.theme && rc.theme.axisText) ?? '#8b91a7';
  const lineColor =
    (rc.canvasOptions && rc.canvasOptions.scales && rc.canvasOptions.scales.lineColor) ??
    (rc.theme && rc.theme.axisLine) ?? '#2a3046';
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

  // Axis top border line
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
    // 1. Collect all visible bars with their date parts and group by month
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
      if (!grp) {
        grp = { firstIdx: i, indices: [], parts: p };
        monthGroups.set(mKey, grp);
      }
      grp.indices.push(i);

      // Month / Year transition boundary -> TradingView shows just the month
      // name ("Jul", "Aug") or year ("2026")
      if (prevParts && (p.year !== prevParts.year || p.month !== prevParts.month)) {
        if (p.year !== prevParts.year && p.month === 1) {
          pushTick(i, String(p.year), 3);
        } else {
          pushTick(i, SHORT_MONTHS[p.month - 1], 2);
        }
      } else if (i === fromIdx && !prevParts) {
        pushTick(i, SHORT_MONTHS[p.month - 1], 2);
      }
      prevParts = p;
    }

    // 2. Within each month, if zoomed in enough that the month spans more
    //    than ~90px, equally divide the month space into N equal segments
    //    and place day numbers (e.g. 8, 15, 22)
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
    // Intraday: group by day, show month name on month transitions, day
    // number on day transitions, and equally divide each day into time
    // ticks (HH:mm)
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
      if (!grp) {
        grp = [];
        dayGroups.set(dKey, grp);
      }
      grp.push(i);

      if (prevParts) {
        if (p.year !== prevParts.year || p.month !== prevParts.month) {
          pushTick(i, SHORT_MONTHS[p.month - 1], 2);
        } else if (p.day !== prevParts.day) {
          pushTick(i, String(p.day), 1);
        }
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
          const timeLabel = showSeconds
            ? `${pad2(p.hour)}:${pad2(p.minute)}:${pad2(p.second)}`
            : `${pad2(p.hour)}:${pad2(p.minute)}`;
          pushTick(barIdx, timeLabel, 0);
        }
      }
    }
  }

  // Sort by priority descending (Year > Month > Day > Sub-tick), then x
  // ascending; accept a candidate when it does not crowd its neighbours.
  const byPriority = [...candidates].sort(
    (a, b) => b.priority - a.priority || a.x - b.x,
  );
  const minGap = 14 * dpr;
  const accepted = [];

  for (const cand of byPriority) {
    const fits = accepted.every(
      (other) =>
        cand.x + cand.half + minGap <= other.x - other.half ||
        cand.x - cand.half - minGap >= other.x + other.half,
    );
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

  // 1. Set crosshair time tag formatter
  if (typeof chart.setTimeFormatter === 'function') {
    chart.setTimeFormatter((utcSeconds) => {
      const tz =
        typeof chart.timezone === 'function' ? chart.timezone() : 'Asia/Kolkata';
      const parts = getDateParts(utcSeconds, tz);
      let label =
        `${SHORT_WEEKDAYS[parts.weekday]} ${pad2(parts.day)} ` +
        `${SHORT_MONTHS[parts.month - 1]} '${String(parts.year).slice(-2)}`;
      if (parts.hour !== 0 || parts.minute !== 0 || parts.second !== 0) {
        label += ` ${pad2(parts.hour)}:${pad2(parts.minute)}`;
        if (parts.second !== 0) label += `:${pad2(parts.second)}`;
      }
      return label;
    });
  }

  // 2. Override pane.paintBase time-axis pass so first & last visible ticks
  //    of each month are computed AFTER collision filtering
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
          const ctx =
            targetCtx ??
            (this._backend && this._backend.overlay2d ? this._backend.overlay2d() : null) ??
            this.base?.ctx;
          if (ctx) {
            const tz =
              rc.timezone ??
              (typeof chart.timezone === 'function' ? chart.timezone() : 'Asia/Kolkata');
            paintTradingViewTimeAxis(ctx, this, rc, tz);
          }
        } catch {
          /* ignore */
        }
      }
    };
  }
}

window.honbaCharts = {
  version: '2.6.7',
  mount,
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
};

/* Let the Rust side know the engine is live even if it beat the module. */
window.dispatchEvent(new Event('honba-charts-ready'));
