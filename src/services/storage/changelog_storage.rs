use crate::models::changes::{Change, Log, Release};
use crate::models::common::Version;
use crate::services::serializers::changelog_serializer;
use crate::utils::matching::score_passes;
use anyhow::Result;
use chrono::Utc;
use fuzzy_matcher::FuzzyMatcher;
use std::fs;
use std::path::{Path, PathBuf};

pub struct ChangelogStorage {
    changelog: Log,
    changelog_file: PathBuf,
}

impl ChangelogStorage {
    pub fn new(changelog_file: &Path, project_name: &str) -> Result<Self> {
        let mut storage = Self {
            changelog: changelog_serializer::empty_log(project_name),
            changelog_file: changelog_file.to_path_buf(),
        };
        storage.load()?;
        Ok(storage)
    }

    fn load(&mut self) -> Result<()> {
        if !self.changelog_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.changelog_file)?;
        self.changelog = changelog_serializer::from_markdown(&content)?;
        changelog_serializer::normalize(&mut self.changelog);
        Ok(())
    }

    pub fn save(&mut self) -> Result<()> {
        if let Some(parent) = self.changelog_file.parent() {
            fs::create_dir_all(parent)?;
        }
        self.changelog.generated_at = Utc::now();
        let markdown = changelog_serializer::to_markdown(&self.changelog);
        fs::write(&self.changelog_file, markdown)?;
        Ok(())
    }

    pub fn log(&self) -> &Log {
        &self.changelog
    }

    pub fn merge_changes_for_version(&mut self, version: &Version, changes: Vec<Change>) -> usize {
        let release_index = self
            .changelog
            .releases
            .iter()
            .position(|r| &r.version == version);

        let mut inserted = 0;

        if let Some(idx) = release_index {
            let release = &self.changelog.releases[idx];
            let mut existing: Vec<Change> = release
                .changes_by_priority
                .values()
                .flat_map(|v| v.iter().cloned())
                .collect();

            for change in changes {
                let duplicate = existing.iter().any(|e| {
                    if let (Some(a), Some(b)) = (&e.commit, &change.commit) {
                        a == b
                    } else {
                        e.description == change.description && e.tags == change.tags
                    }
                });
                if !duplicate {
                    existing.push(change);
                    inserted += 1;
                }
            }

            let refs: Vec<&Change> = existing.iter().collect();
            self.changelog.releases[idx] = Release::from_changes(version.clone(), Utc::now(), refs);
        } else {
            inserted = changes.len();
            let refs: Vec<&Change> = changes.iter().collect();
            self.changelog
                .releases
                .push(Release::from_changes(version.clone(), Utc::now(), refs));
        }

        self.changelog
            .releases
            .sort_by(|a, b| b.version.cmp(&a.version));
        inserted
    }

    pub fn remove_change(
        &mut self,
        query: &str,
        version: Option<&Version>,
        tag_filter: Option<&[String]>,
        min_score: f64,
    ) -> Option<(Version, Change)> {
        use fuzzy_matcher::skim::SkimMatcherV2;

        let matcher = SkimMatcherV2::default();
        let mut best: Option<(usize, usize, i64, Version, Change)> = None;
        let query_match = ReleaseQuery::from_query(query);
        let query = query_match.text.as_str();
        let version = version.or(query_match.version.as_ref());
        let version_only_query = query.is_empty() && version.is_some();

        for (ri, release) in self.changelog.releases.iter().enumerate() {
            if let Some(v) = version
                && &release.version != v
            {
                continue;
            }

            let changes: Vec<Change> = release
                .changes_by_priority
                .values()
                .flat_map(|v| v.iter().cloned())
                .collect();

            for (ci, change) in changes.iter().enumerate() {
                if let Some(tags) = tag_filter
                    && !tags.iter().any(|tag| change.tags.contains(tag))
                {
                    continue;
                }
                let maybe_score = if query.is_empty() {
                    version_only_query.then_some(i64::MAX)
                } else {
                    release_match_score(&matcher, &change.description, query)
                };
                let Some(score) = maybe_score else {
                    continue;
                };

                if !score_passes(score, min_score) {
                    continue;
                }

                {
                    let better = best.as_ref().map(|b| score > b.2).unwrap_or(true);
                    if better {
                        best = Some((ri, ci, score, release.version.clone(), change.clone()));
                    }
                }
            }
        }

        let (ri, ci, _, version, removed) = best?;

        let mut changes: Vec<Change> = self.changelog.releases[ri]
            .changes_by_priority
            .values()
            .flat_map(|v| v.iter().cloned())
            .collect();
        if ci >= changes.len() {
            return None;
        }
        changes.remove(ci);

        if changes.is_empty() {
            self.changelog.releases.remove(ri);
        } else {
            let refs: Vec<&Change> = changes.iter().collect();
            self.changelog.releases[ri] = Release::from_changes(version.clone(), Utc::now(), refs);
        }

        Some((version, removed))
    }

    pub fn remove_changes(
        &mut self,
        query: &str,
        version: Option<&Version>,
        tag_filter: Option<&[String]>,
        min_score: f64,
    ) -> Vec<(Version, Change)> {
        let query_match = ReleaseQuery::from_query(query);
        let query_text = query_match.text.as_str();
        let version = version.or(query_match.version.as_ref());

        if query_text.is_empty()
            && let Some(version) = version
        {
            return self.remove_release_changes(version, tag_filter);
        }

        self.remove_change(query, version, tag_filter, min_score)
            .into_iter()
            .collect()
    }

    fn remove_release_changes(
        &mut self,
        version: &Version,
        tag_filter: Option<&[String]>,
    ) -> Vec<(Version, Change)> {
        let Some(ri) = self
            .changelog
            .releases
            .iter()
            .position(|release| &release.version == version)
        else {
            return Vec::new();
        };

        let changes = ordered_changes(&self.changelog.releases[ri]);
        let (removed, remaining): (Vec<_>, Vec<_>) = changes
            .into_iter()
            .partition(|change| tag_matches(change, tag_filter));

        if removed.is_empty() {
            return Vec::new();
        }

        if remaining.is_empty() {
            self.changelog.releases.remove(ri);
        } else {
            let refs: Vec<&Change> = remaining.iter().collect();
            self.changelog.releases[ri] = Release::from_changes(version.clone(), Utc::now(), refs);
        }

        removed
            .into_iter()
            .map(|change| (version.clone(), change))
            .collect()
    }
}

