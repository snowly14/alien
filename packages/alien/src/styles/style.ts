import type { breakpoint } from "../tokens/breakpoint";
import type { TokenReference } from "../tokens/reference";
import type { Color, TokenColor } from "../values/color";
import type { Custom } from "../values/custom";
import type { Important } from "../values/important";
import type { Fraction, Length, Percent, Time } from "../values/units";
import type { CssKeyword } from "./css";
import type { SimplePseudoClass } from "./pseudo-classes";

type StyleColor = TokenColor | Custom<Color> | CssKeyword<"color">;
type LengthOrPercent = Length | Percent;
type Space = TokenReference<"space"> | Custom<LengthOrPercent>;
type Margin = Space | CssKeyword<"margin">;
type Radius = TokenReference<"radius"> | Custom<LengthOrPercent>;
type Inset = LengthOrPercent | CssKeyword<"inset">;
type GridTrack = LengthOrPercent | Fraction | CssKeyword<"gridTrack">;
type GridTrackList = readonly [GridTrack, ...GridTrack[]];
type FontFamily = CssKeyword<"fontFamily"> | Custom<string>;
type TimingFunction = TokenReference<"ease"> | CssKeyword<"timingFunction">;

type ImportantDeclarations<Declarations> = {
  readonly [Property in keyof Declarations]:
    | Required<Declarations>[Property]
    | Important<Required<Declarations>[Property]>;
};

type Border = ImportantDeclarations<{
  width?: Length | CssKeyword<"borderWidth">;
  style?: CssKeyword<"borderStyle">;
  color?: StyleColor;
}>;

type Layout = {
  display?: CssKeyword<"display">;
  position?: CssKeyword<"position">;
  boxSizing?: CssKeyword<"boxSizing">;
  width?: LengthOrPercent | CssKeyword<"width">;
  height?: LengthOrPercent | CssKeyword<"height">;
  minWidth?: LengthOrPercent | CssKeyword<"minWidth">;
  minHeight?: LengthOrPercent | CssKeyword<"minHeight">;
  maxWidth?: LengthOrPercent | CssKeyword<"maxWidth">;
  maxHeight?: LengthOrPercent | CssKeyword<"maxHeight">;
  aspectRatio?: number | CssKeyword<"aspectRatio">;
  inset?: Inset;
  top?: Inset;
  right?: Inset;
  bottom?: Inset;
  left?: Inset;
  zIndex?: number | CssKeyword<"zIndex">;
  overflow?: CssKeyword<"overflow">;
  overflowX?: CssKeyword<"overflowX">;
  overflowY?: CssKeyword<"overflowY">;

  padding?: Space;
  paddingTop?: Space;
  paddingRight?: Space;
  paddingBottom?: Space;
  paddingLeft?: Space;
  paddingBlock?: Space;
  paddingInline?: Space;
  margin?: Margin;
  marginTop?: Margin;
  marginRight?: Margin;
  marginBottom?: Margin;
  marginLeft?: Margin;
  marginBlock?: Margin;
  marginInline?: Margin;
  gap?: Space | CssKeyword<"gap">;
  rowGap?: Space | CssKeyword<"rowGap">;
  columnGap?: Space | CssKeyword<"columnGap">;

  flex?: number | CssKeyword<"flex">;
  flexDirection?: CssKeyword<"flexDirection">;
  flexWrap?: CssKeyword<"flexWrap">;
  flexGrow?: number;
  flexShrink?: number;
  flexBasis?: LengthOrPercent | CssKeyword<"flexBasis">;
  order?: number;
  alignItems?: CssKeyword<"alignItems">;
  alignSelf?: CssKeyword<"alignSelf">;
  alignContent?: CssKeyword<"alignContent">;
  justifyContent?: CssKeyword<"justifyContent">;
  justifyItems?: CssKeyword<"justifyItems">;
  justifySelf?: CssKeyword<"justifySelf">;

  gridTemplateColumns?: GridTrackList | CssKeyword<"gridTemplateColumns">;
  gridTemplateRows?: GridTrackList | CssKeyword<"gridTemplateRows">;
  gridAutoColumns?: GridTrackList;
  gridAutoRows?: GridTrackList;
  gridAutoFlow?: CssKeyword<"gridAutoFlow">;
  gridColumn?: number | CssKeyword<"gridColumn">;
  gridRow?: number | CssKeyword<"gridRow">;
};

