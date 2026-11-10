export type Percent = Readonly<{ kind: "percent"; value: number }>;

export function percent(value: number): Percent {
  if (!Number.isFinite(value)) {
    throw new RangeError("percent() requires a finite number");
  }

  return { kind: "percent", value };
}

type Unit = "px" | "rem" | "em" | "ch" | "vw" | "vh" | "svh" | "dvh";

type LengthLiteral<U extends Unit> = Readonly<{
  kind: "length";
  value: number;
  unit: U;
}>;

type Subtraction = Readonly<{
  kind: "subtract";
  left: Length;
  right: Length;
}>;

export type Length = LengthLiteral<Unit> | Subtraction;

function length<U extends Unit>(unit: U, value: number): LengthLiteral<U> {
  if (!Number.isFinite(value)) {
    throw new RangeError(`${unit}() requires a finite number`);
  }

  return { kind: "length", value, unit };
}

export function px(value: number): LengthLiteral<"px"> {
  return length("px", value);
}

export function rem(value: number): LengthLiteral<"rem"> {
  return length("rem", value);
}

export function ch(value: number): LengthLiteral<"ch"> {
  return length("ch", value);
}

export function em(value: number): LengthLiteral<"em"> {
  return length("em", value);
}

export function vw(value: number): LengthLiteral<"vw"> {
  return length("vw", value);
}

export function vh(value: number): LengthLiteral<"vh"> {
  return length("vh", value);
}

export function svh(value: number): LengthLiteral<"svh"> {
  return length("svh", value);
}

export function dvh(value: number): LengthLiteral<"dvh"> {
  return length("dvh", value);
}

export type Fraction = Readonly<{ kind: "fraction"; value: number }>;

export function fr(value: number): Fraction {
  if (!Number.isFinite(value) || value < 0) {
    throw new RangeError("fr() requires a finite, non-negative number");
  }

  return { kind: "fraction", value };
}

export type Time = Readonly<{ kind: "time"; value: number; unit: "ms" }>;

export function ms(value: number): Time {
  if (!Number.isFinite(value)) {
    throw new RangeError("ms() requires a finite number");
  }

  return { kind: "time", value, unit: "ms" };
}

export const calc = {
  sub(left: Length, right: Length): Subtraction {
    return { kind: "subtract", left, right };
  },
};
