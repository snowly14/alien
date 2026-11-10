import { expect, test } from "bun:test";
import { alien, tokens } from "alien";
import { alpha, dvh, fr, hwb, mix, ms, percent, px, rem, rgb } from "alien/values";

test("the public entry points share the built-in palette", () => {
  expect(alien.color).toBe(tokens.color);
  expect(tokens.color.green[600].path).toBe("color.green.600");
});

test("CSS units reject non-finite values and preserve signed lengths", () => {
  for (const value of [NaN, Infinity, -Infinity]) {
    expect(() => px(value)).toThrow(RangeError);
    expect(() => rem(value)).toThrow(RangeError);
    expect(() => percent(value)).toThrow(RangeError);
  }

  expect(px(-5).value).toBe(-5);
  expect(percent(125).value).toBe(125);
});

test("grid fractions and transition times keep their own types and ranges", () => {
  expect(dvh(100)).toEqual({ kind: "length", value: 100, unit: "dvh" });
  expect(fr(1)).toEqual({ kind: "fraction", value: 1 });
  expect(ms(-150)).toEqual({ kind: "time", value: -150, unit: "ms" });
  expect(() => fr(-1)).toThrow(RangeError);
  expect(() => fr(Infinity)).toThrow(RangeError);
  expect(() => ms(Infinity)).toThrow(RangeError);
});

test("color operations validate channels and their own percentage ranges", () => {
  expect(rgb(0, 127.5, 255).green).toBe(127.5);
  expect(() => rgb(256, 0, 0)).toThrow(RangeError);
  expect(() => rgb(0, NaN, 0)).toThrow(RangeError);
  expect(() => rgb(0, 0, -1)).toThrow(RangeError);
  expect(() => alpha(tokens.color.white, percent(101))).toThrow(RangeError);
  expect(() => mix(tokens.color.black, tokens.color.white, percent(-1))).toThrow(RangeError);
  expect(hwb(0, percent(80), percent(80)).blackness.value).toBe(80);
});
