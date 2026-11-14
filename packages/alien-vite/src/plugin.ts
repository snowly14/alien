import { resolve } from "node:path";
import { moduleIdentity, projectNamespace } from "alien-compiler";
import MagicString from "magic-string";
import { normalizePath, type Plugin, type ViteDevServer } from "vite";
import { isSupported, type ModuleStyles, transformModule } from "./components.ts";
import { prepareProject } from "./configuration.ts";
import type { NativeCompiler } from "./native.ts";

import { resetCss } from "./reset.ts";

const resetUrl = "/@alien/reset.css";
const cssPrefix = "virtual:alien/";
const resolvedCssPrefix = `\0${cssPrefix}`;
const resetId = `${cssPrefix}reset.css`;

export type AlienOptions = {
  /** Project root when the framework uses a different Vite root (for example Nuxt). */
  root?: string;
  /** Configuration path relative to the Alien project root. Defaults to alien.globals.ts. */
  globals?: string;
  /** Stable namespace for CSS shared by independently built applications. */
  namespace?: string;
};

export default function alien(options: AlienOptions = {}): Plugin {
  let compiler: NativeCompiler;
  const modules = new Map<string, { source: string; result: ModuleStyles | null }>();
  const versions = new Map<string, symbol>();
  const classOwners = new Map<string, string>();
  let root = process.cwd();
  let namespace = options.namespace ?? "";
  let globalsPath = "";

  function identify(path: string) {
    const { id: moduleId, digest, prefix } = moduleIdentity(root, namespace, path);
    const owner = classOwners.get(prefix);
    if (owner !== undefined && owner !== moduleId)
      throw new Error(`Alien class-name collision between ${owner} and ${moduleId}`);
    classOwners.set(prefix, moduleId);
    return { moduleId, cssId: `${cssPrefix}${digest}.css` };
  }

  async function compile(
    source: string,
    path: string,
    moduleId: string,
    cssId: string,
  ): Promise<ModuleStyles | null> {
    const cached = modules.get(cssId);
    if (cached?.source === source) return cached.result;
    const request = Symbol();
    const activeCompiler = compiler;
    versions.set(cssId, request);
    const result = await transformModule(activeCompiler, source, path, moduleId);
    if (compiler !== activeCompiler) return compile(source, path, moduleId, cssId);
    if (versions.get(cssId) === request) modules.set(cssId, { source, result });
    return result;
  }

  let reloadQueue = Promise.resolve();
  function reloadConfiguration(server: ViteDevServer) {
    const reload = reloadQueue.then(async () => {
      compiler = await prepareProject(root, options.globals);
      modules.clear();
      versions.clear();
      server.moduleGraph.invalidateAll();
      server.ws.send({ type: "full-reload" });
    });
    reloadQueue = reload.catch(() => {});
    return reload;
  }

  return {
    name: "alien",
    enforce: "pre",
    async configResolved(config) {
      root = resolve(config.root, options.root ?? ".");
      globalsPath = normalizePath(resolve(root, options.globals ?? "alien.globals.ts"));
      namespace = await projectNamespace(root, options.namespace);
    },
    async buildStart() {
      compiler = await prepareProject(root, options.globals);
      modules.clear();
      versions.clear();
      classOwners.clear();
      this.addWatchFile(globalsPath);
    },
    configureServer(server) {
      server.watcher.add(globalsPath);
      const onConfigurationCreatedOrRemoved = (file: string) => {
        if (normalizePath(file) !== globalsPath) return;
        void reloadConfiguration(server).catch((error: unknown) => {
          server.ws.send({
            type: "error",
            err: {
              message: error instanceof Error ? error.message : String(error),
              stack: "",
              plugin: "alien",
            },
          });
        });
      };
      server.watcher.on("add", onConfigurationCreatedOrRemoved);
      server.watcher.on("unlink", onConfigurationCreatedOrRemoved);
      server.watcher.on("unlink", (file) => {
        if (isSupported(file)) {
          const { cssId } = identify(normalizePath(file));
          modules.delete(cssId);
          versions.delete(cssId);
        }
      });
    },
    transform: {
      order: "pre",
      async handler(source, id) {
        const path = normalizePath(id);
        if (
          !isSupported(path) ||
          path.includes("/node_modules/") ||
          path.includes("?") ||
          path.startsWith("\0")
        )
          return null;
        if (path === globalsPath) {
          this.error(
            "Do not import alien.globals.ts at runtime. Import alien and tokens from 'alien'; the plugin loads configuration statically.",
          );
        }
        const { moduleId, cssId } = identify(path);
        const result = await compile(source, path, moduleId, cssId);
        if (!result) return null;
        const code = new MagicString(source);
        for (const edit of result.edits) code.overwrite(edit.start, edit.end, edit.replacement);
        if (result.css) {
          const imports = `${compiler.reset ? `import ${JSON.stringify(resetId)};\n` : ""}import ${JSON.stringify(cssId)};\n`;
          code.appendLeft(result.importPosition, `\n${imports}`);
        }
        return {
          code: code.toString(),
          map: code.generateMap({ hires: true, source: path, includeContent: true }),
        };
      },
    },
    transformIndexHtml: {
      order: "pre",
      handler() {
        if (!compiler.reset) return;
        return [
          { tag: "link", attrs: { rel: "stylesheet", href: resetUrl }, injectTo: "head-prepend" },
        ];
      },
    },
    resolveId(id) {
      if (id === "alien-vite/reset.css") return `\0${resetId}`;
      if (id === resetUrl || id.startsWith(`${resetUrl}?`)) {
        return `\0${resetId}${id.slice(resetUrl.length)}`;
      }
      if (id.startsWith(cssPrefix)) return `\0${id}`;
    },
    load(id) {
      if (id.split("?", 1)[0] === `\0${resetId}`) return resetCss(compiler.reset);
      if (id.startsWith(resolvedCssPrefix)) return modules.get(id.slice(1))?.result?.css ?? "";
    },
    handleHotUpdate: {
      order: "post",
      async handler({ file, modules: changed, read, server }) {
        if (normalizePath(file) === globalsPath) {
          await reloadConfiguration(server);
          return [];
        }
        if (!isSupported(file)) return;
        const { moduleId, cssId } = identify(file);
        const cssModule = server.moduleGraph.getModuleById(`\0${cssId}`);
        if (!cssModule) return;
        await compile(await read(), file, moduleId, cssId);
        return [...new Set([...changed, cssModule])];
      },
    },
  };
}
