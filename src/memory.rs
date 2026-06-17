use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryInitOptions {
    pub root: PathBuf,
    pub project: String,
    pub channel: Option<String>,
    pub agent: Option<String>,
    pub date: String,
    pub force: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryStatusOptions {
    pub root: PathBuf,
    pub project: String,
    pub channel: Option<String>,
    pub agent: Option<String>,
    pub date: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryInitReport {
    pub root: PathBuf,
    pub written_files: Vec<PathBuf>,
    pub skipped_files: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryStatusReport {
    pub root: PathBuf,
    pub memory_file_exists: bool,
    pub memory_dir_exists: bool,
    pub markdown_file_count: usize,
    pub missing_paths: Vec<PathBuf>,
}

impl MemoryStatusReport {
    pub fn ready(&self) -> bool {
        self.memory_file_exists && self.memory_dir_exists && self.missing_paths.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MemoryLayout {
    root: PathBuf,
    project_slug: String,
    channel_slug: Option<String>,
    agent_slug: Option<String>,
    date_slug: String,
}

pub fn init(options: &MemoryInitOptions) -> Result<MemoryInitReport> {
    let layout = MemoryLayout::from_init_options(options)?;
    ensure_root_directory(&layout.root)?;
    for dir in layout.expected_dirs() {
        ensure_child_dir(&layout.root, &dir)?;
    }

    let mut written_files = Vec::new();
    let mut skipped_files = Vec::new();
    for (path, contents) in scaffold_files(&layout) {
        write_scaffold_file(
            &layout.root,
            path,
            contents,
            options.force,
            &mut written_files,
            &mut skipped_files,
        )?;
    }

    Ok(MemoryInitReport {
        root: layout.root,
        written_files,
        skipped_files,
    })
}

pub fn status(options: &MemoryStatusOptions) -> Result<MemoryStatusReport> {
    let layout = MemoryLayout::from_status_options(options)?;
    validate_existing_root(&layout.root)?;
    let memory_dir = layout.memory_dir();
    let mut missing_paths = Vec::new();
    for dir in layout.expected_dirs() {
        if !is_dir_without_symlink(&dir)? {
            missing_paths.push(dir);
        }
    }
    for file in layout.expected_files() {
        if !is_file_without_symlink(&file)? {
            missing_paths.push(file);
        }
    }

    Ok(MemoryStatusReport {
        root: layout.root.clone(),
        memory_file_exists: is_file_without_symlink(&layout.memory_file())?,
        memory_dir_exists: is_dir_without_symlink(&memory_dir)?,
        markdown_file_count: count_markdown_files(&memory_dir)?,
        missing_paths,
    })
}

impl MemoryInitReport {
    pub fn render(&self) -> String {
        let mut output = format!(
            "memory scaffold initialized: {}\nwritten_files={}\nskipped_files={}\n",
            self.root.display(),
            self.written_files.len(),
            self.skipped_files.len()
        );
        for path in &self.written_files {
            output.push_str(&format!("  wrote {}\n", display_relative(&self.root, path)));
        }
        for path in &self.skipped_files {
            output.push_str(&format!("  kept {}\n", display_relative(&self.root, path)));
        }
        output
    }
}

impl MemoryStatusReport {
    pub fn render(&self) -> String {
        let mut output = format!(
            "memory root: {}\nMEMORY.md: {}\nmemory/: {}\nmarkdown_files={}\n",
            self.root.display(),
            yes_no(self.memory_file_exists),
            yes_no(self.memory_dir_exists),
            self.markdown_file_count
        );
        if self.ready() {
            output.push_str("status: ready\n");
        } else {
            output.push_str("status: incomplete\nmissing:\n");
            for path in &self.missing_paths {
                output.push_str(&format!("  - {}\n", display_relative(&self.root, path)));
            }
        }
        output
    }
}

impl MemoryLayout {
    fn from_init_options(options: &MemoryInitOptions) -> Result<Self> {
        Self::build(
            options.root.clone(),
            &options.project,
            options.channel.as_deref(),
            options.agent.as_deref(),
            &options.date,
        )
    }

    fn from_status_options(options: &MemoryStatusOptions) -> Result<Self> {
        Self::build(
            options.root.clone(),
            &options.project,
            options.channel.as_deref(),
            options.agent.as_deref(),
            &options.date,
        )
    }

    fn build(
        root: PathBuf,
        project: &str,
        channel: Option<&str>,
        agent: Option<&str>,
        date: &str,
    ) -> Result<Self> {
        Ok(Self {
            root,
            project_slug: slugify(project)?,
            channel_slug: channel.map(slugify).transpose()?,
            agent_slug: agent.map(slugify).transpose()?,
            date_slug: validate_date_slug(date)?,
        })
    }

    fn memory_file(&self) -> PathBuf {
        self.root.join("MEMORY.md")
    }

    fn memory_dir(&self) -> PathBuf {
        self.root.join("memory")
    }

    fn memory_index_file(&self) -> PathBuf {
        self.memory_dir().join("README.md")
    }

    fn daily_dir(&self) -> PathBuf {
        self.memory_dir().join("daily")
    }

    fn daily_file(&self) -> PathBuf {
        self.daily_dir().join(format!("{}.md", self.date_slug))
    }

    fn projects_dir(&self) -> PathBuf {
        self.memory_dir().join("projects")
    }

    fn project_file(&self) -> PathBuf {
        self.projects_dir()
            .join(format!("{}.md", self.project_slug))
    }

    fn channels_dir(&self) -> PathBuf {
        self.memory_dir().join("channels")
    }

    fn channel_file(&self) -> Option<PathBuf> {
        self.channel_slug
            .as_ref()
            .map(|slug| self.channels_dir().join(format!("{slug}.md")))
    }

    fn agents_dir(&self) -> PathBuf {
        self.memory_dir().join("agents")
    }

    fn agent_file(&self) -> Option<PathBuf> {
        self.agent_slug
            .as_ref()
            .map(|slug| self.agents_dir().join(format!("{slug}.md")))
    }

    fn topics_dir(&self) -> PathBuf {
        self.memory_dir().join("topics")
    }

    fn rules_file(&self) -> PathBuf {
        self.topics_dir().join("rules.md")
    }

    fn lessons_file(&self) -> PathBuf {
        self.topics_dir().join("lessons.md")
    }

    fn handoffs_dir(&self) -> PathBuf {
        self.memory_dir().join("handoffs")
    }

    fn archive_dir(&self) -> PathBuf {
        self.memory_dir().join("archive")
    }

    fn expected_dirs(&self) -> Vec<PathBuf> {
        vec![
            self.memory_dir(),
            self.daily_dir(),
            self.projects_dir(),
            self.channels_dir(),
            self.agents_dir(),
            self.topics_dir(),
            self.handoffs_dir(),
            self.archive_dir(),
        ]
    }

    fn expected_files(&self) -> Vec<PathBuf> {
        let mut files = vec![
            self.memory_file(),
            self.memory_index_file(),
            self.daily_file(),
            self.project_file(),
            self.rules_file(),
            self.lessons_file(),
            self.handoffs_dir().join(".gitkeep"),
            self.archive_dir().join(".gitkeep"),
        ];
        if let Some(path) = self.channel_file() {
            files.push(path);
        }
        if let Some(path) = self.agent_file() {
            files.push(path);
        }
        files
    }
}

fn scaffold_files(layout: &MemoryLayout) -> Vec<(PathBuf, String)> {
    let mut files = vec![
        (layout.memory_file(), render_memory_md(layout)),
        (layout.memory_index_file(), render_memory_index(layout)),
        (layout.daily_file(), render_daily_file(layout)),
        (layout.project_file(), render_project_file(layout)),
        (layout.rules_file(), render_rules_file()),
        (layout.lessons_file(), render_lessons_file()),
        (layout.handoffs_dir().join(".gitkeep"), String::new()),
        (layout.archive_dir().join(".gitkeep"), String::new()),
    ];
    if let Some(path) = layout.channel_file() {
        files.push((path, render_channel_file(layout)));
    }
    if let Some(path) = layout.agent_file() {
        files.push((path, render_agent_file(layout)));
    }
    files
}

fn render_memory_md(layout: &MemoryLayout) -> String {
    let mut pointers = vec![
        format!(
            "- Project status: `memory/projects/{}.md`",
            layout.project_slug
        ),
        format!(
            "- Today's execution log: `memory/daily/{}.md`",
            layout.date_slug
        ),
        "- Durable rules: `memory/topics/rules.md`".to_string(),
        "- Durable lessons: `memory/topics/lessons.md`".to_string(),
        "- Full subtree guide: `memory/README.md`".to_string(),
    ];
    if let Some(channel) = &layout.channel_slug {
        pointers.insert(
            2,
            format!("- Channel state: `memory/channels/{channel}.md`"),
        );
    }
    if let Some(agent) = &layout.agent_slug {
        pointers.insert(3, format!("- Agent profile: `memory/agents/{agent}.md`"));
    }

    format!(
        "# MEMORY\n\nThis file is a compact pointer layer. Keep detailed state under `memory/`.\n\n## Pointers\n\n{}\n",
        pointers.join("\n")
    )
}

fn render_memory_index(layout: &MemoryLayout) -> String {
    format!(
        "# memory/README.md\n\nUse this subtree for durable Hermeship memory shards.\n\n- Daily logs: `daily/{}.md`\n- Project state: `projects/{}.md`\n- Rules: `topics/rules.md`\n- Lessons: `topics/lessons.md`\n",
        layout.date_slug, layout.project_slug
    )
}

fn render_daily_file(layout: &MemoryLayout) -> String {
    format!(
        "# {}\n\n- Scaffold created by `hermeship memory init`.\n- Project: `{}`.\n",
        layout.date_slug, layout.project_slug
    )
}

fn render_project_file(layout: &MemoryLayout) -> String {
    format!(
        "# Project: {}\n\n## Current State\n\n- Add concise project state here.\n",
        layout.project_slug
    )
}

fn render_channel_file(layout: &MemoryLayout) -> String {
    let channel = layout.channel_slug.as_deref().unwrap_or("channel");
    format!("# Channel: {channel}\n\nCanonical memory for one conversation or workflow lane.\n")
}

fn render_agent_file(layout: &MemoryLayout) -> String {
    let agent = layout.agent_slug.as_deref().unwrap_or("agent");
    format!("# Agent: {agent}\n\nPreferences and handoff expectations for this agent.\n")
}

fn render_rules_file() -> String {
    "# Rules\n\n- Keep root MEMORY.md small and pointer-oriented.\n".to_string()
}

fn render_lessons_file() -> String {
    "# Lessons\n\n- Promote durable lessons here after verification.\n".to_string()
}

fn slugify(raw: &str) -> Result<String> {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in raw.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if ch == '-' || ch == '_' || ch.is_ascii_whitespace() {
            if !last_dash && !slug.is_empty() {
                slug.push('-');
                last_dash = true;
            }
        } else {
            anyhow::bail!("slug contains unsupported character {ch:?}");
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        anyhow::bail!("slug must not be empty");
    }
    Ok(slug)
}

fn validate_date_slug(raw: &str) -> Result<String> {
    let value = raw.trim();
    let valid_shape = value.len() == 10
        && value.as_bytes()[4] == b'-'
        && value.as_bytes()[7] == b'-'
        && value
            .chars()
            .enumerate()
            .all(|(idx, ch)| idx == 4 || idx == 7 || ch.is_ascii_digit());
    if !valid_shape {
        anyhow::bail!("date must use YYYY-MM-DD");
    }
    let year = value[0..4]
        .parse::<i32>()
        .map_err(|_| anyhow::anyhow!("date must use YYYY-MM-DD"))?;
    let month = value[5..7]
        .parse::<u8>()
        .map_err(|_| anyhow::anyhow!("date must use YYYY-MM-DD"))?;
    let day = value[8..10]
        .parse::<u8>()
        .map_err(|_| anyhow::anyhow!("date must use YYYY-MM-DD"))?;
    let month =
        time::Month::try_from(month).map_err(|_| anyhow::anyhow!("date must use YYYY-MM-DD"))?;
    time::Date::from_calendar_date(year, month, day)
        .map_err(|_| anyhow::anyhow!("date must be a real calendar date"))?;
    Ok(value.to_string())
}

fn count_markdown_files(root: &Path) -> Result<usize> {
    if !is_dir_without_symlink(root)? {
        return Ok(0);
    }
    let mut count = 0;
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        for entry in fs::read_dir(&path)
            .with_context(|| format!("failed to read memory directory {}", path.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            let metadata = metadata_without_symlink(&path)?;
            if metadata.is_dir() {
                stack.push(path);
            } else if metadata.is_file()
                && path.extension().and_then(|value| value.to_str()) == Some("md")
            {
                count += 1;
            }
        }
    }
    Ok(count)
}

fn ensure_root_directory(root: &Path) -> Result<()> {
    match fs::symlink_metadata(root) {
        Ok(metadata) => {
            reject_symlink(root, &metadata)?;
            if !metadata.is_dir() {
                anyhow::bail!("memory root must be a directory: {}", root.display());
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            fs::create_dir_all(root)
                .with_context(|| format!("failed to create memory root {}", root.display()))?;
            let metadata = fs::symlink_metadata(root)
                .with_context(|| format!("failed to inspect memory root {}", root.display()))?;
            reject_symlink(root, &metadata)?;
            if !metadata.is_dir() {
                anyhow::bail!("memory root must be a directory: {}", root.display());
            }
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to inspect memory root {}", root.display()));
        }
    }
    Ok(())
}

fn validate_existing_root(root: &Path) -> Result<()> {
    match fs::symlink_metadata(root) {
        Ok(metadata) => {
            reject_symlink(root, &metadata)?;
            if !metadata.is_dir() {
                anyhow::bail!("memory root must be a directory: {}", root.display());
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to inspect memory root {}", root.display()));
        }
    }
    Ok(())
}

fn ensure_child_dir(root: &Path, dir: &Path) -> Result<()> {
    let relative = dir
        .strip_prefix(root)
        .with_context(|| format!("memory path escapes root: {}", dir.display()))?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        match component {
            Component::Normal(part) => current.push(part),
            Component::CurDir => continue,
            _ => anyhow::bail!(
                "memory path contains unsupported component: {}",
                dir.display()
            ),
        }
        match fs::symlink_metadata(&current) {
            Ok(metadata) => {
                reject_symlink(&current, &metadata)?;
                if !metadata.is_dir() {
                    anyhow::bail!("memory path must be a directory: {}", current.display());
                }
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                fs::create_dir(&current).with_context(|| {
                    format!("failed to create memory directory {}", current.display())
                })?;
                let metadata = fs::symlink_metadata(&current).with_context(|| {
                    format!("failed to inspect memory directory {}", current.display())
                })?;
                reject_symlink(&current, &metadata)?;
                if !metadata.is_dir() {
                    anyhow::bail!("memory path must be a directory: {}", current.display());
                }
            }
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("failed to inspect memory directory {}", current.display())
                });
            }
        }
    }
    Ok(())
}

fn write_scaffold_file(
    root: &Path,
    path: PathBuf,
    contents: String,
    force: bool,
    written_files: &mut Vec<PathBuf>,
    skipped_files: &mut Vec<PathBuf>,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_child_dir(root, parent)?;
    }
    match fs::symlink_metadata(&path) {
        Ok(metadata) => {
            reject_symlink(&path, &metadata)?;
            if !metadata.is_file() {
                anyhow::bail!("memory scaffold path must be a file: {}", path.display());
            }
            if !force {
                skipped_files.push(path);
                return Ok(());
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to inspect memory scaffold {}", path.display()));
        }
    }
    fs::write(&path, contents)
        .with_context(|| format!("failed to write memory scaffold {}", path.display()))?;
    let metadata = fs::symlink_metadata(&path)
        .with_context(|| format!("failed to inspect memory scaffold {}", path.display()))?;
    reject_symlink(&path, &metadata)?;
    if !metadata.is_file() {
        anyhow::bail!("memory scaffold path must be a file: {}", path.display());
    }
    written_files.push(path);
    Ok(())
}

fn is_dir_without_symlink(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            reject_symlink(path, &metadata)?;
            Ok(metadata.is_dir())
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => {
            Err(error).with_context(|| format!("failed to inspect memory path {}", path.display()))
        }
    }
}

fn is_file_without_symlink(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            reject_symlink(path, &metadata)?;
            Ok(metadata.is_file())
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => {
            Err(error).with_context(|| format!("failed to inspect memory path {}", path.display()))
        }
    }
}

fn metadata_without_symlink(path: &Path) -> Result<fs::Metadata> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("failed to inspect memory path {}", path.display()))?;
    reject_symlink(path, &metadata)?;
    Ok(metadata)
}

