# Generate and update

1. Identify the authoritative MD, full required scope, current model (if any),
   standard editions, language dictionary, subject glossary, related schemas,
   and available evidence. Read the source independently of an existing claim list.
   Inventory applicable requirements, including the parts of compound requirements.
   Record decisions and missing inputs; do not supply invented answers.
2. Author the argument in canonical English under ASD-STE100 Issue 9. Use G/S/Sn
   support and G/S → C/A/J context. Account for each applicable source requirement
   and its origin with the small documented metadata profile where supported.
   Source coverage can be explained in the author report; it is not a second authored
   node registry. Keep evidence and formal goals distinct from GSN prose Goals.
3. In update mode, preserve IDs for unchanged entities. Explain additions, removals,
   splits and changed meaning. Identify affected references, child and parent
   inferences, assumptions, evidence, and prior reviews. An old verdict does not
   apply automatically after an input changes.
4. Run `gsn2x --check --contract draft --diagnostics json MODEL` while authoring.
   Fix actual errors and repeat affected checks. Keep missing support visible.
   Before completion run the complete profile and every other applicable project
   check, including the actual ADR resolver if supplied. Do not silently certify
   unsupported external references. Record executable/version, commands, exits,
   and outputs. A plan for running commands is not their result.
5. Send the exact candidate to a separate Codex session with this skill in `review`
   mode. Supply the full source MD, modules, contracts, evidence, dictionary and
   glossary, and actual check results. The reviewer independently reads the source
   and applies all five GSN v3 review stages. It must detect omitted source
   requirements even when the graph passes structural checks.
6. Carry material findings forward. Repair in scope without weakening a requirement
   or substituting a convenient criterion. Repeat affected deterministic checks;
   have the separate reviewer recheck changed claims, parents, evidence and language.
   If a finding needs a decision or unavailable evidence, report the unresolved
   obligation and next action. Repeating the same review does not close it.
7. Close only when complete-profile and applicable project checks pass on the exact
   final candidate, the whole required scope has been reviewed, STE rules and actual
   vocabulary references have been checked, and all material findings are resolved.
   Record exact identities for model, source, modules, evidence and vocabulary
   inputs (for example file digest or repository revision plus dirty-file digests).
   These bindings belong in the review record, not in an invented GSN registry.
   Any later input change invalidates the affected verdict until it is rechecked.

Deliver GSN, actual diagnostics/check results, and a final review record. Explain
whether the decision is recorded, the design contract is complete, and the
implementation is verified. A design may define clear future implementation tests,
but those plans cannot support a claim that verification already succeeded.
