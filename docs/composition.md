# Atomic composition

The public `cx` helper accepts atomic descriptors in addition to ordinary class strings:

```ts
import { cx, type AtomicClass } from "alien/cx";

const first: readonly AtomicClass[] = [
  { property: "color", condition: "", className: "red" },
  { property: "padding-top", condition: "", className: "paddingTop" },
];
const second: readonly AtomicClass[] = [
  { property: "color", condition: "", className: "blue" },
];

cx(first, second); // "paddingTop blue"
```

Each descriptor must refer to an existing CSS rule containing exactly its declared longhand at its declared condition. This API does not generate CSS. Use a consistent condition representation for equivalent conditions, and a unique class name for each distinct rule. Expand shorthands into independent longhand classes before composing: one class that assigns all four padding sides cannot be partially removed.

Later descriptors replace earlier descriptors for the same property and exact condition. Base, hover and media-query conditions are independent and still interact through normal CSS cascading. Ordinary strings are opaque and only deduplicated; their declarations cannot be inferred from their names.

The compiler currently returns ordinary class strings from `alien.create()`. It does **not yet generate these descriptors**, so this API does not change existing generated styles or benchmark results. Mixed physical/logical properties and overlapping conditions require further compiler work before automatic composition can be enabled. Descriptor-based composition is a runtime foundation, not a claim that the compiler migration is complete.
