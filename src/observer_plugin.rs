use std::fs;
use std::io::{ErrorKind, Write};
use std::path::Component;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub const OBSERVER_PLUGIN_NAME: &str = "hermeship-observer";
const MANAGED_MARKER_FILE: &str = ".hermeship-managed.json";
pub const OBSERVER_PLUGIN_MANIFEST_TEMPLATE: &str =
    include_str!("../templates/hermes-plugin/plugin.yaml");
pub const OBSERVER_PLUGIN_INIT_TEMPLATE: &str =
    include_str!("../templates/hermes-plugin/__init__.py");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObserverPluginInstallOptions {
    pub hermes_home: PathBuf,
    pub force: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObserverPluginInstallReport {
    pub plugin_dir: PathBuf,
    pub planned_files: Vec<PathBuf>,
    pub written_files: Vec<PathBuf>,
    pub skipped_files: Vec<PathBuf>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ManagedPluginMarker {
    version: u32,
    files: Vec<ManagedPluginFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ManagedPluginFile {
    path: String,
    checksum: String,
}

pub fn install_observer_plugin(
    options: &ObserverPluginInstallOptions,
) -> Result<ObserverPluginInstallReport> {
    let plugin_dir = observer_plugin_dir(&options.hermes_home);
    let files = managed_plugin_files(&plugin_dir);
    let planned_files = files
        .iter()
        .map(|(path, _, _)| path.clone())
        .chain(std::iter::once(plugin_dir.join(MANAGED_MARKER_FILE)))
        .collect::<Vec<_>>();
    let mut report = ObserverPluginInstallReport {
        plugin_dir,
        planned_files,
        written_files: Vec::new(),
        skipped_files: Vec::new(),
        dry_run: options.dry_run,
    };

    if options.dry_run {
        return Ok(report);
    }

    ensure_observer_plugin_directory(&options.hermes_home, &report.plugin_dir)?;
    let marker_path = report.plugin_dir.join(MANAGED_MARKER_FILE);
    validate_install_targets(&files, &marker_path)?;

    let mut managed_entries = Vec::new();
    for (path, content, marker) in files {
        if safe_file_exists(&path)? && !options.force {
            report.skipped_files.push(path.clone());
            if read_regular_file(&path).ok().as_deref() == Some(content.as_str()) {
                managed_entries.push(marker);
            }
            continue;
        }

        write_regular_file(&path, &content).with_context(|| {
            format!(
                "failed to write Hermes observer plugin file {}",
                path.display()
            )
        })?;
        managed_entries.push(marker);
        report.written_files.push(path);
    }

    if !managed_entries.is_empty() || safe_file_exists(&marker_path)? {
        write_marker(&marker_path, managed_entries)?;
        report.written_files.push(marker_path);
    }

    Ok(report)
}

pub fn render_enable_instructions(hermes_home: &Path, dry_run: bool) -> String {
    let plugin_dir = observer_plugin_dir(hermes_home);
    let prefix = if dry_run {
        "hermes observer plugin enable dry-run"
    } else {
        "hermes observer plugin enable instructions"
    };
    let mut output = format!("{prefix}: {}\n", plugin_dir.display());
    output.push_str("Hermeship does not modify Hermes config or run Hermes automatically.\n");
    output.push_str("After installing the plugin template, enable it with:\n");
    output.push_str("  hermes plugins enable hermeship-observer\n");
    output
}

fn observer_plugin_dir(hermes_home: &Path) -> PathBuf {
    hermes_home.join("plugins").join(OBSERVER_PLUGIN_NAME)
}

fn managed_plugin_files(plugin_dir: &Path) -> Vec<(PathBuf, String, ManagedPluginFile)> {
    vec![
        (
            plugin_dir.join("plugin.yaml"),
            OBSERVER_PLUGIN_MANIFEST_TEMPLATE.to_string(),
            ManagedPluginFile {
                path: "plugin.yaml".to_string(),
                checksum: checksum(OBSERVER_PLUGIN_MANIFEST_TEMPLATE),
            },
        ),
        (
            plugin_dir.join("__init__.py"),
            OBSERVER_PLUGIN_INIT_TEMPLATE.to_string(),
            ManagedPluginFile {
                path: "__init__.py".to_string(),
                checksum: checksum(OBSERVER_PLUGIN_INIT_TEMPLATE),
            },
        ),
    ]
}

fn checksum(content: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in content.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3_u64);
    }
    format!("{hash:016x}")
}

fn ensure_observer_plugin_directory(hermes_home: &Path, plugin_dir: &Path) -> Result<()> {
    ensure_base_directory(hermes_home, "Hermes home")?;
    ensure_child_directory(hermes_home, plugin_dir, "Hermes observer plugin directory")
}

fn ensure_base_directory(path: &Path, label: &str) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            reject_symlink(path, &metadata, label)?;
            if !metadata.is_dir() {
                anyhow::bail!("{label} must be a directory: {}", path.display());
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            fs::create_dir_all(path)
                .with_context(|| format!("failed to create {label} {}", path.display()))?;
            let metadata = fs::symlink_metadata(path)
                .with_context(|| format!("failed to inspect {label} {}", path.display()))?;
            reject_symlink(path, &metadata, label)?;
            if !metadata.is_dir() {
                anyhow::bail!("{label} must be a directory: {}", path.display());
            }
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to inspect {label} {}", path.display()));
        }
    }
    Ok(())
}

