export default {
  extends: ["stylelint-config-standard"],
  rules: {
    // Allow Tailwind v4 directives
    "at-rule-no-deprecated": [true, { ignoreAtRules: ["source", "custom-variant", "theme"] }],
    "at-rule-no-unknown": [true, { ignoreAtRules: ["source", "custom-variant", "theme"] }],

    // Colors
    "color-no-invalid-hex": true,
    "color-hex-length": "short",
    "color-function-notation": "modern",

    // Fonts
    "font-family-no-missing-generic-family-keyword": true,
    "font-family-name-quotes": "always-where-recommended",

    // Functions
    "function-url-no-scheme-relative": true,
    "function-no-unknown": true,

    // Values
    "length-zero-no-unit": true,
    "number-max-precision": 4,

    // Properties
    "property-no-unknown": true,
    "declaration-block-no-duplicate-properties": true,

    // Selectors — relaxed: allow kebab-case BEM AND camelCase (common for JS-driven classes)
    "selector-pseudo-class-no-unknown": true,
    "selector-pseudo-element-no-unknown": true,
    "selector-class-pattern": null,

    // Units — allow ch (character-based sizing) alongside standard units
    "unit-no-unknown": true,
    "unit-allowed-list": ["px", "rem", "em", "%", "vh", "vw", "s", "ms", "deg", "fr", "ch"],

    // No vendor prefixes (Vite handles autoprefixing)
    "property-no-vendor-prefix": true,
    "selector-no-vendor-prefix": true,
    "value-no-vendor-prefix": true,
    "value-keyword-case": ["lower", { camelCaseSvgKeywords: true }],

    // Shorthand
    "shorthand-property-no-redundant-values": true,

    // Comments
    "comment-no-empty": true,

    // Nesting
    "max-nesting-depth": 4,

    // Duplication
    "no-duplicate-selectors": true,
    "no-duplicate-at-import-rules": true,

    // Custom properties
    "custom-property-pattern": [
      "^([a-z][a-z0-9]*)(-[a-z0-9]+)*$",
      { message: "Custom property must use kebab-case" },
    ],
    "custom-property-no-missing-var-function": true,

    // Keyframes — relaxed: allow camelCase for JS-driven animations
    "keyframes-name-pattern": null,

    // !important — allowed (used for animation overrides), just warn
    "declaration-no-important": null,

    // Specificity ordering — disabled (too many false positives in large CSS)
    "no-descending-specificity": null,

    // Allow @import in tokens.css for Tailwind (stylelint wants it at top, but
    // tokens.css structure is intentional)
    "no-invalid-position-at-import-rule": null,
  },
};
