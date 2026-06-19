use crate::models::changes::{Change, Log, Release};
use crate::models::common::Version;
use crate::services::serializers::changelog_serializer;
use anyhow::Result;
use chrono::Utc;
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

        self.changelog.releases.sort_by(|a, b| b.version.cmp(&a.version));
        inserted
    }

    pub fn filtered_releases(&self, from: Option<&Version>, to: Option<&Version>) -> Vec<Release> {
        self.changelog
            .releases
            .iter()
            .filter(|release| {
                if let Some(from_v) = from
                    && release.version < *from_v
                {
                    return false;
                }
                if let Some(to_v) = to
                    && release.version > *to_v
                {
                    return false;
                }
                true
            })
            .cloned()
            .collect()
    }

    pub fn remove_change(
        &mut self,
        query: &str,
        version: Option<&Version>,
        tag_filter: Option<&[String]>,
    ) -> Option<(Version, Change)> {
        use fuzzy_matcher::skim::SkimMatcherV2;
        use fuzzy_matcher::FuzzyMatcher;

        let matcher = SkimMatcherV2::default();
        let mut best: Option<(usize, usize, i64, Version, Change)> = None;
        let query_match = ReleaseQuery::from_query(query);
        let query = query_match.text.as_str();
        let version = version.or(query_match.version.as_ref());

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
                let score = if query.is_empty() {
                    exact_release_query_score(&changes, tag_filter)?
                } else {
                    matcher.fuzzy_match(&change.description, query)?
                };

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
}

fn exact_release_query_score(changes: &[Change], tag_filter: Option<&[String]>) -> Option<i64> {
    let matching_count = changes
        .iter()
        .filter(|change| {
            tag_filter
                .map(|tags| tags.iter().any(|tag| change.tags.contains(tag)))
                .unwrap_or(true)
        })
        .take(2)
        .count();

    (matching_count == 1).then_some(i64::MAX)
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
    fn remove_change_uses_semver_tag_as_release_filter() {
        let v1 = Version::new(1, 0, 0, false);
        let v2 = Version::new(2, 0, 0, false);
        let mut storage = storage_with_releases(vec![
            Release::from_changes(v1.clone(), Utc::now(), vec![&change("fix parser", &[])]),
            Release::from_changes(v2.clone(), Utc::now(), vec![&change("fix parser", &[])]),
        ]);

        let (version, removed) = storage
            .remove_change("v2.0.0 fix parser", None, None)
            .expect("tagged release match");

        assert_eq!(version, v2);
        assert_eq!(removed.description, "fix parser");
        assert_eq!(storage.changelog.releases.len(), 1);
        assert_eq!(storage.changelog.releases[0].version, v1);
    }

    #[test]
    fn remove_change_can_match_single_entry_by_semver_tag_only() {
        let version = Version::new(1, 2, 3, false);
        let mut storage = storage_with_releases(vec![Release::from_changes(
            version.clone(),
            Utc::now(),
            vec![&change("single released task", &[])],
        )]);

        let (removed_version, removed) = storage
            .remove_change("v1.2.3", None, None)
            .expect("single release entry");

        assert_eq!(removed_version, version);
        assert_eq!(removed.description, "single released task");
        assert!(storage.changelog.releases.is_empty());
    }

    #[test]
    fn semver_tag_only_does_not_pick_from_multi_entry_release() {
        let version = Version::new(1, 2, 3, false);
        let mut storage = storage_with_releases(vec![Release::from_changes(
            version,
            Utc::now(),
            vec![&change("first task", &[]), &change("second task", &[])],
        )]);

        assert!(storage.remove_change("v1.2.3", None, None).is_none());
        assert_eq!(storage.changelog.releases.len(), 1);
    }
}
