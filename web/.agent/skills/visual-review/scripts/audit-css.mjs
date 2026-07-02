/**
 * CSS Audit Runner
 * Eval script for Playwright CLI — checks computed styles against design standards.
 *
 * Uso:
 *   playwright-cli eval "$(cat .agent/skills/visual-review/scripts/audit-css.mjs)"
 *
 * Output: JSON array of issues with severity, category, selector, msg
 */

(function () {
  var s = getComputedStyle.bind(window);
  var issues = [];

  // ════════════════════════════════════════════
  // 1. MINIMUM FONT SIZE (11px for readability)
  // ════════════════════════════════════════════
  var textSelectors = [
    "p", "label", "span", "a", "li", "button", "td", "th",
    "figcaption", "small", "blockquote", "dd", "dt", "address",
    ".tl-tag", ".fg-label", ".framebar-label", ".hero-stat-label",
    ".term-cell-tag", ".term-cell-title", ".sp-step-tag", ".sp-step-title",
    ".tl-legend", ".sp-sidebar-num", ".sp-sidebar-icon", ".footer-col-title",
    ".section-eyebrow", ".section-sub", ".term-prompt", ".term-query",
    ".term-footer", ".quickstart-step",
  ];

  textSelectors.forEach(function (sel) {
    try {
      document.querySelectorAll(sel).forEach(function (el) {
        var fs = parseFloat(s(el).fontSize);
        if (fs > 0 && fs < 11) {
          issues.push({
            severity: "warn",
            category: "font-size",
            selector: sel,
            tag: el.tagName.toLowerCase(),
            class: (el.className || "").slice(0, 60),
            value: fs + "px",
            msg: 'Text under 11px',
          });
        }
      });
    } catch (e) {
      // invalid selector, skip
    }
  });

  // ════════════════════════════════════════════
  // 2. LOW OPACITY TEXT (potential contrast issue)
  // ════════════════════════════════════════════
  document.querySelectorAll("*").forEach(function (el) {
    var fs = parseFloat(s(el).fontSize);
    if (fs > 0 && fs < 24) {
      var op = parseFloat(s(el).opacity);
      if (op > 0 && op < 0.3) {
        var tag = el.tagName.toLowerCase();
        if (tag === "script" || tag === "style") return;
        issues.push({
          severity: "warn",
          category: "contrast",
          selector: tag + "." + ((el.className || "").slice(0, 30) || "(no class)"),
          value: op,
          msg: "Opacity " + op + " < 0.3 on text (" + fs + "px)",
        });
      }
    }
  });

  // ════════════════════════════════════════════
  // 3. HEADING HIERARCHY SKIPS
  // ════════════════════════════════════════════
  var headings = document.querySelectorAll("h1, h2, h3, h4, h5, h6");
  var lastLevel = 0;
  headings.forEach(function (h) {
    var level = parseInt(h.tagName[1]);
    if (level - lastLevel > 1 && lastLevel > 0) {
      issues.push({
        severity: "warn",
        category: "heading-hierarchy",
        selector: h.tagName,
        value: (h.textContent || "").trim().slice(0, 60),
        msg: "Heading skip: h" + lastLevel + " → " + h.tagName,
      });
    }
    lastLevel = level;
  });

  // ════════════════════════════════════════════
  // 4. TOUCH TARGET SIZE (mobile)
  // ════════════════════════════════════════════
  document.querySelectorAll("a, button, [role=button], input, select, textarea").forEach(function (el) {
    var w = parseFloat(s(el).width);
    var h = parseFloat(s(el).height);
    if (w > 0 && h > 0 && (w < 44 || h < 44)) {
      issues.push({
        severity: "info",
        category: "touch-target",
        selector: el.tagName.toLowerCase() + "." + ((el.className || "").slice(0, 30) || "(no class)"),
        value: w + "x" + h + "px",
        msg: "Touch target smaller than 44x44px",
      });
    }
  });

  // ════════════════════════════════════════════
  // 5. EMPTY ALT ON IMAGES
  // ════════════════════════════════════════════
  document.querySelectorAll("img:not([alt]), img[alt='']").forEach(function (el) {
    issues.push({
      severity: "info",
      category: "accessibility",
      selector: "img" + (el.className ? "." + el.className.slice(0, 30) : ""),
      value: el.src.slice(0, 80),
      msg: "Image missing alt text or has empty alt",
    });
  });

  // ════════════════════════════════════════════
  // 6. OVERFLOW / CLIPPED ELEMENTS
  // ════════════════════════════════════════════
  document.querySelectorAll("*").forEach(function (el) {
    var tag = el.tagName.toLowerCase();
    if (tag === "html" || tag === "body" || tag === "script" || tag === "style") return;
    var overflow = s(el).overflow;
    var overflowX = s(el).overflowX;
    var overflowY = s(el).overflowY;
    var clip = (overflow === "hidden" || overflowX === "hidden" || overflowY === "hidden");
    if (clip) {
      var rect = el.getBoundingClientRect();
      var parent = el.parentElement;
      if (parent) {
        var parentRect = parent.getBoundingClientRect();
        // Check if element extends beyond parent
        if (rect.right > parentRect.right + 2 || rect.bottom > parentRect.bottom + 2) {
          issues.push({
            severity: "info",
            category: "overflow",
            selector: tag + "." + ((el.className || "").slice(0, 30) || "(no class)"),
            value: overflow,
            msg: "Element clipped by overflow:hidden on parent",
          });
        }
      }
    }
  });

  // ════════════════════════════════════════════
  // 7. CONSOLE ERRORS (from page context)
  // ════════════════════════════════════════════
  // This is populated by the pipeline wrapper

  return JSON.stringify(
    issues.sort(function (a, b) {
      var order = { error: 0, warn: 1, info: 2 };
      return (order[a.severity] || 99) - (order[b.severity] || 99);
    }),
    null,
    2,
  );
})();
