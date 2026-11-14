import { expect, test } from "bun:test";
import { mkdtemp, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import react from "@vitejs/plugin-react";
import vue from "@vitejs/plugin-vue";
import { build, type PluginOption } from "vite";
import solid from "vite-plugin-solid";
import alien from "../src/plugin.ts";

const repository = fileURLToPath(new URL("../../../", import.meta.url));
const entryPoint = join(repository, "packages/alien/src/index.ts");
const styles =
  'import {alien,tokens} from "alien";const title="👽";const styles=alien.create({button:{color:tokens.color.brand}});';
const cases: { name: string; file: string; source: string; plugins: () => PluginOption[] }[] = [
  {
    name: "React JSX",
    file: "App.jsx",
    source: `${styles}export default function App(){return <button className={styles.button}>{title}</button>}`,
    plugins: () => [react()],
  },
  {
    name: "Solid JSX",
    file: "App.jsx",
    source: `${styles}export default function App(){return <button class={styles.button}>{title}</button>}`,
    plugins: () => [solid()],
  },
  {
    name: "Vue script setup",
    file: "App.vue",
    source: `<script setup lang="ts">${styles}</script><template><button :class="styles.button">{{title}}</button></template>`,
    plugins: () => [vue()],
  },
  {
    name: "Svelte instance script",
    file: "App.svelte",
    source: `<script lang="ts">${styles}</script><button class={styles.button}>{title}</button>`,
    plugins: () => [svelte()],
  },
];

for (const fixture of cases)
  test(`${fixture.name}: client and SSR builds share classes and emit CSS/reset`, async () => {
    const root = await mkdtemp(join(tmpdir(), "alien-framework-"));
    try {
      await symlink(join(repository, "node_modules"), join(root, "node_modules"), "dir");
      await writeFile(
        join(root, "package.json"),
        JSON.stringify({ name: "fixture", type: "module" }),
      );
      await writeFile(
        join(root, "alien.globals.ts"),
        'import {alien} from "alien";alien.globals({mode:"strict",reset:"light",color:{brand:"#123456"}});',
      );
      await writeFile(join(root, fixture.file), fixture.source);
      const outputs: { css: string; js: string }[] = [];
      for (const ssr of [false, true]) {
        const result = await build({
          configFile: false,
          root,
          logLevel: "silent",
          plugins: [alien({ namespace: "framework-fixture" }), ...fixture.plugins()],
          resolve: { alias: [{ find: /^alien$/, replacement: entryPoint }] },
          build: {
            write: false,
            minify: false,
            sourcemap: true,
            ssr: ssr ? join(root, fixture.file) : false,
            ssrEmitAssets: true,
            lib: { entry: join(root, fixture.file), formats: ["es"], fileName: "app" },
          },
        });
        const assets = (Array.isArray(result) ? result : [result]).flatMap((result) =>
          "output" in result ? result.output : [],
        );
        outputs.push({
          css: assets
            .filter((item) => item.type === "asset" && item.fileName.endsWith(".css"))
            .map((item) => (item.type === "asset" ? item.source.toString() : ""))
            .join(""),
          js: assets
            .filter((item) => item.type === "chunk")
            .map((item) => (item.type === "chunk" ? item.code : ""))
            .join(""),
        });
      }
      for (const output of outputs) {
        expect(output.css).toContain("#123456");
        expect(output.css).toContain("alien-reset");
        expect(output.js).not.toContain("must be compiled by Alien");
        expect(output.js).not.toContain("alien.create(");
      }
      expect(outputs[0]!.css.match(/\.a_[\w]+/g)).toEqual(outputs[1]!.css.match(/\.a_[\w]+/g));
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  }, 30_000);
