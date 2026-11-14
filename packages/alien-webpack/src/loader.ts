import { createHash, randomUUID } from "node:crypto";
import { mkdir, readFile, rename, rm, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { loadCompiler, moduleIdentity, projectNamespace, resetCss } from "alien-compiler";
import MagicString from "magic-string";
import type { LoaderContext } from "webpack";
import type { AlienWebpackOptions } from "./index.ts";

export default function loader(this: LoaderContext<AlienWebpackOptions>, source: string) {
  const done = this.async();
  const options = this.getOptions();
  const root = resolve(options.root ?? this.rootContext);
  const globalsPath = resolve(root, options.globals ?? "alien.globals.ts");
  const run = async () => {
    if (this.resourcePath === globalsPath)
      throw new Error("Do not import alien.globals.ts at runtime.");
    let configuration: string | undefined;
    try {
      configuration = await readFile(globalsPath, "utf8");
      this.addDependency(globalsPath);
    } catch (error) {
      if (!(error instanceof Error && "code" in error && error.code === "ENOENT")) throw error;
      this.addMissingDependency(globalsPath);
      if (options.globals !== undefined)
        throw new Error(`Alien configuration not found: ${globalsPath}`);
    }
    const compiler = loadCompiler(configuration, globalsPath);
    const namespace = await projectNamespace(root, options.namespace);
    this.addDependency(join(root, "package.json"));
    const { id } = moduleIdentity(root, namespace, this.resourcePath);
    const result = compiler.transform(source, this.resourcePath, id);
    if (!result) return { code: source, map: undefined };
    const code = new MagicString(source);
    for (const edit of result.edits) code.overwrite(edit.start, edit.end, edit.replacement);
    if (result.css) {
      const directory = join(root, ".alien", "css");
      await mkdir(directory, { recursive: true });
      // Content-addressed files cannot be overwritten by concurrent client/SSR
      // compilers. They are ordinary CSS dependencies handled by the host bundler.
      for (const css of [resetCss(compiler.reset), result.css]) {
        if (!css) continue;
        const digest = createHash("sha256").update(css).digest("hex");
        const path = join(directory, `${digest}.css`);
        let existing: string | undefined;
        try {
          existing = await readFile(path, "utf8");
        } catch (error) {
          if (!(error instanceof Error && "code" in error && error.code === "ENOENT")) throw error;
        }
        if (existing !== css) {
          const temporary = `${path}.${randomUUID()}.tmp`;
          try {
            await writeFile(temporary, css, "utf8");
            await rename(temporary, path);
          } finally {
            await rm(temporary, { force: true });
          }
        }
        code.append(`\nimport ${JSON.stringify(path.replaceAll("\\", "/"))};\n`);
      }
    }
    return {
      code: code.toString(),
      map: code.generateMap({ hires: true, source: this.resourcePath, includeContent: true }),
    };
  };
  void run().then(
    ({ code, map }) => done(null, code, map),
    (error: unknown) => done(error instanceof Error ? error : new Error(String(error))),
  );
}
