import { describe, expect, it } from "vitest";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

/**
 * FIND-238 — `Client.create()` must not flood the console with the core's
 * `DEBUG` traces by default.
 *
 * The smoke spawns a fresh Node process per case: the real "first impression"
 * scenario (no vitest module cache, no prior tracing init — the subscriber is
 * global and one-shot per process) and inspects the actual stdout+stderr of
 * client creation.
 *
 * Contract:
 *  - default: no `DEBUG` lines, client creation succeeds;
 *  - `globalThis.VANTADB_LOG = "debug"`: `DEBUG` lines are back (documented gate).
 */
const tsRoot = fileURLToPath(new URL("../..", import.meta.url));

const CREATE_CLIENT = `
  import("vantadb-wasm").then((m) => {
    const db = new m.Client();
    db.close();
    console.log("CLIENT_CREATED_OK");
  }).catch((e) => {
    console.error(e);
    process.exit(1);
  });
`;

function runClientScript(body: string): { output: string; status: number | null } {
  const res = spawnSync(process.execPath, ["--input-type=module", "-e", body], {
    cwd: tsRoot,
    encoding: "utf8",
  });
  return { output: `${res.stdout}${res.stderr}`, status: res.status };
}

describe("FIND-238 — console output of Client.create()", () => {
  it("does not emit DEBUG lines by default", () => {
    const { output, status } = runClientScript(CREATE_CLIENT);

    expect(status).toBe(0);
    expect(output).toContain("CLIENT_CREATED_OK");
    expect(output).not.toMatch(/DEBUG/);
  });

  it("emits DEBUG lines when globalThis.VANTADB_LOG=debug (documented gate)", () => {
    const { output, status } = runClientScript(
      `globalThis.VANTADB_LOG = "debug";\n${CREATE_CLIENT}`,
    );

    expect(status).toBe(0);
    expect(output).toMatch(/DEBUG/);
  });
});
