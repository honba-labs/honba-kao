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
 * ========================================================================== */

import { createChart, darkTheme, lightTheme } from './openalgo-charts.mjs';
/* Side-effect tiers: 112 built-in indicators and the drawing toolkit. */
import './openalgo-charts.indicators.mjs';
import './openalgo-charts.draw.mjs';

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

window.honbaCharts = {
  version: '2.6.7',
  mount,
  setSeries,
  refreshThemeAll,
  destroy,
};

/* Let the Rust side know the engine is live even if it beat the module. */
window.dispatchEvent(new Event('honba-charts-ready'));
