import { expect, test } from "bun:test";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import MiniCssExtractPlugin from "mini-css-extract-plugin";
import webpack from "webpack";
import { alienRule } from "../dist/index.js";

const require = createRequire(import.meta.url);
const alienEntry = fileURLToPath(new URL("../../alien/dist/index.js", import.meta.url));

test("webpack extracts static CSS, keeps directives and includes the configured reset", async () => {
  const root = await mkdtemp(join(tmpdir(), "alien-webpack-"));
  try {
    await writeFile(
      join(root, "package.json"),
      JSON.stringify({ name: "webpack-test", type: "module" }),
    );
    await writeFile(
      join(root, "alien.globals.ts"),
      'import {alien} from "alien";alien.globals({mode:"strict",reset:"light",color:{brand:"#123456"}});',
    );
    await writeFile(
      join(root, "entry.js"),
      '"use client";import {alien,tokens} from "alien";export const s=alien.create({button:{color:tokens.color.brand}});',
    );
    const compiler = webpack({
      mode: "production",
      context: root,
      entry: join(root, "entry.js"),
      output: { path: join(root, "dist"), filename: "app.js", library: { type: "module" } },
      experiments: { outputModule: true },
      resolve: { alias: { alien: alienEntry } },
      module: {
        rules: [
          alienRule({ namespace: "webpack-test" }),
          { test: /\.css$/, use: [MiniCssExtractPlugin.loader, require.resolve("css-loader")] },
        ],
      },
      plugins: [new MiniCssExtractPlugin({ filename: "styles.css" })],
      devtool: "source-map",
    });
    if (!compiler) throw new Error("No webpack compiler");
    try {
      await new Promise<void>((resolve, reject) =>
        compiler.run((error, stats) => {
          if (error) reject(error);
          else if (stats?.hasErrors())
            reject(new Error(stats.toString({ all: false, errors: true })));
          else resolve();
        }),
      );
    } finally {
      await new Promise<void>((resolve, reject) =>
        compiler.close((error) => (error ? reject(error) : resolve())),
      );
    }
    const css = await readFile(join(root, "dist/styles.css"), "utf8");
    const js = await readFile(join(root, "dist/app.js"), "utf8");
    expect(css).toContain("#123456");
    expect(css).toContain("alien-reset");
    expect(js).not.toContain("must be compiled by Alien");
    expect(js).not.toContain("alien.create(");
    expect(js.match(/a_[\w-]+/g)?.some((name) => css.includes(`.${name}`))).toBe(true);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}, 30_000);
