const auto = { auto: "auto" } as const;
const normal = { normal: "normal" } as const;
const intrinsicSize = {
  minContent: "min-content",
  maxContent: "max-content",
  fitContent: "fit-content",
} as const;
const size = { ...auto, ...intrinsicSize } as const;
const maxSize = { none: "none", ...intrinsicSize } as const;
const overflow = { visible: "visible", hidden: "hidden", clip: "clip", auto: "auto", scroll: "scroll" } as const;
const alignPosition = {
  normal: "normal",
  center: "center",
  start: "start",
  end: "end",
  flexStart: "flex-start",
  flexEnd: "flex-end",
  stretch: "stretch",
} as const;
const alignItems = { ...alignPosition, baseline: "baseline", selfStart: "self-start", selfEnd: "self-end" } as const;
const distribution = {
  ...alignPosition,
  spaceBetween: "space-between",
  spaceAround: "space-around",
  spaceEvenly: "space-evenly",
} as const;
const gridTemplate = { none: "none", subgrid: "subgrid" } as const;

export const layoutKeywords = {
  display: {
    none: "none",
    contents: "contents",
    block: "block",
    inline: "inline",
    inlineBlock: "inline-block",
    flowRoot: "flow-root",
    flex: "flex",
    inlineFlex: "inline-flex",
    grid: "grid",
    inlineGrid: "inline-grid",
    table: "table",
    inlineTable: "inline-table",
    listItem: "list-item",
  },
  position: { static: "static", relative: "relative", absolute: "absolute", fixed: "fixed", sticky: "sticky" },
  boxSizing: { contentBox: "content-box", borderBox: "border-box" },
  width: size,
  height: size,
  minWidth: size,
  minHeight: size,
  maxWidth: maxSize,
  maxHeight: maxSize,
  aspectRatio: auto,
  inset: auto,
  zIndex: auto,
  overflow,
  overflowX: overflow,
  overflowY: overflow,
  margin: auto,
  gap: normal,
  rowGap: normal,
  columnGap: normal,
  flex: { auto: "auto", none: "none" },
  flexDirection: { row: "row", rowReverse: "row-reverse", column: "column", columnReverse: "column-reverse" },
  flexWrap: { nowrap: "nowrap", wrap: "wrap", wrapReverse: "wrap-reverse" },
  flexBasis: { ...size, content: "content" },
  alignItems,
  alignSelf: { ...auto, ...alignItems },
  alignContent: { ...distribution, baseline: "baseline" },
  justifyContent: { ...distribution, left: "left", right: "right" },
  justifyItems: { ...alignItems, left: "left", right: "right" },
  justifySelf: { ...auto, ...alignItems, left: "left", right: "right" },
  gridTemplateColumns: gridTemplate,
  gridTemplateRows: gridTemplate,
  gridTrack: { ...auto, minContent: "min-content", maxContent: "max-content" },
  gridAutoFlow: { row: "row", column: "column", dense: "dense", rowDense: "row dense", columnDense: "column dense" },
  gridColumn: auto,
  gridRow: auto,
} as const;
