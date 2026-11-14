import { createRequire } from "node:module";
import { nativeFilename } from "./native-filename.ts";

export type SourceEdit = { start: number; end: number; replacement: string };
export type TransformedModule = { code: string; css: string; edits: SourceEdit[] };
export type ProjectInput = { source: string; path: string; moduleId: string };
export type TransformedProject = { modules: TransformedModule[]; css: string };
export type NativeCompiler = {
  readonly mode: "free" | "strict";
  readonly reset: "light" | "medium" | "strong" | null | undefined;
  readonly tokenPaths: string[];
  transformProject(sources: ProjectInput[]): TransformedProject;
  transform(source: string, path: string, moduleId: string): TransformedModule | null;
};
type NativeBinding = { Compiler: new (source?: string, path?: string) => NativeCompiler };

export function loadCompiler(source?: string, path?: string): NativeCompiler {
  const filename = nativeFilename();
  let binding: NativeBinding;
  try {
    binding = createRequire(import.meta.url)(`../native/${filename}`) as NativeBinding;
    if (typeof binding.Compiler !== "function")
      throw new Error("Incompatible native compiler version");
  } catch (cause) {
    throw new Error(
      `Alien could not load ${filename}. In the Alien repository, run bun run build:native. For an installed package, install a build matching your OS and architecture.`,
      { cause },
    );
  }
  // Keep configuration diagnostics separate from native loading failures.
  return new binding.Compiler(source, path);
}
