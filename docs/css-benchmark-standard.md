# CSS size comparison

Run `bun run bench:css --standard` to regenerate this report, the [HTML report](benchmarks-standard/index.html), [JSON results](benchmarks-standard/results.json), and the measured CSS files. The terminal prints winners, percentage differences and scenario totals. Add `--check` to require outright Alien wins on all three metrics; `--check=brotli --min-savings=20` requires at least 20% savings against every competitor in every scenario. A failed check exits with status 1 after writing the reports.

Matching style declarations compiled with Alien, Tailwind 4.3.5, and StyleX 0.19.2. Lower is better; all sizes are in bytes. Bold values mark the smallest output in each scenario.

Tailwind uses its standard theme without Preflight. All outputs use Lightning CSS 1.33.0, gzip level 9, and Brotli quality 11. Each scenario produces one stylesheet, including the 20-module scenario. JavaScript, HTML class names, resets, build time, and runtime performance are excluded. The alternating-color fixture tests conflicting values; Alien preserves source-order composition, while utility systems have their own composition rules. These fixtures do not establish a universal winner or framework compatibility. Alien is measured with standard complete-project compilation (alien generate --standard) consistently in all six scenarios. There is one Alien result per scenario, never a best-of selection between compilation modes. These results do not measure the incremental Vite/webpack path. Percentage savings are (competitor bytes − Alien bytes) / competitor bytes × 100; negative savings mean Alien is larger.

## Winners at a glance

Smallest CSS by scenario and metric. Ties credit every tied tool. This ranks CSS size, not overall framework quality.

| Scenario | Minified CSS winner | gzip winner | Brotli winner |
| --- | --- | --- | --- |
| One button | Alien | Alien | Alien |
| Button with focus and breakpoint | Alien | Alien | Alien |
| 100 alternating color variants | StyleX | StyleX | StyleX |
| 100 cards and buttons, one module | Tailwind | Alien | Alien |
| 100 cards and buttons, 20 modules | Tailwind | Alien | Alien |
| 100 distinct width/height pairs | StyleX | Alien | Alien |

| Tool | Minified wins | gzip wins | Brotli wins |
| --- | ---: | ---: | ---: |
| Alien | 2/6 | 5/6 | 5/6 |
| Tailwind | 2/6 | 0/6 | 0/6 |
| StyleX | 2/6 | 1/6 | 1/6 |


## One button

**Winners — minified: Alien · gzip: Alien · brotli: Alien**

| Tool | Minified CSS | gzip | Brotli |
| --- | ---: | ---: | ---: |
| Alien | **205** | **178** | **113** |
| Tailwind | 912 | 494 | 415 |
| StyleX | 351 | 227 | 171 |

- Alien vs Tailwind — minified: 77.5% smaller, gzip: 64.0% smaller, brotli: 72.8% smaller
- Alien vs StyleX — minified: 41.6% smaller, gzip: 21.6% smaller, brotli: 33.9% smaller

## Button with focus and breakpoint

**Winners — minified: Alien · gzip: Alien · brotli: Alien**

| Tool | Minified CSS | gzip | Brotli |
| --- | ---: | ---: | ---: |
| Alien | **343** | **229** | **153** |
| Tailwind | 1,094 | 547 | 459 |
| StyleX | 495 | 288 | 222 |

- Alien vs Tailwind — minified: 68.6% smaller, gzip: 58.1% smaller, brotli: 66.7% smaller
- Alien vs StyleX — minified: 30.7% smaller, gzip: 20.5% smaller, brotli: 31.1% smaller

## 100 alternating color variants

**Winners — minified: StyleX · gzip: StyleX · brotli: StyleX**

| Tool | Minified CSS | gzip | Brotli |
| --- | ---: | ---: | ---: |
| Alien | 3,798 | 319 | 202 |
| Tailwind | 147 | 137 | 119 |
| StyleX | **40** | **55** | **44** |

- Alien vs Tailwind — minified: 2483.7% larger, gzip: 132.8% larger, brotli: 69.7% larger
- Alien vs StyleX — minified: 9395.0% larger, gzip: 480.0% larger, brotli: 359.1% larger

## 100 cards and buttons, one module

**Winners — minified: Tailwind · gzip: Alien · brotli: Alien**

| Tool | Minified CSS | gzip | Brotli |
| --- | ---: | ---: | ---: |
| Alien | 7,896 | **747** | **450** |
| Tailwind | **3,505** | 1,000 | 672 |
| StyleX | 5,375 | 1,110 | 846 |

- Alien vs Tailwind — minified: 125.3% larger, gzip: 25.3% smaller, brotli: 33.0% smaller
- Alien vs StyleX — minified: 46.9% larger, gzip: 32.7% smaller, brotli: 46.8% smaller

## 100 cards and buttons, 20 modules

**Winners — minified: Tailwind · gzip: Alien · brotli: Alien**

| Tool | Minified CSS | gzip | Brotli |
| --- | ---: | ---: | ---: |
| Alien | 7,900 | **749** | **456** |
| Tailwind | **3,505** | 1,000 | 672 |
| StyleX | 5,375 | 1,110 | 846 |

- Alien vs Tailwind — minified: 125.4% larger, gzip: 25.1% smaller, brotli: 32.1% smaller
- Alien vs StyleX — minified: 47.0% larger, gzip: 32.5% smaller, brotli: 46.1% smaller

## 100 distinct width/height pairs

**Winners — minified: StyleX · gzip: Alien · brotli: Alien**

| Tool | Minified CSS | gzip | Brotli |
| --- | ---: | ---: | ---: |
| Alien | 5,248 | **744** | **386** |
| Tailwind | 5,197 | 937 | 513 |
| StyleX | **4,383** | 1,572 | 1,201 |

- Alien vs Tailwind — minified: 1.0% larger, gzip: 20.6% smaller, brotli: 24.8% smaller
- Alien vs StyleX — minified: 19.7% larger, gzip: 52.7% smaller, brotli: 67.9% smaller

## Reading the results

Percentages use (competitor bytes − Alien bytes) / competitor bytes; “larger” means Alien loses. Compare Brotli or gzip for CSS transfer size, and minified CSS for the uncompressed stylesheet. A smaller stylesheet does not necessarily mean a smaller application: extra class names also consume HTML and JavaScript bytes. Alien selects sharing plans using CSS plus its generated class map, rather than CSS alone.

Alien preserves declaration order. Independent transforms retain module-specific class names; complete-project generation uses a shared namespace and optimizes safe declarations across modules. It shares safe repeated declarations individually or in groups when that reduces its cost model. Tailwind and StyleX reuse utilities across modules; the split-module fixture deliberately measures that difference. No tool is guaranteed to win for every workload.
