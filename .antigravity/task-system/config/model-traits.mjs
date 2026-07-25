// Model traits registry — configuración por modelo
// El harness consulta estos traits según -m <modelo>
// Formato: model name → { tool_mode, reasoning, history_window, num_ctx, ... }

const TIERS = [
  { tier: 0, label: "first attempt", models: ["haiku"], cost: "low" },
  { tier: 1, label: "1st retry", models: ["sonnet", "gpt-4o"], cost: "medium" },
  { tier: 2, label: "2nd retry", models: ["deepseek-v4", "deepseek-v4-flash-free"], cost: "high" },
  { tier: 3, label: "human escalation", models: [], cost: "human" },
]

const TRAITS = {
  "deepseek-v4-flash-free": {
    tool_mode: "native",
    reasoning: true,
    response_field: "content",
    history_window: 10,
    max_full_read_lines: 300,
    max_diff_lines: 500,
    unescape_tool_args: false,
    single_quote_json: false,
    num_ctx: 200000,
  },
  "deepseek-v4": {
    tool_mode: "native",
    reasoning: true,
    response_field: "content",
    history_window: 10,
    max_full_read_lines: 300,
    max_diff_lines: 500,
    unescape_tool_args: false,
    single_quote_json: false,
    num_ctx: 128000,
  },
  "sonnet": {
    tool_mode: "native",
    reasoning: false,
    response_field: "content",
    history_window: 8,
    max_full_read_lines: 200,
    max_diff_lines: 300,
    unescape_tool_args: false,
    single_quote_json: false,
    num_ctx: 100000,
  },
  "haiku": {
    tool_mode: "raw",
    reasoning: false,
    response_field: "content",
    history_window: 5,
    max_full_read_lines: 100,
    max_diff_lines: 150,
    unescape_tool_args: true,
    single_quote_json: true,
    num_ctx: 48000,
  },
  "gpt-4o": {
    tool_mode: "native",
    reasoning: false,
    response_field: "content",
    history_window: 8,
    max_full_read_lines: 200,
    max_diff_lines: 300,
    unescape_tool_args: false,
    single_quote_json: false,
    num_ctx: 128000,
  },
  "default": {
    tool_mode: "raw",
    reasoning: false,
    response_field: "content",
    history_window: 5,
    max_full_read_lines: 100,
    max_diff_lines: 200,
    unescape_tool_args: true,
    single_quote_json: false,
    num_ctx: 32768,
  },
}

function getTraits(modelName) {
  return TRAITS[modelName] || TRAITS["default"]
}

function listModels() {
  return Object.keys(TRAITS).filter(k => k !== "default")
}

function escalateTier(currentTier) {
  const next = TIERS[currentTier + 1]
  if (!next) return { tier: currentTier, label: "max", models: [], cost: "human" }
  return next
}

function tierForModel(modelName) {
  for (const t of TIERS) {
    if (t.models.includes(modelName)) return t.tier
  }
  return 0
}

export { getTraits, listModels, TRAITS, TIERS, escalateTier, tierForModel }
