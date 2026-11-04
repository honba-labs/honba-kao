#!/usr/bin/env node
'use strict';
/* Deterministic generator: reads the class-token inventory and emits
 * crates/honba-kao-app/assets/style/utilities.css (dark-first, no media queries). */
const fs = require('fs');
const path = require('path');

const SRC = path.join(__dirname, 'classes.txt');
const OUT = path.join(__dirname, '..', 'crates', 'honba-kao-app', 'assets', 'style', 'utilities.css');
const NUM = /^\d+(\.\d+)?$/;
const rem = n => (Number(n) === 0 ? '0' : `${Number(n) * 0.25}rem`);
const esc = c => c.replace(/[.[\]:/%]/g, m => '\\' + m);
const rule = (c, d) => `.${esc(c)} { ${d} }`;
const SIB = ' > :not([hidden]) ~ :not([hidden])';

/* ---- semantic + tailwind palette: [solid value, hex used for opacity mixes] ---- */
const P = {
  'surface-base': ['var(--hf-surface-base)', '#11141b'],
  'surface-subtle': ['var(--hf-surface-subtle)', '#1a1f2d'],
  'surface-elevated': ['var(--hf-surface-elevated)', '#232a3a'],
  'border-soft': ['var(--hf-border-soft)', '#232a3a'],
  'border-strong': ['var(--hf-border-strong)', '#333c52'],
  'text-high': ['var(--hf-text-high)', '#e5e7eb'],
  'text-muted': ['var(--hf-text-muted)', '#9ca3af'],
  'text-faint': ['var(--hf-text-faint)', '#6b7280'],
  'primary-400': ['#34d399', '#34d399'],
  'primary-500': ['var(--hf-primary)', '#10b981'],
  'primary-600': ['#059669', '#059669'],
  accent: ['var(--hf-accent)', '#fbbf24'],
  'accent-primary': ['var(--hf-accent)', '#fbbf24'],
  'semantic-error': ['var(--hf-danger)', '#ef4444'],
  'semantic-success': ['#22c55e', '#22c55e'],
  success: ['#22c55e', '#22c55e'],
  warning: ['#f59e0b', '#f59e0b'],
  error: ['#ef4444', '#ef4444'],
  'terminal-dark': ['#0b0e14', '#0b0e14'],
  'text-primary': ['#e5e7eb', '#e5e7eb'],
  'text-inverted': ['#0b0e14', '#0b0e14'],
  transparent: ['transparent', 'transparent'],
  black: ['#000000', '#000000'],
  white: ['#ffffff', '#ffffff'],
  'amber-400': ['#fbbf24', '#fbbf24'], 'amber-500': ['#f59e0b', '#f59e0b'], 'amber-600': ['#d97706', '#d97706'],
  'blue-400': ['#60a5fa', '#60a5fa'], 'blue-500': ['#3b82f6', '#3b82f6'], 'blue-600': ['#2563eb', '#2563eb'],
  'cyan-400': ['#22d3ee', '#22d3ee'], 'cyan-500': ['#06b6d4', '#06b6d4'], 'cyan-600': ['#0891b2', '#0891b2'],
  'emerald-300': ['#6ee7b7', '#6ee7b7'], 'emerald-400': ['#34d399', '#34d399'], 'emerald-500': ['#10b981', '#10b981'],
  'emerald-600': ['#059669', '#059669'], 'emerald-700': ['#047857', '#047857'], 'emerald-800': ['#065f46', '#065f46'],
  'emerald-950': ['#022c22', '#022c22'],
  'orange-400': ['#fb923c', '#fb923c'], 'pink-400': ['#f472b6', '#f472b6'],
  'purple-300': ['#d8b4fe', '#d8b4fe'], 'purple-400': ['#c084fc', '#c084fc'], 'purple-500': ['#a855f7', '#a855f7'],
  'red-500': ['#ef4444', '#ef4444'],
  'rose-400': ['#fb7185', '#fb7185'], 'rose-500': ['#f43f5e', '#f43f5e'], 'rose-600': ['#e11d48', '#e11d48'],
  'rose-950': ['#4c0519', '#4c0519'],
  'slate-800': ['#1e293b', '#1e293b'], 'teal-400': ['#2dd4bf', '#2dd4bf'],
};
function paintVal(s) {
  const i = s.indexOf('/');
  const p = i < 0 ? s : s.slice(0, i);
  const e = P[p];
  if (!e) return null;
  if (i < 0) return e[0];
  const op = Number(s.slice(i + 1));
  if (!Number.isFinite(op)) return null;
  if (e[0] === 'transparent') return 'transparent';
  return `color-mix(in srgb, ${e[1]} ${op}%, transparent)`;
}
const col = (s, prop) => { const v = paintVal(s); return v == null ? null : `${prop}: ${v}`; };

