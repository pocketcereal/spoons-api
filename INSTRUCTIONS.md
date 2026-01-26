# Ralph Loop

*If the serena mcp server is avail, make sure to reference the LSP mappings. If Context7 mcp is avail for documentation, rely on this for looking up API's and potential issues. Using any agents, mcps, skills at your disposal do the following tasks.*

- Run the /comprehensive_review command for an overview of potential <ISSUE>s.
- Identify the <ISSUE>s that have only one obvious / best solution and fix them.
- For the rest, write these to a docs/REVIEW.md in the form of:

```md
# <IDENTIFIED <ISSUE>>
<CONCISE DESCRIPTION>

- ### <OPTION TO FIX 1..N>

#### <LIKELY BEST SOLUTION>
```

- Systematically re-review the docs/REVIEW.md file, if external libraries are involved, confirm proper API usage. If external REST/GraphQL API's are involved, confirm proper schemas / response types.
- Issues that remove dead/duplicated code should just be removed. Update the CHANGELOG.md
- Identify the coding pattern used around this <ISSUE>, for example: _An issue has been identified that has an optimization to a programming pattern where properties are gathered to create one interface. This might be identified as a builder pattern_. Once the overall pattern has been identified, review the related code as a whole and follow symbols to uncover any seemingly unrelated but important details that effect the issue that had been identified.
- Your goal for each of these re-reviews is to have a focused context of the architecture of the project and patterns as a whole that influence the issue or perhaps require the <ISSUE> to be re-framed into another <ISSUE> or broken into several. Once these are identified, update the docs/REVIEW.md item for this <ISSUE> with enriched context.
- If at this point a <LIKELY BEST SOLUTION> is not clear, provide a report in the file docs/UNCLEAR_SOLUTIONS.md to be addressed later.
- If the re-review uncovers that the issue requires a large refactor, provide a concise detail of why and potential paths forward for this in a file: docs/LARGER_REFACTOR.md and continue.
- Fix the <ISSUE>s that have clear solutions, for each run `task check` and any other tests, update the CHANGELOG.md
- Follow the fixed code's symbols, where variables are used elsewhere and related functions to identify any duplicate or dead code and remove it.
- Review any project READEME.md, AGENT.md or user document and systematically update any stale information.
- Update the original docs/REVIEW.md to reflect the state of the items. add checkmarks or "**DONE**" headers if that makes more sense.
- Commit the fixes to the issues that were clear with the prefix [REVIEW FIXES]: <desc>.
- For each item in the docs/LARGER_REFACTOR.md systematically review each one. Break the potential solutions into their smallest constituants. For each element trace all code paths to build an accurate tree of dependencies (functions, structs, external libraries, etc) and begin to create a clear step by step plan for the refactor. Provide one or two potential ideas / paths forward and research which fits best with the code style and patterns laid forth then save / update these notes in a docs/[DOMAIN_OF_REFACTOR]_REFACTOR.md file. You will reference this throughout the refactoring process.
- Start the refactor following the instructions in the docs/[DOMAIN_OF_REFACTOR]_REFACTOR.md file.
- Commit often with the prefix [refactor]: [DOMAIN-OF-REFACTOR]-<short-desc> and run all checks and tests after each step and update the CHANGELOG.md
- Once complete, review each document created in the docs/* dir. Remove any completed elements, delete any files that are not necessary any more.
- Commit with the prefix [refactor]: docs-cleanup-<N> where N is the itteration of cleanup.
- Update the project docs (README.md, AGENTS.md) and commit.
- Start this loop over. This loop is considered complete when no documentation / items exist in the `docs/*` dir because we have completed all of the items. when done print a newline with the text `<promise>DONE</promise>`