fn ensure_child_directory(root: &Path, dir: &Path, label: &str) -> Result<()> {
    let relative = dir
        .strip_prefix(root)
        .with_context(|| format!("{label} escapes Hermes home: {}", dir.display()))?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        match component {
            Component::Normal(part) => current.push(part),
            Component::CurDir => continue,
            _ => anyhow::bail!(
                "{label} contains unsupported path component: {}",
                dir.display()
            ),
        }
        match fs::symlink_metadata(&current) {
            Ok(metadata) => {
                reject_symlink(&current, &metadata, label)?;
                if !metadata.is_dir() {
                    anyhow::bail!("{label} must be a directory: {}", current.display());
                }
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                fs::create_dir(&current)
                    .with_context(|| format!("failed to create {label} {}", current.display()))?;
                let metadata = fs::symlink_metadata(&current)
                    .with_context(|| format!("failed to inspect {label} {}", current.display()))?;
                reject_symlink(&current, &metadata, label)?;
                if !metadata.is_dir() {
                    anyhow::bail!("{label} must be a directory: {}", current.display());
                }
            }
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("failed to inspect {label} {}", current.display()));
            }
        }
    }
    Ok(())
}

fn validate_install_targets(
    files: &[(PathBuf, String, ManagedPluginFile)],
    marker_path: &Path,
) -> Result<()> {
    for (path, _, _) in files {
        validate_regular_file_target(path, "Hermes observer plugin file")?;
    }
    validate_regular_file_target(marker_path, "Hermes observer plugin marker")
}

fn safe_file_exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            reject_symlink(path, &metadata, "Hermes observer plugin file")?;
            if !metadata.is_file() {
                anyhow::bail!(
                    "Hermes observer plugin file must be a regular file: {}",
                    path.display()
                );
            }
            Ok(true)
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| {
            format!(
                "failed to inspect Hermes observer plugin file {}",
                path.display()
            )
        }),
    }
}

fn validate_regular_file_target(path: &Path, label: &str) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            reject_symlink(path, &metadata, label)?;
            if !metadata.is_file() {
                anyhow::bail!("{label} must be a regular file: {}", path.display());
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to inspect {label} {}", path.display()));
        }
    }
    Ok(())
}

fn read_regular_file(path: &Path) -> Result<String> {
    validate_regular_file_target(path, "Hermes observer plugin file")?;
    fs::read_to_string(path).with_context(|| {
        format!(
            "failed to read Hermes observer plugin file {}",
            path.display()
        )
    })
}

