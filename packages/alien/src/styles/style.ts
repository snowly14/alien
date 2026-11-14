import type { Mode, ProjectMode } from "../config";
import type { breakpoint } from "../tokens/breakpoint";
import type { TokenReference } from "../tokens/reference";
import type { Color, TokenColor } from "../values/color";
import type { Custom } from "../values/custom";
import type { Important } from "../values/important";
import type { Fraction, Length, LengthString, Percent, Time } from "../values/units";
import type { CssKeyword } from "./css";
import type { SimplePseudoClass } from "./pseudo-classes";

type StyleColor<M extends Mode> =
  | TokenColor
  | CssKeyword<"color">
  | (M extends "free" ? Color | string | Custom<Color> : never);
type LengthOrPercent = Length | Percent | LengthString;
type Space<M extends Mode> =
  | TokenReference<"space">
  | (M extends "free" ? LengthOrPercent | Custom<LengthOrPercent> : never);
type Margin<M extends Mode> = Space<M> | CssKeyword<"margin">;
type Radius<M extends Mode> =
  | TokenReference<"radius">
  | (M extends "free" ? LengthOrPercent | Custom<LengthOrPercent> : never);
type Inset = LengthOrPercent | CssKeyword<"inset">;
type GridTrack = LengthOrPercent | Fraction | CssKeyword<"gridTrack">;
type GridTrackList = readonly [GridTrack, ...GridTrack[]];
type FontFamily = CssKeyword<"fontFamily"> | Custom<string>;
type Duration<M extends Mode> = TokenReference<"duration"> | (M extends "free" ? Time : never);
type TimingFunction = TokenReference<"ease"> | CssKeyword<"timingFunction">;
type TranslateAxes = {
  readonly x?: LengthOrPercent;
  readonly y?: LengthOrPercent;
  readonly z?: Length;
};
type Translate =
  | CssKeyword<"translate">
  | (TranslateAxes &
      ({ readonly x: LengthOrPercent } | { readonly y: LengthOrPercent } | { readonly z: Length }));

type ImportantDeclarations<Declarations> = {
  readonly [Property in keyof Declarations]:
    | Required<Declarations>[Property]
    | Important<Required<Declarations>[Property]>;
};

type Border<M extends Mode> = ImportantDeclarations<{
  width?: Length | CssKeyword<"borderWidth">;
  style?: CssKeyword<"borderStyle">;
  color?: StyleColor<M>;
}>;

type Layout<M extends Mode> = {
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

  padding?: Space<M>;
  paddingTop?: Space<M>;
  paddingRight?: Space<M>;
  paddingBottom?: Space<M>;
  paddingLeft?: Space<M>;
  paddingBlock?: Space<M>;
  paddingInline?: Space<M>;
  margin?: Margin<M>;
  marginTop?: Margin<M>;
  marginRight?: Margin<M>;
  marginBottom?: Margin<M>;
  marginLeft?: Margin<M>;
  marginBlock?: Margin<M>;
  marginInline?: Margin<M>;
  gap?: Space<M> | CssKeyword<"gap">;
  rowGap?: Space<M> | CssKeyword<"rowGap">;
  columnGap?: Space<M> | CssKeyword<"columnGap">;

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

type Typography<M extends Mode> = {
  fontFamily?: FontFamily | readonly [FontFamily, ...FontFamily[]];
  fontStyle?: CssKeyword<"fontStyle">;
  fontWeight?:
    | CssKeyword<"fontWeight">
    | TokenReference<"fontWeight">
    | (M extends "free" ? number | Custom<number> : never);
  fontSize?: TokenReference<"fontSize"> | (M extends "free" ? LengthOrPercent : never);
  lineHeight?:
    | TokenReference<"lineHeight">
    | CssKeyword<"lineHeight">
    | (M extends "free" ? number | LengthOrPercent : never);
  letterSpacing?: Length | CssKeyword<"letterSpacing">;
  textAlign?: CssKeyword<"textAlign">;
  textDecorationLine?: CssKeyword<"textDecorationLine">;
  textDecorationStyle?: CssKeyword<"textDecorationStyle">;
  textDecorationColor?: StyleColor<M>;
  textOverflow?: CssKeyword<"textOverflow">;
  textTransform?: CssKeyword<"textTransform">;
  whiteSpace?: CssKeyword<"whiteSpace">;
  overflowWrap?: CssKeyword<"overflowWrap">;
  wordBreak?: CssKeyword<"wordBreak">;
};

type Appearance<M extends Mode> = {
  color?: StyleColor<M>;
  backgroundColor?: StyleColor<M>;
  backgroundRepeat?: CssKeyword<"backgroundRepeat">;
  backgroundSize?: LengthOrPercent | CssKeyword<"backgroundSize">;
  backgroundPosition?: LengthOrPercent | CssKeyword<"backgroundPosition">;
  opacity?: number | Percent;
  objectFit?: CssKeyword<"objectFit">;
  objectPosition?: LengthOrPercent | CssKeyword<"objectPosition">;
  boxShadow?: TokenReference<"shadow">;
  borderRadius?: Radius<M>;
  borderTopLeftRadius?: Radius<M>;
  borderTopRightRadius?: Radius<M>;
  borderBottomLeftRadius?: Radius<M>;
  borderBottomRightRadius?: Radius<M>;
  outlineColor?: StyleColor<M>;
  outlineStyle?: CssKeyword<"outlineStyle">;
  outlineWidth?: Length | CssKeyword<"outlineWidth">;
  outlineOffset?: Length;
  border?: Border<M>;
  borderTop?: Border<M>;
  borderRight?: Border<M>;
  borderBottom?: Border<M>;
  borderLeft?: Border<M>;
  visibility?: CssKeyword<"visibility">;
  cursor?: CssKeyword<"cursor">;
  pointerEvents?: CssKeyword<"pointerEvents">;
  userSelect?: CssKeyword<"userSelect">;
  appearance?: CssKeyword<"appearance">;
};

type TransitionProperty = CssKeyword<"transitionProperty">;
type TransitionTarget = Exclude<TransitionProperty, "all" | "none">;

type Transition<M extends Mode> = ImportantDeclarations<{
  property?: TransitionProperty | readonly [TransitionTarget, ...TransitionTarget[]];
  duration?: Duration<M>;
  delay?: Duration<M>;
  ease?: TimingFunction;
}>;

type Animation<M extends Mode> = ImportantDeclarations<{
  name?: CssKeyword<"animationName"> | Custom<string>;
  duration?: Duration<M>;
  delay?: Duration<M>;
  ease?: TimingFunction;
  iterationCount?: number | CssKeyword<"animationIterationCount">;
  direction?: CssKeyword<"animationDirection">;
  fillMode?: CssKeyword<"animationFillMode">;
  playState?: CssKeyword<"animationPlayState">;
}>;

type Motion<M extends Mode> = {
  translate?: Translate;
  transition?: Transition<M>;
  animation?: Animation<M>;
};

export type Style<
  M extends Mode = ProjectMode,
  InBreakpoint extends boolean = false,
> = ImportantDeclarations<Layout<M> & Typography<M> & Appearance<M> & Motion<M>> &
  Readonly<{
    on?: Readonly<
      Partial<Record<SimplePseudoClass, Style<M, InBreakpoint>>> &
        (InBreakpoint extends true
          ? { breakpoint?: never }
          : {
              breakpoint?: Readonly<Partial<Record<keyof typeof breakpoint, Style<M, true>>>>;
            })
    >;
  }>;
