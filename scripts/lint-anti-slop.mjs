#!/usr/bin/env node
import fs from 'fs';
import path from 'path';

const SRC_DIR = path.resolve(process.cwd(), 'web/src');
const EXTENSIONS = /\.(tsx|css|ts)$/i;
const SKIP_DIRS = new Set(['node_modules', 'dist']);

function walkDir(dir) {
  const files = [];
  let entries;
  try { entries = fs.readdirSync(dir, { withFileTypes: true }); }
  catch { return files; }
  for (const entry of entries) {
    if (SKIP_DIRS.has(entry.name)) continue;
    const fp = path.join(dir, entry.name);
    if (entry.isDirectory()) files.push(...walkDir(fp));
    else if (entry.isFile() && EXTENSIONS.test(entry.name)) files.push(fp);
  }
  return files;
}

function extract(line, re) {
  const m = line.match(re);
  return m ? m[0].trim() : '';
}

function isDotGridPattern(l) {
  return /radial-gradient\(.*\d+\.?\d*px.*\btransparent\b/i.test(l);
}

const checks = [
  {
    label: 'border-left:',
    test: (l) => {
      if (!/border-left:/.test(l)) return false;
      const m = l.match(/border-left:\s*([^;]+)/);
      if (!m) return true;
      const val = m[1].trim();
      if (/^none$/i.test(val)) return false;
      if (/^2px\s+solid\s+transparent$/i.test(val)) return false;
      if (/^1px\b/.test(val)) return false;
      if (/^3px\s+solid$/i.test(val)) return false;
      return true;
    },
    hit: (l) => extract(l, /border-left:\s*[^;]+/),
  },
  {
    label: 'radial-gradient(',
    test: (l) => {
      if (!/radial-gradient\(/.test(l)) return false;
      if (isDotGridPattern(l)) return false;
      return true;
    },
    hit: (l) => extract(l, /radial-gradient\([^)]*\)/),
  },
  {
    label: 'backdrop-filter: blur(',
    test: (l) => /backdrop-filter:\s*blur\(/.test(l),
    hit: (l) => extract(l, /backdrop-filter:\s*blur\([^)]*\)/),
  },
  {
    label: 'border-radius (non-zero)',
    test: (l) => {
      if (!/border-radius:\s+[^0]/.test(l)) return false;
      if (/border-radius:\s*(50%|9999px|999px)/.test(l)) return false;
      return true;
    },
    hit: (l) => extract(l, /border-radius:\s*[^;]+/),
  },
  {
    label: "font-family 'Inter'",
    test: (l) => /['"]Inter['"]/.test(l),
    hit: (l) => extract(l, /['"]Inter['"]/),
  },
];

const files = walkDir(SRC_DIR);
const issues = [];

for (const file of files) {
  const rel = path.relative(process.cwd(), file);
  const content = fs.readFileSync(file, 'utf-8');
  const lines = content.split('\n');
  let inBlockComment = false;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const lineNum = i + 1;
    const trimmed = line.trim();

    if (inBlockComment) {
      if (trimmed.includes('*/')) inBlockComment = false;
      continue;
    }

    if (trimmed.startsWith('//')) continue;
    if (trimmed.startsWith('/*')) {
      if (!trimmed.includes('*/')) inBlockComment = true;
      continue;
    }

    for (const check of checks) {
      if (check.test(line)) {
        issues.push({ file: rel, lineNum, label: check.label, hit: check.hit(line), line: line.trim() });
      }
    }
  }
}

issues.sort((a, b) => a.file.localeCompare(b.file) || a.lineNum - b.lineNum);

console.log(`${files.length} files scanned, ${issues.length} issues found`);

let currentFile = '';
for (const issue of issues) {
  if (issue.file !== currentFile) {
    console.log(`\n${issue.file}`);
    currentFile = issue.file;
  }
  console.log(`  Line ${issue.lineNum}  [${issue.label}]  ${issue.hit}`);
  console.log(`    ${issue.line}`);
}

process.exit(issues.length > 0 ? 1 : 0);
