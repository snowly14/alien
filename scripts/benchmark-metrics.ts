export const metrics = ["minified", "gzip", "brotli"] as const;
export type Metric = (typeof metrics)[number];
export type Measurement = { scenario: string; tool: string } & Record<Metric, number>;

/** Positive means smaller than the competitor; zero is a tie. */
export function savingsPercent(bytes: number, competitorBytes: number): number {
  if (
    !Number.isFinite(bytes) ||
    !Number.isFinite(competitorBytes) ||
    bytes < 0 ||
    competitorBytes < 0
  )
    throw new Error("CSS sizes must be finite, non-negative numbers");
  if (competitorBytes === 0) return bytes === 0 ? 0 : Number.NEGATIVE_INFINITY;
  return ((competitorBytes - bytes) / competitorBytes) * 100;
}

export function formatSavings(value: number): string {
  if (value === 0) return "tie";
  if (!Number.isFinite(value)) return "larger (competitor: 0 bytes)";
  return `${Math.abs(value).toFixed(1)}% ${value > 0 ? "smaller" : "larger"}`;
}

export function parseBenchmarkOptions(args: string[]) {
  let check: Metric | "all" | undefined;
  let minimumSavings = 0;
  for (const arg of args) {
    if (arg === "--check") check = "all";
    else if (arg.startsWith("--check=")) {
      const value = arg.slice("--check=".length);
      if (value !== "all" && !metrics.includes(value as Metric))
        throw new Error("--check expects all, minified, gzip, or brotli");
      check = value as Metric | "all";
    } else if (arg.startsWith("--min-savings=")) {
      const value = arg.slice("--min-savings=".length);
      minimumSavings = Number(value);
      if (
        !value.trim() ||
        !Number.isFinite(minimumSavings) ||
        minimumSavings < 0 ||
        minimumSavings > 100
      )
        throw new Error("--min-savings expects a percentage between 0 and 100");
    } else throw new Error(`Unknown benchmark argument: ${arg}`);
  }
  if (minimumSavings > 0 && !check) throw new Error("--min-savings requires --check");
  return { check, minimumSavings };
}

export function checkAdvantages(
  rows: Measurement[],
  selected: readonly Metric[],
  minimumSavings: number,
) {
  if (rows.length === 0) throw new Error("Cannot check an empty benchmark");
  const failures: { scenario: string; metric: Metric; competitor: string; result: string }[] = [];
  for (const scenario of new Set(rows.map((row) => row.scenario))) {
    const results = rows.filter((row) => row.scenario === scenario);
    const alien = results.find((row) => row.tool === "Alien");
    const rivals = results.filter((row) => row.tool !== "Alien");
    if (!alien || rivals.length === 0) throw new Error(`Incomplete scenario: ${scenario}`);
    for (const metric of selected) {
      for (const rival of rivals) {
        const savings = savingsPercent(alien[metric], rival[metric]);
        // A tie is not an outright win, even when the requested margin is zero.
        if (savings <= 0 || savings < minimumSavings)
          failures.push({
            scenario,
            metric,
            competitor: rival.tool,
            result: formatSavings(savings),
          });
      }
    }
  }
  return failures;
}
