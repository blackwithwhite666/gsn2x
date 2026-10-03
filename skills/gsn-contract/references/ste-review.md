# Review method

## Reference and scope

Use [ste-rules.md](ste-rules.md) to check writing against ASD-STE100 Issue 9. Use the Issue 9 controlled dictionary to check vocabulary and the applicable subject glossary to check technical terms.

For dictionary use, check each word's entry, part of speech, forms, meanings, alternatives, and contextual restrictions. An alternative can have a different part of speech or can be a technical noun or verb. Check its meaning in the sentence. The dictionary is not a full list of technical nouns or verbs.

## Review record

Default to review and report with suggested corrections. Apply changes only when the user requests rewriting or correction.

Before review, record:

- Target revision or document identity, and the review date.
- Exact files, sections, or passages in scope, including exclusions.
- Review mode: procedural, descriptive, or mixed; classify each passage by its function.
- Issue 9 dictionary source and edition, applicable company/industry/subject glossary, and any official style or terminology directives.
- Method used to inspect prose and preserve code, identifiers, schema syntax, and exact quotations.
- Rules reviewed and any rules not applicable, with a reason; unresolved vocabulary and missing evidence.

Inspect prose where it appears, including titles, headings, labels, tables, notes, warnings, captions, ADRs, and human-readable HTML. In the declared HTML scope, inspect editable natural-language labels, including text and natural-language attribute values; their presentation does not exempt them from review. Keep quoted or fixed source text unchanged where the standard or scope requires it; identify quoted text separately from editable prose. Preserve surrounding code, program identifiers, markup, schema syntax, and exact quotations unless the user puts them in scope for correction.

## Vocabulary checks

For each ordinary word that affects compliance, verify its dictionary entry, approved part of speech, approved meaning, and approved forms in the Issue 9 dictionary. A word being familiar, short, or commonly used does not prove that its use is approved. Where a replacement changes part of speech or cannot preserve meaning word for word, suggest a different sentence construction. If correction is requested, apply it and recheck the result for meaning.

For technical nouns and verbs, consult the actual glossary or terminology source for the target subject. Check whether the term is an approved technical term for that context and whether the text uses it consistently and in the permitted grammatical role. The categories and examples in Rule 1 help identify candidates; they do not approve every similar term. If the applicable glossary is missing, mark the term unresolved and the related coverage pending. Do not guess dictionary entries or construct a reusable vocabulary database.

## Review and findings

Apply all rules that bear on the text. Check both local sentence issues and document-level terminology, paragraph structure, and consistency. Do not review only short or conspicuous sentences. Distinguish requirements for procedures from requirements for descriptions, including their different sentence limits and the rule on imperative forms. Check passive voice against the specific exception in Rule 3.6.

Count words according to Rules 8.4–8.7, not by a word processor's default counter. In particular, account for colons in vertical lists, parenthetical text, numbers and numbers with units, abbreviations, alphanumeric identifiers, quoted text, titles and headings, placard/label text, proper nouns, and hyphenated words. Apply the applicable sentence limit after these rules: 20 words for procedural sentences and 25 for descriptive sentences; note sentences have their own limit in Rule 5.5. Count a paragraph's sentences and check the six-sentence maximum for descriptive text.

For Rule 8.4, count the introduction before a list's colon and each following item as separate units for the applicable word limit. For Rule 6.6, count the paragraph's grammatical sentences; list fragments do not each add a sentence. An introduction with three list fragments counts as one sentence. A preceding sentence plus an introduction with four fragments counts as two.

For Rule 8.6, fixed titles, headings, and placard/label text that cannot be changed count as one word when referenced or quoted in another sentence. Review editable headings and labels as prose in their own right. Count their words separately.

For each known violation, report:

- A stable source location (file and line, heading, page, or quoted short span).
- The Rule ID and the specific requirement not met.
- A suggested correction that preserves the technical meaning, or a concise explanation of why context from the author is required before correction.

Separate proven violations from unresolved vocabulary and other missing evidence. When rewriting or correction is requested, apply corrections and re-review changed text, nearby references, and any affected terminology for unintended meaning changes. A short sentence, a successful automated check, or a list of issues found does not establish complete rule coverage or full compliance.

## Completion statement

State the target revision and scope, the reference and method, the rule coverage, known violations and their disposition, unresolved vocabulary, and missing evidence. Say that the review is partial or pending whenever required dictionary, glossary, or scope evidence is absent. Claim full compliance only after the declared natural-language scope has been reviewed against every relevant rule and required vocabulary evidence, and known violations have been corrected and rechecked.