fn write_regular_file(path: &Path, content: &str) -> Result<()> {
    validate_regular_file_target(path, "Hermes observer plugin file")?;
    let temp_path = unique_temp_path(path)?;
    let write_result = (|| -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .with_context(|| {
                format!(
                    "failed to create temporary Hermes observer plugin file {}",
                    temp_path.display()
                )
            })?;
        file.write_all(content.as_bytes()).with_context(|| {
            format!(
                "failed to write temporary Hermes observer plugin file {}",
                temp_path.display()
            )
        })?;
        file.sync_all().with_context(|| {
            format!(
                "failed to sync temporary Hermes observer plugin file {}",
                temp_path.display()
            )
        })?;
        Ok(())
    })();
    if let Err(error) = write_result {
        let _ = fs::remove_file(&temp_path);
        return Err(error);
    }

    if safe_file_exists(path)? {
        fs::remove_file(path).with_context(|| {
            format!(
                "failed to replace Hermes observer plugin file {}",
                path.display()
            )
        })?;
    }
    fs::rename(&temp_path, path).with_context(|| {
        let _ = fs::remove_file(&temp_path);
        format!(
            "failed to install Hermes observer plugin file {}",
            path.display()
        )
    })?;
    validate_regular_file_target(path, "Hermes observer plugin file")
}

fn write_marker(path: &Path, files: Vec<ManagedPluginFile>) -> Result<()> {
    let marker = ManagedPluginMarker { version: 1, files };
    let marker_json = serde_json::to_string_pretty(&marker)
        .context("failed to serialize Hermes observer plugin marker")?;
    write_regular_file(path, &marker_json).with_context(|| {
        format!(
            "failed to write Hermes observer plugin marker {}",
            path.display()
        )
    })
}

fn unique_temp_path(path: &Path) -> Result<PathBuf> {
    let parent = path.parent().with_context(|| {
        format!(
            "Hermes observer plugin file has no parent directory: {}",
            path.display()
        )
    })?;
    let file_name = path.file_name().with_context(|| {
        format!(
            "Hermes observer plugin file has no file name: {}",
            path.display()
        )
    })?;
    Ok(parent.join(format!(
        ".{}.{}.tmp",
        file_name.to_string_lossy(),
        uuid::Uuid::new_v4()
    )))
}

