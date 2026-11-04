const fs = require('fs'), path = require('path');
const ROOT = '/home/pmallapp/Devel/Finance/honba-workspace/honba-kao/crates';
const dirs = [
  path.join(ROOT, 'honba-kao-app/src'),
  path.join(ROOT, 'honba-kao-app-nsh/src'),
  path.join(ROOT, 'honba-kao-ui-core/src'),
  path.join(ROOT, 'honba-kao-client/src'),
];
const tokens = new Map();
const walk = d => fs.readdirSync(d, { withFileTypes: true }).forEach(e => {
  const p = path.join(d, e.name);
  if (e.isDirectory()) walk(p);
  else if (p.endsWith('.rs')) {
    const src = fs.readFileSync(p, 'utf8');
    // every double-quoted literal
    const lits = src.match(/"(?:[^"\\]|\\.)*"/g) || [];
    for (const lit of lits) {
      const s = lit.slice(1, -1);
      if (!s.trim() || /[:{}<>|]/.test(s) || /\n/.test(s) && s.length > 200) continue;
      for (const t of s.split(/\s+/)) {
        if (!/^[a-z][a-z0-9_.\-\/]*$/.test(t)) continue;
        if (t.length < 2) continue;
        // semantic-or-tailwind-like tokens only
        if (t.includes('-') || t.includes(':') || t === 'flex' || t === 'grid' || t === 'block' || t === 'hidden' || t === 'table' || t === 'truncate') {
          tokens.set(t, (tokens.get(t) || 0) + 1);
        }
      }
    }
  }
});
dirs.forEach(d => fs.existsSync(d) && walk(d));
const list = [...tokens.entries()].sort((a, b) => a[0].localeCompare(b[0]));
fs.writeFileSync(path.join(__dirname, 'classes.txt'), list.map(([t, n]) => `${n}\t${t}`).join('\n') + '\n');
console.log('distinct tokens:', list.length, '(written to scripts/classes.txt)');
