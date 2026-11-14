import { expect, test } from "bun:test";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const cli = fileURLToPath(new URL("../src/cli.ts", import.meta.url));
async function run(root: string, args: string[]) {
  const child = Bun.spawn([process.execPath, cli, ...args], {
    cwd: root,
    stdout: "pipe",
    stderr: "pipe",
  });
  const [status, stdout, stderr] = await Promise.all([
    child.exited,
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
  ]);
  return { status, stdout, stderr };
}

test("prepare supports the same custom globals path as the Vite plugin", async () => {
  const root = await mkdtemp(join(tmpdir(), "alien-cli-"));
  try {
    await writeFile(
      join(root, "design.tokens.ts"),
      'import {alien} from "alien";export default alien.globals({mode:"strict",radius:{control:"6px"}});',
    );
    for (const args of [
      ["prepare", root, "--globals", "design.tokens.ts"],
      ["prepare", "--globals", "design.tokens.ts", root],
      ["prepare", "--globals", "design.tokens.ts"],
    ]) {
      const result = await run(root, args);
      expect(result.status).toBe(0);
      expect(result.stdout).toContain("strict mode");
      expect(await readFile(join(root, "alien-env.d.ts"), "utf8")).toContain('"control"');
    }
    for (const args of [
      ["prepare", "--globals"],
      ["prepare", root, root],
      ["prepare", "--unknown"],
      ["prepare", "--globals", "a.ts", "--globals", "b.ts"],
    ]) {
      const result = await run(root, args);
      expect(result.status).toBe(1);
      expect(result.stderr).toContain("Usage:");
    }
    const missing = await run(root, ["prepare", "--globals", "missing.ts"]);
    expect(missing.status).toBe(1);
    expect(missing.stderr).toContain("Alien configuration not found");
    expect(await readFile(join(root, "alien-env.d.ts"), "utf8")).toContain('mode: "strict"');
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
