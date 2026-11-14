import type { AlienProject } from "./index";
import type { Color } from "./values/color";
import type { Length, LengthString, Percent, Time } from "./values/units";

export type ResetLevel = "light" | "medium" | "strong";
export type Mode = "free" | "strict";
export type ProjectMode = AlienProject extends { mode: infer M extends Mode } ? M : "free";
export type ProjectTokenNames<Family extends string> = AlienProject extends {
  tokens: infer Tokens;
}
  ? Family extends keyof Tokens
    ? Extract<Tokens[Family], string>
    : never
  : never;

/** Literal project values are compiled once; built-in token names cannot be replaced. */
export type Globals = Readonly<{
  mode?: Mode;
  reset?: false | ResetLevel;
  color?: Readonly<Record<string, Color | string>>;
  space?: Readonly<Record<string, Length | Percent | LengthString>>;
  radius?: Readonly<Record<string, Length | Percent | LengthString>>;
  fontWeight?: Readonly<Record<string, number>>;
  fontSize?: Readonly<Record<string, Length | Percent | LengthString>>;
  lineHeight?: Readonly<Record<string, number | Length | Percent | LengthString>>;
  duration?: Readonly<Record<string, Time>>;
}>;

export function globals(_config: Globals): Globals {
  throw new Error(
    "alien.globals() is compile-time configuration. Do not import alien.globals.ts at runtime.",
  );
}