/* ---- static tables ---- */
const DISP = { hidden: 'none', block: 'block', table: 'table', flex: 'flex', grid: 'grid', 'inline-block': 'inline-block', 'inline-flex': 'inline-flex' };
const TS = {
  xs: '0.75rem/1rem', sm: '0.875rem/1.25rem', base: '1rem/1.5rem', lg: '1.125rem/1.75rem',
  xl: '1.25rem/1.75rem', '2xl': '1.5rem/2rem', '3xl': '1.875rem/2.25rem', '4xl': '2.25rem/2.5rem', '5xl': '3rem/1',
};
const TA = { center: 'center', left: 'left', right: 'right', justify: 'justify' };
const FW = { light: '300', normal: '400', medium: '500', semibold: '600', bold: '700', extrabold: '800', black: '900' };
const TK = { tight: '-0.025em', normal: '0', wide: '0.025em', wider: '0.05em', widest: '0.1em' };
const LD = { none: '1', tight: '1.25', snug: '1.375', normal: '1.5', relaxed: '1.625', loose: '2' };
const RD = { sm: '0.125rem', md: '0.375rem', lg: '0.5rem', '2xl': '1rem', '3xl': '1.5rem', full: '9999px' };
const SH = { sm: '0 1px 2px rgba(0,0,0,.35)', md: '0 4px 12px rgba(0,0,0,.3)', lg: '0 8px 24px rgba(0,0,0,.3)', '2xl': '0 16px 48px rgba(0,0,0,.45)' };
const SIDE = { r: 'right', l: 'left', t: 'top', b: 'bottom' };
const AI = { start: 'flex-start', end: 'flex-end', center: 'center', baseline: 'baseline', stretch: 'stretch' };
const JC = { start: 'flex-start', end: 'flex-end', center: 'center', between: 'space-between', around: 'space-around', evenly: 'space-evenly' };
const MAXW = { md: '28rem', lg: '32rem', xl: '36rem', '3xl': '48rem', '5xl': '64rem', full: '100%' };
const dim = (v, k) => v === 'full' ? '100%' : v === 'screen' ? (k === 'w' ? '100vw' : '100vh')
  : v === 'fit' ? 'fit-content' : v === 'px' ? '1px' : NUM.test(v) ? rem(v) : null;
const pos = v => v === 'full' ? '100%' : v === 'auto' ? 'auto'
  : /^\d+\/\d+$/.test(v) ? `${(v.split('/')[0] / v.split('/')[1]) * 100}%` : NUM.test(v) ? rem(v) : null;

const ROOT = `:root {
  --hf-bg-dark: #0b0e14;
  --hf-surface-base: #11141b;
  --hf-surface-subtle: #1a1f2d;
  --hf-surface-elevated: #232a3a;
  --hf-border-soft: #232a3a;
  --hf-border-strong: #333c52;
  --hf-text-high: #e5e7eb;
  --hf-text-muted: #9ca3af;
  --hf-text-faint: #6b7280;
  --hf-primary: #10b981;
  --hf-accent: #fbbf24;
  --hf-danger: #ef4444;
  --hf-surface-radius: 12px;
  --hf-surface-shadow: 0 12px 32px rgba(0,0,0,.35), 0 2px 8px rgba(0,0,0,.25);
}`;
const KEYS = '@keyframes spin { from { transform: rotate(0deg) } to { transform: rotate(360deg) } }';

