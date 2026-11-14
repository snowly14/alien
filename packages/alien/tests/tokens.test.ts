import { expect, test } from "bun:test";
import { fileURLToPath } from "node:url";
import { tokens } from "alien";

test("every built-in token path exists in both Rust and TypeScript", () => {
  const inventory = Bun.spawnSync(
    ["cargo", "run", "--quiet", "-p", "alien-tokens", "--example", "builtin_paths"],
    { cwd: fileURLToPath(new URL("../../../", import.meta.url)), stderr: "pipe" },
  );

  if (!inventory.success) {
    throw new Error(inventory.stderr.toString());
  }

  const { color, ...families } = tokens;
  const colorPaths = Object.values(color).flatMap((entry) =>
    "path" in entry ? [entry.path] : Object.values(entry).map((token) => token.path),
  );
  const otherPaths = Object.values(families).flatMap((family) =>
    Object.values(family).map((token) => token.path),
  );

  expect([...colorPaths, ...otherPaths].sort()).toEqual(
    inventory.stdout.toString().trim().split("\n").sort(),
  );
});