fn reject_symlink(path: &Path, metadata: &fs::Metadata, label: &str) -> Result<()> {
    if metadata.file_type().is_symlink() {
        anyhow::bail!("{label} must not follow symlink: {}", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::{
        ManagedPluginMarker, OBSERVER_PLUGIN_INIT_TEMPLATE, OBSERVER_PLUGIN_MANIFEST_TEMPLATE,
        ObserverPluginInstallOptions, install_observer_plugin, render_enable_instructions,
    };

    #[test]
    fn install_dry_run_reports_plugin_paths_without_writing() {
        let home = temp_dir("install-dry-run");

        let report = install_observer_plugin(&ObserverPluginInstallOptions {
            hermes_home: home.clone(),
            force: false,
            dry_run: true,
        })
        .unwrap();

        assert!(report.dry_run);
        assert_eq!(report.plugin_dir, home.join("plugins/hermeship-observer"));
        assert_eq!(
            report.planned_files,
            vec![
                home.join("plugins/hermeship-observer/plugin.yaml"),
                home.join("plugins/hermeship-observer/__init__.py"),
                home.join("plugins/hermeship-observer/.hermeship-managed.json"),
            ]
        );
        assert!(report.written_files.is_empty());
        assert!(!home.join("plugins/hermeship-observer/plugin.yaml").exists());

        remove_temp_dir(&home);
    }

    #[test]
    fn install_writes_observer_plugin_template_to_fake_hermes_home() {
        let home = temp_dir("install-writes");

        let report = install_observer_plugin(&ObserverPluginInstallOptions {
            hermes_home: home.clone(),
            force: false,
            dry_run: false,
        })
        .unwrap();

        let plugin_dir = home.join("plugins/hermeship-observer");
        let manifest = plugin_dir.join("plugin.yaml");
        let init = plugin_dir.join("__init__.py");
        let marker = plugin_dir.join(".hermeship-managed.json");

        assert_eq!(report.plugin_dir, plugin_dir);
        assert!(report.written_files.contains(&manifest));
        assert!(report.written_files.contains(&init));
        assert!(report.written_files.contains(&marker));
        assert_eq!(
            fs::read_to_string(&manifest).unwrap(),
            OBSERVER_PLUGIN_MANIFEST_TEMPLATE
        );
        assert_eq!(
            fs::read_to_string(&init).unwrap(),
            OBSERVER_PLUGIN_INIT_TEMPLATE
        );
        assert_marker_entries(
            &marker,
            vec![
                (
                    "plugin.yaml",
                    super::checksum(OBSERVER_PLUGIN_MANIFEST_TEMPLATE),
                ),
                (
                    "__init__.py",
                    super::checksum(OBSERVER_PLUGIN_INIT_TEMPLATE),
                ),
            ],
        );

        remove_temp_dir(&home);
    }

    #[test]
    fn install_marker_excludes_skipped_local_modifications() {
        let home = temp_dir("install-marker-skipped");
        let plugin_dir = home.join("plugins/hermeship-observer");
        fs::create_dir_all(&plugin_dir).unwrap();
        let init = plugin_dir.join("__init__.py");
        fs::write(&init, "# local observer plugin\n").unwrap();

        let report = install_observer_plugin(&ObserverPluginInstallOptions {
            hermes_home: home.clone(),
            force: false,
            dry_run: false,
        })
        .unwrap();

        let manifest = plugin_dir.join("plugin.yaml");
        let marker = plugin_dir.join(".hermeship-managed.json");
        assert!(report.written_files.contains(&manifest));
        assert!(report.written_files.contains(&marker));
        assert!(report.skipped_files.contains(&init));
        assert_marker_entries(
            &marker,
            vec![(
                "plugin.yaml",
                super::checksum(OBSERVER_PLUGIN_MANIFEST_TEMPLATE),
            )],
        );

        remove_temp_dir(&home);
    }

    #[test]
    fn install_preserves_existing_files_without_force_and_overwrites_with_force() {
        let home = temp_dir("install-force");
        let plugin_dir = home.join("plugins/hermeship-observer");
        fs::create_dir_all(&plugin_dir).unwrap();
        let init = plugin_dir.join("__init__.py");
        fs::write(&init, "# local observer plugin\n").unwrap();

        let skipped = install_observer_plugin(&ObserverPluginInstallOptions {
            hermes_home: home.clone(),
            force: false,
            dry_run: false,
        })
        .unwrap();

        assert_eq!(
            fs::read_to_string(&init).unwrap(),
            "# local observer plugin\n"
        );
        assert!(skipped.skipped_files.contains(&init));

        let overwritten = install_observer_plugin(&ObserverPluginInstallOptions {
            hermes_home: home.clone(),
            force: true,
            dry_run: false,
        })
        .unwrap();

        assert_eq!(
            fs::read_to_string(&init).unwrap(),
            OBSERVER_PLUGIN_INIT_TEMPLATE
        );
        assert!(overwritten.written_files.contains(&init));

        remove_temp_dir(&home);
    }

    #[test]
    fn install_clears_stale_marker_when_all_files_are_local_modifications() {
        let home = temp_dir("install-clears-stale-marker");
        install_observer_plugin(&ObserverPluginInstallOptions {
            hermes_home: home.clone(),
            force: false,
            dry_run: false,
        })
        .unwrap();
        let plugin_dir = home.join("plugins/hermeship-observer");
        let manifest = plugin_dir.join("plugin.yaml");
        let init = plugin_dir.join("__init__.py");
        let marker = plugin_dir.join(".hermeship-managed.json");
        fs::write(&manifest, "name: local-observer\n").unwrap();
        fs::write(&init, "# local observer plugin\n").unwrap();

        let report = install_observer_plugin(&ObserverPluginInstallOptions {
            hermes_home: home.clone(),
            force: false,
            dry_run: false,
        })
        .unwrap();

        assert!(report.skipped_files.contains(&manifest));
        assert!(report.skipped_files.contains(&init));
        assert!(report.written_files.contains(&marker));
        assert_marker_entries(&marker, Vec::new());

        remove_temp_dir(&home);
    }

    #[cfg(unix)]
    #[test]
    fn install_rejects_symlinked_plugin_directory_without_writing_target() {
        use std::os::unix::fs::symlink;

        let home = temp_dir("install-symlink-dir");
        let outside = temp_dir("install-symlink-dir-outside");
        fs::create_dir_all(home.join("plugins")).unwrap();
        fs::write(outside.join("plugin.yaml"), "outside original").unwrap();
        symlink(&outside, home.join("plugins/hermeship-observer")).unwrap();

        let error = install_observer_plugin(&ObserverPluginInstallOptions {
            hermes_home: home.clone(),
            force: true,
            dry_run: false,
        })
        .unwrap_err()
        .to_string();

        assert!(error.contains("symlink"), "{error}");
        assert_eq!(
            fs::read_to_string(outside.join("plugin.yaml")).unwrap(),
            "outside original"
        );

        remove_temp_dir(&home);
        remove_temp_dir(&outside);
    }

    #[cfg(unix)]
    #[test]
    fn install_rejects_symlinked_template_file_without_writing_target() {
        use std::os::unix::fs::symlink;

        let home = temp_dir("install-symlink-file");
        let outside = temp_dir("install-symlink-file-outside");
        let plugin_dir = home.join("plugins/hermeship-observer");
        fs::create_dir_all(&plugin_dir).unwrap();
        let outside_manifest = outside.join("outside-plugin.yaml");
        fs::write(&outside_manifest, "outside original").unwrap();
        symlink(&outside_manifest, plugin_dir.join("plugin.yaml")).unwrap();

        let error = install_observer_plugin(&ObserverPluginInstallOptions {
            hermes_home: home.clone(),
            force: true,
            dry_run: false,
        })
        .unwrap_err()
        .to_string();

        assert!(error.contains("symlink"), "{error}");
        assert_eq!(
            fs::read_to_string(&outside_manifest).unwrap(),
            "outside original"
        );

        remove_temp_dir(&home);
        remove_temp_dir(&outside);
    }

    #[cfg(unix)]
    #[test]
    fn install_rejects_symlinked_marker_without_partial_template_writes() {
        use std::os::unix::fs::symlink;

        let home = temp_dir("install-symlink-marker");
        let outside = temp_dir("install-symlink-marker-outside");
        let plugin_dir = home.join("plugins/hermeship-observer");
        fs::create_dir_all(&plugin_dir).unwrap();
        let outside_marker = outside.join("outside-marker.json");
        fs::write(&outside_marker, "outside original").unwrap();
        symlink(&outside_marker, plugin_dir.join(".hermeship-managed.json")).unwrap();

        let error = install_observer_plugin(&ObserverPluginInstallOptions {
            hermes_home: home.clone(),
            force: true,
            dry_run: false,
        })
        .unwrap_err()
        .to_string();

        assert!(error.contains("symlink"), "{error}");
        assert_eq!(
            fs::read_to_string(&outside_marker).unwrap(),
            "outside original"
        );
        assert!(!plugin_dir.join("plugin.yaml").exists());
        assert!(!plugin_dir.join("__init__.py").exists());

        remove_temp_dir(&home);
        remove_temp_dir(&outside);
    }

    #[test]
    fn enable_instructions_do_not_run_hermes_or_modify_config() {
        let home = temp_dir("enable-instructions");
        let output = render_enable_instructions(&home, true);

        assert!(output.contains("hermes plugins enable hermeship-observer"));
        assert!(output.contains("dry-run"));
        assert!(
            output.contains(
                &home
                    .join("plugins/hermeship-observer")
                    .display()
                    .to_string()
            )
        );
        assert!(!home.join("config.yaml").exists());

        remove_temp_dir(&home);
    }

    fn assert_marker_entries(path: &Path, expected: Vec<(&str, String)>) {
        let marker: ManagedPluginMarker =
            serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        let actual = marker
            .files
            .into_iter()
            .map(|entry| (entry.path, entry.checksum))
            .collect::<Vec<_>>();
        let expected = expected
            .into_iter()
            .map(|(path, checksum)| (path.to_string(), checksum))
            .collect::<Vec<_>>();

        assert_eq!(marker.version, 1);
        assert_eq!(actual, expected);
    }

    fn temp_dir(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "hermeship-observer-plugin-{name}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn remove_temp_dir(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }
}