type Typography = {
  fontFamily?: FontFamily | readonly [FontFamily, ...FontFamily[]];
  fontStyle?: CssKeyword<"fontStyle">;
  fontWeight?: CssKeyword<"fontWeight"> | TokenReference<"fontWeight"> | Custom<number>;
  fontSize?: LengthOrPercent | TokenReference<"fontSize">;
  lineHeight?: number | LengthOrPercent | CssKeyword<"lineHeight">;
  letterSpacing?: Length | CssKeyword<"letterSpacing">;
  textAlign?: CssKeyword<"textAlign">;
  textDecorationLine?: CssKeyword<"textDecorationLine">;
  textDecorationStyle?: CssKeyword<"textDecorationStyle">;
  textDecorationColor?: StyleColor;
  textOverflow?: CssKeyword<"textOverflow">;
  textTransform?: CssKeyword<"textTransform">;
  whiteSpace?: CssKeyword<"whiteSpace">;
  overflowWrap?: CssKeyword<"overflowWrap">;
  wordBreak?: CssKeyword<"wordBreak">;
};

type Appearance = {
  color?: StyleColor;
  backgroundColor?: StyleColor;
  backgroundRepeat?: CssKeyword<"backgroundRepeat">;
  backgroundSize?: LengthOrPercent | CssKeyword<"backgroundSize">;
  backgroundPosition?: LengthOrPercent | CssKeyword<"backgroundPosition">;
  opacity?: number | Percent;
  objectFit?: CssKeyword<"objectFit">;
  objectPosition?: LengthOrPercent | CssKeyword<"objectPosition">;
  borderRadius?: Radius;
  borderTopLeftRadius?: Radius;
  borderTopRightRadius?: Radius;
  borderBottomLeftRadius?: Radius;
  borderBottomRightRadius?: Radius;
  outlineColor?: StyleColor;
  outlineStyle?: CssKeyword<"outlineStyle">;
  outlineWidth?: Length | CssKeyword<"outlineWidth">;
  outlineOffset?: Length;
  border?: Border;
  borderTop?: Border;
  borderRight?: Border;
  borderBottom?: Border;
  borderLeft?: Border;
  visibility?: CssKeyword<"visibility">;
  cursor?: CssKeyword<"cursor">;
  pointerEvents?: CssKeyword<"pointerEvents">;
  userSelect?: CssKeyword<"userSelect">;
  appearance?: CssKeyword<"appearance">;
};

type Transition = ImportantDeclarations<{
  property?: CssKeyword<"transitionProperty">;
  duration?: Time;
  delay?: Time;
  ease?: TimingFunction;
}>;

type Animation = ImportantDeclarations<{
  name?: CssKeyword<"animationName"> | Custom<string>;
  duration?: Time;
  delay?: Time;
  ease?: TimingFunction;
  iterationCount?: number | CssKeyword<"animationIterationCount">;
  direction?: CssKeyword<"animationDirection">;
  fillMode?: CssKeyword<"animationFillMode">;
  playState?: CssKeyword<"animationPlayState">;
}>;

type Motion = {
  transition?: Transition;
  animation?: Animation;
};

export type Style = ImportantDeclarations<Layout & Typography & Appearance & Motion> &
  Readonly<{
    on?: Readonly<
      Partial<Record<SimplePseudoClass, Style>> & {
        breakpoint?: Readonly<Partial<Record<keyof typeof breakpoint, Style>>>;
      }
    >;
  }>;
