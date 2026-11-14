/** Joins class names while ignoring absent conditional values. */
export function cx(...classes: readonly (string | false | null | undefined)[]): string {
  return classes
    .filter((value): value is string => typeof value === "string" && value.length > 0)
    .join(" ");
}
