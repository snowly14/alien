import { rm } from "node:fs/promises";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
for (const [name, entries] of [
  ["alien", ["index.ts", "values/index.ts", "cx.ts"]],
  ["alien-compiler", ["index.ts", "cli.ts"]],
  ["alien-vite", ["index.ts", "cli.ts"]],
  ["alien-webpack", ["index.ts", "loader.ts"]],
] as const) {
  const directory = join(root, "packages", name);
  await rm(join(directory, "dist"), { recursive: true, force: true });
  const result = await Bun.build({
    entrypoints: entries.map((entry) => join(directory, "src", entry)),
    root: join(directory, "src"),
    outdir: join(directory, "dist"),
    target: "node",
    format: "esm",
    splitting: true,
    packages: "external",
    sourcemap: "external",
  });
  if (!result.success) throw new AggregateError(result.logs, `Failed to build ${name}`);
  const declarations = Bun.spawn(
    ["bun", "run", "tsc", "-p", join(directory, "tsconfig.build.json")],
    { cwd: root, stdout: "inherit", stderr: "inherit" },
  );
  if ((await declarations.exited) !== 0) throw new Error(`Failed to emit declarations for ${name}`);
}
