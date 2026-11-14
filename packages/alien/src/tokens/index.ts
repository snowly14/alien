import { breakpoint } from "./breakpoint";
import { color } from "./color";
import { duration } from "./duration";
import { ease } from "./ease";
import { fontSize } from "./font-size";
import { fontWeight } from "./font-weight";
import { lineHeight } from "./line-height";
import { radius } from "./radius";
import { shadow } from "./shadow";
import { space } from "./space";

// These references use the names defined in the Rust crate alien-tokens.
export const tokens: Readonly<{
  breakpoint: typeof breakpoint;
  color: typeof color;
  duration: typeof duration;
  ease: typeof ease;
  fontSize: typeof fontSize;
  fontWeight: typeof fontWeight;
  lineHeight: typeof lineHeight;
  radius: typeof radius;
  shadow: typeof shadow;
  space: typeof space;
}> = {
  breakpoint,
  color,
  duration,
  ease,
  fontSize,
  fontWeight,
  lineHeight,
  radius,
  shadow,
  space,
} as const;
