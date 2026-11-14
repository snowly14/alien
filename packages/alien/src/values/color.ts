import type { Percent } from "./units";

type HexColor = Readonly<{ kind: "hex"; value: `#${string}` }>;
type OklchColor = Readonly<{
  kind: "oklch";
  lightness: number;
  chroma: number;
  hue: number;
}>;

type RgbColor = Readonly<{
  kind: "rgb";
  red: number;
  green: number;
  blue: number;
}>;

type HslColor = Readonly<{
  kind: "hsl";
  hue: number;
  saturation: Percent;
  lightness: Percent;
}>;

type HwbColor = Readonly<{
  kind: "hwb";
  hue: number;
  whiteness: Percent;
  blackness: Percent;
}>;

type HsvColor = Readonly<{
  kind: "hsv";
  hue: number;
  saturation: Percent;
  value: Percent;
}>;

export type ColorToken = Readonly<{
  kind: "color-token";
  path: string;
}>;

interface AlphaColor<Value> {
  readonly kind: "alpha";
  readonly color: Value;
  readonly opacity: Percent;
}

interface MixedColor<First, Second> {
  readonly kind: "mix";
  readonly first: First;
  readonly second: Second;
  /** Percentage of the second color. */
  readonly amount: Percent;
}

export type Color =
  | HexColor
  | OklchColor
  | RgbColor
  | HslColor
  | HwbColor
  | HsvColor
  | ColorToken
  | AlphaColor<Color>
  | MixedColor<Color, Color>;

export type TokenColor = ColorToken | AlphaColor<TokenColor> | MixedColor<TokenColor, TokenColor>;

function isInRange(value: number, min: number, max: number): boolean {
  return Number.isFinite(value) && value >= min && value <= max;
}

function validateHue(hue: number, name: string): void {
  if (!Number.isFinite(hue)) {
    throw new RangeError(`${name}() hue must be a finite number`);
  }
}

function validatePercentage(value: Percent, name: string): void {
  if (!isInRange(value.value, 0, 100)) {
    throw new RangeError(`${name} must be between 0% and 100%`);
  }
}

export function hex(value: `#${string}`): HexColor {
  if (!/^#(?:[0-9a-f]{3,4}|[0-9a-f]{6}|[0-9a-f]{8})$/i.test(value)) {
    throw new RangeError("hex() requires a CSS hex color");
  }

  return { kind: "hex", value };
}

export function rgb(red: number, green: number, blue: number): RgbColor {
  if (!isInRange(red, 0, 255) || !isInRange(green, 0, 255) || !isInRange(blue, 0, 255)) {
    throw new RangeError("rgb() components must be finite numbers between 0 and 255");
  }

  return { kind: "rgb", red, green, blue };
}

export function oklch(lightness: number, chroma: number, hue: number): OklchColor {
  if (!isInRange(lightness, 0, 1)) {
    throw new RangeError("oklch() lightness must be between 0 and 1");
  }
  if (!Number.isFinite(chroma) || chroma < 0 || !Number.isFinite(hue)) {
    throw new RangeError("oklch() requires non-negative chroma and a finite hue");
  }

  return { kind: "oklch", lightness, chroma, hue };
}

export function hsl(hue: number, saturation: Percent, lightness: Percent): HslColor {
  validateHue(hue, "hsl");
  validatePercentage(saturation, "hsl() saturation");
  validatePercentage(lightness, "hsl() lightness");
  return { kind: "hsl", hue, saturation, lightness };
}

export function hwb(hue: number, whiteness: Percent, blackness: Percent): HwbColor {
  validateHue(hue, "hwb");
  validatePercentage(whiteness, "hwb() whiteness");
  validatePercentage(blackness, "hwb() blackness");
  return { kind: "hwb", hue, whiteness, blackness };
}

export function hsv(hue: number, saturation: Percent, value: Percent): HsvColor {
  validateHue(hue, "hsv");
  validatePercentage(saturation, "hsv() saturation");
  validatePercentage(value, "hsv() value");
  return { kind: "hsv", hue, saturation, value };
}

export function mix<const First extends Color, const Second extends Color>(
  first: First,
  second: Second,
  amount: Percent,
): MixedColor<First, Second> {
  validatePercentage(amount, "mix() amount");
  return { kind: "mix", first, second, amount };
}

export function alpha<const Value extends Color>(
  color: Value,
  opacity: Percent,
): AlphaColor<Value> {
  validatePercentage(opacity, "alpha() opacity");
  return { kind: "alpha", color, opacity };
}
