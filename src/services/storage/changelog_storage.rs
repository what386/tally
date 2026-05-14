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

    pub fn remove_change(&mut self, query: &str, version: Option<&Version>) -> Option<(Version, Change)> {
        use fuzzy_matcher::skim::SkimMatcherV2;
        use fuzzy_matcher::FuzzyMatcher;

        let matcher = SkimMatcherV2::default();
        let mut best: Option<(usize, usize, i64, Version, Change)> = None;

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
                if let Some(score) = matcher.fuzzy_match(&change.description, query) {
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
