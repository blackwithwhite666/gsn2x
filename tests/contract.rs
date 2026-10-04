use assert_cmd::cargo;
use assert_fs::prelude::*;
use serde_json::Value;
use std::process::{Command, Output};

const COMPLETE: &str = "G1:\n  text: Each local reference identifies one existing object of the required type.\n  supportedBy: [Sn1]\nSn1:\n  text: Inspection of the reference cases supports the claim.\n";

fn run(dir: &assert_fs::TempDir, args: &[&str]) -> Output {
    Command::new(cargo::cargo_bin!())
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}
fn json_failure(output: Output, code: &str) -> Value {
    assert!(!output.status.success(), "Unexpected pass: {:?}", output);
    assert_eq!(output.status.code(), Some(1));
    let document: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(document["version"], 1);
    assert!(
        document["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["code"] == code),
        "{document}"
    );
    document
}
fn check(text: &str, mode: &str) -> Output {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child("model.yaml").write_str(text).unwrap();
    run(
        &temp,
        &[
            "--check",
            "--contract",
            mode,
            "--diagnostics",
            "json",
            "model.yaml",
        ],
    )
}

#[test]
fn prose_goal_complete_and_default_compatibility() {
    assert!(check(COMPLETE, "complete").status.success());
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child("model.yaml")
        .write_str("G1: {text: A prose claim}\n")
        .unwrap();
    assert!(run(&temp, &["--check", "model.yaml"]).status.success());
    assert!(
        check("G1: {text: A prose claim, undeveloped: true}\n", "draft")
            .status
            .success()
    );
    assert!(
        check("G1: {text: A prose claim}\n", "draft")
            .status
            .success()
    );
    for marker in ["", ", undeveloped: true"] {
        json_failure(
            check(
                &format!("G1: {{text: A prose claim{marker}}}\n"),
                "complete",
            ),
            "P05",
        );
    }
}

#[test]
fn strict_edges_blank_text_and_context() {
    for child in ["S2", "Sn1"] {
        let text = format!(
            "G1: {{text: Claim, supportedBy: [S1]}}\nS1: {{text: Reason, supportedBy: [{child}]}}\n{child}: {{text: Child, undeveloped: true}}\n"
        );
        let temp = assert_fs::TempDir::new().unwrap();
        temp.child("model.yaml").write_str(&text).unwrap();
        assert!(run(&temp, &["--check", "model.yaml"]).status.success());
        let d = json_failure(check(&text, "draft"), "P03");
        let bad = d["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["code"] == "P03")
            .unwrap();
        assert_eq!(bad["node"], "S1");
        assert_eq!(bad["field"], "supportedBy");
        assert!(!bad["input"].is_null());
    }
    json_failure(
        check(
            &COMPLETE.replace(
                "Inspection of the reference cases supports the claim.",
                "'   '",
            ),
            "complete",
        ),
        "P02",
    );
    json_failure(
        check(
            "G1: {text: Claim, inContextOf: [Sn1]}\nSn1: {text: Evidence}\n",
            "draft",
        ),
        "V04",
    );
    let valid = "G1: {text: Claim, supportedBy: [S1], inContextOf: [C1, A1, J1]}\nS1: {text: Reason, supportedBy: [G2]}\nG2: {text: Subclaim, supportedBy: [Sn1]}\nSn1: {text: Evidence}\nC1: {text: Context}\nA1: {text: Assumption}\nJ1: {text: Justification}\n";
    assert!(check(valid, "complete").status.success());
    json_failure(
        check(
            "G1: {text: Claim, supportedBy: [CG1]}\nCG1: {text: Counterclaim}\n",
            "draft",
        ),
        "P01",
    );
}

#[test]
fn metadata_shape_versions_and_typed_refs() {
    for metadata in [
        "[]",
        "{version: 1, surprise: true}",
        "{version: 1, nodeRefs: [{id: Sn1}]}",
        "{version: 1, sources: [{path: 7}]}",
        "{version: 1, sources: [{path: ' '}]}",
    ] {
        json_failure(
            check(
                &format!("{COMPLETE}  gsnContract: {metadata}\n"),
                "complete",
            ),
            "M01",
        );
    }
    json_failure(
        check(
            &format!("{COMPLETE}  gsnContract: {{version: 2}}\n"),
            "complete",
        ),
        "M02",
    );
    for (id, node_type, code) in [("missing", "Goal", "M03"), ("G1", "Solution", "M04")] {
        json_failure(
            check(
                &format!(
                    "{COMPLETE}  gsnContract: {{version: 1, nodeRefs: [{{id: {id}, type: {node_type}}}]}}\n"
                ),
                "complete",
            ),
            code,
        );
    }
}

#[test]
fn local_files_anchors_owners_and_layers() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child("sub/source.md")
        .write_str("# Requirement {#required}\n")
        .unwrap();
    let model = format!(
        "{COMPLETE}  colors: {{custom: visual}}\n  gsnContract:\n    version: 1\n    evidence: [{{path: source.md, anchor: required}}]\n    nodeRefs: [{{id: G1, type: Goal, owner: model.yaml}}]\n"
    );
    temp.child("sub/model.yaml").write_str(&model).unwrap();
    assert!(
        run(
            &temp,
            &["--check", "--contract", "complete", "sub/model.yaml"]
        )
        .status
        .success()
    );
    for (from, to, code) in [
        ("source.md", "absent.md", "M06"),
        ("anchor: required", "anchor: absent", "M07"),
        ("owner: model.yaml", "owner: source.md", "M05"),
        ("source.md", "https://example.org/source.md", "M08"),
    ] {
        temp.child("sub/model.yaml")
            .write_str(&model.replace(from, to))
            .unwrap();
        json_failure(
            run(
                &temp,
                &[
                    "--check",
                    "--contract",
                    "complete",
                    "--diagnostics",
                    "json",
                    "sub/model.yaml",
                ],
            ),
            code,
        );
    }
}

#[test]
fn bad_yaml_missing_inputs_and_no_stale_index() {
    let temp = assert_fs::TempDir::new().unwrap();
    for (text, code) in [("G1: [unterminated", "I02"), ("G1: {text: Claim}\n", "P05")] {
        temp.child("model.yaml").write_str(COMPLETE).unwrap();
        let args = [
            "--check",
            "--contract",
            "complete",
            "--diagnostics",
            "json",
            "--index-json",
            "nodes.json",
            "model.yaml",
        ];
        assert!(run(&temp, &args).status.success());
        let prior = std::fs::read(temp.child("nodes.json")).unwrap();
        temp.child("model.yaml").write_str(text).unwrap();
        let document = json_failure(run(&temp, &args), code);
        if code == "I02" {
            assert_eq!(prior, std::fs::read(temp.child("nodes.json")).unwrap());
            assert!(
                document["diagnostics"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|d| d["code"] == "O02")
            );
            std::fs::remove_file(temp.child("nodes.json")).unwrap();
        } else {
            assert!(!temp.child("nodes.json").path().exists());
        }
    }
    json_failure(
        run(&temp, &["--check", "--diagnostics", "json", "missing.yaml"]),
        "I01",
    );
    json_failure(run(&temp, &["--check", "--diagnostics", "json"]), "I01");
}

#[test]
fn complete_cannot_bypass_references_modules_cycles_or_reachability() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child("model.yaml")
        .write_str("module: {name: Main}\nG1: {text: Claim, supportedBy: [G2]}\n")
        .unwrap();
    json_failure(
        run(
            &temp,
            &[
                "--check",
                "--contract",
                "complete",
                "--diagnostics",
                "json",
                "--exclude=Main",
                "model.yaml",
            ],
        ),
        "P06",
    );
    json_failure(
        check(
            "module: {name: Main, uses: [absent.yaml]}\nG1: {text: Claim}\n",
            "complete",
        ),
        "I01",
    );
    json_failure(
        check(
            "G0: {text: Root, supportedBy: [G1]}\nG1: {text: A, supportedBy: [G2]}\nG2: {text: B, supportedBy: [G1]}\n",
            "complete",
        ),
        "C04",
    );
    json_failure(
        check(
            "G0: {text: Root, supportedBy: [Sn1]}\nSn1: {text: Evidence}\nG1: {text: A, supportedBy: [G2]}\nG2: {text: B, supportedBy: [G1]}\n",
            "complete",
        ),
        "C08",
    );
}