fn reject_symlink(path: &Path, metadata: &fs::Metadata) -> Result<()> {
    if metadata.file_type().is_symlink() {
        anyhow::bail!(
            "memory scaffold must not follow symlink: {}",
            path.display()
        );
    }
    Ok(())
}

fn display_relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::{MemoryInitOptions, MemoryStatusOptions, init, status};

    #[test]
    fn memory_init_creates_filesystem_offload_scaffold() {
        let root = temp_root("init");

        let report = init(&MemoryInitOptions {
            root: root.clone(),
            project: "Hermeship".to_string(),
            channel: Some("ops".to_string()),
            agent: Some("codex".to_string()),
            date: "2026-06-17".to_string(),
            force: false,
        })
        .unwrap();

        assert!(report.written_files.contains(&root.join("MEMORY.md")));
        assert!(root.join("MEMORY.md").is_file());
        assert!(root.join("memory/README.md").is_file());
        assert!(root.join("memory/daily/2026-06-17.md").is_file());
        assert!(root.join("memory/projects/hermeship.md").is_file());
        assert!(root.join("memory/channels/ops.md").is_file());
        assert!(root.join("memory/agents/codex.md").is_file());
        assert!(root.join("memory/topics/rules.md").is_file());
        assert!(root.join("memory/topics/lessons.md").is_file());

        let memory_md = fs::read_to_string(root.join("MEMORY.md")).unwrap();
        assert!(memory_md.contains("memory/projects/hermeship.md"));
        assert!(memory_md.contains("memory/channels/ops.md"));
        assert!(memory_md.contains("memory/agents/codex.md"));
        assert!(!memory_md.contains("token"));
        assert!(!memory_md.contains("secret"));

        remove_temp_root(&root);
    }

    #[test]
    fn memory_init_does_not_overwrite_without_force() {
        let root = temp_root("no-overwrite");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("MEMORY.md"), "custom memory").unwrap();

        let report = init(&MemoryInitOptions {
            root: root.clone(),
            project: "hermeship".to_string(),
            channel: None,
            agent: None,
            date: "2026-06-17".to_string(),
            force: false,
        })
        .unwrap();

        assert!(report.skipped_files.contains(&root.join("MEMORY.md")));
        assert_eq!(
            fs::read_to_string(root.join("MEMORY.md")).unwrap(),
            "custom memory"
        );

        remove_temp_root(&root);
    }

    #[test]
    fn memory_status_reports_missing_and_ready_paths() {
        let root = temp_root("status");
        fs::create_dir_all(root.join("memory")).unwrap();

        let missing = status(&MemoryStatusOptions {
            root: root.clone(),
            project: "hermeship".to_string(),
            channel: Some("ops".to_string()),
            agent: Some("codex".to_string()),
            date: "2026-06-17".to_string(),
        })
        .unwrap();

        assert!(missing.memory_dir_exists);
        assert!(!missing.memory_file_exists);
        assert!(missing.missing_paths.contains(&root.join("MEMORY.md")));
        assert!(!missing.ready());

        init(&MemoryInitOptions {
            root: root.clone(),
            project: "hermeship".to_string(),
            channel: Some("ops".to_string()),
            agent: Some("codex".to_string()),
            date: "2026-06-17".to_string(),
            force: false,
        })
        .unwrap();

        let ready = status(&MemoryStatusOptions {
            root: root.clone(),
            project: "hermeship".to_string(),
            channel: Some("ops".to_string()),
            agent: Some("codex".to_string()),
            date: "2026-06-17".to_string(),
        })
        .unwrap();

        assert!(ready.ready());
        assert!(ready.markdown_file_count >= 6);

        remove_temp_root(&root);
    }

    #[test]
    fn memory_scaffold_rejects_invalid_slugs_and_dates() {
        let root = temp_root("invalid");

        let bad_slug = init(&MemoryInitOptions {
            root: root.clone(),
            project: "../secret".to_string(),
            channel: None,
            agent: None,
            date: "2026-06-17".to_string(),
            force: false,
        })
        .unwrap_err()
        .to_string();
        assert!(bad_slug.contains("slug"), "{bad_slug}");

        let bad_date = init(&MemoryInitOptions {
            root: root.clone(),
            project: "hermeship".to_string(),
            channel: None,
            agent: None,
            date: "today".to_string(),
            force: false,
        })
        .unwrap_err()
        .to_string();
        assert!(bad_date.contains("date must use YYYY-MM-DD"), "{bad_date}");

        let impossible_date = init(&MemoryInitOptions {
            root: root.clone(),
            project: "hermeship".to_string(),
            channel: None,
            agent: None,
            date: "2026-02-31".to_string(),
            force: false,
        })
        .unwrap_err()
        .to_string();
        assert!(
            impossible_date.contains("date must be a real calendar date"),
            "{impossible_date}"
        );

        remove_temp_root(&root);
    }

    #[cfg(unix)]
    #[test]
    fn memory_init_rejects_symlinked_scaffold_paths_without_writing_outside_root() {
        use std::os::unix::fs::symlink;

        let root = temp_root("symlink-init");
        let outside = temp_root("symlink-outside");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        let outside_memory = outside.join("outside-memory.md");
        fs::write(&outside_memory, "outside original").unwrap();
        symlink(&outside_memory, root.join("MEMORY.md")).unwrap();

        let error = init(&MemoryInitOptions {
            root: root.clone(),
            project: "hermeship".to_string(),
            channel: None,
            agent: None,
            date: "2026-06-17".to_string(),
            force: true,
        })
        .unwrap_err()
        .to_string();

        assert!(error.contains("symlink"), "{error}");
        assert_eq!(
            fs::read_to_string(&outside_memory).unwrap(),
            "outside original"
        );

        remove_temp_root(&root);
        remove_temp_root(&outside);
    }

    #[cfg(unix)]
    #[test]
    fn memory_status_rejects_symlinked_memory_directory_without_scanning_target() {
        use std::os::unix::fs::symlink;

        let root = temp_root("symlink-status");
        let outside = temp_root("symlink-status-outside");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("secret.md"), "outside").unwrap();
        symlink(&outside, root.join("memory")).unwrap();

        let error = status(&MemoryStatusOptions {
            root: root.clone(),
            project: "hermeship".to_string(),
            channel: None,
            agent: None,
            date: "2026-06-17".to_string(),
        })
        .unwrap_err()
        .to_string();

        assert!(error.contains("symlink"), "{error}");

        remove_temp_root(&root);
        remove_temp_root(&outside);
    }

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "hermeship-memory-{label}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ))
    }

    fn remove_temp_root(root: &Path) {
        let _ = fs::remove_dir_all(root);
    }
}
