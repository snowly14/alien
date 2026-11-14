import { mkdir, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { brotliCompressSync, constants, gzipSync } from "node:zlib";
import { transformSync } from "@babel/core";
import stylex, { type Rule } from "@stylexjs/babel-plugin";
import { compile } from "@tailwindcss/node";
import { transform } from "lightningcss";
import { loadCompiler, type ProjectInput } from "../packages/alien-compiler/src/native.ts";

type Declaration = [property: string, value: string, candidate: string, condition?: "focus" | "md"];
const button: Declaration[] = [
  ["display", "inline-flex", "inline-flex"],
  ["alignItems", "center", "items-center"],
  ["justifyContent", "center", "justify-center"],
  ["paddingInline", "1rem", "px-4"],
  ["height", "2rem", "h-8"],
  ["borderRadius", "0.5rem", "rounded-lg"],
  ["fontWeight", "600", "font-semibold"],
  ["backgroundColor", "#fff", "bg-white"],
  ["color", "#000", "text-black"],
  ["cursor", "pointer", "cursor-pointer"],
];
const card: Declaration[] = [
  ["display", "grid", "grid"],
  ["gap", "1rem", "gap-4"],
  ["padding", "1rem", "p-4"],
  ["borderRadius", "0.5rem", "rounded-lg"],
  ["backgroundColor", "#fff", "bg-white"],
  ["color", "#000", "text-black"],
];
const width = (px: number): Declaration => ["width", `${px}px`, `w-[${px}px]`];
const mixed = Array.from({ length: 100 }, (_, i) => [
  ...(i % 2 === 0 ? card : button),
  width(200 + i),
]);
const scenarios: { name: string; styles: Declaration[][]; stylesPerModule?: number }[] = [
  { name: "One button", styles: [button] },
  {
    name: "Button with focus and breakpoint",
    styles: [
      [
        ...button,
        ["color", "#fff", "focus:text-white", "focus"],
        ["backgroundColor", "#000", "focus:bg-black", "focus"],
        ["paddingInline", "1.5rem", "md:px-6", "md"],
      ],
    ],
  },
  {
    name: "100 alternating color variants",
    styles: Array.from({ length: 100 }, (_, i) => [
      ["color", i % 2 === 0 ? "red" : "blue", i % 2 === 0 ? "text-[red]" : "text-[blue]"],
    ]),
  },
  { name: "100 cards and buttons, one module", styles: mixed },
  { name: "100 cards and buttons, 20 modules", styles: mixed, stylesPerModule: 5 },
  {
    name: "100 distinct width/height pairs",
    styles: Array.from({ length: 100 }, (_, i) => [
      width(200 + i),
      ["height", `${100 + i}px`, `h-[${100 + i}px]`],
    ]),
  },
];

const outputDirectory = new URL("../docs/benchmarks/", import.meta.url);
await mkdir(outputDirectory, { recursive: true });
const compiler = loadCompiler();
const require = createRequire(import.meta.url);
const versions = {
  tailwind: require("tailwindcss/package.json").version,
  stylex: require("@stylexjs/babel-plugin/package.json").version,
};
type Result = { scenario: string; tool: string; minified: number; gzip: number; brotli: number };
const rows: Result[] = [];
const artifactName = (name: string, tool: string) =>
  `${name.toLowerCase().replace(/[^a-z0-9]+/g, "-")}-${tool.toLowerCase().replace(/[^a-z0-9]+/g, "-")}.css`;
for (const { name, styles, stylesPerModule = styles.length } of scenarios) {
  const projectInputs: ProjectInput[] = [];
  const stylexRules: Rule[] = [];
  for (let start = 0; start < styles.length; start += stylesPerModule) {
    const input = styles.slice(start, start + stylesPerModule);
    const alienStyles = Object.fromEntries(
      input.map((declarations, i) => {
        const values = (condition?: "focus" | "md") =>
          Object.fromEntries(
            declarations
              .filter(([, , , state]) => state === condition)
              .map(([property, value]) => [
                property,
                property === "fontWeight" ? Number(value) : value,
              ]),
          );
        const focus = values("focus");
        const md = values("md");
        const on = {
          ...(Object.keys(focus).length && { focus }),
          ...(Object.keys(md).length && { breakpoint: { md } }),
        };
        return [`s${i}`, { ...values(), ...(Object.keys(on).length && { on }) }];
      }),
    );
    const stylexStyles = Object.fromEntries(
      input.map((declarations, i) => {
        const properties = new Map<string, Record<string, string | number>>();
        for (const [property, value, , condition] of declarations) {
          const values = properties.get(property) ?? {};
          const selector =
            condition === "md"
              ? "@media (min-width: 48rem)"
              : condition === "focus"
                ? ":focus"
                : "default";
          values[selector] = property === "fontWeight" ? Number(value) : value;
          properties.set(property, values);
        }
        return [
          `s${i}`,
          Object.fromEntries(
            [...properties].map(([property, values]) => [
              property,
              Object.keys(values).length === 1 && values.default !== undefined
                ? values.default
                : values,
            ]),
          ),
        ];
      }),
    );
    const filename = `benchmark-${start}`;
    const projectInput = {
      source: `import {alien} from "alien"; export const styles = alien.create(${JSON.stringify(alienStyles)});`,
      path: `${filename}.ts`,
      moduleId: `./${filename}.ts`,
    };
    projectInputs.push(projectInput);
    const result = transformSync(
      `import * as stylex from "@stylexjs/stylex";export const styles = stylex.create(${JSON.stringify(stylexStyles)});`,
      {
        filename: `${filename}.js`,
        babelrc: false,
        configFile: false,
        plugins: [[stylex, { dev: false, runtimeInjection: false }]],
      },
    );
    const metadata = result?.metadata;
    const rules = metadata && "stylex" in metadata ? (metadata.stylex as Rule[]) : undefined;
    if (!rules?.length) throw new Error("StyleX emitted no CSS rules");
    stylexRules.push(...rules);
  }
  const tailwind = await compile(
    '@layer theme, utilities; @import "tailwindcss/theme.css" layer(theme); @import "tailwindcss/utilities.css" layer(utilities);',
    { base: process.cwd(), onDependency() {} },
  );
  const candidates = [
    ...new Set(styles.flatMap((declarations) => declarations.map(([, , candidate]) => candidate))),
  ];
  const tailwindCss = tailwind.build(candidates);
  for (const candidate of candidates) {
    const selector = candidate.replace(/[^\w-]/g, "\\$&");
    if (!tailwindCss.includes(`.${selector}`))
      throw new Error(`Tailwind did not emit ${candidate}`);
  }
  for (const [tool, css] of [
    ["Alien", compiler.transformProject(projectInputs).css],
    ["Tailwind", tailwindCss],
    ["StyleX", stylex.processStylexRules(stylexRules)],
  ] as const) {
    const { code } = transform({ filename: "benchmark.css", code: Buffer.from(css), minify: true });
    await writeFile(new URL(artifactName(name, tool), outputDirectory), code);
    rows.push({
      scenario: name,
      tool,
      minified: code.length,
      gzip: gzipSync(code, { level: 9 }).length,
      brotli: brotliCompressSync(code, { params: { [constants.BROTLI_PARAM_QUALITY]: 11 } }).length,
    });
  }
}
const metrics = ["minified", "gzip", "brotli"] as const;
const percentageResults: {
  scenario: string;
  competitor: string;
  metric: string;
  savingsPercent: number;
}[] = [];
const number = (value: number) => value.toLocaleString("en-US");
const introduction = `Matching style declarations compiled with Alien, Tailwind ${versions.tailwind}, and StyleX ${versions.stylex}. Lower is better; all sizes are in bytes. Bold values mark the smallest output in each scenario.

Tailwind uses its standard theme without Preflight. All outputs use Lightning CSS ${require("lightningcss/package.json").version}, gzip level 9, and Brotli quality 11. Each scenario produces one stylesheet, including the 20-module scenario. JavaScript, HTML class names, resets, build time, and runtime performance are excluded. The alternating-color fixture tests conflicting values; Alien preserves source-order composition, while utility systems have their own composition rules. These fixtures do not establish a universal winner or framework compatibility. Alien is measured with complete-project compilation (alien generate) consistently in all six scenarios. There is one Alien result per scenario, never a best-of selection between compilation modes. These results do not measure the incremental Vite/webpack path. Percentage savings are (competitor bytes − Alien bytes) / competitor bytes × 100; negative savings mean Alien is larger.`;
let markdown = `# CSS size comparison

Run \`bun run bench:css\` to regenerate this report, the [HTML report](benchmarks/index.html), [JSON results](benchmarks/results.json), and the measured CSS files.

${introduction}
`;
let sections = "";
for (const { name } of scenarios) {
  const results = rows.filter((row) => row.scenario === name);
  const minimum = Object.fromEntries(
    metrics.map((metric) => [metric, Math.min(...results.map((row) => row[metric]))]),
  );
  markdown += `
## ${name}

| Tool | Minified CSS | gzip | Brotli |
| --- | ---: | ---: | ---: |
`;
  let table = "";
  for (const row of results) {
    markdown += `| ${row.tool} | ${metrics.map((metric) => (row[metric] === minimum[metric] ? `**${number(row[metric])}**` : number(row[metric]))).join(" | ")} |
`;
    table += `<tr><th scope="row"><a href="${artifactName(name, row.tool)}">${row.tool}</a></th>${metrics.map((metric) => `<td${row[metric] === minimum[metric] ? ' class="best"' : ""}>${number(row[metric])}${row[metric] === minimum[metric] ? "<span>Smallest</span>" : ""}</td>`).join("")}</tr>`;
  }
  const comparisons: string[] = [];
  for (const alien of results.filter((row) => row.tool === "Alien")) {
    for (const rival of results.filter((row) => row.tool === "Tailwind" || row.tool === "StyleX")) {
      const parts = metrics.map((metric) => {
        const difference = ((rival[metric] - alien[metric]) / rival[metric]) * 100;
        percentageResults.push({
          scenario: name,
          competitor: rival.tool,
          metric,
          savingsPercent: difference,
        });
        return `${metric}: ${Math.abs(difference).toFixed(1)}% ${difference > 0 ? "smaller" : difference < 0 ? "larger" : "equal"}`;
      });
      comparisons.push(`${alien.tool} vs ${rival.tool} — ${parts.join(", ")}`);
    }
  }
  markdown += `\n${comparisons.map((line) => `- ${line}`).join("\n")}\n`;
  sections += `<section><h2>${name}</h2><div class="table"><table><thead><tr><th scope="col">Tool / CSS file</th><th scope="col">Minified CSS</th><th scope="col">gzip</th><th scope="col">Brotli</th></tr></thead><tbody>${table}</tbody></table></div><p>Relative to each competitor: ${comparisons.join("<br>")}</p></section>`;
  console.log(`\n${name} — bytes, lower is better`);
  console.table(results.map(({ scenario: _, ...result }) => result));
}
markdown += `
## Reading the results

Percentages use (competitor bytes − Alien bytes) / competitor bytes; “larger” means Alien loses. Compare Brotli or gzip for CSS transfer size, and minified CSS for the uncompressed stylesheet. A smaller stylesheet does not necessarily mean a smaller application: extra class names also consume HTML and JavaScript bytes. Alien selects sharing plans using CSS plus its generated class map, rather than CSS alone.

Alien preserves declaration order. Independent transforms retain module-specific class names; complete-project generation uses a shared namespace and optimizes safe declarations across modules. It shares safe repeated declarations individually or in groups when that reduces its cost model. Tailwind and StyleX reuse utilities across modules; the split-module fixture deliberately measures that difference. No tool is guaranteed to win for every workload.
`;
await writeFile(new URL("../css-benchmark.md", outputDirectory), markdown);
await writeFile(
  new URL("results.json", outputDirectory),
  `${JSON.stringify({ versions: { ...versions, lightningcss: require("lightningcss/package.json").version }, compression: { gzip: 9, brotli: 11 }, alienCompilation: "whole-project", percentageResults, rows }, null, 2)}\n`,
);
await writeFile(
  new URL("index.html", outputDirectory),
  `<!doctype html>
<html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>Alien · CSS size comparison</title>
<style>
:root{font-family:system-ui,sans-serif;color:#18243a;background:#f3f6fa;color-scheme:light}body{max-width:960px;margin:0 auto;padding:48px 20px}h1{font-size:clamp(2rem,5vw,3.2rem);letter-spacing:-.04em;margin:12px 0}p{line-height:1.7;max-width:85ch;color:#475569}.eyebrow{font-weight:700;color:#35644d;letter-spacing:.12em;text-transform:uppercase;font-size:.8rem}section{background:white;border:1px solid #dce3ed;border-radius:16px;padding:24px;margin:24px 0}h2{font-size:1.2rem;margin:0 0 20px}.table{overflow:auto}table{width:100%;border-collapse:collapse;font-variant-numeric:tabular-nums;white-space:nowrap}th,td{padding:16px 12px;text-align:right;border-bottom:1px solid #e7ecf3}th:first-child{text-align:left}thead{font-size:.8rem;color:#58657a}td.best{font-weight:750;color:#14613e;background:#edf8f0}td span{display:block;font-size:.65rem;font-weight:500;margin-top:4px}a{color:#2459a6}code{background:#e8edf5;padding:3px 6px;border-radius:4px}footer{font-size:.9rem;color:#475569;line-height:1.7}
</style><header><div class="eyebrow">Reproducible benchmark</div><h1>How much CSS gets shipped?</h1><p>${introduction.replaceAll("\n\n", "</p><p>")}</p><p><code>bun run bench:css</code> · <a href="results.json">Download JSON</a> · Click a tool name to inspect its CSS.</p></header>${sections}<footer>Compare gzip or Brotli for transfer size. Alien evaluates CSS plus generated class-map size; this report measures CSS only. Module boundaries, composition, code splitting, and real application content can change the ranking. Framework compatibility and universal superiority are not established by these measurements.</footer></html>`,
);
console.log("\nReports: docs/css-benchmark.md and docs/benchmarks/index.html");
