import { expect, test } from "bun:test";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const root = fileURLToPath(new URL("../../../", import.meta.url));
function diagnostics(source: string, registry = "") {
  const path = resolve(root, "packages/alien/tests/__type_fixture.ts");
  const declaration = resolve(root, "packages/alien/tests/__project_fixture.d.ts");
  const options: ts.CompilerOptions = {
    strict: true,
    exactOptionalPropertyTypes: true,
    noEmit: true,
    skipLibCheck: true,
    target: ts.ScriptTarget.ESNext,
    module: ts.ModuleKind.ESNext,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
    types: [],
    paths: {
      alien: [resolve(root, "packages/alien/src/index.ts")],
      "alien/values": [resolve(root, "packages/alien/src/values/index.ts")],
    },
  };
  const host = ts.createCompilerHost(options);
  const getSourceFile = host.getSourceFile.bind(host);
  host.getSourceFile = (file, languageVersion, ...args) => {
    if (file === path || file === declaration)
      return ts.createSourceFile(file, file === path ? source : registry, languageVersion, true);
    return getSourceFile(file, languageVersion, ...args);
  };
  return ts
    .getPreEmitDiagnostics(ts.createProgram([path, declaration], options, host))
    .map((diagnostic) => ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"));
}

test("free styles and strict styles have distinct, checked contracts", () => {
  expect(
    diagnostics(`
    import {alien, tokens, custom, type Style} from "alien";
    import {hex, px, rgb} from "alien/values";
    const free: Style<"free"> = {color:"red",padding:"12px",fontWeight:650};
    const strict: Style<"strict"> = {color:tokens.color.black,padding:tokens.space.md,width:"100%",display:"flex"};
    // @ts-expect-error strict components use color tokens
    const badColor: Style<"strict"> = {color:"red"};
    // @ts-expect-error strict components cannot bypass the policy with custom
    const bypass: Style<"strict"> = {color:custom(hex("#fff"))};
    // @ts-expect-error strict spacing uses tokens
    const badSpacing: Style<"strict"> = {padding:px(4)};
    // @ts-expect-error properties still have their own types in free mode
    const badUnit: Style<"free"> = {padding:rgb(1,2,3)};
    // @ts-expect-error unknown display keyword
    const badKeyword: Style<"free"> = {display:"flxe"};
    // @ts-expect-error nested breakpoints are unsupported
    const nested: Style = {on:{breakpoint:{md:{on:{breakpoint:{lg:{}}}}}}};
  `),
  ).toEqual([]);
});

test("project augmentation exposes only declared names and applies strict mode to create", () => {
  expect(
    diagnostics(
      `
    import {alien,tokens} from "alien";
    alien.create({button:{color:tokens.color.brand,padding:tokens.space.control}});
    // @ts-expect-error stale token names must fail
    alien.create({button:{color:tokens.color.removed}});
    // @ts-expect-error project mode applies to the default API
    alien.create({button:{padding:"12px"}});
  `,
      `import "alien"; declare module "alien" {interface AlienProject {mode:"strict";tokens:{color:"brand";space:"control"}}}`,
    ),
  ).toEqual([]);
});
