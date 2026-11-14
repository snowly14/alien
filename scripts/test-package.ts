import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const temporary = await mkdtemp(join(tmpdir(), "alien-package-"));
async function run(args: string[], cwd: string) {
  const process = Bun.spawn([Bun.which("bun") ?? "bun", ...args], {
    cwd,
    stdout: "inherit",
    stderr: "inherit",
  });
  const exitCode = await process.exited;
  if (exitCode !== 0) throw new Error(`bun ${args.join(" ")} failed with exit code ${exitCode}`);
}
try {
  for (const name of ["alien", "alien-compiler", "alien-vite", "alien-webpack"]) {
    await run(
      ["pm", "pack", "--ignore-scripts", "--filename", join(temporary, `${name}.tgz`)],
      join(root, "packages", name),
    );
  }
  await writeFile(
    join(temporary, "package.json"),
    JSON.stringify({
      name: "alien-installed-package-test",
      private: true,
      type: "module",
      overrides: { "alien-compiler": `file:${join(temporary, "alien-compiler.tgz")}` },
      dependencies: {
        alien: `file:${join(temporary, "alien.tgz")}`,
        "alien-compiler": `file:${join(temporary, "alien-compiler.tgz")}`,
        "alien-vite": `file:${join(temporary, "alien-vite.tgz")}`,
        "alien-webpack": `file:${join(temporary, "alien-webpack.tgz")}`,
        vite: "8.3.8",
        typescript: "5.9.3",
      },
    }),
  );
  await writeFile(
    join(temporary, "alien.globals.ts"),
    'import {alien} from "alien";export default alien.globals({mode:"strict",color:{brand:"#123456"}});',
  );
  await writeFile(
    join(temporary, "vite.config.ts"),
    'import alien from "alien-vite";export default {plugins:[alien()]};',
  );
  await writeFile(join(temporary, "index.html"), '<script type="module" src="/main.ts"></script>');
  await writeFile(
    join(temporary, "main.ts"),
    'import {alien,tokens} from "alien";const styles=alien.create({root:{color:tokens.color.brand}});document.body.className=styles.root;',
  );
  await writeFile(
    join(temporary, "type-check.ts"),
    `import {tokens, type Style} from "alien";
// @ts-expect-error strict mode also survives packaged declaration emission
const literal: Style = {color:"red"};
// @ts-expect-error undeclared project tokens are rejected
const missing = tokens.color.missing;
`,
  );
  await writeFile(
    join(temporary, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        strict: true,
        noEmit: true,
        skipLibCheck: true,
        target: "ES2023",
        module: "ESNext",
        moduleResolution: "Bundler",
        allowImportingTsExtensions: true,
      },
      include: ["main.ts", "type-check.ts", "alien-env.d.ts"],
    }),
  );
  await run(["install"], temporary);
  const smoke = (await readFile(join(root, "scripts/smoke-native.mjs"), "utf8")).replace(
    "../packages/alien-compiler/dist/index.js",
    "alien-compiler",
  );
  await writeFile(join(temporary, "native-smoke.mjs"), smoke);
  await run(["run", "native-smoke.mjs"], temporary);
  await run(["run", "alien", "prepare"], temporary);
  await run(["run", "tsc", "--noEmit"], temporary);
  await run(["run", "vite", "build"], temporary);
  const generated = await readFile(join(temporary, "alien-env.d.ts"), "utf8");
  if (!generated.includes('"brand"'))
    throw new Error("Installed CLI did not generate project tokens");
  console.log("Packed installation, native loading, generated types, and Vite build passed.");
} finally {
  await rm(temporary, { recursive: true, force: true });
}
