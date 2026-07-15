import { tool } from "@opencode-ai/plugin"

export default tool({
  description: "Ejecuta un comando de validación y devuelve resultado estructurado. Contrato: cargo nextest, cargo check, cargo fmt, etc.",
  args: {
    command: tool.schema.string().describe("Comando a ejecutar (ej: 'cargo check -p vantadb-litellm', 'cargo nextest run --profile audit --workspace --build-jobs 2')"),
    expectedExitCode: tool.schema.number().optional().default(0).describe("Exit code esperado (default: 0)"),
    timeout: tool.schema.number().optional().default(300).describe("Timeout en segundos (default: 300)"),
    taskId: tool.schema.string().optional().describe("ID de tarea asociada para logging"),
  },
  async execute(args) {
    const startTime = Date.now()
    const result = await Bun.$`${args.command}`.nothrow().timeout(args.timeout * 1000)
    const elapsed = ((Date.now() - startTime) / 1000).toFixed(1)

    const stdout = result.stdout?.toString().trim() || ""
    const stderr = result.stderr?.toString().trim() || ""
    const exitCode = result.exitCode ?? -1
    const passed = exitCode === args.expectedExitCode

    // Extraer resumen de nextest si aplica
    const nextestMatch = stdout.match(/(\d+)\s+passed.*?(\d+)\s+failed/s)
    const summary = nextestMatch
      ? { passed: parseInt(nextestMatch[1]), failed: parseInt(nextestMatch[2]) }
      : null

    return {
      passed,
      exitCode,
      expectedExitCode: args.expectedExitCode,
      elapsed: `${elapsed}s`,
      taskId: args.taskId || null,
      summary,
      stdout: stdout.length > 2000 ? stdout.slice(0, 2000) + `\n... [truncated, ${stdout.length} total chars]` : stdout,
      stderr: stderr.length > 1000 ? stderr.slice(0, 1000) + `\n... [truncated, ${stderr.length} total chars]` : stderr,
    }
  },
})
