import { expect, test } from "bun:test";
import { type AtomicClass, composeAtomicStyles } from "../src/styles/compose.ts";

const atom = (property: string, className: string, condition = ""): AtomicClass => ({
  property,
  className,
  condition,
});

test("last style wins per property without dropping unrelated declarations", () => {
  const red = [atom("color", "red"), atom("padding-top", "top")];
  const blue = [atom("color", "blue")];
  expect(composeAtomicStyles([red, blue])).toBe("top blue");
  expect(composeAtomicStyles([blue, red])).toBe("red top");
});

test("expanded shorthands retain sides that a later longhand does not override", () => {
  const padding = ["top", "right", "bottom", "left"].map((side) =>
    atom(`padding-${side}`, `p-${side}`),
  );
  const top = [atom("padding-top", "override-top")];
  expect(composeAtomicStyles([padding, top])).toBe("p-right p-bottom p-left override-top");
  expect(composeAtomicStyles([top, padding])).toBe("p-top p-right p-bottom p-left");
});

test("hover and responsive rules remain independent of base declarations", () => {
  const first = [
    atom("color", "red"),
    atom("color", "hover-red", ":hover"),
    atom("color", "md-red", "@media(min-width:48rem)"),
  ];
  const second = [atom("color", "blue"), atom("color", "hover-blue", ":hover")];
  expect(composeAtomicStyles([first, second])).toBe("md-red blue hover-blue");
});

test("condition and property identities cannot collide through delimiter concatenation", () => {
  expect(composeAtomicStyles([[atom("a:b", "first", "c"), atom("b", "second", "c:a")]])).toBe(
    "first second",
  );
});

test("missing styles and duplicate tokens do not mutate compiler metadata", () => {
  const style = Object.freeze([Object.freeze(atom("color", "red"))]);
  expect(composeAtomicStyles([undefined, style, false, style, null])).toBe("red");
  expect(composeAtomicStyles([])).toBe("");
  expect(style[0]?.className).toBe("red");
});
