# Alien CSS for webpack

```sh
bun add @alien-css/core
bun add -d @alien-css/webpack
```

```js
import { alienRule } from "@alien-css/webpack";

export default { module: { rules: [alienRule()] } };
```

Keep your application's CSS loaders configured. See the [framework guide](https://github.com/snowly14/alien/blob/main/docs/frameworks.md) for webpack and Next.js integration.
