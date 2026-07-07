# VantaDB Visual Regression Tests

Playwright-based visual regression tests for the VantaDB HTTP server.

## Prerequisites

- Node.js 18+
- Playwright installed globally or in a parent project:
  ```bash
  npx playwright install chromium
  ```
- Rust toolchain (to build `vantadb-server`)

## How to run

```bash
cd e2e/visual
npx playwright test
```

The test configuration automatically builds and starts `vantadb-server` as a `webServer` before running tests.

## View the report

```bash
npx playwright show-report
```

## Updating baselines

When the server UI or response format changes intentionally, update the baselines:

```bash
npx playwright test --update-snapshots
```

This overwrites the `.png` files in `baselines/` with current screenshots. Commit them alongside the code change.

## How it works

1. The config starts `vantadb-server` on a random port (or `4173`).
2. Tests navigate to `/health`, `/metrics`, and `/` endpoints.
3. Screenshots are taken and compared against stored baselines.
4. Diffs are saved to `diffs/` when mismatches occur.

## Directory structure

```
e2e/visual/
├── playwright.config.ts    # Playwright configuration
├── vanta-visual.spec.ts    # Test spec
├── README.md               # This file
├── baselines/              # Stored reference screenshots
├── actual/                 # Screenshots from the last run
├── diffs/                  # Visual diff images (only on failure)
└── test-data/              # Temporary VantaDB data directory
```

## CI integration

In CI, set the `CI` environment variable to enable retries and HTML reporting:

```bash
CI=true npx playwright test
```
