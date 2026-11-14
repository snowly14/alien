import { expect, test } from "bun:test";
import { loadCompiler, type ProjectInput, type TransformedModule } from "../src/native.ts";

const compiler = loadCompiler();
const input = (id: number, declarations: string): ProjectInput => ({
  source: `import {alien} from 'alien';export const label='👽';export const styles=alien.create({same:{${declarations}}});`,
  path: `${id}.alien.ts`,
  moduleId: `test/${id}.alien.ts`,
});
function classes(output: TransformedModule): Record<string, string> {
  const edit = output.edits[0];
  if (!edit) throw new Error("Missing source edit");
  return new Function(`return ${edit.replacement};`)();
}
function computed(css: string, names: string) {
  const selected = new Set(names.split(" "));
  const values: Record<string, string> = {};
  for (const [, name, declarations] of css.matchAll(/\.([\w-]+)\{([^{}]*)\}/g)) {
    if (!name || !declarations || !selected.has(name)) continue;
    for (const declaration of declarations.split(";")) {
      const colon = declaration.indexOf(":");
      values[declaration.slice(0, colon)] = declaration.slice(colon + 1);
    }
  }
  return values;
}

test("project compilation shares across files and preserves scoped keys, edits and individual styles", () => {
  const inputs = Array.from({ length: 30 }, (_, i) =>
    input(i, `color:'red',backgroundColor:'white',fontWeight:600,width:'${i}px'`),
  );
  const output = compiler.transformProject(inputs);
  expect(output.modules).toHaveLength(30);
  expect(output.css.match(/color:red/g)).toHaveLength(1);
  expect(output.css.match(/background-color:white/g)).toHaveLength(1);
  for (const [index, module] of output.modules.entries()) {
    const source = inputs[index]?.source;
    if (!source) throw new Error("Missing input");
    let reconstructed = source;
    for (const edit of module.edits.toReversed()) {
      reconstructed =
        reconstructed.slice(0, edit.start) + edit.replacement + reconstructed.slice(edit.end);
    }
    expect(reconstructed).toBe(module.code);
    expect(module.css).toBe("");
    const names = classes(module).same;
    if (!names) throw new Error("Missing style name");
    expect(computed(output.css, names)).toEqual({
      color: "red",
      "background-color": "white",
      "font-weight": "600",
      width: `${index}px`,
    });
  }
  expect(compiler.transformProject(inputs)).toEqual(output);
});

test("project sharing preserves cross-module conflicting declarations and stylesheet order", () => {
  const inputs = [
    input(0, "color:'red',display:'flex'"),
    input(1, "color:'blue',display:'grid'"),
    input(2, "color:'red',display:'flex'"),
  ];
  const output = compiler.transformProject(inputs);
  const maps = output.modules.map(classes);
  for (const [left, right] of [
    [0, 1],
    [1, 2],
    [0, 2],
  ] as const) {
    const names = `${maps[left]?.same} ${maps[right]?.same}`;
    expect(computed(output.css, names)).toEqual({
      color: right === 1 ? "blue" : "red",
      display: right === 1 ? "grid" : "flex",
    });
  }
});

test("multiple calls share rules while retaining repeated keys and empty calls", () => {
  const source = `import {alien} from 'alien';${Array.from({ length: 30 }, (_, i) => `export const s${i}=alien.create({same:{color:'red',backgroundColor:'white',fontWeight:600,width:'${i}px'}});`).join("")}export const empty=alien.create({});`;
  const output = compiler.transform(source, "calls.ts", "calls.ts");
  expect(output?.edits).toHaveLength(31);
  expect(output?.css.match(/color:red/g)).toHaveLength(1);
  expect(output?.edits.at(-1)?.replacement).toBe("({})");
});

test("project compilation rejects duplicate identities and missing styles before returning output", () => {
  const source = input(0, "color:'red'");
  expect(() => compiler.transformProject([source, source])).toThrow("duplicate module identity");
  expect(() => compiler.transformProject([{ ...source, source: "export const x=1;" }])).toThrow(
    "alien.create()",
  );
  expect(compiler.transformProject([])).toEqual({ modules: [], css: "" });
  expect(() => compiler.transformProject([input(0, "color:'not-a-color'")])).toThrow();
});

test("project sharing keeps conditional overrides attached to their original classes", () => {
  const output = compiler.transformProject([
    input(0, "color:'red',width:'10px'"),
    input(1, "color:'red',width:'20px',on:{hover:{color:'blue'}}"),
  ]);
  expect(output.css.match(/color:red/g)).toHaveLength(2);
  const second = output.modules[1];
  if (!second) throw new Error("Missing second module");
  expect(output.css).toContain(`.${classes(second).same}:hover{color:blue}`);
});

test("project class maps preserve special keys as own properties", () => {
  const source = input(0, "color:'red'");
  source.source = source.source.replace("same:", "'__proto__':");
  const output = compiler.transformProject([source]);
  const first = output.modules[0];
  if (!first) throw new Error("Missing first module");
  const map = classes(first);
  expect(Object.hasOwn(map, "__proto__")).toBe(true);
  expect(Object.getPrototypeOf(map)).toBe(Object.prototype);
});

test("cross-call conflicts retain profitable sharing inside each call", () => {
  const block = (weight: number) =>
    Array.from(
      { length: 30 },
      (_, i) =>
        `s${i}:{color:mix(tokens.color.green[600],tokens.color.blue[600],percent(${weight})),width:'${i}px'}`,
    ).join(",");
  const output = compiler.transform(
    `import {alien,tokens} from 'alien';import {mix,percent} from 'alien/values';export const a=alien.create({${block(25)}});export const b=alien.create({${block(75)}});`,
    "fallback.ts",
    "fallback.ts",
  );
  expect(output?.css.match(/color:color-mix/g)).toHaveLength(2);
  expect(output?.edits).toHaveLength(2);
  const maps = output?.edits.map(
    (edit) => new Function(`return ${edit.replacement};`)() as Record<string, string>,
  );
  const first = new Set(Object.values(maps?.[0] ?? {}).flatMap((value) => value.split(" ")));
  const second = new Set(Object.values(maps?.[1] ?? {}).flatMap((value) => value.split(" ")));
  expect([...first].filter((name) => second.has(name))).toHaveLength(0);
});
