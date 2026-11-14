import type { NativeCompiler, SourceEdit } from "alien-compiler";

export const isScript = (path: string) =>
  /\.(?:[cm]?[jt]s|[jt]sx)$/.test(path) && !/\.d\.[cm]?ts$/.test(path);
export const isComponent = (path: string) => /\.(vue|svelte|astro)$/.test(path);
export const isSupported = (path: string) => isScript(path) || isComponent(path);

type ScriptBlock = { start: number; end: number; extension: string; name: string };
export type ModuleStyles = { css: string; edits: SourceEdit[]; importPosition: number };

async function scriptBlocks(source: string, path: string): Promise<ScriptBlock[]> {
  if (path.endsWith(".vue")) {
    const { parse } = await import("vue/compiler-sfc");
    const { descriptor, errors } = parse(source, { filename: path });
    if (errors.length) throw new Error(`${path}: ${errors.map(String).join("\n")}`);
    return [descriptor.script, descriptor.scriptSetup].flatMap((script, index) => {
      if (!script || script.src) return [];
      if (script.lang && !["js", "jsx", "ts", "tsx"].includes(script.lang)) return [];
      return [
        {
          start: script.loc.start.offset,
          end: script.loc.end.offset,
          extension: script.lang ?? "js",
          name: index === 0 ? "script" : "setup",
        },
      ];
    });
  }
  if (path.endsWith(".svelte")) {
    const { parse } = await import("svelte/compiler");
    const ast = parse(source, { modern: true, filename: path });
    return [ast.module, ast.instance].flatMap((script, index) => {
      if (
        !script ||
        !("start" in script.content) ||
        !("end" in script.content) ||
        typeof script.content.start !== "number" ||
        typeof script.content.end !== "number"
      )
        return [];
      return [
        {
          start: script.content.start,
          end: script.content.end,
          extension: "ts",
          name: index === 0 ? "module" : "instance",
        },
      ];
    });
  }
  const { parse } = await import("@astrojs/compiler");
  const { ast, diagnostics } = await parse(source, { position: true });
  const errors = diagnostics.filter((diagnostic) => diagnostic.severity === 1);
  if (errors.length) throw new Error(`${path}: ${errors.map((error) => error.text).join("\n")}`);
  return ast.children.flatMap((node) => {
    if (node.type !== "frontmatter" || !node.position) return [];
    // Frontmatter starts with a three-character delimiter. Its value is the
    // original script text; use JS string lengths to preserve UTF-16 offsets.
    const start = node.position.start.offset + 3;
    return [{ start, end: start + node.value.length, extension: "ts", name: "frontmatter" }];
  });
}

/** Compile scripts before framework plugins lower them into component functions. */
export async function transformModule(
  compiler: NativeCompiler,
  source: string,
  path: string,
  moduleId: string,
): Promise<ModuleStyles | null> {
  if (isScript(path)) {
    const result = compiler.transform(source, path, moduleId);
    return result && { css: result.css, edits: result.edits, importPosition: source.length };
  }
  const blocks = await scriptBlocks(source, path);
  const edits: SourceEdit[] = [];
  let css = "";
  let importPosition = 0;
  for (const block of blocks) {
    const result = compiler.transform(
      source.slice(block.start, block.end),
      `${path}.${block.extension}`,
      `${moduleId}#${block.name}`,
    );
    if (!result) continue;
    if (!edits.length) importPosition = block.end;
    css += result.css;
    for (const edit of result.edits) {
      edits.push({ ...edit, start: block.start + edit.start, end: block.start + edit.end });
    }
  }
  return edits.length ? { css, edits, importPosition } : null;
}
