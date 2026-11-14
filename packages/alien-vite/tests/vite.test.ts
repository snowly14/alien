import { expect, test } from "bun:test";
import { mkdtemp, readFile, realpath, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { build } from "vite";
import { prepareProject } from "../src/configuration.ts";
import alien from "../src/plugin.ts";

const entryPoint = fileURLToPath(new URL("../../alien/src/index.ts", import.meta.url));

test("a real Vite build emits strict project CSS and source maps without runtime styling", async () => {
  const root = await mkdtemp(join(tmpdir(), "alien-vite-"));
  try {
    await writeFile(
      join(root, "alien.globals.ts"),
      'import {alien} from "alien";export default alien.globals({mode:"strict",color:{brand:"#123456"},space:{control:"12px"}})',
    );
    await writeFile(
      join(root, "main.ts"),
      'import {alien,tokens} from "alien";const title="👽";export const styles=alien.create({button:{color:tokens.color.brand,padding:tokens.space.control}});export {title};',
    );
    const result = await build({
      configFile: false,
      root,
      logLevel: "silent",
      plugins: [alien({ namespace: "integration" })],
      resolve: { alias: { alien: entryPoint } },
      build: {
        write: false,
        sourcemap: true,
        minify: false,
        lib: { entry: join(root, "main.ts"), formats: ["es"], fileName: "main" },
      },
    });
    const builds = Array.isArray(result) ? result : [result];
    const outputs = builds.flatMap((build) => ("output" in build ? build.output : []));
    const css = outputs
      .filter((output) => output.type === "asset" && output.fileName.endsWith(".css"))
      .map((output) => (output.type === "asset" ? output.source.toString() : ""))
      .join("");
    const chunks = outputs.filter((output) => output.type === "chunk");
    expect(css).toContain("#123456");
    expect(css).toContain("12px");
    expect(chunks.length).toBeGreaterThan(0);
    expect(chunks[0]!.code).not.toContain("must be compiled by Alien");
    expect(chunks[0]!.map?.sources.some((source) => source.endsWith("main.ts"))).toBe(true);
    const types = await readFile(join(root, "alien-env.d.ts"), "utf8");
    expect(types).toContain('mode: "strict"');
    expect(types).toContain('"brand"');
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("preparation removes stale token names and preserves handwritten declarations", async () => {
  const root = await mkdtemp(join(tmpdir(), "alien-prepare-"));
  try {
    const config = join(root, "alien.globals.ts");
    const types = join(root, "alien-env.d.ts");
    await writeFile(
      config,
      'import {alien} from "alien";alien.globals({mode:"strict",color:{old:"red"}})',
    );
    await prepareProject(root);
    expect(await readFile(types, "utf8")).toContain('"old"');
    await writeFile(
      config,
      'import {alien} from "alien";alien.globals({mode:"free",color:{new:"blue"}})',
    );
    await prepareProject(root);
    const updated = await readFile(types, "utf8");
    expect(updated).not.toContain('"old"');
    expect(updated).toContain('"new"');
    await expect(prepareProject(root, "missing.ts")).rejects.toThrow(
      "Alien configuration not found",
    );
    expect(await readFile(types, "utf8")).toBe(updated);
    await rm(config);
    await prepareProject(root);
    expect(await readFile(types, "utf8")).not.toContain('"new"');
    await writeFile(types, "// A user-owned file\n");
    await expect(prepareProject(root)).rejects.toThrow("Refusing to overwrite");
    expect(await readFile(types, "utf8")).toBe("// A user-owned file\n");
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("development updates refresh CSS and reload project configuration", async () => {
  const { createServer } = await import("vite");
  const root = await realpath(await mkdtemp(join(tmpdir(), "alien-hmr-")));
  const entry = join(root, "main.ts");
  const config = join(root, "alien.globals.ts");
  const source = (color: string) =>
    `import {alien} from "alien";export const s=alien.create({a:{color:${color}}});`;
  await writeFile(entry, source('"red"'));
  const server = await createServer({
    configFile: false,
    root,
    logLevel: "silent",
    plugins: [alien({ namespace: "hmr-test" })],
    resolve: { alias: { alien: entryPoint } },
    server: { middlewareMode: true, watch: null },
    optimizeDeps: { noDiscovery: true },
  });
  const messages: unknown[] = [];
  const send = server.ws.send.bind(server.ws);
  server.ws.send = ((...args: Parameters<typeof send>) => {
    messages.push(args[0]);
    return send(...args);
  }) as typeof send;
  const hot = server.environments.client!.hot;
  const hotSend = hot.send.bind(hot);
  hot.send = ((...args: Parameters<typeof hotSend>) => {
    messages.push(args[0]);
    return hotSend(...args);
  }) as typeof hotSend;
  async function waitForUpdate() {
    const deadline = Date.now() + 3_000;
    while (
      !messages.some(
        (value) =>
          typeof value === "object" &&
          value !== null &&
          "type" in value &&
          (value.type === "full-reload" || value.type === "update"),
      )
    ) {
      if (Date.now() > deadline) throw new Error("Vite did not send an HMR update");
      await new Promise((resolve) => setTimeout(resolve, 10));
    }
    messages.length = 0;
  }
  const html = "<html><head></head><body></body></html>";
  try {
    expect(await server.transformIndexHtml("/index.html", html)).not.toContain("/@alien/reset.css");
    const first = await server.transformRequest("/main.ts");
    const cssId = first?.code.match(/virtual:alien\/[^"']+\.css/)?.[0];
    if (!cssId) throw new Error("Missing virtual stylesheet import");
    expect((await server.transformRequest(cssId))?.code).toContain("color:red");
    await writeFile(entry, source('"blue"'));
    server.watcher.emit("change", entry);
    await waitForUpdate();
    expect((await server.transformRequest(cssId))?.code).toContain("color:blue");
    await writeFile(
      config,
      'import {alien} from "alien";alien.globals({mode:"strict",reset:"medium",color:{brand:"#123456"}});',
    );
    server.watcher.emit("add", config);
    await waitForUpdate();
    expect(await readFile(join(root, "alien-env.d.ts"), "utf8")).toContain('mode: "strict"');
    expect(await server.transformIndexHtml("/index.html", html)).toContain("/@alien/reset.css");
    expect((await server.transformRequest("/@alien/reset.css?direct"))?.code).toContain(
      "max-inline-size",
    );
    await expect(server.transformRequest("/main.ts")).rejects.toThrow("color token");
    await rm(config);
    server.watcher.emit("unlink", config);
    await waitForUpdate();
    expect((await server.transformRequest("/main.ts"))?.code).toContain("virtual:alien/");
    expect(await readFile(join(root, "alien-env.d.ts"), "utf8")).toContain('mode: "free"');
    expect(await server.transformIndexHtml("/index.html", html)).not.toContain("/@alien/reset.css");
  } finally {
    await server.close();
    await rm(root, { recursive: true, force: true });
  }
});

test("the optional reset is emitted in HTML builds without Alien style calls", async () => {
  const root = await mkdtemp(join(tmpdir(), "alien-reset-"));
  try {
    await writeFile(
      join(root, "index.html"),
      "<!doctype html><html><head></head><body>Hello</body></html>",
    );
    for (const [option, markers] of [
      ["", []],
      ["false", []],
      ['"light"', ["box-sizing"]],
      ['"medium"', ["box-sizing", "max-inline-size"]],
      ['"strong"', ["box-sizing", "max-inline-size", "border-collapse"]],
    ] as const) {
      await writeFile(
        join(root, "alien.globals.ts"),
        `import {alien} from "alien"; alien.globals({${option ? `reset:${option}` : ""}});`,
      );
      const result = await build({
        configFile: false,
        root,
        base: "/demo/",
        plugins: [alien()],
        logLevel: "silent",
        build: { write: false },
      });
      const outputs = (Array.isArray(result) ? result : [result]).flatMap((build) =>
        "output" in build ? build.output : [],
      );
      const html = outputs.find(
        (output) => output.type === "asset" && output.fileName === "index.html",
      );
      const css = outputs.filter(
        (output) => output.type === "asset" && output.fileName.endsWith(".css"),
      );
      expect(css).toHaveLength(markers.length ? 1 : 0);
      if (markers.length) {
        expect(html?.type === "asset" && html.source.toString()).toContain("/demo/assets/");
        const output = css[0];
        const source = output?.type === "asset" ? output.source.toString() : "";
        expect(source).toContain("@layer alien-reset");
        for (const marker of ["box-sizing", "max-inline-size", "border-collapse"]) {
          expect(source.includes(marker)).toBe(markers.some((value) => value === marker));
        }
      }
    }
    await writeFile(
      join(root, "alien.globals.ts"),
      'import {alien} from "alien"; alien.globals({reset:"maximum"});',
    );
    await expect(prepareProject(root)).rejects.toThrow("reset must be false");
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

for (const framework of ["vue", "svelte"] as const)
  test(`${framework} updates compile before the framework and refresh virtual CSS`, async () => {
    const { createServer } = await import("vite");
    const frameworkPlugin =
      framework === "vue"
        ? (await import("@vitejs/plugin-vue")).default()
        : (await import("@sveltejs/vite-plugin-svelte")).svelte();
    const root = await realpath(await mkdtemp(join(tmpdir(), "alien-vue-hmr-")));
    await symlink(
      fileURLToPath(new URL("../../../node_modules", import.meta.url)),
      join(root, "node_modules"),
      "dir",
    );
    const fileName = `App.${framework}`;
    const entry = join(root, fileName);
    const source = (color: string) => {
      const script = `import {alien} from "alien";const s=alien.create({button:{color:"${color}"}});`;
      return framework === "vue"
        ? `<script setup lang="ts">${script}</script><template><button :class="s.button">Hello</button></template>`
        : `<script lang="ts">${script}</script><button class={s.button}>Hello</button>`;
    };
    await writeFile(entry, source("red"));
    const server = await createServer({
      configFile: false,
      root,
      logLevel: "silent",
      plugins: [alien(), frameworkPlugin],
      resolve: { alias: { alien: entryPoint } },
      server: { middlewareMode: true, watch: null },
      optimizeDeps: { noDiscovery: true },
    });
    try {
      const first = await server.transformRequest(`/${fileName}`);
      const cssId = first?.code.match(/virtual:alien\/[^"']+\.css/)?.[0];
      if (!cssId) throw new Error("Missing component virtual stylesheet");
      expect(first?.code).not.toContain("alien.create(");
      expect((await server.transformRequest(cssId))?.code).toContain("color:red");
      await writeFile(entry, source("blue"));
      server.watcher.emit("change", entry);
      const deadline = Date.now() + 3000;
      while (!(await server.transformRequest(cssId))?.code.includes("color:blue")) {
        if (Date.now() > deadline) throw new Error("Component CSS did not update");
        await new Promise((resolve) => setTimeout(resolve, 20));
      }
      expect((await server.transformRequest(`/${fileName}`))?.code).not.toContain("alien.create(");
    } finally {
      await server.close();
      await rm(root, { recursive: true, force: true });
    }
  });
