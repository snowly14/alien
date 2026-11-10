export type Custom<Value> = Readonly<{ kind: "custom"; value: Value }>;

export function custom<const Value>(value: Value): Custom<Value> {
  return { kind: "custom", value };
}
