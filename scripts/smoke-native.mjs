import assert from "node:assert/strict";
import { loadCompiler } from "../packages/alien-compiler/dist/index.js";

const output = loadCompiler().transform(
  'import { alien } from "alien"; export const styles = alien.create({ button: { display: "flex" } });',
  "button.ts",
  "./button.ts",
);
assert.match(output?.css ?? "", /display:flex/);
assert.match(output?.code ?? "", /"button"/);

const project = loadCompiler().transformProject(
  ["first", "second"].map((name) => ({
    source: 'import { alien } from "alien"; export const styles = alien.create({ button: { display: "flex" } });',
    path: `${name}.ts`,
    moduleId: `./${name}.ts`,
  })),
);
assert.equal(project.modules.length, 2);
assert.equal(project.css.match(/display:flex/g)?.length, 1);
assert.ok(project.modules.every((module) => module.css === "" && !module.code.includes("alien.create(")));
