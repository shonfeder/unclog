use crate::changelog::fs_utils::{read_and_filter_dir, read_to_string_opt};
use crate::changelog::parsing_utils::trim_newlines;
use crate::{ChangeSetSection, Config, Error, Result};
use log::debug;
use std::fs;
use std::path::{Path, PathBuf};

/// A set of changes, either associated with a release or not.
#[derive(Debug, Clone)]
pub struct ChangeSet {
    /// An optional high-level summary of the set of changes.
    pub maybe_summary: Option<String>,
    /// The sections making up the change set.
    pub sections: Vec<ChangeSetSection>,
}

impl ChangeSet {
    /// Returns true if this change set has no summary and no entries
    /// associated with it.
    pub fn is_empty(&self) -> bool {
        self.maybe_summary.as_ref().map_or(true, String::is_empty) && self.are_sections_empty()
    }

    /// Returns whether or not all the sections are empty.
    pub fn are_sections_empty(&self) -> bool {
        self.sections.iter().all(ChangeSetSection::is_empty)
    }

    /// Attempt to read a single change set from the given directory.
    pub fn read_from_dir<P>(config: &Config, path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        debug!("Loading change set from {}", path.display());
        let summary = read_to_string_opt(path.join(&config.change_sets.summary_filename))?
            .map(|s| trim_newlines(&s).to_owned());
        let section_dirs = read_and_filter_dir(path, change_set_section_filter)?;
        let mut sections = section_dirs
            .into_iter()
            .map(|path| ChangeSetSection::read_from_dir(config, path))
            .collect::<Result<Vec<ChangeSetSection>>>()?;
        // Sort sections alphabetically
        sections.sort_by(|a, b| a.title.cmp(&b.title));
        Ok(Self {
            maybe_summary: summary,
            sections,
        })
    }

    /// Attempt to read a single change set from the given directory, like
    /// [`ChangeSet::read_from_dir`], but return `Option::None` if the
    /// directory does not exist.
    pub fn read_from_dir_opt<P>(config: &Config, path: P) -> Result<Option<Self>>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        // The path doesn't exist
        if fs::metadata(path).is_err() {
            return Ok(None);
        }
        Self::read_from_dir(config, path).map(Some)
    }

    pub fn render(&self, config: &Config) -> String {
        let mut paragraphs = Vec::new();
        if let Some(summary) = self.maybe_summary.as_ref() {
            paragraphs.push(summary.clone());
        }
        self.sections
            .iter()
            .filter(|s| !s.is_empty())
            .for_each(|s| paragraphs.push(s.render(config)));
        paragraphs.join("\n\n")
    }
}

fn change_set_section_filter(e: fs::DirEntry) -> Option<Result<PathBuf>> {
    let meta = match e.metadata() {
        Ok(m) => m,
        Err(e) => return Some(Err(Error::Io(e))),
    };
    if meta.is_dir() {
        Some(Ok(e.path()))
    } else {
        None
    }
}
