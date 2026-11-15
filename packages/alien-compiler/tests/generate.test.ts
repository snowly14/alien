import { expect, test } from "bun:test";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { cx } from "../../alien/src/cx.ts";
import { generateProject } from "../src/generate.ts";

test("default atomic generation writes shared CSS and composable descriptors", async () => {
  const root = await mkdtemp(join(tmpdir(), "alien-atomic-generate-"));
  try {
    await mkdir(join(root, "src"));
    const source = (color: string) =>
      `import {alien} from '@alien-css/core';export const s=alien.create({item:{color:'${color}'}});`;
    await writeFile(join(root, "src/a.alien.ts"), source("red"));
    await writeFile(join(root, "src/b.alien.ts"), source("blue"));
    await writeFile(join(root, "src/c.alien.ts"), source("red"));
    const result = await generateProject(root);
    const css = await readFile(result.cssPath, "utf8");
    expect(css.match(/color:red/g)).toHaveLength(1);
    expect(css.match(/color:blue/g)).toHaveLength(1);
    const generated = await readFile(join(root, "src/c.alien.generated.ts"), "utf8");
    expect(generated).toMatch(/"[A-Za-z][A-Za-z0-9_-]{3}a~color"/);
    const [a, b, c] = await Promise.all(
      ["a", "b", "c"].map(
        (name) => import(pathToFileURL(join(root, `src/${name}.alien.generated.ts`)).href),
      ),
    );
    const className = cx(a.s.item, b.s.item, c.s.item);
    const markup = renderToStaticMarkup(
      createElement("button", { className, type: "button" }, "Save"),
    );
    expect(markup).toContain(`class="${className}"`);
    expect(css).toContain(`.${className}{color:red}`);
    const cli = new URL("../src/cli.ts", import.meta.url).pathname;
    const child = Bun.spawn([process.execPath, cli, "generate", root], {
      stdout: "pipe",
      stderr: "pipe",
    });
    const status = await child.exited;
    expect(status).toBe(0);
    expect(await readFile(result.cssPath, "utf8")).toBe(css);
    const strong = Bun.spawn(
      [
        process.execPath,
        cli,
        "generate",
        root,
        "--experimental-atomic",
        "--atomic-isolation",
        "strong",
      ],
      { stdout: "pipe", stderr: "pipe" },
    );
    expect(await strong.exited).toBe(0);
    expect(await readFile(result.cssPath, "utf8")).toMatch(/\.a_[\w-]{22}a\{/);
    const standard = Bun.spawn([process.execPath, cli, "generate", root, "--standard"], {
      stdout: "pipe",
      stderr: "pipe",
    });
    expect(await standard.exited).toBe(0);
    expect(await readFile(join(root, "src/c.alien.generated.ts"), "utf8")).toMatch(
      /"item":"a_[\w-]+"/,
    );
    await expect(
      generateProject(root, { standard: true, atomicIsolation: "strong" }),
    ).rejects.toThrow("cannot be used with standard");
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("portable generation keeps source files, validates before emitting, and removes owned stale output", async () => {
  const root = await mkdtemp(join(tmpdir(), "alien-generate-"));
  try {
    await mkdir(join(root, "src"));
    await writeFile(
      join(root, "alien.globals.ts"),
      'import {alien} from "@alien-css/core";alien.globals({mode:"strict",reset:"light",color:{brand:"#123456"}});',
    );
    const input = join(root, "src/button.alien.ts");
    const generated = join(root, "src/button.alien.generated.ts");
    const source =
      'import {alien,tokens} from "@alien-css/core";export const styles=alien.create({button:{color:tokens.color.brand}});';
    await writeFile(input, source);
    const first = await generateProject(root, { standard: true });
    expect(first.modules).toBe(1);
    expect(await readFile(input, "utf8")).toBe(source);
    const css = await readFile(first.cssPath, "utf8");
    expect(css).toContain("#123456");
    expect(css).toContain("alien-reset");
    const module = await readFile(generated, "utf8");
    expect(module).not.toContain("alien.create(");
    await writeFile(input, source.replace("tokens.color.brand", '"red"'));
    await expect(generateProject(root, { standard: true })).rejects.toThrow("color token");
    expect(await readFile(first.cssPath, "utf8")).toBe(css);
    expect(await readFile(generated, "utf8")).toBe(module);
    await writeFile(input, source);
    await writeFile(generated, "// Handwritten\n");
    await expect(generateProject(root, { standard: true })).rejects.toThrow(
      "Refusing to overwrite",
    );
    await writeFile(generated, module);
    await rm(input);
    await generateProject(root, { standard: true });
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
    `import {alien} from "@alien-css/core";export const s=alien.create({button:{color:"${color}"}});`;
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
    await writeFile(
      globals,
      'import {alien} from "@alien-css/core";alien.globals({reset:"strong"});',
    );
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
