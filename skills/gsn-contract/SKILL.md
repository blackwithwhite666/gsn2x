---
name: gsn-contract
description: Generate, update, or independently review a GSN v3 contract from authoritative Markdown, with structural checks, source coverage, ASD-STE100 language review, inferences, and actual evidence.
---

# GSN contract

Use `generate` for a new model, `update` to change an existing model while preserving
stable IDs, and `review` for an independent assessment. Read the authoritative MD,
applicable machine contracts, scope, glossary, and available evidence. Keep chosen
decisions distinct from proposals, requirements from assumptions, and design claims
from claims about executed verification. Do not reduce source scope to pass a check.

A Goal states a checkable result in prose, with context where needed. Do not require
an outcome block, a numeric metric for a nonquantitative claim, or duplicate parent
tests. Strategy text explains why the children establish the parent. Solution text
identifies relevant real evidence. Missing decisions or evidence remain unresolved;
a planned test never proves an executed result.

Read [the author and completion loop](references/workflow.md) for generate/update.
Read [the five review stages](references/model-review.md) and
[acceptance and reporting](references/acceptance.md) for review. For canonical
English, also read [the STE method](references/ste-review.md) and
[the bundled STE rule summaries](references/ste-rules.md). Check vocabulary against
the actual ASD-STE100 Issue 9 dictionary and the applicable subject glossary.
The summaries do not replace the dictionary or establish compliance by themselves.

Use the configured gsn2x executable and inspect its version/help. For this fork,
[the profile and CLI contract](references/profile.md) defines supported checks,
metadata, and the derived index. Use the bundled profile only with a matching executable;
if the installed tool differs, obtain its supported schema before authoring metadata. Run actual commands and record results. A structural pass
cannot establish semantic coverage, evidence relevance, or freshness.

The author and reviewer use separate Codex sessions with this same skill. Pass the
reviewer the exact candidate and source inputs and request `review` mode. Use the
available authorized launcher/model; no specific launcher or model is required.
The author cannot label its own review independent. If separate review or required
references are unavailable, report an incomplete contract and the missing work.
Do not launch sessions when the task forbids delegation.

Completion requires checks on the final inputs, full source coverage, all five
review stages, STE rules and vocabulary review, checkable Goals, sufficient
inferences, relevant evidence, no undeveloped branches, and no material unresolved
findings. Report recorded decision, complete design contract, and verified
implementation as separate verdicts. Bind every final verdict to exact inspected
inputs. Preserve project-specific human acceptance gates.
