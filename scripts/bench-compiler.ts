import { arch, cpus, loadavg, platform } from "node:os";
import { loadCompiler } from "../packages/alien-vite/src/native";

const compiler = loadCompiler();
const button = `import {alien,tokens} from "alien";import {px} from "alien/values";
export const styles=alien.create({button:{display:"flex",padding:tokens.space.md,color:tokens.color.green[600],width:px(4*20),on:{hover:{color:tokens.color.green[700]},breakpoint:{md:{padding:tokens.space.lg}}}}});`;
function styles(count: number, shared: boolean) {
  return `import {alien,tokens} from "alien";import {px} from "alien/values";
export const styles=alien.create({${Array.from(
    { length: count },
    (_, i) =>
      `s${i}:{width:px(${i + 1}),${shared ? 'display:"flex",position:"relative",boxSizing:"border-box",alignItems:"center",justifyContent:"space-between",flexDirection:"column",overflow:"hidden",fontStyle:"normal",textAlign:"center",textTransform:"uppercase",whiteSpace:"nowrap",userSelect:"none",pointerEvents:"auto",backgroundColor:tokens.color.green[600],borderRadius:tokens.radius.md,boxShadow:tokens.shadow.md' : `height:px(${i + 2})`}}`,
  ).join(",")}});`;
}

const scenarios = [
  { name: "single-style", source: button, iterations: 5_000 },
  { name: "shared-100", source: styles(100, true), iterations: 50 },
  { name: "shared-1000", source: styles(1_000, true), iterations: 5 },
  { name: "unique-1000", source: styles(1_000, false), iterations: 50 },
];
const results = scenarios.map(({ name, source, iterations }) => {
  for (let i = 0; i < 10; i++) compiler.transform(source, "bench.ts", "bench/styles.ts");
  const samples: number[] = [];
  for (let round = 0; round < 5; round++) {
    const start = performance.now();
    for (let i = 0; i < iterations; i++) compiler.transform(source, "bench.ts", "bench/styles.ts");
    samples.push((performance.now() - start) / iterations);
  }
  samples.sort((a, b) => a - b);
  const output = compiler.transform(source, "bench.ts", "bench/styles.ts");
  if (!output) throw new Error(`Benchmark ${name} did not produce compiled styles`);
  return {
    name,
    iterationsPerRound: iterations,
    medianMillisecondsPerModule: samples[2],
    minMillisecondsPerModule: samples[0],
    maxMillisecondsPerModule: samples.at(-1),
    cssBytes: Buffer.byteLength(output.css),
    classMapBytes: output.edits.reduce(
      (size, edit) => size + Buffer.byteLength(edit.replacement),
      0,
    ),
  };
});
console.log(
  JSON.stringify(
    {
      scope: "native transform, including JS/native boundary; no Vite cache",
      environment: {
        platform: platform(),
        arch: arch(),
        cpu: cpus()[0]?.model,
        bun: Bun.version,
        node: process.versions.node,
        loadAverage: loadavg(),
      },
      timingNote:
        "Wall-clock measurements include host contention; compare runs on an otherwise idle machine.",
      rounds: 5,
      results,
    },
    null,
    2,
  ),
);
