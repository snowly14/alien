# Atomic composition

The public `cx` helper accepts atomic descriptors in addition to ordinary class strings:

```ts
import { cx, type AtomicClass } from "@alien-css/core/cx";

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

Later descriptors replace earlier descriptors for the same property and exact condition, except that an `important: true` declaration takes priority over a normal one. Base, hover and media-query conditions are independent and still interact through normal CSS cascading. Ordinary strings are opaque and only deduplicated; their declarations cannot be inferred from their names.

`alien generate` now emits compact descriptors by default. Call `cx()` even for a single generated style; this is a breaking change for applications that previously used the generated value as a class string. The generated descriptors handle overlapping physical/logical padding and margin with compiler-emitted ordering variants. Conditions have a fixed compiler-defined precedence, and the compact project prefix can collide across independently generated stylesheets. Use `alien generate --atomic-isolation strong` for a longer namespace, or `alien generate --standard` for the prior string-class and source-order contract. Vite and webpack incremental transforms continue to emit ordinary class strings.
