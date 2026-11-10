import { appearanceKeywords } from "./keywords/appearance";
import { layoutKeywords } from "./keywords/layout";
import { motionKeywords } from "./keywords/motion";
import { typographyKeywords } from "./keywords/typography";

export const css = {
  ...layoutKeywords,
  ...appearanceKeywords,
  ...typographyKeywords,
  ...motionKeywords,
} as const;

export type CssKeyword<Property extends keyof typeof css> =
  (typeof css)[Property][keyof (typeof css)[Property]];
