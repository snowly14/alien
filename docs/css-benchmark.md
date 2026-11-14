# CSS size comparison

Run `bun run bench:css` to regenerate this report, the [HTML report](benchmarks/index.html), [JSON results](benchmarks/results.json), and the measured CSS files.

Matching style declarations compiled with Alien, Tailwind 4.3.5, and StyleX 0.19.2. Lower is better; all sizes are in bytes. Bold values mark the smallest output in each scenario.

Tailwind uses its standard theme without Preflight. All outputs use Lightning CSS 1.33.0, gzip level 9, and Brotli quality 11. Each scenario produces one stylesheet, including the 20-module scenario. JavaScript, HTML class names, resets, build time, and runtime performance are excluded. The alternating-color fixture tests conflicting values; Alien preserves source-order composition, while utility systems have their own composition rules. These fixtures do not establish a universal winner or framework compatibility. Alien is measured with complete-project compilation (alien generate) consistently in all six scenarios. There is one Alien result per scenario, never a best-of selection between compilation modes. These results do not measure the incremental Vite/webpack path. Percentage savings are (competitor bytes − Alien bytes) / competitor bytes × 100; negative savings mean Alien is larger.

## One button

| Tool | Minified CSS | gzip | Brotli |
| --- | ---: | ---: | ---: |
| Alien | **206** | **179** | **114** |
| Tailwind | 912 | 494 | 415 |
| StyleX | 351 | 227 | 171 |

- Alien vs Tailwind — minified: 77.4% smaller, gzip: 63.8% smaller, brotli: 72.5% smaller
- Alien vs StyleX — minified: 41.3% smaller, gzip: 21.1% smaller, brotli: 33.3% smaller

## Button with focus and breakpoint

| Tool | Minified CSS | gzip | Brotli |
| --- | ---: | ---: | ---: |
| Alien | **346** | **230** | **154** |
| Tailwind | 1,094 | 547 | 459 |
| StyleX | 495 | 288 | 222 |

- Alien vs Tailwind — minified: 68.4% smaller, gzip: 58.0% smaller, brotli: 66.4% smaller
- Alien vs StyleX — minified: 30.1% smaller, gzip: 20.1% smaller, brotli: 30.6% smaller

## 100 alternating color variants

| Tool | Minified CSS | gzip | Brotli |
| --- | ---: | ---: | ---: |
| Alien | 3,940 | 315 | 219 |
| Tailwind | 147 | 137 | 119 |
| StyleX | **40** | **55** | **44** |

- Alien vs Tailwind — minified: 2580.3% larger, gzip: 129.9% larger, brotli: 84.0% larger
- Alien vs StyleX — minified: 9750.0% larger, gzip: 472.7% larger, brotli: 397.7% larger

## 100 cards and buttons, one module

| Tool | Minified CSS | gzip | Brotli |
| --- | ---: | ---: | ---: |
| Alien | 8,040 | **742** | **426** |
| Tailwind | **3,505** | 1,000 | 672 |
| StyleX | 5,375 | 1,110 | 846 |

- Alien vs Tailwind — minified: 129.4% larger, gzip: 25.8% smaller, brotli: 36.6% smaller
- Alien vs StyleX — minified: 49.6% larger, gzip: 33.2% smaller, brotli: 49.6% smaller

## 100 cards and buttons, 20 modules

| Tool | Minified CSS | gzip | Brotli |
| --- | ---: | ---: | ---: |
| Alien | 8,044 | **743** | **424** |
| Tailwind | **3,505** | 1,000 | 672 |
| StyleX | 5,375 | 1,110 | 846 |

- Alien vs Tailwind — minified: 129.5% larger, gzip: 25.7% smaller, brotli: 36.9% smaller
- Alien vs StyleX — minified: 49.7% larger, gzip: 33.1% smaller, brotli: 49.9% smaller

## 100 distinct width/height pairs

| Tool | Minified CSS | gzip | Brotli |
| --- | ---: | ---: | ---: |
| Alien | 5,390 | **750** | **376** |
| Tailwind | 5,197 | 937 | 513 |
| StyleX | **4,383** | 1,572 | 1,201 |

- Alien vs Tailwind — minified: 3.7% larger, gzip: 20.0% smaller, brotli: 26.7% smaller
- Alien vs StyleX — minified: 23.0% larger, gzip: 52.3% smaller, brotli: 68.7% smaller

## Reading the results

Percentages use (competitor bytes − Alien bytes) / competitor bytes; “larger” means Alien loses. Compare Brotli or gzip for CSS transfer size, and minified CSS for the uncompressed stylesheet. A smaller stylesheet does not necessarily mean a smaller application: extra class names also consume HTML and JavaScript bytes. Alien selects sharing plans using CSS plus its generated class map, rather than CSS alone.

Alien preserves declaration order. Independent transforms retain module-specific class names; complete-project generation uses a shared namespace and optimizes safe declarations across modules. It shares safe repeated declarations individually or in groups when that reduces its cost model. Tailwind and StyleX reuse utilities across modules; the split-module fixture deliberately measures that difference. No tool is guaranteed to win for every workload.