/* ---- classifier: returns a full rule line, or null when the token is unknown ---- */
function classify(t) {
  let m, d, v;
  if (t.startsWith('bg-')) { d = col(t.slice(3), 'background-color'); return d && rule(t, d); }
  if (t.startsWith('text-')) {
    const s = t.slice(5);
    if (TS[s]) return rule(t, `font-size: ${TS[s].split('/')[0]}; line-height: ${TS[s].split('/')[1]}`);
    if (TA[s]) return rule(t, `text-align: ${TA[s]}`);
    d = col(s, 'color'); return d && rule(t, d);
  }
  if (t === 'border' || /^border-(0|2|4)$/.test(t)) return rule(t, `border-width: ${t === 'border' ? 1 : t.slice(7)}px; border-style: solid`);
  if ((m = t.match(/^border-([rltb])(?:-(\d))?$/))) return rule(t, `border-${SIDE[m[1]]}-width: ${m[2] || 1}px; border-style: solid`);
  if (t === 'border-x' || t === 'border-y') { const s = t === 'border-x' ? ['left', 'right'] : ['top', 'bottom']; return rule(t, s.map(x => `border-${x}-width: 1px`).join('; ') + '; border-style: solid'); }
  if (t === 'border-collapse') return rule(t, 'border-collapse: collapse');
  if (t.startsWith('border-')) { d = col(t.slice(7), 'border-color'); return d && rule(t, d); }
  if (DISP[t]) return rule(t, `display: ${DISP[t]}`);
  if (t === 'flex-1') return rule(t, 'flex: 1 1 0%');
  if (t === 'flex-col') return rule(t, 'flex-direction: column');
  if (t === 'flex-col-reverse') return rule(t, 'flex-direction: column-reverse');
  if (t === 'flex-row') return rule(t, 'flex-direction: row');
  if (t === 'flex-wrap') return rule(t, 'flex-wrap: wrap');
  if (t === 'flex-nowrap') return rule(t, 'flex-wrap: nowrap');
  if (t === 'flex-grow') return rule(t, 'flex-grow: 1');
  if (t === 'shrink-0' || t === 'flex-shrink-0') return rule(t, 'flex-shrink: 0');
  if ((m = t.match(/^items-(start|end|center|baseline|stretch)$/))) return rule(t, `align-items: ${AI[m[1]]}`);
  if ((m = t.match(/^justify-(start|end|center|between|around|evenly)$/))) return rule(t, `justify-content: ${JC[m[1]]}`);
  if ((m = t.match(/^grid-cols-(\d+)$/))) return rule(t, `grid-template-columns: repeat(${m[1]}, minmax(0, 1fr))`);
  if ((m = t.match(/^col-span-(\d+)$/))) return rule(t, `grid-column: span ${m[1]} / span ${m[1]}`);
  if ((m = t.match(/^row-span-(\d+)$/))) return rule(t, `grid-row: span ${m[1]} / span ${m[1]}`);
  if ((m = t.match(/^gap-(?:(x|y)-)?(\d+(?:\.\d+)?)$/))) return rule(t, `${m[1] === 'x' ? 'column-gap' : m[1] === 'y' ? 'row-gap' : 'gap'}: ${rem(m[2])}`);
  if ((m = t.match(/^([pm])([xytblr]?)-(.+)$/))) {
    const prop = m[1] === 'p' ? 'padding' : 'margin';
    const val = m[3] === 'auto' && m[1] === 'm' ? 'auto' : NUM.test(m[3]) ? rem(m[3]) : null;
    if (val == null) return null;
    const sides = { '': [''], x: ['-left', '-right'], y: ['-top', '-bottom'], t: ['-top'], b: ['-bottom'], l: ['-left'], r: ['-right'] }[m[2]];
    if (!sides) return null;
    return rule(t, sides.map(s => `${prop}${s}: ${val}`).join('; '));
  }
  if ((m = t.match(/^space-y-(\d+(?:\.\d+)?)$/))) return `.${esc(t)}${SIB} { margin-top: ${rem(m[1])} }`;
  if (t === 'divide-y') return `.${esc(t)}${SIB} { border-top-width: 1px; border-style: solid }`;
  if (t === 'divide-x') return `.${esc(t)}${SIB} { border-left-width: 1px; border-style: solid }`;
  if (t.startsWith('divide-')) { d = col(t.slice(7), 'border-color'); return d && `.${esc(t)}${SIB} { ${d} }`; }
  if ((m = t.match(/^(min-|max-)?([wh])-\[([^\]]+)\]$/))) return rule(t, `${m[1] || ''}${m[2] === 'w' ? 'width' : 'height'}: ${m[3]}`);
  if ((m = t.match(/^(min|max)-(w|h)-(.+)$/))) {
    const prop = `${m[1]}-${m[2] === 'w' ? 'width' : 'height'}`;
    const val = m[3] === 'full' ? '100%' : m[3] === 'screen' ? (m[2] === 'w' ? '100vw' : '100vh')
      : NUM.test(m[3]) ? rem(m[3]) : (m[1] === 'max' && m[2] === 'w' ? MAXW[m[3]] : null);
    return val == null ? null : rule(t, `${prop}: ${val}`);
  }
  if (t.startsWith('w-')) { v = dim(t.slice(2), 'w'); return v == null ? null : rule(t, `width: ${v}`); }
  if (t.startsWith('h-')) { v = dim(t.slice(2), 'h'); return v == null ? null : rule(t, `height: ${v}`); }
  if (t === 'inset-0') return rule(t, 'top: 0; right: 0; bottom: 0; left: 0');
  if (t === 'inset-y-0') return rule(t, 'top: 0; bottom: 0');
  if (t === 'inset-x-0') return rule(t, 'left: 0; right: 0');
  if ((m = t.match(/^(top|right|bottom|left)-(.+)$/))) { v = pos(m[2]); return v == null ? null : rule(t, `${m[1]}: ${v}`); }
  if ((m = t.match(/^z-(\d+)$/))) return rule(t, `z-index: ${m[1]}`);
  if ((m = t.match(/^overflow-(?:(x|y)-)?(auto|hidden|scroll|visible)$/))) return rule(t, `${m[1] ? 'overflow-' + m[1] : 'overflow'}: ${m[2]}`);
  if ((m = t.match(/^cursor-(pointer|not-allowed|default|grab|crosshair|wait)$/))) return rule(t, `cursor: ${m[1]}`);
  if ((m = t.match(/^opacity-(\d+)$/))) return rule(t, `opacity: ${Number(m[1]) / 100}`);
  if (t === 'pointer-events-none') return rule(t, 'pointer-events: none');
  if (t === 'pointer-events-auto') return rule(t, 'pointer-events: auto');
  if (t === 'select-none') return rule(t, 'user-select: none');
  if ((m = t.match(/^font-(light|normal|medium|semibold|bold|extrabold|black)$/))) return rule(t, `font-weight: ${FW[m[1]]}`);
  if (t === 'font-mono') return rule(t, "font-family: 'IBM Plex Mono', ui-monospace, monospace");
  if (t === 'font-sans') return rule(t, "font-family: Inter, -apple-system, 'Segoe UI', sans-serif");
  if ((m = t.match(/^tracking-(tight|normal|wide|wider|widest)$/))) return rule(t, `letter-spacing: ${TK[m[1]]}`);
  if ((m = t.match(/^leading-(none|tight|snug|normal|relaxed|loose)$/))) return rule(t, `line-height: ${LD[m[1]]}`);
  if ((m = t.match(/^leading-(\d+(?:\.\d+)?)$/))) return rule(t, `line-height: ${rem(m[1])}`);
  if ((m = t.match(/^whitespace-(nowrap|pre|pre-wrap|pre-line)$/))) return rule(t, `white-space: ${m[1]}`);
  if (t === 'truncate') return rule(t, 'overflow: hidden; text-overflow: ellipsis; white-space: nowrap');
  if (t === 'break-words') return rule(t, 'overflow-wrap: break-word');
  if (t === 'tabular-nums') return rule(t, 'font-variant-numeric: tabular-nums');
  if (t === 'uppercase') return rule(t, 'text-transform: uppercase');
  if (t === 'align-middle') return rule(t, 'vertical-align: middle');
  if (t === 'rounded') return rule(t, 'border-radius: 0.25rem');
  if ((m = t.match(/^rounded-(sm|md|lg|xl|2xl|3xl|full)$/))) return rule(t, `border-radius: ${m[1] === 'xl' ? 'var(--hf-surface-radius)' : RD[m[1]]}`);
  if (t === 'rounded-t') return rule(t, 'border-top-left-radius: 0.25rem; border-top-right-radius: 0.25rem');
  if (t === 'shadow-inner') return rule(t, 'box-shadow: inset 0 2px 4px 0 rgba(0,0,0,0.25)');
  if (t === 'shadow' || t === 'shadow-sm') return rule(t, SH.sm);
  if ((m = t.match(/^shadow-(sm|md|lg|xl|2xl)$/))) return rule(t, `box-shadow: ${m[1] === 'xl' ? 'var(--hf-surface-shadow)' : SH[m[1]]}`);
  if ((m = t.match(/^shadow-([a-z]+)\/(\d+)$/))) { const c = paintVal(`${m[1]}/${m[2]}`); return c == null ? null : rule(t, `box-shadow: 0 8px 24px ${c}`); }
  if ((m = t.match(/^backdrop-blur(-(sm|md|lg|none))?$/))) return rule(t, `backdrop-filter: blur(${m[2] === 'sm' ? 4 : m[2] === 'md' ? 8 : m[2] === 'lg' ? 16 : 0}px)`);
  if ((m = t.match(/^transition(-(all|colors|transform|opacity|shadow))?$/))) {
    const props = { all: 'all', colors: 'color, background-color, border-color, text-decoration-color, fill, stroke', transform: 'transform', opacity: 'opacity', shadow: 'box-shadow' };
    const p = m[2] ? props[m[2]] : 'color, background-color, border-color, opacity, box-shadow, transform, filter';
    return rule(t, `transition-property: ${p}; transition-duration: 150ms; transition-timing-function: cubic-bezier(.4, 0, .2, 1)`);
  }
  if ((m = t.match(/^duration-(\d+)$/))) return rule(t, `transition-duration: ${m[1]}ms`);
  if (t === 'ease-in-out') return rule(t, 'transition-timing-function: cubic-bezier(.4, 0, .2, 1)');
  if (t === 'animate-spin') return rule(t, 'animation: spin 1s linear infinite');
  if (t === 'outline-none') return rule(t, 'outline: 2px solid transparent; outline-offset: 2px');
  if ((m = t.match(/^ring-(\d+)$/))) return rule(t, `box-shadow: 0 0 0 ${m[1]}px rgb(59 130 246 / .5)`);
  if (t.startsWith('ring-')) { d = paintVal(t.slice(5)); return d == null ? null : rule(t, `box-shadow: 0 0 0 2px ${d}`); }
  if ((m = t.match(/^stroke-(\d+(?:\.\d+)?)$/))) return rule(t, `stroke-width: ${m[1]}`);
  if (t === 'stroke-current') return rule(t, 'stroke: currentColor');
  if (t.startsWith('stroke-')) { d = col(t.slice(7), 'stroke'); return d && rule(t, d); }
  if (t === 'fill-none') return rule(t, 'fill: none');
  if (t.startsWith('fill-')) { d = col(t.slice(5), 'fill'); return d && rule(t, d); }
  if ((m = t.match(/^translate-([xy])-(full|\d+(?:\.\d+)?)$/))) return rule(t, `transform: translate${m[1].toUpperCase()}(${m[2] === 'full' ? '100%' : rem(m[2])})`);
  return null;
}

