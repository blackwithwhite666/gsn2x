# Opt-in GSN contract profile (version 1)

Ordinary commands retain upstream checks and visual layers. Select `--contract draft`
while authoring or `--contract complete` for structural completion. Both modes check
the effective graph after `uses` and `extends` expansion: support is Goal →
Goal/Strategy/Solution or Strategy → Goal; context is Goal/Strategy →
Context/Assumption/Justification. Other node types are outside this core profile.
Every node needs nonblank text. This is a structural check, not a judgement of
meaning, checkability, coverage, inference strength, or evidence truth.

Draft permits unsupported and undeveloped Goals/Strategies. Complete rejects both,
even without an `undeveloped` marker. Removing the marker does not complete a claim.
All upstream reference, cycle and reachability checks remain required. `--exclude`
is rejected in either contract mode. Missing imported modules fail input loading.

## Typed metadata wire form

In `model.yaml`, the optional `gsnContract` node attribute is a versioned object, written as a YAML
mapping (or its YAML/JSON text in a block scalar). It uses the existing additional
attribute storage, so unrelated visual layers remain available. Only these fields
are accepted; unknown fields, wrong shapes, and versions other than 1 fail:

```yaml
G1:
  text: Each local reference identifies an existing object of the required type.
  supportedBy: [Sn1]
  gsnContract:
    version: 1
    sources:
      - path: requirements.md
        anchor: local-references
    nodeRefs:
      - id: Sn1
        type: Solution
        owner: model.yaml
Sn1:
  text: Inspection of the local reference cases supports this claim.
  gsnContract:
    version: 1
    evidence:
      - path: inspection.md
```

`version` is required when the attribute is present. `sources`, `evidence`, and
`nodeRefs` are optional lists. File entries require nonblank `path`; `anchor` is
optional and must be nonblank. Paths resolve relative to the actual YAML input
owner, not the current working directory or module display name. They must identify
readable regular local files. Anchors are supported only as explicit Markdown
heading IDs (`# Heading {#local-references}`) or HTML `id="local-references"`
(or single quotes). Automatic heading slugs, PDF page selectors, remote URLs,
external ADR references and transitive Git evidence freshness are unsupported.
Unsupported schemes fail; no successful check certifies external resolution or
freshness. Existence is not proof of relevance or successful execution. Existing node `url`
fields remain visual hyperlinks; the profile does not certify their targets.

Node entries require `id` and `type` (Goal, Strategy, Solution, Context, Assumption,
or Justification). The ID must exist in the expanded input set with that actual
type. Optional `owner` resolves relative to the referencing input and must match
the target node's actual canonical input path. Metadata links do not create support
edges or causal dependencies. Source completeness remains a reviewer obligation.
No `outcome`, numeric metric, or duplicate parent verification record is required.

## Commands and output

```sh
gsn2x --check --contract draft model.yaml
gsn2x --check --contract complete --diagnostics json --index-json nodes.json model.yaml
```

`--diagnostics json` writes exactly one JSON document to stderr, with `version: 1`
and `diagnostics`: an array of records with `code`, `input`, `module`, `node`,
`field`, `severity`, and `message`. Unknown locations are null. Codes are stable;
messages may change. I01/I02 mean missing/unreadable input and invalid YAML; I03 means an invalid
document role; P*
are profile errors; M* are metadata errors; O01 is output failure or an unsafe output
destination; O02 means incomplete input/reference discovery with an index requested. Upstream C*/V*
retain their codes, with GSN_CHECK for unclassified checks. Use `--check` without statistics/YAML-dump outputs to keep
stdout empty; rendering otherwise retains its usual stdout progress. CLI argument
errors are clap's ordinary text, before input validation. Exit 1 denotes
invalid inputs/checks or an output failure; clap argument errors exit 2. Help and
version queries exit 0. Warnings alone do not fail. Without
new output flags, legacy text diagnostics remain available.

`--index-json FILE` requires either contract profile and writes only after all checks
for that profile pass. Draft indexes may contain unsupported and undeveloped
Goals/Strategies, so they do not mean the graph is complete. Contract mode checks all actual output destinations before any output mutation:
module graphs, combined views, evidence lists, statistics, YAML dumps, index staging,
and copied stylesheets. Destinations cannot alias inputs, imports, declared local
source/evidence/owner paths, or stylesheet sources, through normalized paths,
symlinks, or hardlinks. Conflicting output destinations also fail.

Cleanup removes an existing index only after complete input/reference discovery and
successful output preflight, and only if it matches the version 2 derived index
shape, explicit `gsn-contract` profile, and canonical owners from the loaded modules. An arbitrary existing file is
preserved and rejected. Typed `attributes` must match the model's serialized
field names and value shapes; unknown keys belong in `additionalAttributes`. If YAML or metadata parsing prevents complete discovery,
all existing outputs are preserved; O02 reports that a requested existing index is
not current. A fully discovered but invalid graph can remove an identified prior
index without writing a replacement. Consumers must require exit 0 for the current
inputs: retained outputs after failure cannot certify those inputs. A version 1
index is not recognized or removed; choose a fresh exclusive output path instead
of deleting an earlier file without checking its ownership. Do not use an input or
evidence file as an output path.

The index is a derived document with `version: 2`, a `profile` object, and `nodes`,
ordered by ID. The profile is `{name: "gsn-contract", version: 1, mode: "draft"}`
or the same object with `mode: "complete"`; it reports which structural check
passed. Each node contains `id`, `type`, `module`, `input` (canonical owner path),
effective `text`, `undeveloped`, sorted `supportedBy` and `inContextOf` relations,
typed non-graph rendering `attributes`, opaque string-valued
`additionalAttributes`, and parsed GSN contract `metadata` (or null). Effective
module extensions are included. The consumer uses these actual expanded nodes and
does not repeat expansion or resolve owners from comments. The index adds no hashes
or new identity registry. It records structure; it is not a semantic success receipt.

Example draft index (the input path is canonicalized at runtime):

```json
{"version":2,"profile":{"name":"gsn-contract","version":1,"mode":"draft"},"nodes":[{"id":"G1","type":"Goal","module":"Main","input":"/absolute/model.yaml","text":"A claim still in progress.","undeveloped":true,"supportedBy":[],"inContextOf":[],"attributes":{},"additionalAttributes":{},"metadata":null}]}
```

The index cannot attest source coverage, implementation evidence, claim meaning,
or complete contract validity. Arbitrary metadata and visual layers stay descriptive.
Independent review must bind its verdict to the exact source, model, evidence,
standard and vocabulary inputs it inspected.

## Profile diagnostic codes

| Code | Meaning |
| --- | --- |
| P01 | Node type outside the core profile |
| P02 | Blank node text |
| P03 | Invalid effective support edge |
| P04 | Invalid effective context edge |
| P05 | Undeveloped or unsupported Goal/Strategy in complete mode |
| P06 | Attempt to exclude contract reference checks |
| M01 | Malformed metadata or blank reference value |
| M02 | Unsupported metadata version |
| M03 | Missing referenced GSN node |
| M04 | Wrong expected or actual node type |
| M05 | Wrong or missing declared node owner |
| M06 | Missing, unreadable, or non-file local target |
| M07 | Missing explicit anchor or unreadable text target |
| M08 | Unsupported external reference scheme |
| O01 | Output failure or unsafe/conflicting destination |
| O02 | Incomplete discovery; outputs preserved and requested index is not current |

Upstream validation can reject an invalid authored edge with V04 before profile
checks run. P03 also checks edges introduced by module development. A failed check
can stop dependent checks; an empty list of findings from those skipped checks is
not an assessment of their scope.
