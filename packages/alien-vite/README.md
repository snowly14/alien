# Alien CSS for Vite

```sh
bun add @alien-css/core
bun add -d @alien-css/vite
```

```ts
import { defineConfig } from "vite";
import alien from "@alien-css/vite";

export default defineConfig({ plugins: [alien()] });
```

Place the Alien plugin before framework plugins. See the [framework guide](https://github.com/snowly14/alien/blob/main/docs/frameworks.md) for configuration and supported frameworks.
