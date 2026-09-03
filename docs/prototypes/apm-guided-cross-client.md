# Guided APM cross-client prototype

Related decision ticket: [Learn APM through a guided cross-client prototype](https://github.com/asmundwien/agentlens/issues/61)

## Question

How does APM represent project and user scope, pin dependencies, select agent-client targets, materialize skills and hooks, project MCP declarations, report collisions, update dependencies, audit drift, and remove generated output? Does that ownership model fit this environment?

## Isolation and versions

The prototype ran under a disposable directory with a synthetic `HOME`. Claude, Codex, Copilot, and XDG homes were explicitly redirected beneath that directory. No real client configuration was targeted.

The installed CLI was inspected first at `0.28.0` (`e041462`). During the guided run it was manually updated to `0.29.0` (`5ac6733`), so behavioral results below are from `0.29.0` unless stated otherwise. That self-update changed `/usr/local/bin/apm`; `HOME` isolation does not isolate the APM binary.

Primary documentation consulted:

- [What is APM?](https://microsoft.github.io/apm/concepts/what-is-apm/)
- [`apm install`](https://microsoft.github.io/apm/reference/cli/install/)
- [Install MCP servers](https://microsoft.github.io/apm/consumer/install-mcp-servers/)
- [Hooks and commands](https://microsoft.github.io/apm/producer/author-primitives/hooks-and-commands/)
- [`apm uninstall`](https://microsoft.github.io/apm/reference/cli/uninstall/)

## Results

### Manifest, lockfile, and dependency pins

`apm init --yes --target claude,copilot,codex` created only `apm.yml`. Installing `microsoft/apm-sample-package#v1.0.0` added manifest intent and created `apm.lock.yaml`, `apm_modules/`, and target-native output.

The lockfile pinned:

- `microsoft/apm-sample-package` `v1.0.0` to commit `fb2851683be0e0e7711421d518bd8dba23b0b1f6` with content hash `sha256:744cca54cc8ff7ca90aa1dd621c2f98c6291cd793815afe8518001cc94b8aba9`.
- Its transitive `github/awesome-copilot/skills/review-and-refactor` dependency to commit `2ba72cd14253500bbb747b5f01e72dd03fbafcb0`. APM warned that the upstream declaration was unpinned even though the lockfile had an exact commit.

Each deployment entry recorded its path, target, scope, owners, active owner, and content hash. `apm update` later normalized the transitive dependency from no named ref to `main` without changing its commit.

### Target selection

Install honored the manifest targets and reported `source: apm.yml`. Before installation, however, `apm targets --json` reported Claude, Codex, and Copilot as inactive despite those explicit manifest targets. After APM created `.claude/`, `.codex/`, and `.github/`, the same command reported them active based on filesystem markers.

Observed conclusion: `apm targets` displays detected filesystem activation rather than the manifest's effective install selection, despite describing itself as showing resolved targets.

APM `0.29.0` has no OMP or Pi target. It cannot own their native configuration directly.

### Dry-run behavior

`apm install microsoft/apm-sample-package#v1.0.0 --dry-run --verbose` validated the GitHub source and said it would add the package, but then parsed the unchanged manifest, found no dependencies, and previewed no resolution or deployment paths. It made no changes.

`apm update --dry-run --verbose` was stronger: it resolved upstream state, rendered the dependency plan, and left the lockfile and deployed skill hashes byte-identical.

`apm uninstall --dry-run` previewed declaration and package-cache removal. Current documentation correctly warns that pre-uninstall scripts may still run and shared-slot survivor staging is skipped.

### Skill and primitive materialization

Project scope produced:

- Shared Codex/Copilot skills under `.agents/skills/`.
- Claude-native skill copies under `.claude/skills/`.
- Claude rules, commands, and agents under `.claude/`.
- Copilot instructions, prompts, and agents under `.github/`.
- A Codex agent under `.codex/agents/`.

APM dropped unsupported `mode` frontmatter from two Claude commands and warned after installation.

User scope created a separate manifest, lockfile, and package cache under synthetic `~/.apm/`, then materialized supported non-skill primitives under synthetic `.claude/`, `.codex/`, and `.copilot/` directories. It copied skill sources into the global package cache but created only empty deployment directories under synthetic `~/.agents/skills/` and `~/.claude/skills/`; no `SKILL.md` files were deployed.

The global lockfile also marked Claude and Codex deployments as `scope: project`; only Copilot deployments were marked `scope: user`.

### MCP declarations

A self-defined project MCP declaration was stored once in `apm.yml`, then projected to:

- Claude `.mcp.json` using JSON `mcpServers`.
- Copilot `.github/mcp.json` using JSON `mcpServers` plus target-specific `tools` and `id` fields.
- Codex `.codex/config.toml` using TOML `mcp_servers`.

The lockfile represented the root project as owner `.` and recorded one logical MCP server mapped to three runtimes.

`apm install --global --mcp ...` exited immediately with `MCP servers are project-scoped; --global is not supported for MCP entries`. This contradicts both the installed `0.29.0` help and current primary documentation, which describe global MCP projection for supported clients.

### Hook materialization

One project-owned `.apm/hooks/` descriptor was translated into three native forms:

- Claude `.claude/settings.json`, retaining `PreToolUse` and rewriting the command through `${CLAUDE_PROJECT_DIR}`.
- Codex `.codex/hooks.json`, retaining `PreToolUse` and using a project-relative command.
- Copilot `.github/hooks/<name>.json`, translating the event to `preToolUse` and adding `version: 1`.

Claude and Codex received `apm-hooks.json` ownership sidecars with source `_local/project`. Hook scripts were copied into each target's namespaced hook directory. Installation did not execute the hook.

### Drift and audit

After a managed skill was edited, `apm audit --ci --no-policy` exited `1`, reported both hash drift and replay drift for the exact file, and recommended `apm install`. Reinstall restored the locked bytes and exited `0` without requiring `--force`. This is consistent with APM treating a ledger-owned destination as generated output rather than user-editable content.

After update, audit replayed five package entries from cache and passed all ten checks, including lockfile presence, ref consistency, owner validity, deployed-file presence, MCP consistency, content integrity, includes consent, and drift.

### Collision behavior

An existing, unowned `.agents/skills/collision-demo/SKILL.md` was silently overwritten when a local package first claimed that destination. Install exited `0` with no collision warning or `--force` requirement.

When a second direct local package claimed the same skill, install again exited `0`; the last installed package won. A warning appeared only after replacement. The lockfile listed both owners and selected the second as `active_owner`, but both dependency entries recorded the winner's deployed hash rather than each source's own hash.

Observed conclusion: loose-skill collisions are permissive and order-dependent. Target deployment directories cannot safely mix APM-managed and independently authored files unless overwrites are acceptable.

### Update behavior

`apm outdated --verbose` reported all dependencies current. Immediately afterward, `apm update --dry-run --verbose` reported one update: the transitive skill ref changed from unnamed to `main` while the exact commit stayed unchanged. The real update applied that metadata change and preserved the manifest-order collision winner. Audit then passed.

### Uninstall behavior

Removing the second collision package correctly restored the first package as active owner of the shared skill. The uninstall also removed Claude and Codex hook integrations owned by the root project's local `.apm/` content, even though that root content was unrelated to the removed package. It left the corresponding ownership sidecars and produced four drift findings:

- unintegrated `.claude/apm-hooks.json`
- modified `.claude/settings.json`
- unintegrated `.codex/apm-hooks.json`
- modified `.codex/hooks.json`

Observed conclusion: package uninstall can disturb unrelated root-owned hooks and requires a subsequent install to reconcile them.

### Cleanup behavior

`apm deps clean --dry-run` said it would remove two packages, and `apm deps clean --yes` deleted `apm_modules/`. It did not remove dependency declarations from `apm.yml`, lock entries, or deployed output. The next `apm install` therefore downloaded and deployed both packages again. This contradicts the command help's unqualified description, “Remove all APM dependencies”; the command is package-cache cleanup in this run.

After the root hook source and MCP declaration were removed, `apm install` removed the copied hook scripts, the Copilot hook descriptor, and all three MCP entries. It left Claude and Codex hook configurations and ownership sidecars pointing at deleted scripts. A later package uninstall happened to reconcile those unrelated hook remnants. Empty `.mcp.json`, `.github/mcp.json`, and `.codex/config.toml` files remained after MCP removal.

Uninstalling all remaining project packages removed their manifest declarations, transitive dependency, lockfile, cache content, deployed primitives, and hook sidecars. It left empty `apm_modules/` and target configuration files. The no-dependency audit passed its sole lockfile check, and `apm prune --dry-run` found no orphaned packages.

Global uninstall removed the fake-user package, its transitive dependency, and their deployed primitives while preserving the user-scope manifest and APM configuration.

The preserved report was committed before the 5.4 MiB disposable root was deleted. No synthetic home, project output, or collision-package source remains on disk.

## Ownership verdict

APM `0.29.0` demonstrates a useful project-scope model: one manifest, exact lock pins, target-native projection, explicit deployment owners, deterministic repair, cache-only audit replay, and native cleanup commands.

It does not yet fit as the sole cross-client owner for this environment:

1. OMP and Pi are unsupported targets.
2. Global MCP behavior contradicts the installed help and current documentation.
3. Global skill deployment produced empty directories.
4. Global Claude and Codex scope metadata was incorrect.
5. Unowned files in generated skill paths were overwritten silently.
6. Package-to-package skill collisions use last-installed-wins semantics and blur losing-source hashes.
7. Uninstalling one package disturbed unrelated root-owned hooks.
8. `apm deps clean` removed only cached packages, not declared dependencies or deployed output.
9. Removing root hook source left dangling Claude and Codex hook configuration until an unrelated package uninstall reconciled it.
10. `apm targets` and add-style install dry-run present incomplete or misleading previews.

Safe near-term use is narrower: project scope only, committed manifest and lockfile, APM-exclusive generated target paths, explicit targets, and `apm audit --ci` after install/update/uninstall. Global ownership and migration of existing mixed-ownership client directories should wait until the observed defects are resolved or explicitly accepted. OMP and Pi require a separate owner regardless.
