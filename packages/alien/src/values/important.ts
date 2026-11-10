export type Important<Value> = Readonly<{ kind: "important"; value: Value }>;

export function important<const Value>(value: Value): Important<Value> {
  return { kind: "important", value };
}
