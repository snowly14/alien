import { expect, test } from "bun:test";
import { cx } from "../../alien/src/cx.ts";
import { loadCompiler, type ProjectInput, type TransformedModule } from "../src/native.ts";

const compiler = loadCompiler();

test("experimental atomic output composes repeated conflicting colors through cx", () => {
  const inputs = [input(0, "color:'red'"), input(1, "color:'blue'"), input(2, "color:'red'")];
  const output = compiler.transformAtomicProject(inputs);
  const maps = output.modules.map((module) => {
    const edit = module.edits[0];
    if (!edit) throw new Error("Missing source edit");
    return new Function(`return ${edit.replacement};`)();
  });
  expect(output.css.match(/color:red/g)).toHaveLength(1);
  expect(computed(output.css, cx(maps[0].same, maps[1].same, maps[2].same))).toEqual({
    color: "red",
  });
  expect(computed(output.css, cx(maps[2].same, maps[1].same))).toEqual({ color: "blue" });
});

test("experimental atomic output preserves important through compact tokens", () => {
  const important = input(0, "color:important('red')");
  important.source = `import {important} from '@alien-css/core/values';${important.source}`;
  const output = compiler.transformAtomicProject([important, input(1, "color:'blue'")]);
  const styles = output.modules.map((module) => {
    const edit = module.edits[0];
    if (!edit) throw new Error("Missing source edit");
    return new Function(`return ${edit.replacement};`)().same;
  });
  expect(computed(output.css, cx(styles[0], styles[1]))).toEqual({ color: "red!important" });
  expect(computed(output.css, cx(styles[1], styles[0]))).toEqual({ color: "red!important" });
});

test("experimental atomic output derives a stable short namespace from project identities", () => {
  const source = input(0, "color:'red'");
  const first = compiler.transformAtomicProject([source]);
  expect(compiler.transformAtomicProject([source])).toEqual(first);
  const other = compiler.transformAtomicProject([
    { ...source, moduleId: `other/${source.moduleId}` },
  ]);
  expect(first.css).not.toEqual(other.css);
});

test("experimental atomic output can use a 128-bit namespace with ordered variants", () => {
  const inputs = [input(0, "padding:'1rem'"), input(1, "paddingInline:'2rem'")];
  const first = compiler.transformAtomicProject(inputs, true);
  expect(first).toEqual(compiler.transformAtomicProject(inputs, true));
  const other = compiler.transformAtomicProject(
    inputs.map((source) => ({ ...source, moduleId: `other/${source.moduleId}` })),
    true,
  );
  const className = first.css.match(/\.([\w-]+)\{/)?.[1];
  expect(className).toMatch(/^a_[\w-]{22}a$/);
  expect(other.css).not.toEqual(first.css);
  const styles = first.modules.map((module) => {
    const edit = module.edits[0];
    if (!edit) throw new Error("Missing source edit");
    return new Function(`return ${edit.replacement};`)().same;
  });
  expect(cx(styles[0], styles[1]).split(" ")).toHaveLength(5);
  expect(cx(styles[1], styles[0]).split(" ")).toHaveLength(5);
});

test("experimental atomic output composes padding shorthand with a later side", () => {
  const output = compiler.transformAtomicProject([
    input(0, "padding:'1rem'"),
    input(1, "paddingTop:'2rem'"),
  ]);
  const maps = output.modules.map((module) => {
    const edit = module.edits[0];
    if (!edit) throw new Error("Missing source edit");
    return new Function(`return ${edit.replacement};`)();
  });
  expect(computed(output.css, cx(maps[0].same, maps[1].same))).toEqual({
    "padding-top": "2rem",
    "padding-right": "1rem",
    "padding-bottom": "1rem",
    "padding-left": "1rem",
  });
  expect(computed(output.css, cx(maps[1].same, maps[0].same))).toEqual({
    "padding-top": "1rem",
    "padding-right": "1rem",
    "padding-bottom": "1rem",
    "padding-left": "1rem",
  });
});

test("experimental atomic output orders logical and physical padding in either composition order", () => {
  const output = compiler.transformAtomicProject([
    input(0, "padding:'1rem'"),
    input(1, "paddingInline:'2rem'"),
  ]);
  const styles = output.modules.map((module) => {
    const edit = module.edits[0];
    if (!edit) throw new Error("Missing source edit");
    return new Function(`return ${edit.replacement};`)().same;
  });
  const orderedProperties = (names: string) => {
    const selected = new Set(names.split(" "));
    return [...output.css.matchAll(/\.([\w-]+)\{([^:{}]+):[^{}]+\}/g)]
      .filter(([, name]) => name && selected.has(name))
      .map(([, , property]) => property);
  };
  expect(orderedProperties(cx(styles[0], styles[1])).at(-1)).toBe("padding-inline");
  expect(orderedProperties(cx(styles[1], styles[0]))[0]).toBe("padding-inline");
});

test("experimental atomic output expands gap and border radius for independent overrides", () => {
  const output = compiler.transformAtomicProject([
    input(0, "gap:'1rem',borderRadius:'4px'"),
    input(1, "rowGap:'2rem',borderTopLeftRadius:'8px'"),
  ]);
  const styles = output.modules.map((module) => {
    const edit = module.edits[0];
    if (!edit) throw new Error("Missing source edit");
    return new Function(`return ${edit.replacement};`)().same;
  });
  expect(computed(output.css, cx(styles[0], styles[1]))).toEqual({
    "row-gap": "2rem",
    "column-gap": "1rem",
    "border-top-left-radius": "8px",
    "border-top-right-radius": "4px",
    "border-bottom-right-radius": "4px",
    "border-bottom-left-radius": "4px",
  });
});

test("experimental atomic output composes flex shorthand and longhands", () => {
  const output = compiler.transformAtomicProject([
    input(0, "flex:2"),
    input(1, "flexGrow:3"),
    input(2, "flex:'none'"),
    input(3, "flex:'auto'"),
  ]);
  const styles = output.modules.map((module) => {
    const edit = module.edits[0];
    if (!edit) throw new Error("Missing source edit");
    return new Function(`return ${edit.replacement};`)().same;
  });
  expect(computed(output.css, cx(styles[0], styles[1]))).toEqual({
    "flex-grow": "3",
    "flex-shrink": "1",
    "flex-basis": "0%",
  });
  expect(computed(output.css, cx(styles[1], styles[0]))).toEqual({
    "flex-grow": "2",
    "flex-shrink": "1",
    "flex-basis": "0%",
  });
  expect(computed(output.css, cx(styles[2]))).toEqual({
    "flex-grow": "0",
    "flex-shrink": "0",
    "flex-basis": "auto",
  });
  expect(computed(output.css, cx(styles[3]))).toEqual({
    "flex-grow": "1",
    "flex-shrink": "1",
    "flex-basis": "auto",
  });
});
const input = (id: number, declarations: string): ProjectInput => ({
  source: `import {alien} from '@alien-css/core';export const label='👽';export const styles=alien.create({same:{${declarations}}});`,
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
  const source = `import {alien} from '@alien-css/core';${Array.from({ length: 30 }, (_, i) => `export const s${i}=alien.create({same:{color:'red',backgroundColor:'white',fontWeight:600,width:'${i}px'}});`).join("")}export const empty=alien.create({});`;
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
    `import {alien,tokens} from '@alien-css/core';import {mix,percent} from '@alien-css/core/values';export const a=alien.create({${block(25)}});export const b=alien.create({${block(75)}});`,
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
