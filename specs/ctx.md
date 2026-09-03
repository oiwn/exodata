# Current Task Context: Add a PR-driven OpenCode comment-to-issue workflow
State: in progress

## Plan

- [x] Install the OpenCode GitHub App and generate
  `.github/workflows/opencode.yml` with the GLM Coding Plan model
  `zai-coding-plan/glm-5.3-flash`; `ZHIPU_API_KEY` already exists as a repository
  Actions secret.
- [x] Preserve the generated `/oc` and `/opencode` command job for issue, PR
  conversation, and inline review comments.
- [x] Add an automatic `pull_request` job for `opened`, `synchronize`, `reopened`,
  and `ready_for_review`, restricted to same-repository head branches named
  `todos/**` and serialized per PR with stale runs cancelled.
- [x] Ensure the `code-todo` label exists with color `5319E7` and description
  `Created from actionable source comments`.
- [x] Give the automatic job least privilege: `contents: read`,
  `pull-requests: write`, `issues: write`, the built-in `GITHUB_TOKEN`,
  `use_github_token: true`, `share: false`, disabled edit tools, and no content
  write access. Pull-request write access is required for OpenCode's built-in
  reaction and summary comment.
- [x] Add a constrained prompt that reads repository instructions and scans only
  added or modified `TODO`/`FIXME`/`NOTE`/`HACK` comment blocks in the PR diff,
  excluding unchanged, generated, vendor, data, dependency, lockfile, and
  fixture content.
- [x] Make issue creation persistent and idempotent: compare candidates with all
  open and closed issues, group semantic duplicates, and create one `code-todo`
  issue per unmatched standalone task with deterministic source fingerprints.
- [x] Statically validate the workflow.
- [ ] Open the first genuine PR from a `todos/**` branch and confirm it creates
  only appropriate issues and posts its summary on the PR.
- [ ] On a later natural push to that PR, confirm existing remarks map to their
  issues, only new tasks create issues, and no commits, branches, or additional
  pull requests are created.
- [ ] Record the result on GitHub issue #133 and close it after successful
  verification.

## Findings

- The generated workflow currently runs only when a new issue/PR conversation
  comment or inline PR review comment contains `/oc` or `/opencode`; opening or
  updating a PR does not currently run OpenCode.
- OpenCode officially supports `pull_request` events and automatically supplies
  the PR branch, commits, changed files, discussion, and diff context to the
  prompt. It comments the result on the PR.
- Keep interactive commands and automatic scanning as separate jobs. Supplying
  the scanner's custom prompt to the generated command job would override user
  commands such as `/oc summarize`.
- The automatic scanner is issue-only and cannot change code. The preserved
  interactive job is intentionally different: an explicit `/oc` command on a
  same-repository PR may edit its branch and push fix commits to that PR.
- `pull_request.branches` filters the base branch, not the source branch. Enforce
  the `todos/**` opt-in prefix with a job condition on `github.head_ref`, and
  require `github.event.pull_request.head.repo.full_name == github.repository`
  so repository secrets are never expected for fork PRs.
- The automatic job uses the built-in workflow token rather than the installed
  App token. `use_github_token: true` skips the App/OIDC exchange and makes the
  declared job permissions the hard authorization boundary. The generated
  interactive command job continues using the installed App.
- GitHub Issues are the persistent state. There is no manual `report`/`create`
  handoff and no candidate artifact.
- The model registry expects `ZHIPU_API_KEY` for the Z.AI Coding Plan provider;
  GLM-5.3-Flash is the initial classification model.
- Generated issues use the `code-todo` label and contain
  `<!-- opencode-todo:v1:<sha256> -->`, derived from the relative path, marker
  kind, normalized comment block, and enclosing heading or symbol.
- Issue bodies include PR, branch, and commit provenance, an exact-commit source
  link, the source remark and context, and concise acceptance criteria.
- Changing or removing a source remark does not update or close an existing
  issue in this first version; issue discussion and lifecycle remain manual.
- `anomalyco/opencode/github@latest` dynamically installs the latest OpenCode
  release, so the initial workflow is not fully reproducible. Consider explicit
  version pinning only after the workflow is proven.
- The completed workflow passes `actionlint` 1.7.12 and `git diff --check`. The
  `code-todo` label also exists on `oiwn/exodata` with the specified metadata.
- The first run on PR #136 proved that `issues: write` is sufficient for label
  management but not for OpenCode's mandatory reaction and comment on a pull
  request; the automatic job therefore requires `pull-requests: write`.

## Context

The user creates a same-repository branch from `main` named `todos/<name>`, adds
or edits source/documentation remarks (including through GitHub's web editor),
and opens a PR to `main`. Opening the PR starts the automatic scanner; later
pushes produce `synchronize` events and scan the current PR diff again. Raw
branch pushes without an open PR are intentionally ignored.

Keep both behaviors in the generated `.github/workflows/opencode.yml`:

1. The existing command job handles `/oc` and `/opencode` comments with the
   installed App and its original prompt behavior, including user-requested
   fixes committed to the same PR branch.
2. A new automatic job handles qualifying `pull_request` events with a fixed,
   constrained TODO-to-issue prompt, a 20-minute timeout, full-history
   `actions/checkout@v6` checkout, and per-PR concurrency with
   `cancel-in-progress: true`.

The automatic job must set `contents: read`, `pull-requests: write`, and
`issues: write`; pass `GITHUB_TOKEN` and `ZHIPU_API_KEY`; and set
`use_github_token: true`, `share: false`, and an `OPENCODE_PERMISSION` rule that
denies file editing. Omit `id-token: write` and `contents: write` from this job.
The pull-request write grant exists only because the stock action always reacts
and comments on the triggering PR. Create/update the managed `code-todo` label
deterministically before invoking OpenCode.

The prompt examines only marker lines added or modified by the PR, then opens
enough surrounding file context to classify them. `NOTE` is actionable only
when it clearly describes unfinished work. Ignore headings, examples,
historical/completed notes, vague reminders, generated content, and prose that
merely mentions marker names.

Before creating anything, the agent retrieves every open and closed issue and
excludes pull requests. Exact fingerprints and semantic matching both count as
duplicates; closed matches are not reopened. Multiple remarks describing the
same task produce one issue containing each source fingerprint. The fingerprint
must not include branch, commit, or line number, so line movement does not change
identity. The prompt may create validated unmatched issues and must not edit
files, execute project code, mutate existing issues, commit, push, switch/create
branches, or create pull requests. The later issue-to-code bot is outside scope.

Primary references:

- <https://opencode.ai/docs/github/#pull-request-example>
- <https://github.com/anomalyco/opencode/blob/dev/packages/opencode/src/cli/cmd/github.handler.ts>
- <https://github.com/anomalyco/models.dev/blob/dev/providers%2Fzai-coding-plan%2Fprovider.toml>
- <https://docs.z.ai/devpack/overview>
- <https://github.com/oiwn/exodata/issues/133>

## Next

Merge the pull-request permission correction to `main`, then close and reopen PR
#136 to trigger a fresh initial scan. Later, push a new natural task to the same
PR and verify the idempotent synchronize run before updating issue #133.
