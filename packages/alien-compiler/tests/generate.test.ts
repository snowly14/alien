import { expect, test } from "bun:test";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { generateProject } from "../src/generate.ts";

test("portable generation keeps source files, validates before emitting, and removes owned stale output", async () => {
  const root = await mkdtemp(join(tmpdir(), "alien-generate-"));
  try {
    await mkdir(join(root, "src"));
    await writeFile(
      join(root, "alien.globals.ts"),
      'import {alien} from "alien";alien.globals({mode:"strict",reset:"light",color:{brand:"#123456"}});',
    );
    const input = join(root, "src/button.alien.ts");
    const generated = join(root, "src/button.alien.generated.ts");
    const source =
      'import {alien,tokens} from "alien";export const styles=alien.create({button:{color:tokens.color.brand}});';
    await writeFile(input, source);
    const first = await generateProject(root);
    expect(first.modules).toBe(1);
    expect(await readFile(input, "utf8")).toBe(source);
    const css = await readFile(first.cssPath, "utf8");
    expect(css).toContain("#123456");
    expect(css).toContain("alien-reset");
    const module = await readFile(generated, "utf8");
    expect(module).not.toContain("alien.create(");
    await writeFile(input, source.replace("tokens.color.brand", '"red"'));
    await expect(generateProject(root)).rejects.toThrow("color token");
    expect(await readFile(first.cssPath, "utf8")).toBe(css);
    expect(await readFile(generated, "utf8")).toBe(module);
    await writeFile(input, source);
    await writeFile(generated, "// Handwritten\n");
    await expect(generateProject(root)).rejects.toThrow("Refusing to overwrite");
    await writeFile(generated, module);
    await rm(input);
    await generateProject(root);
    expect(await Bun.file(generated).exists()).toBe(false);
    expect(await readFile(first.cssPath, "utf8")).not.toContain("#123456");
    await expect(generateProject(root, { source: ".." })).rejects.toThrow(
      "inside the project root",
    );
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("watch generation refreshes styles and configuration, recovers from errors and handles deletion", async () => {
  const root = await mkdtemp(join(tmpdir(), "alien-watch-"));
  await mkdir(join(root, "src"));
  const entry = join(root, "src/button.alien.mts");
  const css = join(root, ".alien/styles.css");
  const globals = join(root, "alien.globals.ts");
  const source = (color: string) =>
    `import {alien} from "alien";export const s=alien.create({button:{color:"${color}"}});`;
  await writeFile(entry, source("red"));
  const cli = new URL("../src/cli.ts", import.meta.url).pathname;
  const child = Bun.spawn([process.execPath, cli, "generate", root, "--watch"], {
    stdout: "ignore",
    stderr: "ignore",
  });
  async function until(check: () => Promise<boolean>) {
    const deadline = Date.now() + 5000;
    while (!(await check())) {
      if (Date.now() > deadline) throw new Error("Generator did not update within five seconds");
      await new Promise((resolve) => setTimeout(resolve, 20));
    }
  }
  async function contains(text: string) {
    return (await Bun.file(css).exists()) && (await Bun.file(css).text()).includes(text);
  }
  try {
    await until(() => contains("color:red"));
    await writeFile(entry, source("blue"));
    await until(() => contains("color:blue"));
    await writeFile(entry, source("invalid-color"));
    await writeFile(globals, 'import {alien} from "alien";alien.globals({reset:"strong"});');
    await new Promise((resolve) => setTimeout(resolve, 150));
    expect(await contains("color:blue")).toBe(true);
    await writeFile(entry, source("green"));
    await until(async () => (await contains("color:green")) && (await contains("border-collapse")));
    await rm(entry);
    await until(async () => !(await contains("color:green")));
    expect(await Bun.file(join(root, "src/button.alien.generated.mts")).exists()).toBe(false);
  } finally {
    child.kill("SIGTERM");
    await child.exited;
    await rm(root, { recursive: true, force: true });
  }
}, 10_000);
