import { expect, test } from "bun:test";
import { loadCompiler } from "alien-compiler";
import MagicString from "magic-string";
import { transformModule } from "../src/components.ts";

const compiler = loadCompiler();
const script =
  'import {alien as ui} from "alien";const title="👽";const styles=ui.create({button:{color:"red"}});';
for (const [extension, source] of [
  ["vue", `<template>👽</template><script lang="ts">${script}</script>`],
  [
    "svelte",
    `<script module>${script.replaceAll("styles", "shared")}</script><script>${script}</script><button class={styles.button}>👽</button>`,
  ],
  ["astro", `---\n${script}\n---\n<button class={styles.button}>👽</button>`],
] as const)
  test(`${extension} scripts preserve UTF-16 source offsets and distinct block identities`, async () => {
    const result = await transformModule(
      compiler,
      source,
      `app.${extension}`,
      `test/app.${extension}`,
    );
    expect(result).not.toBeNull();
    const code = new MagicString(source);
    for (const edit of result!.edits) {
      expect(source.slice(edit.start, edit.end)).toStartWith("ui.create(");
      code.overwrite(edit.start, edit.end, edit.replacement);
    }
    expect(code.toString()).not.toContain("ui.create(");
    expect(code.toString()).toContain("👽");
    if (extension === "svelte") {
      expect(result!.edits).toHaveLength(2);
      expect(result!.edits[0]!.replacement).not.toBe(result!.edits[1]!.replacement);
    }
  });

for (const extension of ["js", "jsx", "mjs", "ts", "tsx", "mts", "cts"]) {
  test(`plain ${extension} modules compile before the framework`, async () => {
    const result = await transformModule(compiler, script, `styles.${extension}`, "test/styles");
    expect(result?.css).toContain("color:red");
  });
}
