import { expect, test } from "bun:test";
import {
  checkAdvantages,
  formatSavings,
  parseBenchmarkOptions,
  savingsPercent,
} from "./benchmark-metrics.ts";

test("percentages use the competitor as denominator and distinguish losses and ties", () => {
  expect(savingsPercent(60, 100)).toBe(40);
  expect(savingsPercent(140, 100)).toBe(-40);
  expect(formatSavings(savingsPercent(140, 100))).toBe("40.0% larger");
  expect(formatSavings(savingsPercent(0, 0))).toBe("tie");
  expect(formatSavings(savingsPercent(1, 0))).toBe("larger (competitor: 0 bytes)");
  expect(() => savingsPercent(-1, 10)).toThrow();
});

test("strict victory checks catch ties, insufficient margins and metric-specific losses", () => {
  const rows = [
    { scenario: "test", tool: "Alien", minified: 100, gzip: 60, brotli: 40 },
    { scenario: "test", tool: "StyleX", minified: 100, gzip: 100, brotli: 50 },
  ];
  expect(checkAdvantages(rows, ["minified"], 0)).toHaveLength(1);
  expect(checkAdvantages(rows, ["gzip"], 40)).toHaveLength(0);
  expect(checkAdvantages(rows, ["brotli"], 21)).toHaveLength(1);
  expect(() => checkAdvantages([], ["gzip"], 0)).toThrow();
  expect(() => checkAdvantages(rows.slice(0, 1), ["gzip"], 0)).toThrow();
});

test("benchmark options reject invalid metrics and ambiguous thresholds", () => {
  expect(parseBenchmarkOptions(["--check=brotli", "--min-savings=20"])).toEqual({
    check: "brotli",
    minimumSavings: 20,
  });
  expect(parseBenchmarkOptions(["--check"]).check).toBe("all");
  for (const args of [
    ["--check=fast"],
    ["--min-savings=20"],
    ["--check", "--min-savings="],
    ["--check", "--min-savings=NaN"],
    ["--typo"],
  ])
    expect(() => parseBenchmarkOptions(args)).toThrow();
});
