//! CLI regressions for contract output preservation, using small generic inputs.
use assert_cmd::cargo;
use assert_fs::prelude::*;
use serde_json::Value;
use std::{fs, process::Command};

const MODEL: &str = "G1: {text: The design keeps the required state., supportedBy: [Sn1]}\nSn1: {text: The inspection covers the state requirement.}\n";

fn run(dir: &assert_fs::TempDir, options: &[&str], inputs: &[&str]) -> std::process::Output {
    Command::new(cargo::cargo_bin!())
        .current_dir(dir)
        .args(["--contract", "complete", "--diagnostics", "json"])
        .args(options)
        .args(inputs)
        .output()
        .unwrap()
}
fn failure(output: std::process::Output, code: &str) -> Value {
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let doc: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(doc["version"], 1);
    assert!(
        doc["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["code"] == code),
        "{doc}"
    );
    doc
}
fn referenced(path: &str) -> String {
    MODEL.replace("Sn1: {text: The inspection covers the state requirement.}",
        &format!("Sn1: {{text: The inspection covers the state requirement., gsnContract: {{version: 1, evidence: [{{path: '{path}'}}]}}}}"))
}
fn unchanged(dir: &assert_fs::TempDir, name: &str, bytes: &[u8]) {
    assert_eq!(fs::read(dir.child(name)).unwrap(), bytes, "Changed {name}");
}

#[test]
fn incomplete_yaml_and_import_discovery_preserve_outputs_and_references() {
    for imported in [false, true] {
        let dir = assert_fs::TempDir::new().unwrap();
        dir.child("source.md")
            .write_str("Required source bytes")
            .unwrap();
        if imported {
            dir.child("model.yaml").write_str("module: {name: Main, uses: [bad.yaml, later.yaml]}\nG1: {text: Claim, supportedBy: [Sn1]}\n").unwrap();
            dir.child("bad.yaml")
                .write_str("G2: [unterminated")
                .unwrap();
            dir.child("later.yaml").write_str("module: {name: Later}\nSn1: {text: Evidence, gsnContract: {version: 1, sources: [{path: source.md}]}}\n").unwrap();
        } else {
            dir.child("model.yaml")
                .write_str(&format!(
                    "{}\nbroken: [unterminated",
                    referenced("source.md")
                ))
                .unwrap();
        }
        let before = fs::read(dir.child("model.yaml")).unwrap();
        let doc = failure(
            run(
                &dir,
                &[
                    "--check",
                    "--index-json",
                    "source.md",
                    "--statistics=stats.txt",
                ],
                &["model.yaml"],
            ),
            "I02",
        );
        assert!(
            doc["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d["code"] == "O02")
        );
        unchanged(&dir, "source.md", b"Required source bytes");
        unchanged(&dir, "model.yaml", &before);
        assert!(!dir.child("stats.txt").path().exists());
    }
}

#[test]
fn malformed_metadata_never_removes_or_overwrites_an_undiscovered_target() {
    for metadata in [
        "|\n    version: [unterminated\n    evidence: [{path: evidence.md}]",
        "{version: 1, evidence: {path: evidence.md}}",
        "|\n    version: 1\n    evidence: []\n    evidence: [{path: evidence.md}]",
        "{version: 1, unexpected: true, evidence: [{path: evidence.md}]}",
    ] {
        let dir = assert_fs::TempDir::new().unwrap();
        dir.child("model.yaml").write_str(&format!("G1: {{text: Claim, supportedBy: [Sn1]}}\nSn1:\n  text: Evidence\n  gsnContract: {metadata}\n")).unwrap();
        dir.child("evidence.md")
            .write_str("Actual evidence bytes")
            .unwrap();
        let before = fs::read(dir.child("model.yaml")).unwrap();
        failure(
            run(
                &dir,
                &["--index-json", "evidence.md", "--statistics=stats.txt"],
                &["model.yaml"],
            ),
            "M01",
        );
        unchanged(&dir, "evidence.md", b"Actual evidence bytes");
        unchanged(&dir, "model.yaml", &before);
        assert!(!dir.child("stats.txt").path().exists());
        assert!(!dir.child("model.svg").path().exists());
    }
}

#[test]
fn arbitrary_files_are_not_owned_prior_indexes() {
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child("model.yaml").write_str(MODEL).unwrap();
    for content in [
        "stale",
        "{\"version\":1,\"nodes\":[]}",
        "{\"version\":1,\"nodes\":[{\"id\":\"G1\",\"type\":\"Goal\",\"module\":\"Main\",\"input\":\"/tmp/model.yaml\",\"supportedBy\":[],\"inContextOf\":[],\"metadata\":null}]}",
    ] {
        dir.child("nodes.json").write_str(content).unwrap();
        failure(
            run(
                &dir,
                &[
                    "--check",
                    "--index-json",
                    "nodes.json",
                    "--statistics=stats.txt",
                ],
                &["model.yaml"],
            ),
            "O01",
        );
        unchanged(&dir, "nodes.json", content.as_bytes());
        assert!(!dir.child("stats.txt").path().exists());
    }
}

#[test]
fn owned_index_is_preserved_on_incomplete_discovery_but_invalidated_on_complete_failure() {
    let dir = assert_fs::TempDir::new().unwrap();
    let options = ["--check", "--index-json", "nodes.json"];
    dir.child("model.yaml").write_str(MODEL).unwrap();
    assert!(run(&dir, &options, &["model.yaml"]).status.success());
    let prior = fs::read(dir.child("nodes.json")).unwrap();
    dir.child("model.yaml")
        .write_str(&format!("{MODEL}  gsnContract: {{version: invalid}}\n"))
        .unwrap();
    failure(run(&dir, &options, &["model.yaml"]), "O02");
    unchanged(&dir, "nodes.json", &prior);
    dir.child("model.yaml")
        .write_str("G1: {text: Unsupported claim}\n")
        .unwrap();
    failure(run(&dir, &options, &["model.yaml"]), "P05");
    assert!(!dir.child("nodes.json").path().exists());
}

#[test]
fn default_and_renamed_render_destinations_protect_evidence() {
    for (reference, options) in [
        ("evidence.md", vec![]),
        ("model.svg", vec!["--no-evidence"]),
        ("proof.md", vec!["--evidence=ignored/proof.md"]),
    ] {
        let dir = assert_fs::TempDir::new().unwrap();
        let model = referenced(reference);
        dir.child("model.yaml").write_str(&model).unwrap();
        dir.child(reference).write_str("Reference bytes").unwrap();
        failure(run(&dir, &options, &["model.yaml"]), "O01");
        unchanged(&dir, reference, b"Reference bytes");
        unchanged(&dir, "model.yaml", model.as_bytes());
        if reference != "model.svg" {
            assert!(!dir.child("model.svg").path().exists());
        }
    }
}

#[test]
fn statistics_dump_and_index_reject_protected_hardlink_identities() {
    for target in ["model.yaml", "proof.md"] {
        for option in [
            "--statistics=alias",
            "--dump-yaml=alias",
            "--index-json=alias",
        ] {
            let dir = assert_fs::TempDir::new().unwrap();
            dir.child("model.yaml")
                .write_str(&referenced("proof.md"))
                .unwrap();
            dir.child("proof.md").write_str("Evidence bytes").unwrap();
            fs::hard_link(dir.child(target), dir.child("alias")).unwrap();
            let original = fs::read(dir.child(target)).unwrap();
            failure(run(&dir, &["--check", option], &["model.yaml"]), "O01");
            unchanged(&dir, target, &original);
            unchanged(&dir, "alias", &original);
            assert!(same_file::is_same_file(dir.child(target), dir.child("alias")).unwrap());
        }
    }
}

#[cfg(unix)]
#[test]
fn symlink_and_normalized_output_aliases_are_rejected() {
    let dir = assert_fs::TempDir::new().unwrap();
    let model = referenced("proof.md");
    dir.child("model.yaml").write_str(&model).unwrap();
    dir.child("proof.md").write_str("Evidence bytes").unwrap();
    dir.child("sub").create_dir_all().unwrap();
    std::os::unix::fs::symlink("proof.md", dir.child("alias")).unwrap();
    for option in [
        "--statistics=alias",
        "--dump-yaml=sub/../proof.md",
        "--index-json=alias",
    ] {
        failure(run(&dir, &["--check", option], &["model.yaml"]), "O01");
        unchanged(&dir, "proof.md", b"Evidence bytes");
        unchanged(&dir, "model.yaml", model.as_bytes());
    }
}

#[test]
fn imported_inputs_and_their_owner_relative_references_are_protected() {
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child("model.yaml")
        .write_str(
            "module: {name: Root, uses: [sub/child.yaml]}\nG1: {text: Claim, supportedBy: [Sn1]}\n",
        )
        .unwrap();
    let child = "module: {name: Child}\nSn1: {text: Evidence, gsnContract: {version: 1, sources: [{path: source.md}]}}\n";
    dir.child("sub/child.yaml").write_str(child).unwrap();
    dir.child("sub/source.md")
        .write_str("Source bytes")
        .unwrap();
    for option in ["--statistics=sub/child.yaml", "--dump-yaml=sub/source.md"] {
        failure(run(&dir, &["--check", option], &["model.yaml"]), "O01");
        unchanged(&dir, "sub/child.yaml", child.as_bytes());
        unchanged(&dir, "sub/source.md", b"Source bytes");
    }
}

#[test]
fn later_invalid_output_prevents_earlier_writes_and_index_cleanup() {
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child("model.yaml").write_str(MODEL).unwrap();
    assert!(
        run(
            &dir,
            &["--check", "--index-json", "nodes.json"],
            &["model.yaml"]
        )
        .status
        .success()
    );
    let index = fs::read(dir.child("nodes.json")).unwrap();
    dir.child("stats.txt")
        .write_str("Earlier output bytes")
        .unwrap();
    dir.child("block").write_str("Not a directory").unwrap();
    failure(
        run(
            &dir,
            &[
                "--statistics=stats.txt",
                "--dump-yaml=block/fail.yaml",
                "--index-json",
                "nodes.json",
            ],
            &["model.yaml"],
        ),
        "O01",
    );
    unchanged(&dir, "stats.txt", b"Earlier output bytes");
    unchanged(&dir, "nodes.json", &index);
    unchanged(&dir, "model.yaml", MODEL.as_bytes());
    assert!(!dir.child("model.svg").path().exists());
    assert!(!dir.child("evidence.md").path().exists());
}

#[test]
fn copied_stylesheets_cannot_overwrite_references_or_other_outputs() {
    for reference in ["out/style.css", "unused.md"] {
        let dir = assert_fs::TempDir::new().unwrap();
        dir.child("model.yaml")
            .write_str(&referenced(reference))
            .unwrap();
        dir.child(reference).write_str("Evidence bytes").unwrap();
        dir.child("css/style.css")
            .write_str("svg { color: blue; }")
            .unwrap();
        let options = if reference == "unused.md" {
            vec![
                "--output-dir=out",
                "--stylesheet=css/style.css",
                "--dump-yaml=out/style.css",
            ]
        } else {
            vec!["--output-dir=out", "--stylesheet=css/style.css"]
        };
        failure(run(&dir, &options, &["model.yaml"]), "O01");
        unchanged(&dir, reference, b"Evidence bytes");
        unchanged(&dir, "css/style.css", b"svg { color: blue; }");
        assert!(!dir.child("out/model.svg").path().exists());
    }
}

#[cfg(unix)]
#[test]
fn stylesheet_copy_through_a_symlink_cannot_overwrite_its_source() {
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child("model.yaml").write_str(MODEL).unwrap();
    dir.child("css/style.css")
        .write_str("svg { color: blue; }")
        .unwrap();
    dir.child("out").create_dir_all().unwrap();
    std::os::unix::fs::symlink("../css/style.css", dir.child("out/style.css")).unwrap();
    failure(
        run(
            &dir,
            &["--output-dir=out", "--stylesheet=css/style.css"],
            &["model.yaml"],
        ),
        "O01",
    );
    unchanged(&dir, "css/style.css", b"svg { color: blue; }");
    assert!(!dir.child("out/model.svg").path().exists());
}

#[test]
fn dedicated_outputs_and_stylesheet_copy_still_work_and_index_is_repeatable() {
    let dir = assert_fs::TempDir::new().unwrap();
    let model = referenced("proof.md");
    dir.child("model.yaml").write_str(&model).unwrap();
    dir.child("proof.md").write_str("Actual evidence").unwrap();
    dir.child("css/style.css")
        .write_str("svg { color: blue; }")
        .unwrap();
    let options = [
        "--output-dir=out",
        "--stylesheet=css/style.css",
        "--statistics=stats.txt",
        "--dump-yaml=out/model-dump.yaml",
        "--index-json",
        "out/nodes.json",
    ];
    let result = run(&dir, &options, &["model.yaml"]);
    assert!(result.status.success(), "{result:?}");
    let index = fs::read(dir.child("out/nodes.json")).unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&index).unwrap()["nodes"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    for path in [
        "out/model.svg",
        "out/evidence.md",
        "out/model-dump.yaml",
        "stats.txt",
    ] {
        assert!(!fs::read(dir.child(path)).unwrap().is_empty(), "{path}");
    }
    unchanged(&dir, "out/style.css", b"svg { color: blue; }");
    assert!(run(&dir, &options, &["model.yaml"]).status.success());
    unchanged(&dir, "out/nodes.json", &index);
    unchanged(&dir, "model.yaml", model.as_bytes());
    unchanged(&dir, "proof.md", b"Actual evidence");
}

#[test]
fn indexes_for_other_input_owners_are_preserved_and_rejected() {
    let dir = assert_fs::TempDir::new().unwrap();
    dir.child("other.yaml").write_str(MODEL).unwrap();
    dir.child("model.yaml").write_str(MODEL).unwrap();
    let options = ["--check", "--index-json", "nodes.json"];
    assert!(run(&dir, &options, &["other.yaml"]).status.success());
    let index = fs::read(dir.child("nodes.json")).unwrap();
    failure(run(&dir, &options, &["model.yaml"]), "O01");
    unchanged(&dir, "nodes.json", &index);
}

#[test]
fn multimodule_graph_and_combined_view_destinations_protect_references() {
    for reference in ["architecture.svg", "complete.svg", "sub/child.svg"] {
        let dir = assert_fs::TempDir::new().unwrap();
        dir.child("model.yaml").write_str(&format!("module: {{name: Root, uses: [sub/child.yaml]}}\nG1: {{text: Claim, supportedBy: [Sn1], gsnContract: {{version: 1, sources: [{{path: '{reference}'}}]}}}}\n")).unwrap();
        dir.child("sub/child.yaml")
            .write_str("module: {name: Child}\nSn1: {text: Evidence}\n")
            .unwrap();
        dir.child(reference).write_str("Source bytes").unwrap();
        failure(run(&dir, &[], &["model.yaml"]), "O01");
        unchanged(&dir, reference, b"Source bytes");
        assert!(!dir.child("model.svg").path().exists());
        assert!(!dir.child("evidence.md").path().exists());
    }
}

fn refuses_prior_attributes(attributes: Value) {
    for mode in ["draft", "complete"] {
        for valid_model in [true, false] {
            let dir = assert_fs::TempDir::new().unwrap();
            dir.child("model.yaml").write_str(MODEL).unwrap();
            let args = [
                "--check",
                "--contract",
                mode,
                "--diagnostics",
                "json",
                "--index-json",
                "nodes.json",
                "model.yaml",
            ];
            let invoke = || {
                Command::new(cargo::cargo_bin!())
                    .current_dir(&dir)
                    .args(args)
                    .output()
                    .unwrap()
            };
            assert!(invoke().status.success());
            let mut index: Value =
                serde_json::from_slice(&fs::read(dir.child("nodes.json")).unwrap()).unwrap();
            index["nodes"][0]["attributes"] = attributes.clone();
            let prior = serde_json::to_vec_pretty(&index).unwrap();
            fs::write(dir.child("nodes.json"), &prior).unwrap();
            if !valid_model {
                dir.child("model.yaml")
                    .write_str("G1: {text: Claim, supportedBy: [C1]}\nC1: {text: Invalid support target}\n")
                    .unwrap();
            }
            let model = fs::read(dir.child("model.yaml")).unwrap();
            failure(invoke(), "O01");
            unchanged(&dir, "nodes.json", &prior);
            unchanged(&dir, "model.yaml", &model);
        }
    }
}

#[test]
fn malformed_prior_url_is_preserved() {
    refuses_prior_attributes(serde_json::json!({"url": 17}));
}

#[test]
fn unknown_prior_attribute_is_preserved() {
    refuses_prior_attributes(serde_json::json!({"private-note": {"keep": "this document"}}));
}

#[test]
fn malformed_prior_typed_attributes_are_preserved() {
    for attributes in [
        serde_json::json!([]),
        serde_json::json!({"classes": [17]}),
        serde_json::json!({"url": null}),
        serde_json::json!({"acp": {"inspection": [17]}}),
        serde_json::json!({"acp": []}),
        serde_json::json!({"rankIncrement": -1}),
        serde_json::json!({"rankIncrement": "2"}),
        serde_json::json!({"horizontalIndex": {"relative": "-2"}}),
        serde_json::json!({"horizontalIndex": {"absolute": {"private-note": true}}}),
        serde_json::json!({"horizontalIndex": {"relative": 2, "private-note": true}}),
        serde_json::json!({"charWrap": -1}),
        serde_json::json!({"charWrap": 4294967296_u64}),
        serde_json::json!({"challenges": {"Node": "G1"}}),
        serde_json::json!({"defeated": "true"}),
        serde_json::json!({"defeatedRelation": [17]}),
        serde_json::json!({"text": "hidden duplicate text"}),
        serde_json::json!({"supportedBy": ["Sn1"]}),
    ] {
        refuses_prior_attributes(attributes);
    }
}

#[test]
fn generated_typed_attributes_and_opaque_storage_remain_repeatable() {
    for (horizontal, expected) in [
        ("{relative: -2}", serde_json::json!({"relative": -2})),
        ("{relative: 0}", serde_json::json!({"relative": 0})),
        ("{absolute: 0}", serde_json::json!({"absolute": 0})),
        ("{absolute: last}", serde_json::json!({"absolute": "last"})),
    ] {
        let dir = assert_fs::TempDir::new().unwrap();
        let model = format!(
            "module: {{name: Main, charWrap: 64}}\nG1:\n  text: Claim\n  supportedBy: [Sn1]\n  classes: [reviewed, custom]\n  rankIncrement: 2\n  horizontalIndex: {horizontal}\n  acp: {{inspection: [G1, Sn1]}}\n  defeatedRelation: [Sn1]\n  private-note: {{keep: this document}}\n  gsnContract: {{version: 1}}\nSn1: {{text: Evidence, url: proof.md, charWrap: 80}}\n"
        );
        dir.child("model.yaml").write_str(&model).unwrap();
        for mode in ["draft", "complete"] {
            let args = [
                "--check",
                "--contract",
                mode,
                "--diagnostics",
                "json",
                "--index-json",
                "nodes.json",
                "model.yaml",
            ];
            let invoke = || {
                Command::new(cargo::cargo_bin!())
                    .current_dir(&dir)
                    .args(args)
                    .output()
                    .unwrap()
            };
            let result = invoke();
            assert!(result.status.success(), "{result:?}");
            let first = fs::read(dir.child("nodes.json")).unwrap();
            let index: Value = serde_json::from_slice(&first).unwrap();
            assert_eq!(
                index["nodes"][0]["attributes"],
                serde_json::json!({
                    "classes": ["reviewed", "custom"], "rankIncrement": 2,
                    "horizontalIndex": expected, "acp": {"inspection": ["G1", "Sn1"]},
                    "defeatedRelation": ["Sn1"], "charWrap": 64
                })
            );
            assert_eq!(
                index["nodes"][1]["attributes"],
                serde_json::json!({"url": "proof.md", "charWrap": 80})
            );
            assert_eq!(
                index["nodes"][0]["additionalAttributes"]["private-note"],
                "keep: this document\n"
            );
            assert_eq!(index["nodes"][0]["metadata"]["version"], 1);
            let repeat = invoke();
            assert!(repeat.status.success(), "{repeat:?}");
            unchanged(&dir, "nodes.json", &first);
            unchanged(&dir, "model.yaml", model.as_bytes());
        }
    }
}