/* ---- noise: non-class domain tokens (only consulted when classify() fails) ---- */
const UTILITY = /^(bg|border|text|flex|grid|col|row|gap|space|divide|w|h|min-w|min-h|max-w|max-h|top|right|bottom|left|inset|z|overflow|cursor|font|tracking|leading|whitespace|rounded|shadow|ring|stroke|fill|transition|duration|animate|opacity|select|pointer-events|items|justify|align|outline|backdrop|translate|shrink|break|p|px|py|pt|pb|pl|pr|m|mx|my|mt|mb|ml|mr)-|^(hidden|block|flex|grid|table|truncate|uppercase|inline-block|inline-flex|tabular-nums)$/;
const isNoise = t => t === 'p-value' || t === 'z-score' || t.includes('/') || t.includes('__') || t.endsWith('.') || !UTILITY.test(t);

/* ---- main ---- */
function main() {
  if (!fs.existsSync(SRC)) throw new Error(`missing token inventory: ${SRC}`);
  const raw = fs.readFileSync(SRC, 'utf8').split('\n').map(l => l.split('\t')[1]).filter(Boolean);
  const set = new Set(raw);
  const PREFIX = /^(hover|group-hover|laptop|desktop):/;
  const tokens = [];
  const unmatched = [];
  for (const t of raw) {
    const m = t.match(PREFIX);
    if (!m) { tokens.push(t); continue; }
    const bare = t.slice(m[0].length);
    if (set.has(bare)) continue; // bare form emitted on its own pass
    if (classify(bare) != null) tokens.push(bare); // base-only fallback
    else unmatched.push(t); // variant with no bare form
  }
  const out = [ROOT];
  if (set.has('animate-spin')) out.push(KEYS);
  const seen = new Set();
  const unclassified = [];
  let rules = 0;
  for (const t of tokens) {
    if (seen.has(t)) continue;
    const line = classify(t);
    if (line != null) { seen.add(t); out.push(line); rules++; }
    else if (!isNoise(t)) unclassified.push(t);
  }
  if (unclassified.length) throw new Error(`cannot classify ${unclassified.length} token(s): ${unclassified.join(', ')}`);
  if (unmatched.length) out.push(`/* unmatched (${unmatched.length}): ${unmatched.join(', ')} */`);
  fs.mkdirSync(path.dirname(OUT), { recursive: true });
  fs.writeFileSync(OUT, out.join('\n') + '\n');
  console.log(`wrote ${OUT} — ${rules} rules, ${unmatched.length} unmatched`);
}
main();
