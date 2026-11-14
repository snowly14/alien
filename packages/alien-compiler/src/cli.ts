#!/usr/bin/env node
import { watch } from "node:fs";
import { dirname, resolve } from "node:path";
import { prepareProject } from "./configuration.ts";
import { type GenerateOptions, generateProject, isStyleSource } from "./generate.ts";

const usage =
  "Usage: alien prepare [project-root] [--globals path]\n       alien generate [project-root] [--source src] [--globals path] [--namespace name] [--watch]";
function parseArguments(args: string[]) {
  const [command, ...options] = args;
  if (command !== "prepare" && command !== "generate") throw new Error(usage);
  let root: string | undefined;
  let watching = false;
  const config: GenerateOptions = {};
  const remaining = options.values();
  for (const option of remaining) {
    if (option === "--watch" && command === "generate") {
      watching = true;
      continue;
    }
    if (
      option === "--globals" ||
      (command === "generate" && ["--source", "--namespace"].includes(option))
    ) {
      const value = remaining.next().value;
      const key = option.slice(2) as keyof GenerateOptions;
      if (config[key] !== undefined || !value || value.startsWith("-"))
        throw new Error(`${option} requires exactly one value.\n${usage}`);
      config[key] = value;
    } else if (option.startsWith("-") || root !== undefined) {
      throw new Error(`Unexpected argument: ${option}\n${usage}`);
    } else root = option;
  }
  return { command, root: resolve(root ?? process.cwd()), config, watching };
}

function report(error: unknown) {
  console.error(error instanceof Error ? error.message : String(error));
}

try {
  const { command, root, config, watching } = parseArguments(process.argv.slice(2));
  if (command === "prepare") {
    const compiler = await prepareProject(root, config.globals);
    console.log(
      `Alien: ${compiler.mode} mode; ${compiler.tokenPaths.length} project tokens. Types generated.`,
    );
  } else {
    const generate = async () => {
      const result = await generateProject(root, config);
      console.log(`Alien: ${result.modules} modules generated; CSS: ${result.cssPath}`);
    };
    if (!watching) await generate();
    else {
      let queue = Promise.resolve();
      let timer: ReturnType<typeof setTimeout> | undefined;
      const rebuild = () => {
        clearTimeout(timer);
        timer = setTimeout(() => {
          queue = queue.then(generate).catch(report);
        }, 50);
      };
      const sourceDirectory = resolve(root, config.source ?? "src");
      const globalsPath = resolve(root, config.globals ?? "alien.globals.ts");
      const watcher = watch(sourceDirectory, { recursive: true }, (event, filename) => {
        const path = filename?.replaceAll("\\", "/");
        if (
          path &&
          (path
            .split("/")
            .some((part) => ["node_modules", ".git", ".alien", "dist"].includes(part)) ||
            path.includes(".alien.generated.") ||
            path.endsWith(".tmp"))
        )
          return;
        if (!path || isStyleSource(path) || event === "rename") rebuild();
      });
      const configWatcher = watch(dirname(globalsPath), (_, filename) => {
        if (!filename || resolve(dirname(globalsPath), filename.toString()) === globalsPath)
          rebuild();
      });
      configWatcher.on("error", report);
      watcher.on("error", report);
      const close = () => {
        clearTimeout(timer);
        watcher.close();
        configWatcher.close();
      };
      process.once("SIGINT", close);
      process.once("SIGTERM", close);
      queue = queue.then(generate).catch(report);
      await queue;
    }
  }
} catch (error) {
  report(error);
  process.exitCode = 1;
}
