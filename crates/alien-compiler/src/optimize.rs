//! Conservative sharing with an allocation-free cost model during plan selection.
use crate::{CompiledStyles, compile::ResolvedStyle};
use alien_emit::{write_class_map, write_rule, write_rules};
use alien_ir::Property;
use std::collections::{BTreeMap, BTreeSet};

// A fixed search bound keeps optimization predictable for large style blocks.
const MAX_CANDIDATES: usize = u64::BITS as usize;

struct Candidate {
    signatures: Vec<String>,
    class: String,
    css: String,
    declaration_size: usize,
}

pub(crate) fn optimize(
    styles: &[ResolvedStyle<'_>],
    prefix: &str,
    offset: usize,
    baseline: CompiledStyles,
) -> CompiledStyles {
    if styles.len() < 2 {
        return baseline;
    }
    let candidates = find_candidates(styles, prefix, offset);
    if candidates.is_empty() {
        return baseline;
    }
    let bundled = bundle_candidates(styles, &candidates);
    // A bundle dominates every nonempty subset of its members: they occur in
    // exactly the same styles, its class is no longer than any member's, and
    // adding another declaration removes at least two copies while emitting one.
    // Search only these nondominated choices instead of repeating atomic search.
    let plan = if bundled.len() < candidates.len() {
        &bundled
    } else {
        &candidates
    };
    let model = CostModel::new(styles, plan);
    let selected = select_plan(&model, plan.len());
    if model.size(selected) < size(&baseline) {
        emit(styles, plan, selected)
    } else {
        baseline
    }
}

fn find_candidates(styles: &[ResolvedStyle<'_>], prefix: &str, offset: usize) -> Vec<Candidate> {
    let mut signatures = BTreeMap::new();
    for style in styles {
        for rule in &style.rules {
            if rule.is_base() {
                signatures
                    .entry(rule.signature.as_str())
                    .and_modify(|(_, count)| *count += 1)
                    .or_insert((rule, 1usize));
            }
        }
    }
    if !signatures.values().any(|(_, count)| *count >= 2) {
        return Vec::new();
    }
    // Summarize conflicts once; skip this work for entirely unique styles.
    let mut properties: BTreeMap<Property, (&str, bool)> = BTreeMap::new();
    for style in styles {
        let mut base_properties = BTreeSet::new();
        for rule in &style.rules {
            let duplicate = rule.is_base() && !base_properties.insert(rule.declaration.property);
            properties
                .entry(rule.declaration.property)
                .and_modify(|(signature, safe)| *safe &= !duplicate && *signature == rule.signature)
                .or_insert((&rule.signature, !duplicate));
        }
    }
    let mut ranked: Vec<_> = signatures.into_iter().collect();
    // Prefer potential bytes saved, rather than frequency alone.
    ranked.sort_by(|left, right| {
        let savings = |entry: &(&str, (_, usize))| (entry.1.1 - 1) * entry.0.len();
        savings(right)
            .cmp(&savings(left))
            .then_with(|| left.0.cmp(right.0))
    });
    let mut candidates = Vec::new();
    for (signature, (rule, count)) in ranked {
        if count < 2 {
            continue;
        }
        let property = rule.declaration.property;
        // Moving rules to the end is safe only if every potentially overlapping
        // declaration (including conditional/important rules) agrees exactly.
        let safe = properties.iter().all(|(&other, &(value, consistent))| {
            !property.conflicts_with(other)
                || (property == other && consistent && value == signature)
        });
        if safe {
            let class = format!("{prefix}u{offset}_{}", candidates.len());
            let mut css = String::new();
            write_rule(&mut css, &class, rule.condition, &rule.declaration);
            candidates.push(Candidate {
                signatures: vec![signature.to_owned()],
                class,
                css,
                declaration_size: rule.declaration_size(),
            });
        }
        if candidates.len() == MAX_CANDIDATES {
            break;
        }
    }
    candidates
}

// Declarations with identical occurrence sets can share one class. All members
// already passed the conflict checks above, so bundling cannot change the cascade.
fn bundle_candidates(styles: &[ResolvedStyle<'_>], candidates: &[Candidate]) -> Vec<Candidate> {
    let by_signature: BTreeMap<_, _> = candidates
        .iter()
        .enumerate()
        .flat_map(|(index, candidate)| {
            candidate
                .signatures
                .iter()
                .map(move |signature| (signature.as_str(), index))
        })
        .collect();
    let mut occurrences = vec![Vec::new(); candidates.len()];
    for (style_index, style) in styles.iter().enumerate() {
        for rule in &style.rules {
            if rule.is_base()
                && let Some(&index) = by_signature.get(rule.signature.as_str())
            {
                occurrences[index].push(style_index);
            }
        }
    }
    let mut groups: BTreeMap<Vec<usize>, Vec<&Candidate>> = BTreeMap::new();
    for (candidate, positions) in candidates.iter().zip(occurrences) {
        groups.entry(positions).or_default().push(candidate);
    }
    groups
        .into_values()
        .map(|members| {
            let class = members[0].class.clone();
            let signatures: Vec<_> = members
                .iter()
                .flat_map(|member| member.signatures.iter().cloned())
                .collect();
            let declarations = signatures
                .iter()
                .map(|signature| &signature[3..signature.len() - 1])
                .collect::<Vec<_>>()
                .join(";");
            Candidate {
                css: format!(".{class}{{{declarations}}}"),
                class,
                declaration_size: members.iter().map(|member| member.declaration_size).sum(),
                signatures,
            }
        })
        .collect()
}

struct StyleCost {
    candidates: u64,
    base_count: usize,
    base_bytes: usize,
    conditional_bytes: usize,
    class_bytes: usize,
}

struct CostModel<'a> {
    styles: Vec<StyleCost>,
    candidates: &'a [Candidate],
    map_overhead: usize,
}

impl<'a> CostModel<'a> {
    fn new(styles: &[ResolvedStyle<'_>], candidates: &'a [Candidate]) -> Self {
        let indices: BTreeMap<_, _> = candidates
            .iter()
            .enumerate()
            .flat_map(|(index, candidate)| {
                candidate
                    .signatures
                    .iter()
                    .map(move |signature| (signature.as_str(), index))
            })
            .collect();
        let mut costs = Vec::with_capacity(styles.len());
        let mut names = BTreeMap::new();
        for style in styles {
            let mut cost = StyleCost {
                candidates: 0,
                base_count: 0,
                base_bytes: 0,
                conditional_bytes: 0,
                class_bytes: style.class.len(),
            };
            for rule in &style.rules {
                if rule.is_base() {
                    cost.base_count += 1;
                    cost.base_bytes += rule.declaration_size();
                    if let Some(&index) = indices.get(rule.signature.as_str()) {
                        cost.candidates |= 1 << index;
                    }
                }
            }
            let mut conditional = String::new();
            for group in style
                .rules
                .chunk_by(|left, right| left.condition == right.condition)
            {
                if !group[0].is_base() {
                    write_rules(
                        &mut conditional,
                        &style.class,
                        group[0].condition,
                        group.iter().map(|rule| &rule.declaration),
                    );
                }
            }
            cost.conditional_bytes = conditional.len();
            costs.push(cost);
            names.insert(style.name.to_owned(), String::new());
        }
        let mut javascript = String::new();
        write_class_map(&mut javascript, &names);
        Self {
            styles: costs,
            candidates,
            map_overhead: javascript.len(),
        }
    }

    fn state(&self, style: &StyleCost, selected: u64) -> StyleState {
        let shared = style.candidates & selected;
        let mut state = StyleState {
            count: shared.count_ones() as usize,
            ..StyleState::default()
        };
        for index in indices(shared) {
            state.toggle(&self.candidates[index], true);
        }
        state
    }

    fn size(&self, selected: u64) -> usize {
        self.map_overhead
            + indices(selected)
                .map(|index| self.candidates[index].css.len())
                .sum::<usize>()
            + self
                .styles
                .iter()
                .map(|style| self.state(style, selected).size(style))
                .sum::<usize>()
    }

    // Evaluate every single-bit move in one pass. Each affected style costs O(1)
    // per move instead of rescanning all declarations selected by that move.
    fn toggled_sizes(&self, selected: u64, available: u64) -> [usize; MAX_CANDIDATES] {
        let current = self.size(selected);
        let mut sizes = [current; MAX_CANDIDATES];
        for index in indices(available) {
            if selected & (1 << index) == 0 {
                sizes[index] += self.candidates[index].css.len();
            } else {
                sizes[index] -= self.candidates[index].css.len();
            }
        }
        for style in &self.styles {
            let state = self.state(style, selected);
            let before = state.size(style);
            for index in indices(available & style.candidates) {
                let adding = selected & (1 << index) == 0;
                let mut next = state;
                if adding {
                    next.count += 1;
                } else {
                    next.count -= 1;
                }
                next.toggle(&self.candidates[index], adding);
                sizes[index] = sizes[index] - before + next.size(style);
            }
        }
        sizes
    }
}

#[derive(Clone, Copy, Default)]
struct StyleState {
    count: usize,
    removed: usize,
    declarations: usize,
    classes: usize,
}

impl StyleState {
    fn toggle(&mut self, candidate: &Candidate, adding: bool) {
        if adding {
            self.removed += candidate.signatures.len();
            self.declarations += candidate.declaration_size;
            self.classes += candidate.class.len();
        } else {
            self.removed -= candidate.signatures.len();
            self.declarations -= candidate.declaration_size;
            self.classes -= candidate.class.len();
        }
    }

    fn size(self, style: &StyleCost) -> usize {
        let remaining = style.base_count - self.removed;
        let mut size = self.classes + style.conditional_bytes;
        if remaining > 0 {
            size += style.class_bytes + 3 + style.base_bytes - self.declarations + remaining - 1;
        }
        let grouped = remaining > 0 || style.conditional_bytes > 0 || self.count == 0;
        if grouped {
            size += style.class_bytes;
        }
        size + (self.count + usize::from(grouped)).saturating_sub(1)
    }
}

fn indices(mut mask: u64) -> impl Iterator<Item = usize> {
    std::iter::from_fn(move || {
        if mask == 0 {
            return None;
        }
        let index = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        Some(index)
    })
}

fn select_plan(model: &CostModel<'_>, count: usize) -> u64 {
    let all = u64::MAX >> (MAX_CANDIDATES - count);
    // Search both from grouped and from atomic. Starting from all candidates
    // can retain profitable combinations that single-step additions miss.
    let forward = improve_plan(model, 0, all);
    let backward = improve_plan(model, all, all);
    if model.size(backward) < model.size(forward) {
        backward
    } else {
        forward
    }
}

fn improve_plan(model: &CostModel<'_>, mut selected: u64, all: u64) -> u64 {
    // Each step adds or removes one candidate: at most 64 rounds per direction.
    let adding = selected == 0;
    loop {
        let mut next = selected;
        let mut best_size = model.size(selected);
        let available = if adding { all & !selected } else { selected };
        let sizes = model.toggled_sizes(selected, available);
        for index in indices(available) {
            let proposal = selected ^ (1 << index);
            let proposal_size = sizes[index];
            if proposal_size < best_size {
                next = proposal;
                best_size = proposal_size;
            }
        }
        if next == selected {
            return selected;
        }
        selected = next;
    }
}

fn size(output: &CompiledStyles) -> usize {
    let mut javascript = String::new();
    write_class_map(&mut javascript, &output.classes);
    output.css.len() + javascript.len()
}

fn emit(styles: &[ResolvedStyle<'_>], candidates: &[Candidate], selected: u64) -> CompiledStyles {
    let chosen: BTreeMap<_, _> = indices(selected)
        .flat_map(|index| {
            candidates[index]
                .signatures
                .iter()
                .map(move |signature| (signature.as_str(), &candidates[index]))
        })
        .collect();
    let mut css = String::new();
    let mut classes = BTreeMap::new();
    for style in styles {
        let mut names = Vec::new();
        let remaining: Vec<_> = style
            .rules
            .iter()
            .filter(|rule| {
                if rule.is_base()
                    && let Some(candidate) = chosen.get(rule.signature.as_str())
                {
                    if !names.contains(&candidate.class.as_str()) {
                        names.push(candidate.class.as_str());
                    }
                    false
                } else {
                    true
                }
            })
            .collect();
        if !remaining.is_empty() || names.is_empty() {
            names.insert(0, &style.class);
        }
        for group in remaining.chunk_by(|left, right| left.condition == right.condition) {
            write_rules(
                &mut css,
                &style.class,
                group[0].condition,
                group.iter().map(|rule| &rule.declaration),
            );
        }
        classes.insert(style.name.to_owned(), names.join(" "));
    }
    for index in indices(selected) {
        css.push_str(&candidates[index].css);
    }
    CompiledStyles { css, classes }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::{compile_grouped, resolve_styles};
    use alien_ir::GlobalConfig;
    use alien_parser::parse_styles;
    use std::{borrow::Cow, path::Path};

    fn fixture(count: usize) -> String {
        let entries = (0..count).map(|index| {
            let display = if index % 2 == 0 { "display:'flex'," } else { "" };
            let conditional = if index % 3 == 0 {
                "on:{hover:{color:'red'},breakpoint:{md:{height:px(10)}}},"
            } else { "" };
            format!("'été 👽 {index}':{{{display}{conditional}userSelect:important('none'),fontStyle:'normal',width:px({index})}}")
        }).collect::<Vec<_>>().join(",");
        format!(
            "import {{alien}} from 'alien';import {{px,important}} from 'alien/values';const s=alien.create({{empty:{{}},{entries}}});"
        )
    }

    #[test]
    fn cost_matches_emission_for_every_subset_including_conditions_and_escaped_names() {
        let source = fixture(12);
        let parsed = parse_styles(&source, Path::new("cost.ts")).unwrap();
        let styles: Vec<_> = parsed
            .iter()
            .map(|style| (Cow::Borrowed(style.name.as_str()), &style.rules))
            .collect();
        let config = GlobalConfig::default();
        for prefix in ["a", "a_long_production_module_namespace_"] {
            let resolved = resolve_styles(&styles, prefix, 99, &config).unwrap();
            let candidates = find_candidates(&resolved, prefix, 99);
            assert_eq!(candidates.len(), 3);
            let bundles = bundle_candidates(&resolved, &candidates);
            let atomic_model = CostModel::new(&resolved, &candidates);
            let bundle_model = CostModel::new(&resolved, &bundles);
            for mask in 0..(1 << candidates.len()) {
                let mut bundled_mask = 0;
                for (index, bundle) in bundles.iter().enumerate() {
                    if indices(mask).any(|selected| {
                        candidates[selected]
                            .signatures
                            .iter()
                            .any(|signature| bundle.signatures.contains(signature))
                    }) {
                        bundled_mask |= 1 << index;
                    }
                }
                assert!(bundle_model.size(bundled_mask) <= atomic_model.size(mask));
            }
            for plan in [&candidates, &bundles] {
                let model = CostModel::new(&resolved, plan);
                for mask in 0..(1 << plan.len()) {
                    assert_eq!(model.size(mask), size(&emit(&resolved, plan, mask)));
                    let all = (1 << plan.len()) - 1;
                    let toggled = model.toggled_sizes(mask, all);
                    for index in indices(all) {
                        assert_eq!(toggled[index], model.size(mask ^ (1 << index)));
                    }
                }
            }
            let model = CostModel::new(&resolved, &candidates);
            for mask in 0..(1 << candidates.len()) {
                assert_eq!(
                    model.size(mask),
                    size(&emit(&resolved, &candidates, mask)),
                    "mask {mask}, prefix {prefix}"
                );
            }
        }
    }

    #[test]
    fn plans_are_deterministic_and_never_exceed_the_grouped_baseline() {
        for count in [2, 10, 100, 600] {
            let source = fixture(count);
            let parsed = parse_styles(&source, Path::new("size.ts")).unwrap();
            let styles: Vec<_> = parsed
                .iter()
                .map(|style| (Cow::Borrowed(style.name.as_str()), &style.rules))
                .collect();
            let config = GlobalConfig::default();
            let resolved = resolve_styles(&styles, "a_production_prefix_", 0, &config).unwrap();
            let baseline = compile_grouped(&resolved);
            let baseline_size = size(&baseline);
            let output = optimize(&resolved, "a_production_prefix_", 0, baseline);
            assert!(size(&output) <= baseline_size);
            let repeated = optimize(
                &resolved,
                "a_production_prefix_",
                0,
                compile_grouped(&resolved),
            );
            assert_eq!(output.css, repeated.css);
            assert_eq!(output.classes, repeated.classes);
        }
    }

    #[test]
    fn backward_search_keeps_joint_savings_after_removing_an_expensive_candidate() {
        let candidates: Vec<_> = (0..3)
            .map(|index| Candidate {
                signatures: vec![String::new()],
                class: "a".repeat(10),
                css: "x".repeat(if index == 2 { 500 } else { 30 }),
                declaration_size: 10,
            })
            .collect();
        let mut styles = Vec::new();
        for _ in 0..10 {
            styles.push(StyleCost {
                candidates: 0b011,
                base_count: 2,
                base_bytes: 20,
                conditional_bytes: 0,
                class_bytes: 10,
            });
        }
        styles.push(StyleCost {
            candidates: 0b100,
            base_count: 1,
            base_bytes: 10,
            conditional_bytes: 0,
            class_bytes: 10,
        });
        let model = CostModel {
            styles,
            candidates: &candidates,
            map_overhead: 0,
        };
        assert!(model.size(1) > model.size(0));
        assert!(model.size(2) > model.size(0));
        assert!(model.size(7) > model.size(0));
        assert!(model.size(3) < model.size(0));
        assert_eq!(select_plan(&model, 3), 3);
    }

    #[test]
    fn handles_all_64_candidate_bits() {
        let candidates: Vec<_> = (0..64)
            .map(|_| Candidate {
                signatures: vec![String::new()],
                class: "a".to_owned(),
                css: "x".to_owned(),
                declaration_size: 100,
            })
            .collect();
        let model = CostModel {
            styles: vec![StyleCost {
                candidates: u64::MAX,
                base_count: 64,
                base_bytes: 6400,
                conditional_bytes: 0,
                class_bytes: 2,
            }],
            candidates: &candidates,
            map_overhead: 0,
        };
        assert_eq!(select_plan(&model, 64), u64::MAX);
    }
}

#[cfg(test)]
mod bundle_tests {
    use crate::transform_typescript;
    use std::path::Path;

    #[test]
    fn bundles_common_declarations_without_losing_distinct_widths_or_conditions() {
        let entries = (0..40).map(|index| format!("s{index}:{{color:'red',backgroundColor:'white',fontWeight:600,width:'{index}px',on:{{hover:{{height:'{index}px'}}}}}}"))
            .collect::<Vec<_>>().join(",");
        let source = format!(
            "import {{alien}} from 'alien';export const styles=alien.create({{{entries}}});"
        );
        let result = transform_typescript(&source, Path::new("bundle.ts"), "bundle.ts")
            .unwrap()
            .unwrap();
        for declaration in ["color:red", "background-color:white", "font-weight:600"] {
            assert_eq!(result.css.matches(declaration).count(), 1, "{declaration}");
        }
        for index in 0..40 {
            assert!(result.css.contains(&format!("width:{index}px")));
            assert!(result.css.contains(&format!(":hover{{height:{index}px}}")));
        }
        assert_eq!(result.css.matches("u0_").count(), 1);
    }
}