fn ordered_changes(release: &Release) -> Vec<Change> {
    [
        crate::models::common::Priority::High,
        crate::models::common::Priority::Medium,
        crate::models::common::Priority::Low,
    ]
    .into_iter()
    .filter_map(|priority| release.changes_by_priority.get(&priority))
    .flat_map(|changes| changes.iter().cloned())
    .collect()
}

fn tag_matches(change: &Change, tag_filter: Option<&[String]>) -> bool {
    tag_filter
        .map(|tags| tags.iter().any(|tag| change.tags.contains(tag)))
        .unwrap_or(true)
}

fn release_match_score(
    matcher: &fuzzy_matcher::skim::SkimMatcherV2,
    description: &str,
    query: &str,
) -> Option<i64> {
    if description
        .to_lowercase()
        .contains(query.to_lowercase().as_str())
    {
        return Some(i64::MAX);
    }

    matcher.fuzzy_match(description, query)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReleaseQuery {
    version: Option<Version>,
    text: String,
}

impl ReleaseQuery {
    fn from_query(query: &str) -> Self {
        let mut version = None;
        let mut words = Vec::new();

        for word in query.split_whitespace() {
            if version.is_none()
                && let Some(parsed) = parse_release_tag(word)
            {
                version = Some(parsed);
                continue;
            }

            words.push(word);
        }

        Self {
            version,
            text: words.join(" "),
        }
    }
}

fn parse_release_tag(word: &str) -> Option<Version> {
    let word = word.trim_matches(|ch: char| {
        matches!(
            ch,
            ',' | '.' | ':' | ';' | '(' | ')' | '[' | ']' | '{' | '}'
        )
    });

    let has_tag_prefix = word.starts_with('v') || word.starts_with('V');
    let is_dotted_version = word.chars().filter(|ch| *ch == '.').count() >= 2;
    if !has_tag_prefix && !is_dotted_version {
        return None;
    }

    Version::parse(word).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::common::Priority;
    use chrono::{TimeZone, Utc};

    fn change(description: &str, tags: &[&str]) -> Change {
        Change {
            description: description.to_string(),
            priority: Priority::Medium,
            tags: tags.iter().map(|tag| tag.to_string()).collect(),
            commit: None,
            completed_at: Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap(),
        }
    }

    fn storage_with_releases(releases: Vec<Release>) -> ChangelogStorage {
        ChangelogStorage {
            changelog: Log {
                project_name: "test".to_string(),
                releases,
                generated_at: Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap(),
            },
            changelog_file: PathBuf::from("CHANGELOG.md"),
        }
    }

    #[test]
    fn release_query_extracts_semver_tag_and_keeps_task_text() {
        let query = ReleaseQuery::from_query("v1.2.3 fix parser");

        assert_eq!(query.version, Some(Version::new(1, 2, 3, false)));
        assert_eq!(query.text, "fix parser");
    }

    #[test]
    fn release_query_keeps_colon_prefixed_task_text() {
        let query = ReleaseQuery::from_query("feat:");

        assert_eq!(query.version, None);
        assert_eq!(query.text, "feat:");
    }

    #[test]
    fn remove_change_uses_semver_tag_as_release_filter() {
        let v1 = Version::new(1, 0, 0, false);
        let v2 = Version::new(2, 0, 0, false);
        let mut storage = storage_with_releases(vec![
            Release::from_changes(v1.clone(), Utc::now(), vec![&change("fix parser", &[])]),
            Release::from_changes(v2.clone(), Utc::now(), vec![&change("fix parser", &[])]),
        ]);

        let (version, removed) = storage
            .remove_change("v2.0.0 fix parser", None, None, 50.0)
            .expect("tagged release match");

        assert_eq!(version, v2);
        assert_eq!(removed.description, "fix parser");
        assert_eq!(storage.changelog.releases.len(), 1);
        assert_eq!(storage.changelog.releases[0].version, v1);
    }

    #[test]
    fn remove_change_matches_literal_prefix_at_default_score() {
        let version = Version::new(1, 14, 0, false);
        let mut storage = storage_with_releases(vec![Release::from_changes(
            version.clone(),
            Utc::now(),
            vec![
                &change("feat: pagination for long command outputs", &[]),
                &change("feat: yank now supports semver matching", &[]),
            ],
        )]);

        let (removed_version, removed) = storage
            .remove_change("feat:", None, None, 50.0)
            .expect("literal prefix match");

        assert_eq!(removed_version, version);
        assert_eq!(
            removed.description,
            "feat: pagination for long command outputs"
        );
        assert_eq!(storage.changelog.releases.len(), 1);
    }

    #[test]
    fn remove_change_matches_literal_prefix_after_markdown_parse() {
        let log = changelog_serializer::from_markdown(
            r#"# Changelog — tally

*Generated on 2026-06-19*

## 1.14.0 — 2026-06-19

### Changes

- feat: pagination for long command outputs
- feat: yank now supports semver matching

## 0.13.0 — 2026-05-23

### Changes

- make scan able to init tally
"#,
        )
        .expect("parsed changelog");
        let mut storage = ChangelogStorage {
            changelog: log,
            changelog_file: PathBuf::from("CHANGELOG.md"),
        };

        let (removed_version, removed) = storage
            .remove_change("feat:", None, None, 90.0)
            .expect("literal prefix match from parsed markdown");

        assert_eq!(removed_version, Version::new(1, 14, 0, false));
        assert_eq!(
            removed.description,
            "feat: pagination for long command outputs"
        );
    }

    #[test]
    fn remove_change_can_match_first_entry_by_semver_tag_only() {
        let version = Version::new(1, 2, 3, false);
        let mut storage = storage_with_releases(vec![Release::from_changes(
            version.clone(),
            Utc::now(),
            vec![
                &change("first released task", &[]),
                &change("second released task", &[]),
            ],
        )]);

        let (removed_version, removed) = storage
            .remove_change("v1.2.3", None, None, 50.0)
            .expect("first release entry");

        assert_eq!(removed_version, version);
        assert_eq!(removed.description, "first released task");
        assert_eq!(storage.changelog.releases.len(), 1);
    }

    #[test]
    fn remove_changes_with_semver_tag_removes_whole_release() {
        let version = Version::new(1, 2, 3, false);
        let mut storage = storage_with_releases(vec![Release::from_changes(
            version.clone(),
            Utc::now(),
            vec![
                &change("first released task", &[]),
                &change("second released task", &[]),
            ],
        )]);

        let removed = storage.remove_changes("v1.2.3", None, None, 50.0);

        assert_eq!(removed.len(), 2);
        assert_eq!(removed[0].0, version);
        assert_eq!(removed[0].1.description, "first released task");
        assert_eq!(removed[1].1.description, "second released task");
        assert!(storage.changelog.releases.is_empty());
    }

    #[test]
    fn remove_changes_with_semver_tag_respects_tag_filter() {
        let version = Version::new(1, 2, 3, false);
        let mut storage = storage_with_releases(vec![Release::from_changes(
            version.clone(),
            Utc::now(),
            vec![
                &change("first released task", &["feature"]),
                &change("second released task", &["bug"]),
            ],
        )]);
        let tags = vec!["feature".to_string()];

        let removed = storage.remove_changes("v1.2.3", None, Some(&tags), 50.0);

        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].0, version);
        assert_eq!(removed[0].1.description, "first released task");

        let remaining = ordered_changes(&storage.changelog.releases[0]);
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].description, "second released task");
    }

    #[test]
    fn empty_query_without_release_filter_does_not_match() {
        let version = Version::new(1, 2, 3, false);
        let mut storage = storage_with_releases(vec![Release::from_changes(
            version,
            Utc::now(),
            vec![&change("first task", &[]), &change("second task", &[])],
        )]);

        assert!(storage.remove_change("", None, None, 50.0).is_none());
        assert_eq!(storage.changelog.releases.len(), 1);
    }

    #[test]
    fn remove_change_respects_released_min_score() {
        let version = Version::new(1, 2, 3, false);
        let mut storage = storage_with_releases(vec![Release::from_changes(
            version,
            Utc::now(),
            vec![&change("parser", &[])],
        )]);

        assert!(storage.remove_change("parser", None, None, 101.0).is_none());
        assert_eq!(storage.changelog.releases.len(), 1);
    }
}
