//! ROUGE computations on interned token ids.

use rustc_hash::FxHashMap;

use crate::Score;

pub(crate) fn rouge_n(target: &[u32], prediction: &[u32], n: usize) -> Score {
    let target_counts = ngram_counts(target, n);
    let prediction_counts = ngram_counts(prediction, n);
    let hits: usize = target_counts
        .iter()
        .map(|(ngram, &count)| count.min(prediction_counts.get(ngram).copied().unwrap_or(0)))
        .sum();
    let target_total = ngram_total(target, n).max(1);
    let prediction_total = ngram_total(prediction, n).max(1);
    Score::from_counts(hits, prediction_total, target_total)
}

fn ngram_counts(tokens: &[u32], n: usize) -> FxHashMap<&[u32], usize> {
    let mut counts = FxHashMap::default();
    for ngram in tokens.windows(n) {
        *counts.entry(ngram).or_insert(0) += 1;
    }
    counts
}

fn ngram_total(tokens: &[u32], n: usize) -> usize {
    (tokens.len() + 1).saturating_sub(n)
}

pub(crate) fn rouge_l(target: &[u32], prediction: &[u32]) -> Score {
    if target.is_empty() || prediction.is_empty() {
        return Score::ZERO;
    }
    let lcs = lcs_length(target, prediction);
    Score::from_counts(lcs, prediction.len(), target.len())
}

fn lcs_length(a: &[u32], b: &[u32]) -> usize {
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

/// Summary-level LCS (section 3.2 of Lin, 2004) with the clipped hit
/// counting of ROUGE-1.5.5, as implemented by `rouge-score`.
pub(crate) fn rouge_lsum(target: &[Vec<u32>], prediction: &[Vec<u32>], vocabulary: usize) -> Score {
    let target_len: usize = target.iter().map(Vec::len).sum();
    let prediction_len: usize = prediction.iter().map(Vec::len).sum();
    if target_len == 0 || prediction_len == 0 {
        return Score::ZERO;
    }

    let mut target_counts = vec![0usize; vocabulary];
    let mut prediction_counts = vec![0usize; vocabulary];
    for &token in target.iter().flatten() {
        target_counts[token as usize] += 1;
    }
    for &token in prediction.iter().flatten() {
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
fn union_lcs(reference: &[u32], candidates: &[Vec<u32>]) -> Vec<u32> {
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
        let score = rouge_n(&[1, 1, 2, 3], &[1, 2, 2], 1);
        assert_eq!(score.precision, 2.0 / 3.0);
        assert_eq!(score.recall, 2.0 / 4.0);
    }

    #[test]
    fn handles_texts_shorter_than_n() {
        assert_eq!(rouge_n(&[1], &[1], 2), Score::ZERO);
        assert_eq!(rouge_n(&[], &[], 1), Score::ZERO);
    }

    #[test]
    fn computes_lcs_length() {
        assert_eq!(lcs_length(&[1, 2, 3, 4], &[1, 3, 4, 5]), 3);
        assert_eq!(lcs_length(&[1, 2], &[3, 4]), 0);
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
        let target = [vec![1, 2, 3, 4, 5]];
        let prediction = [vec![1, 2, 6, 7, 8], vec![1, 3, 8, 9, 5]];
        let score = rouge_lsum(&target, &prediction, 10);
        assert_eq!(score.recall, 4.0 / 5.0);
        assert_eq!(score.precision, 4.0 / 10.0);
    }
}
