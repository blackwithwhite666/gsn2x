# Model review method

GSN Community Standard Version 3, section 2:7.4, gives the five review steps. This reference applies that order to a model used as a contract. The standard remains authoritative.

## 1. Understand the argument

Read the root claim and all modules in its declared scope. Identify the required user result, the subject, and the stage of work. Read the source requirements. Do not infer the full requirement set from the model alone.

Identify defined terms, contexts, assumptions, and exclusions. Context and Assumption elements apply through their supported subtree. A Justification explains a local choice. Do not treat it as an inherited premise.

Record uncertainty that prevents a conclusion. Continue independent checks when possible. Do not fill a missing definition with a plausible interpretation and then report it as a fact.

## 2. Check structure and coverage

On a matching version of this fork, run `gsn2x --check --contract complete --diagnostics json` on the declared entrypoint. Inspect version/help before choosing flags. Record the command, version or binary identity, exit status, and output. If only upstream checks are available, record their narrower scope. If the tool is unavailable, report that limit and perform supported manual checks; this does not establish deterministic completion.

Check the effective graph after module expansion. Check unique IDs, resolved references, reachability, and support cycles. Core v3 permits these support edges:

- Goal to Goal, Strategy, or Solution.
- Strategy to Goal.

Only Goals and Strategies can have Context, Assumption, or Justification edges. Some versions of `gsn2x` accept more support types. A native pass does not override the standard.

Check every custom requirement, property, verification, and evidence reference that the model actually uses. Resolve its type, owner, namespace, subject, and revision where needed. Reject ambiguity. Do not require invented fields or rewrite a model into a proposed schema.

Account for each applicable source requirement and each clause within it. Trace it to the claims that address it. A mapping proves that a reference exists. It does not prove that the mapped claim covers the requirement's meaning.

Identify undeveloped Goals and Strategies. Keep the full declared scope when an external module or input is unavailable. Do not exclude a missing branch to get a pass.

## 3. Review prose and observable results

Apply the STE references to the declared English prose. Include editable headings, labels, tables, and HTML text when they belong to the review scope. Keep quoted source text separate. Classify descriptions and procedures before checking their different rules.

Use LLM judgement for every Goal. Read the Goal text, inherited context, and supporting branch. Determine:

- What result the claim states.
- Which subject and conditions the claim covers.
- What observation, analysis, proof, or inspection could confirm the result.
- What observation or counterexample would contradict it.

These questions guide the reviewer. They do not require additional fields in the model. A precise Goal sentence and adequate context can answer them.

A binary result is measurable. A set or relation can have an exact comparison. A state transition can have a required final state. Numeric claims need units, conditions, and an actual required limit. Do not add numeric metrics to a claim that already has an exact criterion.

For a parent Goal, check its own stated result and the inference from its children. Reuse child verification where it supports that inference. Do not require a separate parent test solely because the Goal has children.

Distinguish a precise, testable claim from an executed check. A described method can establish how to check a future implementation. Only actual evidence establishes a claim about a completed run.

If a phrase such as “fast,” “sufficient,” or “correct” has no definition that makes it checkable, report the missing criterion. Suggest a correction that preserves the intended requirement. Do not choose a threshold or silently substitute a weaker claim.

Separate a confirmed defect from an unresolved interpretation. Quote the relevant text and explain its effect. Do not fail a Goal because its result lacks a dedicated metadata block.

## 4. Challenge the argument

Assess coverage, dependency, definition, directness, relevance, and robustness. For each inference, seek a case where its premises hold and its conclusion lacks support. Check whether the Strategy hides a premise or merely lists work.

Check inherited scope and assumptions. A narrow test domain cannot support an unrestricted parent claim. Multiple items based on the same flawed model are not independent evidence. A citation to a requirement does not establish that the implementation meets it.

Assess each verification method. The expected result must have a justified source. Comparing an implementation with its own answer can hide the same error twice. Check boundary cases, faults, and goal-specific counterexamples.

Where a test or automated verifier is used, inspect controls that should make it fail. Use actual available controls or propose a focused missing control. For analysis or proof, seek a counterexample or examine the stated premises. Do not demand executable mutation tests for every kind of evidence.

Check for a vacuous pass. Conditions must be possible, tests must exercise the claimed behavior, and omitted inputs must remain visible. Finite tests do not prove a universal claim. An accepted model can use reasoned engineering evidence within an explicit scope.

Record a challenge against the exact claim, inference, or evidence item. Explain its support and effect. A possible doubt does not automatically defeat the whole argument. A supported material challenge blocks the affected acceptance claim until resolved.

## 5. Audit evidence

Open the evidence actually cited by each relevant Solution. Identify what it establishes and what it leaves open. Inspect the real output of a check. Do not substitute a file name, plan, status flag, or author's success message.

Check the subject, declared inputs, revision, checker configuration, applicable environment, and limits. Compare the relevant input closure with the evaluated candidate. Preserve the source contract's canonical identity and dependency rules.

An older result can remain applicable when relevant inputs and conditions are unchanged. A new review date alone does not make it applicable. A changed relevant input invalidates affected conclusions. An unavailable required input prevents a confirmed pass.

Judge whether the method and evidence support the strength of the Goal. Record counterevidence and trust limits. Keep evidence about a recorded choice, a model, and running code distinct.
