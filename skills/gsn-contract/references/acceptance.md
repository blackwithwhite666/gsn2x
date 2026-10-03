# Acceptance and reporting

## Verdicts

Report the recorded decision, model contract, and implementation separately. Use `satisfied`, `not_satisfied`, or `not_assessed` for each requested acceptance claim. Explain the exact scope and basis. Do not average findings into a score.

A recorded decision identifies the choice and its applicable authority. It does not establish the adequacy of the design.

A complete model contract requires:

- Every applicable source requirement is accounted for within the declared scope.
- Each Goal states a result that the reviewer can understand and check.
- Context defines the needed terms, subject, conditions, and limits.
- Every relevant Goal and Strategy has a complete supporting argument.
- No relevant Goal or Strategy is undeveloped.
- Reviewed child claims support each parent result.
- Cited evidence supports the model claims at their stated strength.
- Language review covers the applicable rules and required vocabulary evidence.
- All five review stages have been completed for the inspected candidate.
- No unresolved material finding or missing required evidence remains.

The result may be stated entirely in prose. Existing evidence and child checks can support parent Goals. Do not require separate outcome records, numeric metrics, or duplicate checks.

Future implementation tests may remain planned in a design contract. Their criteria and methods must be clear enough to evaluate the required behavior. They cannot support a Goal that claims a successful execution. Model claims still require applicable evidence, including analysis or inspection where sufficient.

Implementation verification requires the applicable tests, properties, proofs, and resource obligations from the actual source contract. Check their results, input bindings, and strength. Preserve the selected proof gate. Model review cannot waive it.

A draft can contain undeveloped nodes and unresolved choices. Those facts prevent acceptance as a complete contract. Removing a marker does not create supporting evidence.

## Review record

Start with the outcome and the consequence for the reader. Record:

- Exact model revision or file identities and the declared scope.
- Source requirements, standard editions, glossary, and dictionary references used.
- All reviewed Goal IDs and any Goal not assessed, with the reason.
- Actual mechanical commands and their results.
- Findings from all five stages, including semantic coverage and evidence applicability.
- The separate verdicts, unresolved questions, and limitations.

For each finding, identify the file and node or stable source location, review stage, violated rule or requirement, observation, impact, and smallest correction. A correction must preserve meaning. Label missing information as unresolved rather than guessing its content.

Group passing results when that helps the reader. Give enough coverage information to establish that every Goal was read. Use an existing structured report format if the project has one. Do not require a new model schema or a separate result registry to perform this review.

## Final candidate

The author works from the declared source MD and requirements. The reviewer evaluates the resulting candidate independently. Deterministic checks precede semantic review. The five-stage review still checks the actual scope of those mechanical results.

After each repair, repeat invalidated checks and review the affected claims, parents, and evidence. Do not repeat unchanged checks solely because another turn occurred. Preserve unresolved findings between iterations. Do not remove a requirement or weaken a criterion to close a finding.

Deliver a final model only when its exact revision meets the completion conditions and all material findings are resolved. Bind the final review to that revision. Do not promote a draft on the basis of a native pass or the absence of new comments.

## Corrections and repeat review

Apply changes only within the requested correction scope. Repeat checks affected by those changes, including nearby terms, inherited context, and parent inferences. Review unchanged evidence again only when applicability is uncertain.

Do not claim full ASD-STE100 compliance if dictionary, glossary, or rule coverage is incomplete. Do not claim contract completion if a material part of the argument remains unassessed. Report the missing work and its effect on the requested verdict.

A review recommends acceptance within its inspected scope. Preserve any human acceptance decision required by the project.
