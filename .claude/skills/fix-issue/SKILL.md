---
name: fix-issue
description: Fix a GitHub issue end-to-end — fetch the issue, understand the problem, run baseline tests, implement the fix, verify with tests, create a branch, commit, and open a PR. Use this skill when the user asks to fix an issue, resolve a bug, or address a GitHub issue by number or URL. Trigger phrases include "fix issue #", "修复 #", "resolve #", "fix #", or when they paste a GitHub issue URL with intent to fix it.
disable-model-invocation: true
---

This skill implements a complete issue-fixing workflow: fetch → understand → baseline → fix → verify → branch → commit → PR.

The user triggers this skill by providing a GitHub issue reference — either an issue number (`#42`) or a full URL (`https://github.com/owner/repo/issues/42`). `$ARGUMENTS` contains the user's full input including the issue reference.

## Workflow

Follow these steps in order. If any step fails, stop and report the failure to the user before continuing.

### Step 1: Parse the issue reference

Extract the issue number from `$ARGUMENTS`:
- `fix issue #42` → number is `42`
- `修复 #42` → number is `42`
- `https://github.com/owner/repo/issues/42` → number is `42`
- If the input is ambiguous, ask the user to clarify.

Also extract the remote repository:
- If a full URL is provided, use the `owner/repo` from it.
- Otherwise, use `git remote get-url origin` to determine the current repo.

### Step 2: Fetch the issue

```bash
gh issue view <number> --repo <owner/repo> --json number,title,body,labels,milestone,assignees
```

Read the issue carefully. Understand:
- **Title**: Brief description of the problem
- **Body**: Full description, steps to reproduce, expected vs actual behavior, environment details
- **Labels**: Any categorization hints (bug, enhancement, priority)
- **Context**: Any linked PRs, comments, or references

If the issue body is sparse or unclear, ask the user for clarification before proceeding.

### Step 3: Run baseline tests

Before making any changes, verify the current state passes all tests:

```bash
cargo test --workspace
```

If baseline tests fail, report the failures to the user. The issue may be related to pre-existing breakage. Ask whether to proceed with the fix anyway.

### Step 4: Understand and locate the problem

Based on the issue description:
1. Search the codebase for relevant code using `grep` and file exploration
2. Read the affected files to understand the current behavior
3. Identify the root cause — don't just fix symptoms
4. If the issue references specific files, functions, or error messages, start there

Explain your understanding of the root cause to the user before proceeding to the fix. This gives them a chance to correct your diagnosis.

### Step 5: Implement the fix

Write the minimal fix that addresses the root cause:
- Change only what's necessary — no refactoring, no unrelated cleanup
- Follow the project's existing code style and patterns
- Add a test if the issue is a regression that tests would catch
- If the fix touches multiple files, verify each change is needed

### Step 6: Verify the fix

Run the full test suite to confirm the fix doesn't break anything:

```bash
cargo test --workspace
```

Also run clippy to catch any issues introduced:

```bash
cargo clippy --workspace -- -D warnings
```

If anything fails, fix it before proceeding.

### Step 7: Create a branch

Create a descriptive branch name from the issue:

```bash
git checkout -b fix/<issue-number>-<short-description>
```

The short description should be 2-4 words in kebab-case, derived from the issue title. For example, issue #42 titled "Crash when clicking settings with no config loaded" → `fix/42-crash-settings-no-config`.

### Step 8: Commit the fix

Use conventional commit format with the issue reference:

```bash
git add <changed files>
git commit -m "fix(<scope>): <description> (fixes #<number>)"
```

- `<scope>`: The crate or module affected (e.g., `gui`, `core`, `enhance`)
- `<description>`: Imperative mood, under 50 chars, describes the fix
- Include `(fixes #<number>)` so GitHub auto-closes the issue

Example: `fix(gui): handle missing config in settings page (fixes #42)`

### Step 9: Push and create a PR

Push the branch:

```bash
git push -u origin fix/<issue-number>-<short-description>
```

Create the PR:

```bash
gh pr create \
  --title "fix: <brief description> (fixes #<number>)" \
  --body "$(cat <<'EOF'
## Summary
- <bullet points describing the fix>

## Root cause
<1-2 sentences explaining the root cause>

## Test plan
- [x] Baseline tests pass before fix
- [x] All tests pass after fix
- [ ] Manual verification: <what to check>

Fixes #<number>

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

### Step 10: Report

Summarize what was done:
- Issue number and title
- Root cause identified
- Files changed
- Branch and PR URLs
- Test results (before and after)

## Safety rules

- **Never** push to `main` or `master` directly — always use a feature branch
- **Never** skip hooks (`--no-verify`, `--no-gpg-sign`) unless tests won't pass without them
- **Never** force-push unless the user explicitly requests it
- If the issue requires changes beyond a single focused fix (architectural changes, major refactors), flag this to the user and suggest breaking it into multiple issues
- If multiple issues share the same root cause, mention this in the PR description
- If the fix introduces new dependencies, flag this to the user before adding them
