//! ROUGE computations on interned token ids.

use rustc_hash::FxHashMap;

use crate::Score;

pub(crate) fn rouge_n(target: &[u32], prediction: &[u32], n: usize, vocabulary: usize) -> Score {
    let hits = match n {
        1 => unigram_overlap(target, prediction, vocabulary),
        2 => clipped_overlap(bigrams(target), bigrams(prediction)),
        _ => clipped_overlap(target.windows(n), prediction.windows(n)),
    };
    let target_total = ngram_total(target, n).max(1);
    let prediction_total = ngram_total(prediction, n).max(1);
    Score::from_counts(hits, prediction_total, target_total)
}

/// The sum over distinct unigrams of min(target count, prediction count),
/// with counts in a dense array indexed by token id.
fn unigram_overlap(target: &[u32], prediction: &[u32], vocabulary: usize) -> usize {
    let mut counts = vec![0u32; vocabulary];
    for &token in target {
        counts[token as usize] += 1;
    }
    let mut hits = 0;
    for &token in prediction {
        let count = &mut counts[token as usize];
        if *count > 0 {
            *count -= 1;
            hits += 1;
        }
    }
    hits
}

fn bigrams(tokens: &[u32]) -> impl Iterator<Item = u64> + '_ {
    tokens
        .windows(2)
        .map(|pair| (u64::from(pair[0]) << 32) | u64::from(pair[1]))
}

/// The sum over distinct n-grams of min(target count, prediction count).
fn clipped_overlap<K: std::hash::Hash + Eq>(
    target: impl Iterator<Item = K>,
    prediction: impl Iterator<Item = K>,
) -> usize {
    let mut counts: FxHashMap<K, u32> = FxHashMap::default();
    for ngram in target {
        *counts.entry(ngram).or_insert(0) += 1;
    }
    let mut hits = 0;
    for ngram in prediction {
        if let Some(count) = counts.get_mut(&ngram) {
            if *count > 0 {
                *count -= 1;
                hits += 1;
            }
        }
    }
    hits
}

fn ngram_total(tokens: &[u32], n: usize) -> usize {
    (tokens.len() + 1).saturating_sub(n)
}

pub(crate) fn rouge_l(target: &[u32], prediction: &[u32], vocabulary: usize) -> Score {
    if target.is_empty() || prediction.is_empty() {
        return Score::ZERO;
    }
    let lcs = lcs_length(target, prediction, vocabulary);
    Score::from_counts(lcs, prediction.len(), target.len())
}

/// LCS length with the bit-parallel algorithm of Hyyrö (2004): one bit per
/// position of the shorter sequence, 64 positions per machine word. A zero bit
/// marks a position where the LCS grew, so the length is the number of zeros.
fn lcs_length(a: &[u32], b: &[u32], vocabulary: usize) -> usize {
    let (pattern, text) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    let words = pattern.len().div_ceil(64);

    // Match masks for the tokens that occur in the pattern, one row each.
    let mut row_of = vec![u32::MAX; vocabulary];
    let mut masks: Vec<u64> = Vec::new();
    for (i, &token) in pattern.iter().enumerate() {
        let row = &mut row_of[token as usize];
        if *row == u32::MAX {
            *row = (masks.len() / words) as u32;
            masks.resize(masks.len() + words, 0);
        }
        masks[*row as usize * words + i / 64] |= 1 << (i % 64);
    }

    let mut v = vec![u64::MAX; words];
    for &token in text {
        let row = row_of[token as usize];
        if row == u32::MAX {
            continue;
        }
        let mask = &masks[row as usize * words..][..words];
        let mut carry = false;
        for (word, &m) in v.iter_mut().zip(mask) {
            let matched = *word & m;
            let (sum, overflow1) = word.overflowing_add(matched);
            let (sum, overflow2) = sum.overflowing_add(u64::from(carry));
            carry = overflow1 || overflow2;
            *word = sum | (*word & !m);
        }
    }

    // Bits above the pattern length start as ones and never match, so they
    // stay ones and add no zeros.
    v.iter().map(|word| word.count_zeros() as usize).sum()
}

/// Summary-level LCS (section 3.2 of Lin, 2004) with the clipped hit
/// counting of ROUGE-1.5.5, as implemented by `rouge-score`.
pub(crate) fn rouge_lsum(target: &[&[u32]], prediction: &[&[u32]], vocabulary: usize) -> Score {
    let target_len: usize = target.iter().map(|sentence| sentence.len()).sum();
    let prediction_len: usize = prediction.iter().map(|sentence| sentence.len()).sum();
    if target_len == 0 || prediction_len == 0 {
        return Score::ZERO;
    }

    let mut target_counts = vec![0usize; vocabulary];
    let mut prediction_counts = vec![0usize; vocabulary];
    for &token in target.iter().copied().flatten() {
        target_counts[token as usize] += 1;
    }
    for &token in prediction.iter().copied().flatten() {
        prediction_counts[token as usize] += 1;
    }

    let mut hits = 0;
    for sentence in target {
        for token in union_lcs(sentence, prediction) {
            let t = token as usize;
            if target_counts[t] > 0 && prediction_counts[t] > 0 {
                hits += 1;
                target_counts[t] -= 1;
                prediction_counts[t] -= 1;
            }
        }
    }
    Score::from_counts(hits, prediction_len, target_len)
}

/// Tokens of `reference` that belong to an LCS with at least one candidate,
/// in reference order.
fn union_lcs(reference: &[u32], candidates: &[&[u32]]) -> Vec<u32> {
    let mut in_union = vec![false; reference.len()];
    for candidate in candidates {
        for i in lcs_indices(reference, candidate) {
            in_union[i] = true;
        }
    }
    reference
        .iter()
        .zip(in_union)
        .filter_map(|(&token, keep)| keep.then_some(token))
        .collect()
}

