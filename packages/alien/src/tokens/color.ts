import type { ColorToken } from "../values/color";

const shadeSteps = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950] as const;
const paletteNames = [
  "red",
  "orange",
  "amber",
  "yellow",
  "lime",
  "green",
  "emerald",
  "teal",
  "cyan",
  "sky",
  "blue",
  "indigo",
  "violet",
  "purple",
  "fuchsia",
  "pink",
  "rose",
  "slate",
  "gray",
  "zinc",
  "neutral",
  "stone",
  "mauve",
  "olive",
  "mist",
  "taupe",
] as const;

type ShadeStep = (typeof shadeSteps)[number];
type PaletteName = (typeof paletteNames)[number];
type Palette = Readonly<Record<ShadeStep, ColorToken>>;
type StandaloneColors = Readonly<Record<"black" | "white", ColorToken>>;
type BuiltinColors = StandaloneColors & Readonly<Record<PaletteName, Palette>>;

function colorToken(path: string): ColorToken {
  return { kind: "color-token", path };
}

function colorPalette(name: PaletteName): Palette {
  const prefix = `color.${name}`;
  const shades: Partial<Record<ShadeStep, ColorToken>> = {};

  for (const step of shadeSteps) {
    shades[step] = colorToken(`${prefix}.${step}`);
  }

  return shades as Palette;
}

function builtinColors(): BuiltinColors {
  const colors: StandaloneColors & Partial<Record<PaletteName, Palette>> = {
    black: colorToken("color.black"),
    white: colorToken("color.white"),
  };

  for (const name of paletteNames) {
    colors[name] = colorPalette(name);
  }

  return colors as BuiltinColors;
}

export const color = /* @__PURE__ */ builtinColors();
