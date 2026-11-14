import { mkdir, mkdtemp, readdir, readFile, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { generateProject } from "../packages/alien-compiler/src/generate.ts";

const repository = fileURLToPath(new URL("../", import.meta.url));
const node = Bun.which("node");
if (!node) throw new Error("Node.js is required to test framework CLIs.");
const source =
  'import {alien,tokens} from "alien";export const styles=alien.create({button:{color:tokens.color.brand,padding:tokens.space.control}});';
const globals =
  'import {alien} from "alien";export default alien.globals({mode:"strict",reset:"light",color:{brand:"#123456"},space:{control:"13px"}});';
type Fixture = {
  name: string;
  cli: string;
  args: string[];
  output: string;
  generate?: boolean;
  files: Record<string, string>;
};
const fixtures: Fixture[] = [
  {
    name: "Astro",
    cli: "astro/bin/astro.mjs",
    args: ["build"],
    output: "dist",
    files: {
      "astro.config.mjs":
        'import {defineConfig} from "astro/config";import alien from "alien-vite";export default defineConfig({vite:{plugins:[alien()]}});',
      "src/pages/index.astro": `---\n${source}\n---\n<html><head><title>Alien</title></head><body><button class={styles.button}>Hello</button></body></html>`,
    },
  },
  {
    name: "SvelteKit",
    cli: "vite/bin/vite.js",
    args: ["build"],
    output: "build",
    files: {
      "vite.config.js":
        'import {sveltekit} from "@sveltejs/kit/vite";import adapter from "@sveltejs/adapter-static";import alien from "alien-vite";export default {plugins:[alien(),sveltekit({adapter:adapter()})]};',
      "src/app.html":
        '<!doctype html><html><head>%sveltekit.head%</head><body><div style="display:contents">%sveltekit.body%</div></body></html>',
      "src/routes/+page.svelte": `<script>${source.replace("export const", "const")}</script><button class={styles.button}>Hello</button>`,
      "src/routes/+layout.js": "export const prerender=true;",
    },
  },
  {
    name: "Nuxt",
    cli: "nuxt/bin/nuxt.mjs",
    args: ["generate"],
    output: ".output/public",
    files: {
      "nuxt.config.ts":
        'import alien from "alien-vite";export default defineNuxtConfig({devtools:{enabled:false},telemetry:false,vite:{plugins:[alien({root:process.cwd()})]},compatibilityDate:"2026-11-13"});',
      "app/app.vue": `<script setup>${source.replace("export const", "const")}</script><template><button :class="styles.button">Hello</button></template>`,
    },
  },
  {
    name: "Next webpack",
    cli: "next/dist/bin/next",
    args: ["build", "--webpack"],
    output: "out",
    files: {
      "next.config.mjs":
        'import {alienRule} from "alien-webpack";export default {output:"export",webpack(config){config.module.rules.unshift(alienRule({root:process.cwd()}));return config;}};',
      "app/layout.jsx":
        "export default function Layout({children}){return <html><body>{children}</body></html>}",
      "app/page.jsx": `${source.replace("export const", "const")}import Button from "./Button";export default function Page(){return <main className={styles.button}><Button/></main>}`,
      "app/Button.jsx": `"use client";${source.replace("export const", "const")}export default function Button(){return <button className={styles.button} onClick={()=>{}}>Hello</button>}`,
    },
  },
  {
    name: "Next Turbopack",
    cli: "next/dist/bin/next",
    args: ["build", "--turbopack"],
    output: "out",
    generate: true,
    files: {
      "next.config.mjs": 'export default {output:"export",turbopack:{root:process.cwd()}};',
      "src/button.alien.ts": source,
      "app/layout.jsx":
        'import "../.alien/styles.css";export default function Layout({children}){return <html><body>{children}</body></html>}',
      "app/page.jsx":
        'import {styles} from "../src/button.alien.generated";export default function Page(){return <button className={styles.button}>Hello</button>}',
    },
  },
  {
    name: "Angular",
    cli: "@angular/cli/bin/ng.js",
    args: ["build"],
    output: "dist/browser",
    generate: true,
    files: {
      "src/button.alien.ts": source,
      "src/main.ts":
        'import {Component} from "@angular/core";import {bootstrapApplication} from "@angular/platform-browser";import {styles} from "./button.alien.generated";@Component({selector:"app-root",standalone:true,template:`<button [class]="styles.button">Hello</button>`})class App{readonly styles=styles;}bootstrapApplication(App);',
      "src/index.html":
        '<!doctype html><html><head><title>Alien</title><base href="/"></head><body><app-root></app-root></body></html>',
      "tsconfig.json": JSON.stringify({
        compilerOptions: {
          target: "ES2022",
          module: "preserve",
          moduleResolution: "bundler",
          experimentalDecorators: true,
          strict: true,
          skipLibCheck: true,
          lib: ["ES2022", "dom"],
          types: [],
        },
        files: ["src/main.ts"],
        include: ["alien-env.d.ts"],
        angularCompilerOptions: { strictTemplates: true },
      }),
      "angular.json": JSON.stringify({
        version: 1,
        cli: { analytics: false },
        projects: {
          app: {
            projectType: "application",
            root: "",
            sourceRoot: "src",
            architect: {
              build: {
                builder: "@angular/build:application",
                options: {
                  browser: "src/main.ts",
                  index: "src/index.html",
                  tsConfig: "tsconfig.json",
                  outputPath: "dist",
                  styles: [".alien/styles.css"],
                  assets: [],
                },
              },
            },
          },
        },
      }),
    },
  },
];
async function filesIn(directory: string): Promise<string[]> {
  const output: string[] = [];
  for (const file of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, file.name);
    if (file.isDirectory()) output.push(...(await filesIn(path)));
    else if (file.isFile()) output.push(path);
  }
  return output;
}
const requested = process.argv.slice(2);
for (const fixture of fixtures.filter(
  (fixture) =>
    !requested.length ||
    requested.some((name) => fixture.name.toLowerCase().includes(name.toLowerCase())),
)) {
  const root = await mkdtemp(join(tmpdir(), "alien-framework-build-"));
  console.log(`Building ${fixture.name}…`);
  try {
    if (fixture.name === "Next Turbopack") {
      const pack = Bun.spawn(
        [process.execPath, "pm", "pack", "--ignore-scripts", "--filename", join(root, "alien.tgz")],
        { cwd: join(repository, "packages/alien"), stdout: "ignore", stderr: "inherit" },
      );
      if (await pack.exited) throw new Error("Could not pack Alien for Turbopack fixture");
      const dependencies: Record<string, string> = {
        alien: "file:./alien.tgz",
        typescript: "~6.0.3",
        "@types/react": "^19.0.0",
        "@types/node": "^26.0.0",
      };
      for (const name of ["next", "react", "react-dom"])
        dependencies[name] = JSON.parse(
          await readFile(join(repository, "node_modules", name, "package.json"), "utf8"),
        ).version;
      await writeFile(
        join(root, "package.json"),
        JSON.stringify({
          name: "alien-framework-smoke",
          private: true,
          type: "module",
          dependencies,
        }),
      );
      const install = Bun.spawn([process.execPath, "install"], {
        cwd: root,
        stdout: "ignore",
        stderr: "inherit",
      });
      if (await install.exited) throw new Error("Could not install isolated Turbopack fixture");
    } else {
      await mkdir(join(root, "node_modules"));
      for (const name of await readdir(join(repository, "node_modules"))) {
        await symlink(join(repository, "node_modules", name), join(root, "node_modules", name));
      }
      for (const name of ["alien", "alien-compiler", "alien-vite", "alien-webpack"])
        await symlink(join(repository, "packages", name), join(root, "node_modules", name));
      // Vite is a workspace dependency of the React example, not the repository root.
      if (!(await Bun.file(join(root, "node_modules/vite/package.json")).exists()))
        await symlink(
          join(repository, "examples/react/node_modules/vite"),
          join(root, "node_modules/vite"),
        );
      await writeFile(
        join(root, "package.json"),
        JSON.stringify({ name: "alien-framework-smoke", private: true, type: "module" }),
      );
    }
    await writeFile(join(root, "alien.globals.ts"), globals);
    for (const [path, content] of Object.entries(fixture.files)) {
      await mkdir(dirname(join(root, path)), { recursive: true });
      await writeFile(join(root, path), content);
    }
    if (fixture.generate) await generateProject(root);
    const child = Bun.spawn([node, join(root, "node_modules", fixture.cli), ...fixture.args], {
      cwd: root,
      env: {
        ...process.env,
        NEXT_TELEMETRY_DISABLED: "1",
        NUXT_TELEMETRY_DISABLED: "1",
        ASTRO_TELEMETRY_DISABLED: "1",
        CI: "1",
      },
      stdout: "pipe",
      stderr: "pipe",
    });
    const timeout = setTimeout(() => child.kill(), 180_000);
    const [status, stdout, stderr] = await Promise.all([
      child.exited,
      new Response(child.stdout).text(),
      new Response(child.stderr).text(),
    ]);
    clearTimeout(timeout);
    if (status !== 0) throw new Error(`${fixture.name} failed (${status})\n${stdout}\n${stderr}`);
    const artifacts = await filesIn(join(root, fixture.output));
    const text = (
      await Promise.all(
        artifacts
          .filter((path) => /\.(css|html|js)$/.test(path))
          .map((path) => readFile(path, "utf8")),
      )
    ).join("\n");
    if (!text.includes("#123456") || !text.includes("13px") || !text.includes("alien-reset"))
      throw new Error(`${fixture.name}: missing compiled CSS/reset`);
    if (text.includes("alien.create() must be compiled"))
      throw new Error(`${fixture.name}: runtime compiler stub survived`);
    if (fixture.name !== "Angular") {
      const html = (
        await Promise.all(
          artifacts.filter((path) => path.endsWith(".html")).map((path) => readFile(path, "utf8")),
        )
      ).join("\n");
      if (!/class="a_\w+/.test(html))
        throw new Error(`${fixture.name}: rendered HTML has no Alien class`);
    }
    console.log(
      `${fixture.name}: build, CSS/reset and ${fixture.name === "Angular" ? "AOT compilation" : "rendered classes"} passed.`,
    );
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}
