//! Opt-in checks over actual, expanded nodes. No external ADR resolver.
use crate::{
    diagnostics::Diagnostics,
    gsn::{GsnNode, GsnNodeType, Module},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Metadata {
    version: u32,
    #[serde(default)]
    sources: Vec<FileRef>,
    #[serde(default)]
    evidence: Vec<FileRef>,
    #[serde(default)]
    node_refs: Vec<NodeRef>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FileRef {
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    anchor: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct NodeRef {
    id: String,
    #[serde(rename = "type")]
    node_type: GsnNodeType,
    #[serde(skip_serializing_if = "Option::is_none")]
    owner: Option<String>,
}

pub fn check(
    nodes: &BTreeMap<String, GsnNode>,
    modules: &BTreeMap<String, Module>,
    complete: bool,
    diags: &mut Diagnostics,
) -> BTreeMap<String, Metadata> {
    use GsnNodeType::*;
    let mut parsed = BTreeMap::new();
    for (id, node) in nodes {
        let input = modules[&node.module].canonical_path.as_ref().unwrap();
        let input_str = input.to_string_lossy();
        let mut error = |code: &str, field: &str, message: String| {
            diags.error(
                code,
                Some(&input_str),
                Some(&node.module),
                Some(id),
                Some(field),
                message,
            );
        };
        let core = matches!(
            node.node_type,
            Some(Goal | Strategy | Solution | Context | Assumption | Justification)
        );
        if !core {
            error(
                "P01",
                "nodeType",
                "Only core GSN v3 node types are supported by this profile.".into(),
            );
        }
        if node.text.trim().is_empty() {
            error(
                "P02",
                "text",
                "Node text must be nonblank; meaning requires review.".into(),
            );
        }
        if complete
            && matches!(node.node_type, Some(Goal | Strategy))
            && (node.undeveloped || node.supported_by.is_empty())
        {
            error(
                "P05",
                if node.undeveloped {
                    "undeveloped"
                } else {
                    "supportedBy"
                },
                "A complete Goal or Strategy must be developed and have support.".into(),
            );
        }
        for target in &node.supported_by {
            if let Some(other) = nodes.get(target) {
                let valid = match node.node_type {
                    Some(Goal) => matches!(other.node_type, Some(Goal | Strategy | Solution)),
                    Some(Strategy) => other.node_type == Some(Goal),
                    _ => false,
                };
                if !valid {
                    error(
                        "P03",
                        "supportedBy",
                        format!("Invalid core support edge to {target}."),
                    );
                }
            }
        }
        for target in &node.in_context_of {
            if let Some(other) = nodes.get(target)
                && !(matches!(node.node_type, Some(Goal | Strategy))
                    && matches!(other.node_type, Some(Context | Assumption | Justification)))
            {
                error(
                    "P04",
                    "inContextOf",
                    format!("Invalid core context edge to {target}."),
                );
            }
        }
        if let Some(raw) = node.additional.get("gsnContract") {
            let metadata = match parse_metadata(raw) {
                Ok(m) => m,
                Err(e) => {
                    error("M01", "gsnContract", format!("Invalid metadata shape: {e}"));
                    continue;
                }
            };
            if metadata.version != 1 {
                error(
                    "M02",
                    "gsnContract.version",
                    format!("Unsupported metadata version {}.", metadata.version),
                );
                continue;
            }
            for (field, refs) in [
                ("sources", &metadata.sources),
                ("evidence", &metadata.evidence),
            ] {
                for (i, reference) in refs.iter().enumerate() {
                    let field = format!("gsnContract.{field}[{i}]");
                    if reference.path.trim().is_empty()
                        || reference
                            .anchor
                            .as_ref()
                            .is_some_and(|a| a.trim().is_empty())
                    {
                        error(
                            "M01",
                            &field,
                            "File path and declared anchor must be nonblank.".into(),
                        );
                        continue;
                    }
                    if reference.path.contains("://") || reference.path.starts_with("adr:") {
                        error("M08", &field, "Only local file references are supported; external resolution is deferred.".into());
                        continue;
                    }
                    let path = input.parent().unwrap().join(&reference.path);
                    if !path.is_file() || std::fs::File::open(&path).is_err() {
                        error(
                            "M06",
                            &field,
                            format!(
                                "Local file {} is not readable or does not exist.",
                                path.display()
                            ),
                        );
                        continue;
                    }
                    if let Some(anchor) = &reference.anchor {
                        match std::fs::read_to_string(&path) {
                            Ok(content) if has_anchor(&content, anchor) => {}
                            _ => error(
                                "M07",
                                &field,
                                format!(
                                    "Explicit text anchor {anchor} was not found in {}.",
                                    path.display()
                                ),
                            ),
                        }
                    }
                }
            }
            for (i, reference) in metadata.node_refs.iter().enumerate() {
                let field = format!("gsnContract.nodeRefs[{i}]");
                if reference.id.trim().is_empty()
                    || reference
                        .owner
                        .as_ref()
                        .is_some_and(|o| o.trim().is_empty())
                {
                    error(
                        "M01",
                        &field,
                        "Node ID and declared owner must be nonblank.".into(),
                    );
                    continue;
                }
                if !matches!(
                    reference.node_type,
                    Goal | Strategy | Solution | Context | Assumption | Justification
                ) {
                    error(
                        "M04",
                        &field,
                        "Expected type must be a core GSN v3 type.".into(),
                    );
                }
                match nodes.get(&reference.id) {
                    None => error(
                        "M03",
                        &field,
                        format!("Referenced node {} does not exist.", reference.id),
                    ),
                    Some(target) => {
                        if target.node_type != Some(reference.node_type) {
                            error(
                                "M04",
                                &field,
                                format!(
                                    "Node {} does not have expected type {}.",
                                    reference.id, reference.node_type
                                ),
                            );
                        }
                        if let Some(owner) = &reference.owner {
                            let declared = input.parent().unwrap().join(owner).canonicalize().ok();
                            if declared.as_ref() != modules[&target.module].canonical_path.as_ref()
                            {
                                error(
                                    "M05",
                                    &field,
                                    format!(
                                        "Owner does not match actual input for node {}.",
                                        reference.id
                                    ),
                                );
                            }
                        }
                    }
                }
            }
            parsed.insert(id.to_owned(), metadata);
        }
    }
    parsed
}

fn has_anchor(content: &str, anchor: &str) -> bool {
    content.lines().any(|line| {
        (line.trim_start().starts_with('#') && line.trim_end().ends_with(&format!("{{#{anchor}}}")))
            || line.contains(&format!("id=\"{anchor}\""))
            || line.contains(&format!("id='{anchor}'"))
    })
}

pub fn index(
    nodes: &BTreeMap<String, GsnNode>,
    modules: &BTreeMap<String, Module>,
    metadata: &BTreeMap<String, Metadata>,
) -> serde_json::Value {
    serde_json::json!({"version": 1, "nodes": nodes.iter().map(|(id, n)| {
        let mut support = n.supported_by.clone(); support.sort();
        let mut context = n.in_context_of.clone(); context.sort();
        serde_json::json!({"id": id, "type": n.node_type, "module": n.module,
            "input": modules[&n.module].canonical_path,
            "supportedBy": support, "inContextOf": context, "metadata": metadata.get(id)})
    }).collect::<Vec<_>>()})
}

/// Complete file discovery is required before any contract output mutation.
/// Malformed metadata cannot be ignored when it may hide protected files.
pub fn reference_paths(
    nodes: &BTreeMap<String, GsnNode>,
    modules: &BTreeMap<String, Module>,
) -> Result<Vec<std::path::PathBuf>, ()> {
    let mut paths = Vec::new();
    for node in nodes.values() {
        if let Some(raw) = node.additional.get("gsnContract") {
            let metadata = parse_metadata(raw).map_err(|_| ())?;
            let parent = modules[&node.module]
                .canonical_path
                .as_ref()
                .unwrap()
                .parent()
                .unwrap();
            for reference in metadata.sources.iter().chain(&metadata.evidence) {
                if reference.path.trim().is_empty() {
                    return Err(());
                }
                paths.push(parent.join(&reference.path));
            }
            for reference in &metadata.node_refs {
                if let Some(owner) = &reference.owner {
                    if owner.trim().is_empty() {
                        return Err(());
                    }
                    paths.push(parent.join(owner));
                }
            }
        }
    }
    Ok(paths)
}

// Recognize the published derived index shape and actual input owners, not an
// arbitrary sentinel or JSON document. Protected-file checks must precede this.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PriorIndex {
    version: u32,
    nodes: Vec<PriorIndexNode>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PriorIndexNode {
    id: String,
    #[serde(rename = "type")]
    node_type: GsnNodeType,
    module: String,
    input: std::path::PathBuf,
    #[serde(rename = "supportedBy")]
    _support: Vec<String>,
    #[serde(rename = "inContextOf")]
    _context: Vec<String>,
    // Unlike optional metadata on an authored node, this index field is required.
    metadata: serde_json::Value,
}

pub fn is_prior_index(path: &Path, modules: &BTreeMap<String, Module>) -> bool {
    let Ok(file) = std::fs::File::open(path) else {
        return false;
    };
    let Ok(index) = serde_json::from_reader::<_, PriorIndex>(file) else {
        return false;
    };
    let mut ids = std::collections::BTreeSet::new();
    index.version == 1
        && !index.nodes.is_empty()
        && index.nodes.iter().all(|n| {
            !n.id.trim().is_empty()
                && ids.insert(&n.id)
                && !n.module.trim().is_empty()
                && matches!(
                    n.node_type,
                    GsnNodeType::Goal
                        | GsnNodeType::Strategy
                        | GsnNodeType::Solution
                        | GsnNodeType::Context
                        | GsnNodeType::Assumption
                        | GsnNodeType::Justification
                )
                && n.input.is_absolute()
                && modules.get(&n.module).is_some_and(|m| {
                    n.input.canonicalize().ok().as_ref() == m.canonical_path.as_ref()
                })
                && (n.metadata.is_null()
                    || serde_json::from_value::<Metadata>(n.metadata.clone())
                        .is_ok_and(|m| m.version == 1))
        })
}

// YAML permits coercion into strings. Parse raw values first, then use strict JSON
// deserialization so numeric paths/IDs and string versions cannot change type.
fn parse_metadata(raw: &str) -> Result<Metadata, String> {
    serde_json::from_value(parse_metadata_value(raw)?).map_err(|e| e.to_string())
}

fn parse_metadata_value(raw: &str) -> Result<serde_json::Value, String> {
    let options =
        serde_saphyr::options! { duplicate_keys: serde_saphyr::DuplicateKeyPolicy::Error };
    serde_saphyr::from_str_with_options(raw, options).map_err(|e| e.to_string())
}
