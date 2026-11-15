# CSS size comparison

Run `bun run bench:css` to regenerate this report, the [HTML report](benchmarks/index.html), [JSON results](benchmarks/results.json), and the measured CSS files. The terminal prints winners, percentage differences and scenario totals. Add `--check` to require outright Alien wins on all three metrics; `--check=brotli --min-savings=20` requires at least 20% savings against every competitor in every scenario. A failed check exits with status 1 after writing the reports.

Matching style declarations compiled with Alien, Tailwind 4.3.5, and StyleX 0.19.2. Lower is better; all sizes are in bytes. Bold values mark the smallest output in each scenario.

Tailwind uses its standard theme without Preflight. All outputs use Lightning CSS 1.33.0, gzip level 9, and Brotli quality 11. Each scenario produces one stylesheet, including the 20-module scenario. JavaScript, HTML class names, resets, build time, and runtime performance are excluded. Alien uses its default compact atomic project generation: style values are descriptors composed with cx(), compiler-generated logical and physical properties use ordered class variants, conditions use fixed precedence, and short project prefixes can collide. Its JavaScript and runtime overhead are not measured. These fixtures do not establish a universal winner or framework compatibility. Alien is measured with default atomic complete-project compilation (alien generate) consistently in all six scenarios. There is one Alien result per scenario, never a best-of selection between compilation modes. These results do not measure the incremental Vite/webpack path. Percentage savings are (competitor bytes − Alien bytes) / competitor bytes × 100; negative savings mean Alien is larger.

## Winners at a glance

Smallest CSS by scenario and metric. Ties credit every tied tool. This ranks CSS size, not overall framework quality.

| Scenario | Minified CSS winner | gzip winner | Brotli winner |
| --- | --- | --- | --- |
| One button | Alien | Alien | Alien |
| Button with focus and breakpoint | Alien | Alien | Alien |
| 100 alternating color variants | Alien | Alien | Alien |
| 100 cards and buttons, one module | Alien | Alien | Alien |
| 100 cards and buttons, 20 modules | Alien | Alien | Alien |
| 100 distinct width/height pairs | Alien | Alien | Alien |

| Tool | Minified wins | gzip wins | Brotli wins |
| --- | ---: | ---: | ---: |
| Alien | 6/6 | 6/6 | 6/6 |
| Tailwind | 0/6 | 0/6 | 0/6 |
| StyleX | 0/6 | 0/6 | 0/6 |


## One button

**Winners — minified: Alien · gzip: Alien · brotli: Alien**

| Tool | Minified CSS | gzip | Brotli |
| --- | ---: | ---: | ---: |
| Alien | **248** | **174** | **124** |
| Tailwind | 912 | 494 | 415 |
| StyleX | 351 | 227 | 171 |

- Alien vs Tailwind — minified: 72.8% smaller, gzip: 64.8% smaller, brotli: 70.1% smaller
- Alien vs StyleX — minified: 29.3% smaller, gzip: 23.3% smaller, brotli: 27.5% smaller

## Button with focus and breakpoint

**Winners — minified: Alien · gzip: Alien · brotli: Alien**

| Tool | Minified CSS | gzip | Brotli |
| --- | ---: | ---: | ---: |
| Alien | **359** | **219** | **171** |
| Tailwind | 1,094 | 547 | 459 |
| StyleX | 495 | 288 | 222 |

- Alien vs Tailwind — minified: 67.2% smaller, gzip: 60.0% smaller, brotli: 62.7% smaller
- Alien vs StyleX — minified: 27.5% smaller, gzip: 24.0% smaller, brotli: 23.0% smaller

## 100 alternating color variants

**Winners — minified: Alien · gzip: Alien · brotli: Alien**

| Tool | Minified CSS | gzip | Brotli |
| --- | ---: | ---: | ---: |
| Alien | **35** | **47** | **39** |
| Tailwind | 147 | 137 | 119 |
| StyleX | 40 | 55 | 44 |

- Alien vs Tailwind — minified: 76.2% smaller, gzip: 65.7% smaller, brotli: 67.2% smaller
- Alien vs StyleX — minified: 12.5% smaller, gzip: 14.5% smaller, brotli: 11.4% smaller

## 100 cards and buttons, one module

**Winners — minified: Alien · gzip: Alien · brotli: Alien**

| Tool | Minified CSS | gzip | Brotli |
| --- | ---: | ---: | ---: |
| Alien | **2,886** | **696** | **390** |
| Tailwind | 3,505 | 1,000 | 672 |
| StyleX | 5,375 | 1,110 | 846 |

- Alien vs Tailwind — minified: 17.7% smaller, gzip: 30.4% smaller, brotli: 42.0% smaller
- Alien vs StyleX — minified: 46.3% smaller, gzip: 37.3% smaller, brotli: 53.9% smaller

## 100 cards and buttons, 20 modules

**Winners — minified: Alien · gzip: Alien · brotli: Alien**

| Tool | Minified CSS | gzip | Brotli |
| --- | ---: | ---: | ---: |
| Alien | **2,886** | **695** | **394** |
| Tailwind | 3,505 | 1,000 | 672 |
| StyleX | 5,375 | 1,110 | 846 |

- Alien vs Tailwind — minified: 17.7% smaller, gzip: 30.5% smaller, brotli: 41.4% smaller
- Alien vs StyleX — minified: 46.3% smaller, gzip: 37.4% smaller, brotli: 53.4% smaller

## 100 distinct width/height pairs

**Winners — minified: Alien · gzip: Alien · brotli: Alien**

| Tool | Minified CSS | gzip | Brotli |
| --- | ---: | ---: | ---: |
| Alien | **4,048** | **779** | **412** |
| Tailwind | 5,197 | 937 | 513 |
| StyleX | 4,383 | 1,572 | 1,201 |

- Alien vs Tailwind — minified: 22.1% smaller, gzip: 16.9% smaller, brotli: 19.7% smaller
- Alien vs StyleX — minified: 7.6% smaller, gzip: 50.4% smaller, brotli: 65.7% smaller

## Reading the results

Percentages use (competitor bytes − Alien bytes) / competitor bytes; “larger” means Alien loses. Compare Brotli or gzip for CSS transfer size, and minified CSS for the uncompressed stylesheet. A smaller stylesheet does not necessarily mean a smaller application: extra class names also consume HTML and JavaScript bytes. The atomic mode emits metadata and requires the cx runtime; neither is included in these CSS sizes.

The default atomic mode shares declarations across the complete project and requires cx() to compose descriptor values. Its short namespaces have a collision risk; manually constructed overlapping descriptors without ordering metadata are rejected. Tailwind and StyleX reuse utilities across modules; the split-module fixture deliberately measures that difference. No tool is guaranteed to win for every workload.
