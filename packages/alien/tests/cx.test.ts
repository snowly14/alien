import { expect, test } from "bun:test";
import { cx } from "alien/cx";

test("joins class names and ignores absent conditional values", () => {
  expect(cx("button", false, undefined, "active", null, "")).toBe("button active");
  expect(cx()).toBe("");
});
