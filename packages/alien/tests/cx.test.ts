import { expect, test } from "bun:test";
import { cx } from "alien/cx";

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
