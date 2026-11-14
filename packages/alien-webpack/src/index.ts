import { fileURLToPath } from "node:url";
import type { RuleSetRule } from "webpack";

export type AlienWebpackOptions = {
  /** Project root; defaults to webpack's context. */
  root?: string;
  globals?: string;
  namespace?: string;
};

/** Add before the framework's JS/TS loaders; keep its existing CSS handling. */
export function alienRule(options: AlienWebpackOptions = {}): RuleSetRule {
  return {
    test: /\.(?:[cm]?[jt]s|[jt]sx)$/,
    exclude: [/node_modules/, /\.d\.[cm]?ts$/],
    enforce: "pre",
    use: [{ loader: fileURLToPath(new URL("./loader.js", import.meta.url)), options }],
  };
}
