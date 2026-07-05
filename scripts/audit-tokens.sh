#!/usr/bin/env bash
set -euo pipefail

# audit-tokens.sh — Swiss+Neubrutalism design token linter for VantaDB
# Usage: bash scripts/audit-tokens.sh [--fix]
#
# Checks:
#  1. Inline styles (style={{...}}) in TSX
#  2. Non-zero border-radius in CSS
#  3. Blur shadows in CSS
#  4. Hardcoded hex/rgb colors in CSS (not using vars)
#  5. Decorative gradients in CSS
#  6. Colored border-left in CSS (AI slop pattern)

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WEB="$ROOT/web/src"
FIX="${1:-}"  # --fix not yet implemented

VIOLATIONS=0

print_header() {
  echo "─── $* ───"
}

check_grep() {
  local label="$1" pattern="$2" path="$3"
  local matches
  matches=$(grep -rn --include="*.tsx" --include="*.css" "$pattern" "$path" 2>/dev/null || true)
  if [ -n "$matches" ]; then
    echo "$matches"
    VIOLATIONS=$((VIOLATIONS + 1))
  fi
}

# 1. Inline styles in TSX
print_header "1/6  Inline styles (style={{...}})"
inline_matches=$(grep -rn 'style={{' --include="*.tsx" "$WEB" 2>/dev/null || true)
if [ -n "$inline_matches" ]; then
  echo "Found inline styles in:"
  echo "$inline_matches" | sed 's/^/  /'
  VIOLATIONS=$((VIOLATIONS + 1))
else
  echo "  ✓ No inline styles found"
fi

# 2. Non-zero border-radius in CSS
print_header "2/6  Non-zero border-radius"
radius_matches=$(grep -rn 'border-radius' --include="*.css" "$WEB" 2>/dev/null | grep -vE '(0px|0;)' || true)
if [ -n "$radius_matches" ]; then
  echo "$radius_matches" | sed 's/^/  /'
  VIOLATIONS=$((VIOLATIONS + 1))
else
  echo "  ✓ All border-radius are 0px"
fi

# 3. Blur shadows in CSS
print_header "3/6  Blur shadows"
blur_shadow_matches=$(grep -rn 'box-shadow' --include="*.css" "$WEB" 2>/dev/null | grep -vE '(0px 0px|--shadow-)' || true)
if [ -n "$blur_shadow_matches" ]; then
  echo "Found box-shadow values not using token variables:"
  echo "$blur_shadow_matches" | sed 's/^/  /'
  VIOLATIONS=$((VIOLATIONS + 1))
else
  echo "  ✓ All box-shadows use --shadow-* tokens"
fi

# 4. Hardcoded hex/rgb colors in CSS (not in tokens.css)
print_header "4/6  Hardcoded colors in CSS"
# Skip tokens.css since that's where they're defined
color_matches=$(grep -rn '#[0-9a-fA-F]\{3,\}\|rgba\|rgb(' --include="*.css" "$WEB" 2>/dev/null \
  | grep -v 'tokens.css' \
  | grep -vE '(--.*:|noise-overlay|scanline|::-webkit-scrollbar)' || true)
if [ -n "$color_matches" ]; then
  echo "Hardcoded colors found (use CSS variables instead):"
  echo "$color_matches" | sed 's/^/  /'
  VIOLATIONS=$((VIOLATIONS + 1))
else
  echo "  ✓ No hardcoded colors outside tokens.css"
fi

# 5. Decorative gradients in CSS
print_header "5/6  Decorative gradients"
gradient_matches=$(grep -rn 'linear-gradient\|radial-gradient\|conic-gradient' --include="*.css" "$WEB" 2>/dev/null \
  | grep -v 'scanline' || true)
if [ -n "$gradient_matches" ]; then
  echo "Gradients found (only .scanline is allowed):"
  echo "$gradient_matches" | sed 's/^/  /'
  VIOLATIONS=$((VIOLATIONS + 1))
else
  echo "  ✓ No decorative gradients"
fi

# 6. Colored border-left in CSS (AI slop pattern)
print_header "6/6  Colored border-left (AI slop pattern)"
border_matches=$(grep -rn 'border-left' --include="*.css" "$WEB" 2>/dev/null || true)
if [ -n "$border_matches" ]; then
  echo "border-left found (prefer 2px border instead):"
  echo "$border_matches" | sed 's/^/  /'
  VIOLATIONS=$((VIOLATIONS + 1))
else
  echo "  ✓ No border-left patterns"
fi

echo ""
echo "═══════════════════════════"
if [ "$VIOLATIONS" -eq 0 ]; then
  echo "  ✓ All checks passed"
  exit 0
else
  echo "  ✗ $VIOLATIONS violation groups found"
  exit 1
fi
