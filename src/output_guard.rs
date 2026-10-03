//! Preflight contract outputs before any output mutation. Legacy output is unchanged.
use crate::{
    file_utils::{is_url, translate_to_output_path},
    gsn::Module,
    render::RenderOptions,
};
use anyhow::{Context, Result, bail};
use clap::{ArgMatches, parser::ValueSource};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

// Resolve existing ancestors, including symlinks, before appending missing names.
// Do not lexically remove `..` across a symlink, or hide a non-directory ancestor.
fn resolve(path: &Path) -> Result<PathBuf> {
    match path.canonicalize() {
        Ok(path) => Ok(path),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // A dangling symlink is an uncertain destination, not a missing name.
            if path.symlink_metadata().is_ok() {
                bail!("Unresolved symlink: {}", path.display());
            }
            let name = path
                .file_name()
                .with_context(|| format!("Invalid file path: {}", path.display()))?;
            let parent = path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            let parent = resolve(parent)?;
            if parent.exists() && !parent.is_dir() {
                bail!("Output parent is not a directory: {}", parent.display());
            }
            Ok(parent.join(name))
        }
        Err(e) => Err(e).with_context(|| format!("Cannot resolve path {}", path.display())),
    }
}

fn aliases(a: &Path, b: &Path) -> Result<bool> {
    if a == b {
        return Ok(true);
    }
    if a.exists() && b.exists() {
        return same_file::is_same_file(a, b).with_context(|| {
            format!(
                "Cannot compare file identities: {} and {}",
                a.display(),
                b.display()
            )
        });
    }
    Ok(false)
}

pub fn index_temporary_path(path: &Path) -> Result<PathBuf> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = path
        .file_name()
        .context("Index needs a file name")?
        .to_string_lossy();
    Ok(parent.join(format!(".{name}.gsn2x-{}.tmp", std::process::id())))
}

pub fn preflight(
    matches: &ArgMatches,
    modules: &BTreeMap<String, Module>,
    mut protected: Vec<PathBuf>,
    styles: &[String],
) -> Result<()> {
    let out = matches.get_one::<String>("OUTPUT_DIRECTORY").unwrap();
    let rendering = !matches.get_flag("CHECK_ONLY");
    let embed = matches.get_flag("EMBED_CSS");
    let options = RenderOptions::new(matches, styles.to_vec(), embed, out);
    let mut destinations = Vec::new();
    for module in modules.values() {
        let input = module.canonical_path.as_ref().unwrap();
        protected.push(input.clone());
        protected.extend(
            module
                .meta
                .uses
                .iter()
                .map(|p| input.parent().unwrap().join(p)),
        );
    }
    if rendering {
        let directory = resolve(Path::new(out))?;
        if directory.exists() && !directory.is_dir() {
            bail!("Output directory is not a directory: {out}");
        }
        if !options.skip_argument {
            for module in modules.values() {
                destinations.push(PathBuf::from(
                    module
                        .output_path
                        .as_ref()
                        .context("Missing argument output path")?,
                ));
            }
        }
        if modules.len() > 1 {
            for name in [options.architecture_filename, options.complete_filename]
                .into_iter()
                .flatten()
            {
                destinations.push(PathBuf::from(translate_to_output_path(out, name, None)?));
            }
        }
        if let Some(name) = options.evidence_filename {
            destinations.push(PathBuf::from(translate_to_output_path(out, name, None)?));
        }
        let mut copies = BTreeMap::new();
        for style in styles.iter().filter(|s| !is_url(s)) {
            let source = Path::new(style)
                .canonicalize()
                .with_context(|| format!("Cannot read stylesheet {style}"))?;
            if !source.is_file() {
                bail!("Stylesheet is not a regular file: {style}");
            }
            std::fs::File::open(&source)
                .with_context(|| format!("Cannot read stylesheet {style}"))?;
            protected.push(source.clone());
            if !embed {
                let target =
                    directory.join(source.file_name().context("Stylesheet needs a file name")?);
                // Match the renderer's harmless same-canonical-path no-copy case.
                if source != target
                    && let Some(previous) = copies.insert(target.clone(), source.clone())
                    && previous != source
                {
                    bail!(
                        "Stylesheets share an output destination: {}",
                        target.display()
                    );
                }
            }
        }
        destinations.extend(copies.into_keys());
    }
    for flag in ["STATISTICS", "YAMLDUMP", "INDEX_JSON"] {
        if matches.value_source(flag) == Some(ValueSource::CommandLine)
            && let Some(path) = matches.get_one::<String>(flag)
        {
            destinations.push(PathBuf::from(path));
        }
    }
    if let Some(path) = matches.get_one::<String>("INDEX_JSON") {
        let temporary = index_temporary_path(Path::new(path))?;
        if temporary.symlink_metadata().is_ok() {
            bail!("Index staging path already exists: {}", temporary.display());
        }
        destinations.push(temporary);
    }
    let protected = protected
        .iter()
        .map(|p| resolve(p))
        .collect::<Result<Vec<_>>>()?;
    let mut checked: Vec<PathBuf> = Vec::new();
    for target in destinations {
        let resolved = resolve(&target)?;
        if protected
            .iter()
            .map(|p| aliases(&resolved, p))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .any(|same| same)
        {
            bail!(
                "Output must not overwrite an input, import, source, evidence, or stylesheet: {}",
                target.display()
            );
        }
        for earlier in &checked {
            if aliases(&resolved, earlier)?
                || resolved.starts_with(earlier)
                || earlier.starts_with(&resolved)
            {
                bail!(
                    "Output destinations conflict: {} and {}",
                    target.display(),
                    earlier.display()
                );
            }
        }
        if resolved.exists() {
            if !resolved.is_file() {
                bail!("Output is not a regular file: {}", target.display());
            }
            // Check access without creating, truncating or changing contents.
            std::fs::OpenOptions::new()
                .write(true)
                .open(&target)
                .with_context(|| format!("Output is not writable: {}", target.display()))?;
        }
        checked.push(resolved);
    }
    Ok(())
}

pub fn clear_prior_index(path: &Path, modules: &BTreeMap<String, Module>) -> Result<()> {
    match path.symlink_metadata() {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
        Ok(_) => {
            if !crate::contract::is_prior_index(path, modules) {
                bail!(
                    "Existing index destination is not an identified derived index for these inputs; preserved {}",
                    path.display()
                );
            }
            std::fs::remove_file(path)?;
            Ok(())
        }
    }
}
