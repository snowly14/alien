import { expect, test } from "bun:test";
import { projectDeclarations } from "../src/configuration.ts";
import { loadCompiler } from "../src/native.ts";

const imports = `import { alien, css, custom, tokens } from "alien";
import { alpha, calc, hex, hsl, hsv, hwb, mix, ms, oklch, percent, px, rem, rgb } from "alien/values";`;
const free = loadCompiler();
const strict = loadCompiler(`import {alien} from "alien";
import {hex, rem} from "alien/values";
export default alien.globals({mode:"strict", color:{brand:hex("#123456")}, space:{control:rem(0.625)}, radius:{control:"0.5rem"},fontWeight:{label:650}});`);

function compile(style: string, compiler = free) {
  return compiler.transform(
    `${imports}\nexport const styles = alien.create({button:{${style}}});`,
    "button.ts",
    "tests/button.ts",
  )!;
}

test("free mode compiles strings, tokens, arithmetic and all public color operations", () => {
  for (const [source, expected] of [
    ['color:"tomato"', "color:tomato"],
    ['padding:"12px"', "padding:12px"],
    ['margin:"-5%"', "margin:-5%"],
    ["width:px(4 * 5 + 1)", "width:21px"],
    ["color:custom(rgb(255,0,0))", "color:rgb(255 0 0)"],
    ["color:hsl(120,percent(50),percent(25))", "color:hsl(120 50% 25%)"],
    ["color:hwb(0,percent(80),percent(80))", "color:hwb(0 80% 80%)"],
    ["color:hsv(120,percent(100),percent(100))", "color:rgb(0 255 0)"],
    ["color:oklch(0.5,0.2,20)", "color:oklch(0.5 0.2 20)"],
    ['color:custom(alpha(hex("#fff"),percent(50)))', "color:oklch(from #fff l c h/50%)"],
    [
      "color:mix(tokens.color.black,tokens.color.white,percent(25))",
      "color:color-mix(in oklch,oklch(0 0 0),oklch(1 0 0) 25%)",
    ],
    ["display:css.animationName.none", "display:none"],
  ])
    expect(compile(source!).css).toContain(expected!);
});

test("strict mode permits project tokens, built-ins, CSS keywords and dimensional layout", () => {
  const output = compile(
    'color:tokens.color.brand,padding:tokens.space.control,borderRadius:tokens.radius.control,fontWeight:tokens.fontWeight.label,width:"100%",display:"flex",margin:"auto"',
    strict,
  );
  expect(output.css).toContain(
    "color:#123456;padding:0.625rem;border-radius:0.5rem;font-weight:650;width:100%;display:flex;margin:auto",
  );
  expect(compile("color:alpha(tokens.color.brand,percent(40))", strict).css).toContain(
    "oklch(from #123456 l c h/40%)",
  );
});

test("strict policy also applies to nested and grouped declarations", () => {
  for (const style of [
    'color:"red"',
    "padding:px(4)",
    "padding:custom(px(4))",
    'borderRadius:"2px"',
    "fontWeight:650",
    'border:{color:custom(hex("#fff"))}',
    "on:{hover:{color:rgb(1,2,3)}}",
  ]) {
    expect(() => compile(style, strict)).toThrow();
  }
});

test("invalid source values and empty unknown conditions are rejected", () => {
  for (const style of [
    "color:rgb(300,0,0)",
    'color:hex("#xyz")',
    "color:mix(tokens.color.black,tokens.color.white,percent(101))",
    "width:px(1/0)",
    'padding:"-2px"',
    'color:"red;display:none"',
    "on:{breakpoint:{unknown:{}}}",
  ]) {
    expect(() => compile(style)).toThrow();
  }
});

test("default exports compile, but dynamic calls, namespace calls and aliases fail at build time", () => {
  expect(
    free.transform(`${imports}\nexport default alien.create({a:{display:"flex"}})`, "a.ts", "a.ts")
      ?.css,
  ).toContain("display:flex");
  for (const source of [
    `${imports}\nfunction styles(){return alien.create({a:{display:"flex"}})}`,
    `import * as A from "alien"; const s=A.alien.create({a:{display:"flex"}});`,
    `${imports}\nconst create=alien.create; const styles=create({a:{}});`,
    `${imports}\nconst api=alien; const styles=api.create({a:{}});`,
  ])
    expect(() => free.transform(source, "a.ts", "a.ts")).toThrow("top-level");
  expect(
    free.transform(
      `${imports}\nfunction local(alien: any) {return alien.create({a:{}})}`,
      "a.ts",
      "a.ts",
    ),
  ).toBeNull();
});

test("source edits use UTF-16 offsets and reconstruct code exactly", () => {
  const source = `${imports}\nconst title="Été 👽";\nconst a=alien.create({a:{width:px(4)}});\nconst b=alien.create({b:{height:px(8)}});`;
  const output = free.transform(source, "a.ts", "a.ts")!;
  let rewritten = source;
  for (const edit of output.edits.toReversed()) {
    expect(source.slice(edit.start, edit.end)).toStartWith("alien.create(");
    rewritten = rewritten.slice(0, edit.start) + edit.replacement + rewritten.slice(edit.end);
  }
  expect(rewritten).toBe(output.code);
});

test("configuration is static, checked eagerly, and cannot replace built-in names", () => {
  for (const source of [
    "globalThis.sideEffect=true;",
    'alien.globals({mode:"typo"})',
    'alien.globals({color:{brand:"not-a-color"}})',
    'alien.globals({space:{md:"1rem"}})',
    'alien.globals({color:{brand:"red",brand:"blue"}})',
    'alien.globals({mode:"free"});alien.globals({mode:"strict"})',
    'alien.globals({space:{constructor:"1rem"}})',
  ])
    expect(() => loadCompiler(`import {alien} from "alien";${source}`)).toThrow();
  expect(strict.mode).toBe("strict");
  expect(strict.tokenPaths).toContain("color.brand");
  expect(projectDeclarations(strict)).toContain('mode: "strict"');
  expect(projectDeclarations(free)).not.toContain('"brand"');
});

test("hybrid class maps remain usable across multiple calls with production namespaces", () => {
  const block = Array.from(
    { length: 30 },
    (_, i) =>
      `s${i}:{width:px(${i + 1}),backgroundColor:mix(tokens.color.green[600],tokens.color.blue[600],percent(25))}`,
  ).join(",");
  const output = free.transform(
    `${imports}\nexport const first=alien.create({${block}});\nexport const second=alien.create({${block}});`,
    "hybrid.ts",
    "tests/hybrid.ts",
  )!;
  expect(output.edits).toHaveLength(2);
  expect(output.css.match(/background-color:/g)).toHaveLength(1);
  const classes = output.edits.map((edit) => {
    // The replacement is a compiler-generated object literal, with no source code execution.
    const map = new Function(`return ${edit.replacement}`)() as Record<string, string>;
    expect(Object.keys(map)).toHaveLength(30);
    const names = new Set<string>();
    for (const value of Object.values(map)) {
      const tokens = value.split(" ");
      expect(tokens).toHaveLength(2);
      for (const token of tokens) {
        expect(output.css).toContain(`.${token}{`);
        names.add(token);
      }
    }
    return names;
  });
  expect([...classes[0]!].filter((name) => classes[1]!.has(name))).toHaveLength(1);
});
