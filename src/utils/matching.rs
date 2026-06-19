pub fn score_percent(score: i64) -> f64 {
    (score as f64).min(100.0)
}

pub fn score_passes(score: i64, min_score: f64) -> bool {
    score_percent(score) >= min_score
}

#[cfg(test)]
mod tests {
    use super::{score_passes, score_percent};

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
}
