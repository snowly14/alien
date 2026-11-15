import { expect, test } from "bun:test";
import { cx } from "@alien-css/core/cx";

test("joins class names and ignores absent conditional values", () => {
  expect(cx("button", false, undefined, "active", null, "")).toBe("button active");
  expect(cx()).toBe("");
});

test("normalizes HTML whitespace and keeps the last occurrence of each class", () => {
  expect(cx("button active", "button\twide", false, "active\n", " \r\f ")).toBe(
    "button wide active",
  );
  expect(cx("same same", "same")).toBe("same");
  expect(cx("red", "blue")).toBe("red blue");
  expect(cx("non\u00a0breaking")).toBe("non\u00a0breaking");
});

test("public cx resolves atomic properties while keeping unrelated classes and conditions", () => {
  const first = [
    { property: "color", condition: "", className: "red" },
    { property: "padding-top", condition: "", className: "pad" },
    { property: "color", condition: ":hover", className: "hover-red" },
  ];
  const second = [{ property: "color", condition: "", className: "blue" }];
  expect(cx("external", first, false, second)).toBe("external pad hover-red blue");
  expect(cx(second, first)).toBe("red pad hover-red");
  expect(first).toHaveLength(3);
});

test("public cx preserves important atomic declarations", () => {
  const important = [{ property: "color", condition: "", className: "red", important: true }];
  const normal = [{ property: "color", condition: "", className: "blue" }];
  expect(cx(important, normal)).toBe("red");
  expect(cx(normal, important)).toBe("red");
});

test("public cx rejects partially overlapping atomic properties instead of losing a side", () => {
  const logical = [{ property: "padding-inline", condition: "", className: "logical" }];
  const physical = [{ property: "padding-left", condition: "", className: "physical" }];
  expect(() => cx(logical, physical)).toThrow("overlapping atomic CSS properties");
  expect(() => cx(physical, logical)).toThrow("overlapping atomic CSS properties");
  expect(cx("padding-inline", "padding-left")).toBe("padding-inline padding-left");
});

test("public cx decodes compact ordered class ranges across index boundaries", () => {
  const physical = [
    {
      property: "padding-left",
      condition: "",
      className: "zzX",
      orderGroup: "padding",
      variantStart: 49,
      variantCount: 2,
    },
  ];
  const logical = [
    {
      property: "padding-inline",
      condition: "",
      className: "zzZ",
      orderGroup: "padding",
      variantStart: 51,
      variantCount: 2,
    },
  ];
  expect(cx(physical, logical)).toBe("zzX zzaa");
  expect(cx(logical, physical)).toBe("zzZ zzY");
});

test("public cx composes compact compiler tokens and rejects malformed tokens", () => {
  expect(cx(["red~color"], ["blue~color"])).toBe("blue");
  expect(cx(["red~color~~1"], ["blue~color"])).toBe("red");
  expect(cx(["zzX~padding-left~~0~p~49~2"], ["zzZ~padding-inline~~0~p~51~2"])).toBe("zzX zzaa");
  for (const token of ["invalid", "~color", "class~", "a~color~~bad", "a~color~~0~x~0~2"])
    expect(() => cx([token])).toThrow("Invalid compiled atomic style");
});

test("public cx composes packed tokens with one project prefix", () => {
  expect(cx(["@zz X~padding-left~~0~p~49~2 Z~padding-inline~~0~p~51~2"])).toBe("zzX zzaa");
  expect(cx(["@zz a~color b~color~~1"])).toBe("zzb");
  for (const token of ["@bad! a~color", "@ok", "@ok  a~color"])
    expect(() => cx([token])).toThrow("Invalid packed atomic style");
});