#[test]
fn deterministic_index_tracks_expansion_and_actual_owners() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child("root.yaml").write_str("module: {name: Root, uses: [sub/develop.yaml]}\nG1: {text: Claim, undeveloped: true, inContextOf: [C1]}\nC1: {text: Context}\n").unwrap();
    temp.child("sub/develop.yaml").write_str("module:\n  name: Developer\n  extends:\n    - module: Root\n      develops: {G1: [Sn1]}\nSn1:\n  text: Inspection evidence.\n  gsnContract: {version: 1, nodeRefs: [{id: G1, type: Goal, owner: ../root.yaml}]}\n").unwrap();
    let args = [
        "--check",
        "--contract",
        "complete",
        "--diagnostics",
        "json",
        "--index-json",
        "nodes.json",
        "root.yaml",
    ];
    let out = run(&temp, &args);
    assert!(out.status.success(), "{:?}", out);
    assert!(out.stdout.is_empty());
    let first = std::fs::read(temp.child("nodes.json")).unwrap();
    let index: Value = serde_json::from_slice(&first).unwrap();
    assert_eq!(index["nodes"].as_array().unwrap().len(), 3);
    assert_eq!(index["nodes"][0]["id"], "C1");
    assert_eq!(index["nodes"][1]["supportedBy"], serde_json::json!(["Sn1"]));
    assert_eq!(index["nodes"][1]["inContextOf"], serde_json::json!(["C1"]));
    assert_eq!(index["nodes"][1]["text"], "Claim");
    assert_eq!(index["nodes"][1]["undeveloped"], false);
    assert_eq!(index["nodes"][2]["module"], "Developer");
    assert_eq!(
        index["nodes"][2]["input"],
        temp.child("sub/develop.yaml")
            .path()
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
    );
    assert_eq!(index["nodes"][2]["metadata"]["nodeRefs"][0]["type"], "Goal");
    assert_eq!(index["version"], 2);
    assert_eq!(
        index["profile"],
        serde_json::json!({"name":"gsn-contract", "version":1, "mode":"complete"})
    );
    assert!(run(&temp, &args).status.success());
    assert_eq!(first, std::fs::read(temp.child("nodes.json")).unwrap());
    let draft_args = [
        "--check",
        "--contract",
        "draft",
        "--diagnostics",
        "json",
        "--index-json",
        "nodes.json",
        "root.yaml",
    ];
    assert!(run(&temp, &draft_args).status.success());
    let draft: Value =
        serde_json::from_slice(&std::fs::read(temp.child("nodes.json")).unwrap()).unwrap();
    assert_eq!(draft["profile"]["mode"], "draft");
}