/// Reference positions of one LCS, chosen by the same backtracking order as
/// `rouge-score` so that the summary-level union matches it exactly.
fn lcs_indices(reference: &[u32], candidate: &[u32]) -> Vec<usize> {
    let cols = candidate.len() + 1;
    let mut table = vec![0u32; (reference.len() + 1) * cols];
    for (i, &r) in reference.iter().enumerate() {
        for (j, &c) in candidate.iter().enumerate() {
            table[(i + 1) * cols + j + 1] = if r == c {
                table[i * cols + j] + 1
            } else {
                table[i * cols + j + 1].max(table[(i + 1) * cols + j])
            };
        }
    }

    let mut indices = Vec::new();
    let (mut i, mut j) = (reference.len(), candidate.len());
    while i > 0 && j > 0 {
        if reference[i - 1] == candidate[j - 1] {
            indices.push(i - 1);
            i -= 1;
            j -= 1;
        } else if table[i * cols + j - 1] > table[(i - 1) * cols + j] {
            j -= 1;
        } else {
            i -= 1;
        }
    }
    indices.reverse();
    indices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_clipped_ngram_overlap() {
        let score = rouge_n(&[1, 1, 2, 3], &[1, 2, 2], 1, 4);
        assert_eq!(score.precision, 2.0 / 3.0);
        assert_eq!(score.recall, 2.0 / 4.0);
    }

    #[test]
    fn handles_texts_shorter_than_n() {
        assert_eq!(rouge_n(&[1], &[1], 2, 2), Score::ZERO);
        assert_eq!(rouge_n(&[], &[], 1, 0), Score::ZERO);
    }

    #[test]
    fn computes_lcs_length() {
        assert_eq!(lcs_length(&[1, 2, 3, 4], &[1, 3, 4, 5], 6), 3);
        assert_eq!(lcs_length(&[1, 2], &[3, 4], 5), 0);
    }

    /// Deterministic pseudo-random token sequences for differential tests.
    fn sequences(seed: u64, count: usize) -> Vec<(Vec<u32>, Vec<u32>, usize)> {
        let mut state = seed;
        let mut next = move |bound: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % bound
        };
        (0..count)
            .map(|_| {
                let vocabulary = 1 + next(12) as usize;
                let mut tokens = |len: u64| -> Vec<u32> {
                    (0..next(len))
                        .map(|_| next(vocabulary as u64) as u32)
                        .collect()
                };
                let (a, b) = (tokens(200), tokens(200));
                (a, b, vocabulary)
            })
            .collect()
    }

    fn lcs_by_table(a: &[u32], b: &[u32]) -> usize {
        let mut previous = vec![0usize; b.len() + 1];
        let mut current = vec![0usize; b.len() + 1];
        for &x in a {
            for (j, &y) in b.iter().enumerate() {
                current[j + 1] = if x == y {
                    previous[j] + 1
                } else {
                    previous[j + 1].max(current[j])
                };
            }
            std::mem::swap(&mut previous, &mut current);
        }
        previous[b.len()]
    }

    fn overlap_by_two_maps(a: &[u32], b: &[u32], n: usize) -> usize {
        let counts = |tokens: &[u32]| {
            let mut counts = std::collections::HashMap::new();
            for ngram in tokens.windows(n) {
                *counts.entry(ngram.to_vec()).or_insert(0usize) += 1;
            }
            counts
        };
        let (ca, cb) = (counts(a), counts(b));
        ca.iter()
            .map(|(k, &v)| v.min(cb.get(k).copied().unwrap_or(0)))
            .sum()
    }

    #[test]
    fn bit_parallel_lcs_matches_the_table() {
        for (a, b, vocabulary) in sequences(0x9E37_79B9_7F4A_7C15, 2000) {
            assert_eq!(
                lcs_length(&a, &b, vocabulary),
                lcs_by_table(&a, &b),
                "{a:?} {b:?}"
            );
        }
    }

    #[test]
    fn ngram_overlap_matches_two_maps() {
        for (a, b, vocabulary) in sequences(42, 1000) {
            for n in 1..=4 {
                let hits = match n {
                    1 => unigram_overlap(&a, &b, vocabulary),
                    2 => clipped_overlap(bigrams(&a), bigrams(&b)),
                    _ => clipped_overlap(a.windows(n), b.windows(n)),
                };
                assert_eq!(hits, overlap_by_two_maps(&a, &b, n));
            }
        }
    }

    #[test]
    fn backtracks_like_rouge_score() {
        assert_eq!(lcs_indices(&[1, 2, 1], &[2, 1, 1]), [0, 2]);
        assert_eq!(lcs_indices(&[1, 2], &[2, 1]), [0]);
    }

    #[test]
    fn summary_level_lcs_from_the_paper() {
        // Lin (2004), section 3.2: r = w1 w2 w3 w4 w5, c1 = w1 w2 w6 w7 w8,
        // c2 = w1 w3 w8 w9 w5; the union LCS is w1 w2 w3 w5.
        let target: [&[u32]; 1] = [&[1, 2, 3, 4, 5]];
        let prediction: [&[u32]; 2] = [&[1, 2, 6, 7, 8], &[1, 3, 8, 9, 5]];
        let score = rouge_lsum(&target, &prediction, 10);
        assert_eq!(score.recall, 4.0 / 5.0);
        assert_eq!(score.precision, 4.0 / 10.0);
    }
}
