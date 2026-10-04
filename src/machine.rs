//! Output-mode orchestration; shares loading, expansion and upstream checks.
use crate::{
    contract,
    diagnostics::{DiagType, Diagnostics},
    gsn::{GsnNode, Module},
};
use clap::ArgMatches;
use std::{collections::BTreeMap, path::Path};

pub fn run(matches: &ArgMatches) -> bool {
    let mut diags = Diagnostics::default();
    let mut nodes = BTreeMap::<String, GsnNode>::new();
    let mut modules = BTreeMap::<String, Module>::new();
    let inputs = matches
        .get_many::<String>("INPUT")
        .unwrap()
        .cloned()
        .collect::<Vec<_>>();
    let excluded = matches
        .get_many::<String>("EXCLUDED_MODULE")
        .into_iter()
        .flatten()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let layers = matches
        .get_many::<String>("LAYERS")
        .into_iter()
        .flatten()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let profile = matches.get_one::<String>("CONTRACT");
    let index_path = matches.get_one::<String>("INDEX_JSON");
    let output_dir = matches.get_one::<String>("OUTPUT_DIRECTORY").unwrap();
    let result = (|| -> anyhow::Result<()> {
        let read_result =
            crate::read_inputs(&inputs, &mut nodes, &mut modules, &mut diags, output_dir);
        if profile.is_some() && (read_result.is_err() || diags.errors > 0) {
            discovery_incomplete(&mut diags, index_path);
            return read_result;
        }
        read_result?;
        if profile.is_some() && !excluded.is_empty() {
            diags.error(
                "P06",
                None,
                None,
                None,
                Some("exclude"),
                "Contract checks cannot exclude required references or modules.".into(),
            );
        }
        // Never pass exclusion flags to a contract check.
        let exclusions = if profile.is_some() {
            &[][..]
        } else {
            &excluded[..]
        };
        let validation = crate::validate_and_check(
            &mut nodes,
            &modules,
            &mut diags,
            exclusions,
            &layers,
            matches.get_flag("EXTENDED_CHECK"),
            matches.get_flag("WARN_DIALECTIC"),
        );
        let metadata = if let Some(mode) = profile.filter(|_| validation.is_ok()) {
            contract::check(&nodes, &modules, mode == "complete", &mut diags)
        } else {
            BTreeMap::new()
        };
        let mut styles = matches
            .get_many::<String>("STYLESHEETS")
            .into_iter()
            .flatten()
            .cloned()
            .collect::<Vec<_>>();
        styles.extend(modules.values().flat_map(|m| m.meta.stylesheets.clone()));
        if profile.is_some() {
            let Ok(references) = contract::reference_paths(&nodes, &modules) else {
                discovery_incomplete(&mut diags, index_path);
                return validation;
            };
            let safe_outputs = crate::output_guard::preflight(
                matches, &modules, references, &styles,
            )
            .and_then(|_| match index_path {
                Some(path) => crate::output_guard::clear_prior_index(Path::new(path), &modules),
                None => Ok(()),
            });
            if let Err(e) = safe_outputs {
                diags.error("O01", None, None, None, None, format!("{e:#}"));
                return Ok(());
            }
        }
        validation?;
        if diags.errors > 0 {
            return Ok(());
        }
        if !matches.get_flag("CHECK_ONLY") {
            std::fs::create_dir_all(output_dir)?;
            let embed = matches.get_flag("EMBED_CSS");
            crate::copy_and_prepare_stylesheets(&mut styles, embed, output_dir)?;
            let mut options = crate::render::RenderOptions::new(matches, styles, embed, output_dir);
            // An index must describe real nodes, before ordinary excluded placeholders.
            if profile.is_none() {
                crate::add_missing_nodes_and_modules(&mut nodes, &mut modules, &mut options);
            }
            crate::print_outputs(&nodes, &modules, &options)?;
        }
        if let Some(clap::parser::ValueSource::CommandLine) = matches.value_source("STATISTICS") {
            let mut out: Box<dyn std::io::Write> = match matches.get_one::<String>("STATISTICS") {
                Some(p) => Box::new(std::fs::File::create(p)?),
                None => Box::new(std::io::stdout().lock()),
            };
            crate::outputs::render_statistics(&mut out, &nodes, &modules)?;
        }
        if let Some(clap::parser::ValueSource::CommandLine) = matches.value_source("YAMLDUMP") {
            let mut out = match matches.get_one::<String>("YAMLDUMP") {
                Some(p) => crate::create_file_incl_parent(Path::new(p))?,
                None => Box::new(std::io::stdout().lock()),
            };
            crate::outputs::render_yaml_docs(&mut out, &nodes, &modules)?;
        }
        if let Some(path) = index_path {
            write_index(
                Path::new(path),
                &contract::index(
                    &nodes,
                    &modules,
                    &metadata,
                    profile.expect("--index-json requires --contract"),
                ),
            )?;
        }
        Ok(())
    })();
    if let Err(e) = result
        && !e.is::<crate::ValidationOrCheckError>()
        && diags.errors == 0
    {
        diags.error("O01", None, None, None, None, format!("{e:#}"));
    }
    // Enrich upstream records with actual module owners. Per-node checks supply IDs.
    for d in &mut diags.messages {
        if d.module.is_none() {
            d.module = d
                .node
                .as_ref()
                .and_then(|id| nodes.get(id))
                .map(|n| n.module.clone());
        }
        if d.input.is_none() {
            d.input = d
                .module
                .as_ref()
                .and_then(|m| modules.get(m))
                .map(|m| m.orig_file_name.clone());
        }
    }
    if matches.contains_id("DIAGNOSTICS") {
        eprintln!(
            "{}",
            serde_json::json!({"version": 1, "diagnostics": diags.messages})
        );
    } else {
        for d in &diags.messages {
            eprintln!("{d}");
        }
        if diags.errors > 0 {
            eprintln!(
                "{} errors and {} warnings detected.",
                diags.errors, diags.warnings
            );
        }
    }
    !diags
        .messages
        .iter()
        .any(|d| d.diag_type == DiagType::Error)
}

fn write_index(path: &Path, index: &serde_json::Value) -> anyhow::Result<()> {
    use std::io::Write;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent)?;
    let temporary = crate::output_guard::index_temporary_path(path)?;
    let mut out = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let result = (|| -> anyhow::Result<()> {
        serde_json::to_writer_pretty(&mut out, index)?;
        out.write_all(b"\n")?;
        out.flush()?;
        drop(out);
        std::fs::rename(&temporary, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(temporary);
    }
    result
}

fn discovery_incomplete(diags: &mut Diagnostics, index: Option<&String>) {
    if let Some(path) = index {
        diags.error("O02", None, None, None, Some("index-json"), format!("Input/reference discovery is incomplete; all output files are preserved. Any existing index at {path} is not current."));
    }
}