#[test]
fn draft_index_preserves_effective_fields_and_does_not_claim_completion() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child("model.yaml").write_str("G1:\n  text: A claim still in progress.\n  undeveloped: true\n  classes: [draft-note]\n  gsnContract: {version: 1}\n").unwrap();
    let args = [
        "--check",
        "--contract",
        "draft",
        "--diagnostics",
        "json",
        "--index-json",
        "nodes.json",
        "model.yaml",
    ];
    let draft_run = run(&temp, &args);
    assert!(draft_run.status.success(), "{draft_run:?}");
    let bytes = std::fs::read(temp.child("nodes.json")).unwrap();
    let index: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(index["version"], 2);
    assert_eq!(index["profile"]["mode"], "draft");
    assert_eq!(index["nodes"][0]["text"], "A claim still in progress.");
    assert_eq!(index["nodes"][0]["undeveloped"], true);
    assert_eq!(
        index["nodes"][0]["attributes"]["classes"],
        serde_json::json!(["draft-note"])
    );
    assert!(index["nodes"][0]["attributes"]["nodeType"].is_null());
    assert_eq!(
        index["nodes"][0]["additionalAttributes"]["gsnContract"],
        "version: 1\n"
    );
    assert_eq!(index["nodes"][0]["metadata"]["version"], 1);
    assert!(run(&temp, &args).status.success());
    assert_eq!(bytes, std::fs::read(temp.child("nodes.json")).unwrap());

    let original_model = std::fs::read(temp.child("model.yaml")).unwrap();
    let unsafe_output = run(
        &temp,
        &[
            "--check",
            "--contract",
            "draft",
            "--diagnostics",
            "json",
            "--index-json",
            "model.yaml",
            "model.yaml",
        ],
    );
    json_failure(unsafe_output, "O01");
    assert_eq!(
        original_model,
        std::fs::read(temp.child("model.yaml")).unwrap()
    );
    assert_eq!(bytes, std::fs::read(temp.child("nodes.json")).unwrap());

    let complete = run(
        &temp,
        &[
            "--check",
            "--contract",
            "complete",
            "--diagnostics",
            "json",
            "--index-json",
            "nodes.json",
            "model.yaml",
        ],
    );
    json_failure(complete, "P05");
    assert!(!temp.child("nodes.json").path().exists());

    assert!(run(&temp, &args).status.success());
    let retained = std::fs::read(temp.child("nodes.json")).unwrap();
    temp.child("model.yaml")
        .write_str("G1: [invalid\n")
        .unwrap();
    let invalid = run(&temp, &args);
    let diagnostics = json_failure(invalid, "I02");
    assert!(
        diagnostics["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["code"] == "O02")
    );
    assert_eq!(retained, std::fs::read(temp.child("nodes.json")).unwrap());
}

