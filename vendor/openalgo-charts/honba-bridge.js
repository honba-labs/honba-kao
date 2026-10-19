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
 *
 * API surface used from Rust (see components/charts/openalgo_chart.rs):
 *   window.honbaCharts.mount(element)              -> id
 *   window.honbaCharts.setSeries(id, type, style, bars)
 *   window.honbaCharts.refreshThemeAll()
 *   window.honbaCharts.destroy(id)
 *   window.honbaCharts.indicatorCatalog()          -> JSON
 *   window.honbaCharts.chartIndicators(id)         -> JSON
 *   window.honbaCharts.addIndicator(id, indId, settingsJson, instanceId)
 *   window.honbaCharts.removeIndicator(id, instanceId)
 * ========================================================================== */

import { createChart, darkTheme, lightTheme } from './openalgo-charts.mjs';
/* Side-effect tiers: 112 built-in indicators and the drawing toolkit. */
import './openalgo-charts.draw.mjs';
import { BUILTIN_INDICATORS } from './openalgo-charts.indicators.mjs';

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
  const id = nextId++;
  charts.set(id, { chart, series: [], theme });
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

function refreshThemeAll() {
  if (charts.size === 0) return;
  const theme = themeFromSkin();

  for (const entry of charts.values()) {
    entry.theme = theme;
    entry.chart.setTheme(theme);
    for (const series of entry.series) {
      series.api.applyOptions(seriesStyle(series.kind, series.style, theme));
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

window.honbaCharts = {
  version: '2.6.7',
  mount,
  setSeries,
  refreshThemeAll,
  destroy,
  indicatorCatalog,
  chartIndicators,
  addIndicator,
  removeIndicator,
};

/* Let the Rust side know the engine is live even if it beat the module. */
window.dispatchEvent(new Event('honba-charts-ready'));
