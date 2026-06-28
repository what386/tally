use anyhow::{Result, anyhow};

const AMBIGUOUS_SCORE_DELTA: f64 = 5.0;

pub fn score_percent(score: i64) -> f64 {
    (score as f64).min(100.0)
}

pub fn score_passes(score: i64, min_score: f64) -> bool {
    score_percent(score) >= min_score
}

#[derive(Debug, Clone)]
pub struct MatchCandidate<T> {
    pub value: T,
    pub score: i64,
    pub label: String,
    pub exact: bool,
}

pub fn select_unambiguous<T>(
    mut candidates: Vec<MatchCandidate<T>>,
    min_score: f64,
    target: &str,
) -> Result<Option<MatchCandidate<T>>> {
    candidates.retain(|candidate| score_passes(candidate.score, min_score));
    if candidates.is_empty() {
        return Ok(None);
    }

    candidates.sort_by(|a, b| {
        b.exact
            .cmp(&a.exact)
            .then_with(|| b.score.cmp(&a.score))
            .then_with(|| a.label.cmp(&b.label))
    });

    let best = candidates.remove(0);
    if let Some(second) = candidates.first()
        && ambiguous(&best, second)
    {
        let mut labels = vec![best.label.clone(), second.label.clone()];
        labels.extend(
            candidates
                .iter()
                .take(3)
                .map(|candidate| candidate.label.clone()),
        );
        labels.sort();
        labels.dedup();

        return Err(anyhow!(
            "Ambiguous match for '{}'. Narrow the query or use tags. Candidates:\n{}",
            target,
            labels
                .into_iter()
                .map(|label| format!("- {label}"))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }

    Ok(Some(best))
}

fn ambiguous<T>(best: &MatchCandidate<T>, second: &MatchCandidate<T>) -> bool {
    if best.exact || second.exact {
        return best.exact == second.exact;
    }

    score_percent(second.score) >= score_percent(best.score) - AMBIGUOUS_SCORE_DELTA
}

#[cfg(test)]
mod tests {
    use super::{MatchCandidate, score_passes, score_percent, select_unambiguous};

    #[test]
    fn score_percent_caps_raw_matcher_scores() {
        assert_eq!(score_percent(40), 40.0);
        assert_eq!(score_percent(150), 100.0);
    }

    #[test]
    fn score_passes_compares_percent_score() {
        assert!(score_passes(75, 50.0));
        assert!(!score_passes(25, 50.0));
    }

    #[test]
    fn select_unambiguous_rejects_close_matches() {
        let err = select_unambiguous(
            vec![
                MatchCandidate {
                    value: 1,
                    score: 80,
                    label: "first".to_string(),
                    exact: false,
                },
                MatchCandidate {
                    value: 2,
                    score: 77,
                    label: "second".to_string(),
                    exact: false,
                },
            ],
            50.0,
            "query",
        )
        .unwrap_err();

        assert!(err.to_string().contains("Ambiguous match"));
    }

    #[test]
    fn select_unambiguous_allows_clear_winner() {
        let selected = select_unambiguous(
            vec![
                MatchCandidate {
                    value: 1,
                    score: 80,
                    label: "first".to_string(),
                    exact: false,
                },
                MatchCandidate {
                    value: 2,
                    score: 70,
                    label: "second".to_string(),
                    exact: false,
                },
            ],
            50.0,
            "query",
        )
        .unwrap()
        .unwrap();

        assert_eq!(selected.value, 1);
    }
}