#[test]
fn missing_strategy_support_and_invalid_effective_extension() {
    for marker in ["", ", undeveloped: true"] {
        json_failure(
            check(
                &format!("G1: {{text: Claim, supportedBy: [S1]}}\nS1: {{text: Reason{marker}}}\n"),
                "complete",
            ),
            "P05",
        );
    }
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child("root.yaml").write_str("module: {name: Root, uses: [develop.yaml]}\nG1: {text: Claim, supportedBy: [S1]}\nS1: {text: Reason, undeveloped: true}\n").unwrap();
    temp.child("develop.yaml").write_str("module:\n  name: Developer\n  extends:\n    - module: Root\n      develops: {S1: [Sn1]}\nSn1: {text: Evidence}\n").unwrap();
    json_failure(
        run(
            &temp,
            &[
                "--check",
                "--contract",
                "complete",
                "--diagnostics",
                "json",
                "root.yaml",
            ],
        ),
        "P03",
    );
}

#[test]
fn preserves_arbitrary_visual_layers_and_input_files() {
    let temp = assert_fs::TempDir::new().unwrap();
    let text = format!(
        "{COMPLETE}  arbitraryLayer: {{color: blue}}\n  gsnContract: {{version: 1, sources: [{{path: source.md}}]}}\n"
    );
    temp.child("model.yaml").write_str(&text).unwrap();
    temp.child("source.md")
        .write_str("Source material")
        .unwrap();
    let rendered = run(
        &temp,
        &[
            "--contract",
            "complete",
            "--layer=arbitraryLayer",
            "--no-evidence",
            "model.yaml",
        ],
    );
    assert!(rendered.status.success(), "{rendered:?}");
    assert!(
        std::fs::read_to_string(temp.child("model.svg"))
            .unwrap()
            .contains("color: blue")
    );
    for protected in ["model.yaml", "source.md"] {
        let before = std::fs::read(temp.child(protected)).unwrap();
        json_failure(
            run(
                &temp,
                &[
                    "--check",
                    "--contract",
                    "complete",
                    "--diagnostics",
                    "json",
                    "--index-json",
                    protected,
                    "model.yaml",
                ],
            ),
            "O01",
        );
        assert_eq!(before, std::fs::read(temp.child(protected)).unwrap());
    }
    temp.child("model.yaml")
        .write_str(&text.replace("version: 1", "aCustomVisual: blue"))
        .unwrap();
    assert!(run(&temp, &["--check", "model.yaml"]).status.success());
    assert!(
        run(&temp, &["--check", "--diagnostics", "json", "model.yaml"])
            .status
            .success()
    );
}

#[test]
fn invalid_document_roles_and_aliased_inputs_are_diagnostics() {
    json_failure(check("G1: {name: Unexpected module}\n", "complete"), "I03");
    json_failure(
        check(
            "module: {text: Unexpected node}\nG1: {text: Claim}\n",
            "complete",
        ),
        "I03",
    );
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child("model.yaml").write_str(COMPLETE).unwrap();
    let absolute = temp.child("model.yaml").path().canonicalize().unwrap();
    json_failure(
        run(
            &temp,
            &[
                "--check",
                "--contract",
                "complete",
                "--diagnostics",
                "json",
                "model.yaml",
                absolute.to_str().unwrap(),
            ],
        ),
        "C06",
    );
}

#[test]
fn invalid_metadata_cannot_destroy_declared_evidence() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child("model.yaml")
        .write_str(&format!(
            "{COMPLETE}  gsnContract: {{version: invalid, evidence: [{{path: evidence.json}}]}}\n"
        ))
        .unwrap();
    temp.child("evidence.json")
        .write_str("Evidence to preserve")
        .unwrap();
    json_failure(
        run(
            &temp,
            &[
                "--check",
                "--contract",
                "complete",
                "--diagnostics",
                "json",
                "--index-json",
                "evidence.json",
                "model.yaml",
            ],
        ),
        "M01",
    );
    assert_eq!(
        std::fs::read_to_string(temp.child("evidence.json")).unwrap(),
        "Evidence to preserve"
    );
}
