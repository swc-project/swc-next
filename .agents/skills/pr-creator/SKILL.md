---
name: pr-creator
description: Use when asked to create a pull request for this repository. Follow its branch safety rules, title convention, pull request template, reference policy, and concise English writing style. Create draft PRs by default after a dry run and explicit human approval.
---

# Pull request creator

## Steps

1. Confirm the current branch with `git branch --show-current`.
   If it is the default branch, create and switch to a new branch before doing anything else.
   Use a descriptive branch name, preferably `feat-<topic>` or `fix-<topic>`.

2. Review local changes with `git status --short`.
   Do not revert unrelated user changes.
   Before creating the PR, ensure the intended changes are committed and never commit directly on the default branch.

3. If `.github/PULL_REQUEST_TEMPLATE.md` exists, read it and follow its structure.

4. Draft the PR title in the repository's standard format. If the repository uses Conventional Commits, common patterns include:
   - `feat(core): add ...`
   - `fix(types): ...`
   - `docs: ...`
   - `refactor(types): ...`
   - `chore(deps): ...`
   - `release: v1.2.0`

5. Write the PR body in concise, clear English.
   - In `Motivation`, explain the user-facing problem, maintenance goal, or compatibility constraint that makes the change necessary.
   - In `Changes`, describe what the PR changes and how it addresses the motivation. Include implementation details only when they help reviewers assess the change.
   - Prioritize high-signal information: public API changes, behavior changes, breaking changes, migration notes, and important compatibility implications.
   - Keep each section short: one compact paragraph or a few bullets is usually enough.
   - Include relevant verification results in `Changes` when they help reviewers assess correctness, along with material limitations. Avoid separate `Test plan` or `Validation` sections, exhaustive command lists, generated file lists, and obvious implementation details unless the repository template requires them.
   - Good background examples:
     - `This PR adds support for custom logger injection so CLI output can be isolated per instance.`
     - `This PR fixes incorrect padding in URL labels to keep terminal output aligned across different label lengths.`
     - `This PR updates the English docs to clarify how the extraction option works and when to enable it.`

6. Fill `Related Links` according to the reference policy below, or omit the section when no references are needed.

7. Perform a mandatory **dry run** before pushing or creating the PR:
   - Refresh the base branch's remote-tracking ref and compute the merge-base commit with `git merge-base <base-ref> HEAD`.
   - Print the exact title, target repository, base branch, merge-base SHA, head branch and SHA, draft status (`draft` by default), and full Markdown description in the conversation.
   - Wait for explicit human approval of this preview (the initial request to create a PR does not count). Do not push or create any PR during the dry run.
   - Repeat the dry run and obtain approval again if any of these fields change.

8. After approval, re-check the branch name (never push the default branch), push the head branch, write the approved Markdown body to a temporary file, and run `gh pr create` (`--draft` by default unless ready-for-review was explicitly requested and approved) with explicit `--repo`, `--base`, `--head`, `--title`, and `--body-file` flags.

## Reference policy

- Format `Related Links` as a Markdown bullet list. For GitHub issues and PRs, prefer plain refs (`- #123` or `- Close #123` for the current repository; `- owner/repo#123` for another repository) over full URLs or code spans; other resources (such as specifications, design docs, commits, tags, or release notes) may use Markdown links or URLs when needed.
- Cite only existing, directly supporting refs from these official sources (not forks, mirrors, or unrelated repositories in the same organization):
  - SWC: repositories owned by `swc-project`.
  - Upstream and downstream: official project repositories for Rspack, Next.js, Node.js, and Deno.
  - Standards, references, and JS engines: official project repositories for Test262, Babel, ESTree, and TC39 proposals, plus official V8, JavaScriptCore, and SpiderMonkey implementation repositories.
- Apply these source restrictions across the entire PR body (`Motivation`, `Changes`, and `Related Links`).
- Repositories outside the allowed list cannot serve as evidence for a change. Any change originating from an external source must first be discussed and approved in an `swc-project` issue (with an independent reproduction or specification basis), and the PR must reference that `swc-project` issue instead. If no such issue exists yet, ask the user to open one for discussion and wait for its ref before creating the PR; never create the issue automatically.

## Constraints

- Do not modify code while following this skill.
