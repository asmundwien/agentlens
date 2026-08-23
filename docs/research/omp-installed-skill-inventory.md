# OMP installed-skill inventory surfaces

Research date: 2026-08-23

## Scope and inspected boundary

This report answers which supported OMP surfaces can describe the **effective installed skills** visible to an ordinary macOS agent client session. It does not equate files present on disk with the session's effective installed-skill set.

The inspected supported release is **OMP v18.0.3**, which GitHub marked Latest on the research date; tag `v18.0.3` resolves to commit [`160ed439ac0df594347e7d7018b813a7ffdb5e81`](https://github.com/can1357/oh-my-pi/releases/tag/v18.0.3). All v18.0.3 source links below are pinned to that commit. Historical boundaries are named separately.

## Decision

For an ordinary OMP macOS session, use OMP's **in-process public SDK discovery helper** at `session_start`, with the active session `cwd`, active profile, merged `skills.*` settings, and `disabledExtensions`. `discoverSkills()` returns the closest supported reconstruction of the default effective installed-skill set, including each surviving skill's OMP provenance fields. The SDK documents `discoverSkills(cwd?, _agentDir?, settings?)`, and the extension API injects the coding-agent exports through `pi.pi`; the implementation delegates to the same `loadSkills()` used during session construction. [`docs/sdk.md` discovery helpers](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/docs/sdk.md#L335-L347), [`ExtensionAPI.pi`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/extensions/types.ts#L1198-L1217), [`discoverSkills` implementation](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/sdk.ts#L792-L804)

This boundary is intentionally narrow:

- It is authoritative for **ordinary discovered sessions**, not an SDK host that supplies `CreateAgentSessionOptions.skills`; an explicit list replaces discovery. The exact current list for an SDK-owned session is `AgentSession.skills`. [`session construction choice`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/sdk.ts#L1582-L1591), [`AgentSession.skills`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/session/agent-session.ts#L6659-L6664)
- The `_agentDir` parameter is unused in v18.0.3. It cannot query another profile. Profile selection must already have happened in the process. [`discoverSkills` implementation](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/sdk.ts#L792-L804)
- `pi.getCommands()` can provide a session-bound cross-check by filtering entries whose `source` is `skill`, including `name`, description, and path, but only while `skills.enableSkillCommands` is enabled and without provider/level provenance. It is therefore not the primary inventory surface. [`getCommands` API](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/extensions/types.ts#L1429-L1439), [`skill-command projection`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/extensions/get-commands-handler.ts#L24-L66)
- No supported out-of-process inventory endpoint or change stream was found. Agentlens must report that gap rather than label a filesystem crawl as OMP's effective inventory.

Recommended extension query for this inspected boundary:

```ts
pi.on("session_start", async (_event, ctx) => {
  const skillsSettings = pi.pi.settings.getGroup("skills");
  const { skills, warnings } = await pi.pi.discoverSkills(ctx.cwd, undefined, {
    ...skillsSettings,
    disabledExtensions: pi.pi.settings.get("disabledExtensions") ?? [],
  });
  // Persist a session-scoped snapshot; surface warnings as inventory gaps.
});
```

The call shape above is an **Agentlens recommendation**, not an OMP-authored recipe. Its ingredients are public exports and documented extension context fields. [`public settings and SDK exports`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/index.ts#L10-L17), [`public SDK re-export`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/index.ts#L27-L40), [`ExtensionContext.cwd`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/extensions/types.ts#L455-L479)

## How v18.0.3 computes the effective set

`loadSkills()` performs three passes: registered capability providers, configured custom directories, then managed auto-learn skills. `skills.enabled: false` returns an empty result before discovery. The ordinary defaults enable the master switch, skill commands, and every named source toggle; custom/include/ignore arrays default empty. [`loader`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/skills.ts#L119-L178), [`settings defaults`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/config/settings-schema.ts#L5132-L5164)

### Roots in an ordinary macOS session

All ordinary provider scans use the session `cwd`, `os.homedir()`, and discovered repository root. Each normal root scan is shallow: immediate child directories containing `SKILL.md`; hidden child directories, non-directories, and deeper descendants are not scanned. A skill with `enabled: false` is dropped. Some providers additionally require a description. [`loadCapability` context](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/capability/index.ts#L253-L271), [`scanSkillsFromDir`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/helpers.ts#L349-L439)

| Provider | Roots and scope at v18.0.3 |
| --- | --- |
| `native` | Project: `.omp/skills` at `cwd` and each ancestor, closest first, bounded by repository root or home. User: `<active-agent-dir>/skills`. Native scans require `description`. [`builtin.ts`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/builtin.ts#L281-L306) |
| `omp-managed` | `<active-agent-dir>/managed-skills`; discovery is unconditional under the master skill switch, while auto-learn creation/nudging has a separate setting. [`builtin.ts`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/builtin.ts#L309-L336) |
| `agents` | Project ancestor walk for both `.agent/skills` and `.agents/skills`, excluding home as project scope; user `~/.agent/skills` and `~/.agents/skills`. [`agents.ts`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/agents.ts#L138-L195) |
| `claude` | User active Claude config directory's `skills`; project `.claude/skills` from `cwd` through repository root/home, with home excluded from project scope. [`claude.ts`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/claude.ts#L165-L218) |
| `codex` | User `~/.codex/skills`; project `<cwd>/.codex/skills`. It does not ancestor-walk in v18.0.3. [`codex.ts`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/codex.ts#L233-L259) |
| `opencode` | User `~/.config/opencode/skills`; project `<cwd>/.opencode/skills`. [`path constants`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/helpers.ts#L27-L85), [`opencode.ts`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/opencode.ts#L321-L353) |
| `github` | Project `<cwd>/.github/skills`; no user root. [`path constants`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/helpers.ts#L75-L79), [`github.ts`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/github.ts#L278-L328) |
| `omp-plugins` | `skills/` beside configured extension packages, CLI `--extension` directories, and enabled installed OMP plugins. File entrypoints have no sibling discovery tree. [`omp-plugins.ts`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/omp-plugins.ts#L1-L17), [`extension root order`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/omp-extension-roots.ts#L168-L226) |
| `claude-plugins` | Skills from enabled Claude marketplace/plugin roots. The provider remains a legacy loader for roots not governed by Agent Plugins 1.0.0. [`provider registration`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/claude-plugins.ts#L596-L609), [`Agent Plugin handoff`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/agent-plugins.ts#L1-L16) |
| `agent-plugins` | Agent Plugins 1.0.0 roots from marketplace installs, `--plugin-dir`, and configured extension packages; immediate `skills/*/SKILL.md` children must pass the standard-specific validation and containment checks. [`agent-plugins.ts`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/agent-plugins.ts#L1-L16), [`root collection and scan`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/agent-plugins.ts#L55-L81), [`validation path`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/agent-plugins.ts#L95-L195) |
| `custom` | Every `skills.customDirectories` entry after `~` expansion, treated as user level and requiring `description`. [`custom-directory pass`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/skills.ts#L255-L299) |

### Profiles and configured roots

On macOS, the default config root is `~/.omp` and the default agent directory is `~/.omp/agent`. `OMP_PROFILE` is canonical and wins over legacy `PI_PROFILE`; empty/default selects the default profile. A named profile relocates its config root to `~/.omp/profiles/<name>` and its agent directory to `~/.omp/profiles/<name>/agent`. [`profile resolution`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/utils/src/dirs.ts#L50-L127)

`--profile` is applied before imports that read the agent directory. A named profile deliberately ignores `PI_CODING_AGENT_DIR`; that override applies only to the default profile. [`CLI bootstrap`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/cli.ts#L342-L360), [`DirResolver precedence`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/utils/src/dirs.ts#L243-L255)

Consequences for inventory:

- Native authored skills, managed skills, profile settings, and the OMP plugin registry follow the active profile root. [`native user scan`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/builtin.ts#L294-L320), [`plugin directory resolver`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/utils/src/dirs.ts#L524-L553)
- Third-party user homes such as `~/.claude`, `~/.codex`, `~/.agent[s]`, and `~/.config/opencode` remain home-based. This is a **derived fact from the pinned loaders**, not a stated cross-release compatibility promise. [`provider path constants`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/helpers.ts#L27-L109)
- The profile's `skills.customDirectories` values may point anywhere; the active profile selects the setting, not a mandatory storage location. [`settings default and type`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/config/settings-schema.ts#L5152-L5164), [`custom-directory expansion`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/skills.ts#L255-L299)

### Precedence and deduplication

Capability providers are stored in descending numeric priority; equal priority retains registration order. Capability dedup uses skill name and keeps the first item. [`provider insertion`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/capability/index.ts#L64-L91), [`deduplication`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/capability/index.ts#L183-L211), [`skill key`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/capability/skill.ts#L58-L68)

The actual v18.0.3 order is:

1. `native` 100
2. `omp-plugins` 90
3. `claude` 80
4. `agent-plugins` 75
5. `claude-plugins`, `agents`, then `codex`, each 70
6. `opencode` 55
7. `github` 30
8. `omp-managed` 5

The priorities are defined in the pinned providers, and the 70-priority tie follows provider import order. [`provider imports`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/index.ts#L22-L40), [`native priority`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/builtin.ts#L37-L43), [`managed priority`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/builtin.ts#L309-L335), [`omp-plugins priority`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/omp-plugins.ts#L43-L47), [`Claude priority`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/claude.ts#L33-L36), [`Agent Plugins priority`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/agent-plugins.ts#L36-L41), [`Claude Plugins priority`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/claude-plugins.ts#L29-L33), [`Agents priority`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/agents.ts#L26-L29), [`Codex priority`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/codex.ts#L41-L45), [`OpenCode priority`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/opencode.ts#L42-L46), [`GitHub priority`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/github.ts#L39-L43)

The checked-in `docs/skills.md` provider list omits the v18.0.3 `agent-plugins` priority-75 provider, even though the provider is registered in source. Agentlens must follow the executable source at the inspected commit and retain this documentation discrepancy as a compatibility warning. [`docs list`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/docs/skills.md#L81-L98), [`registered provider`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/agent-plugins.ts#L323-L333)

Within one provider, emitted order remains the tie-breaker and is not uniformly project-over-user: native and Agents emit project candidates first, while Claude, Codex, and OpenCode emit user candidates first. Agentlens should record the winning path rather than infer precedence from `level`. [`native order`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/builtin.ts#L281-L305), [`Agents order`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/agents.ts#L172-L186), [`Claude order`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/claude.ts#L193-L218), [`Codex order`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/codex.ts#L237-L258), [`OpenCode order`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/opencode.ts#L325-L353)

After source toggles and name filters, OMP selects enabled authored skills from the pre-dedup superset. This allows a lower-priority copy to survive when the higher-priority source is disabled. It then suppresses identical files by resolved realpath. Custom-directory skills replace same-name provider winners; the first duplicate among custom directories wins. Managed skills are considered dead last and cannot replace authored skills. [`authored filtering and realpath pass`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/skills.ts#L180-L254), [`custom override and managed pass`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/skills.ts#L255-L344)

### Filtering and disabled behavior

The effective filter sequence is:

1. Remove whole providers named by `disabledProviders`. [`provider filter`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/capability/index.ts#L234-L271)
2. If `skills.enabled` is false, return no skills. [`master switch`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/skills.ts#L129-L149)
3. Drop names listed as `skill:<name>` in `disabledExtensions`.
4. Apply dedicated user/project source toggles for `native`, `claude`, `codex`, and `agents`. Providers without a dedicated toggle, including plugin/OpenCode/GitHub providers, are enabled when any named third-party toggle is enabled; managed skills remain enabled under the master switch.
5. Apply `ignoredSkills` Bun globs, then a non-empty `includeSkills` allowlist.
6. Apply same-name precedence and realpath/custom/managed deduplication described above.

Steps 3–5 are implemented together in `loadSkills()`. [`source gates and filters`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/skills.ts#L150-L218)

`hide: true` and Agent Skills `disable-model-invocation: true` do **not** remove an installed skill. They omit it from the model-facing system-prompt list while leaving `skill://<name>` and, when enabled, `/skill:<name>` reachable. [`skill semantics`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/capability/skill.ts#L12-L30), [`runtime mapping`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/skills.ts#L228-L242)

Disabled extension behavior has three distinct mechanisms:

- `disabledExtensions: ["skill:<name>"]` disables that skill name across roots because the skill capability's extension ID is name-based. [`skill extension ID`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/capability/skill.ts#L58-L68)
- `disabledExtensions: ["extension-module:<name>"]` disables an extension module item, not sibling skill IDs. Configured extension roots are still enumerated independently from `disabledExtensions`. This is a **source-derived boundary**, not an explicit OMP compatibility promise. [`generic capability filtering`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/capability/index.ts#L111-L175), [`configured root enumeration`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/omp-extension-roots.ts#L168-L226)
- Disabling an installed plugin through OMP's plugin manager removes it from `getEnabledPlugins()`, so its installed-package root stops contributing sibling skills. Project overrides can also disable an installed plugin. [`enabled-plugin filtering`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/plugins/loader.ts#L73-L156), [`installed-root enumeration`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/discovery/omp-extension-roots.ts#L229-L272)

OMP does not return a normalized per-skill exclusion reason. Loader warnings cover unreadable/invalid files and some collisions, but a filtered name simply does not survive. Agentlens must not invent a reason when only absence is observed. [`LoadSkillsResult`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/skills.ts#L35-L42), [`filter implementation`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/skills.ts#L196-L218)

## Provenance available to Agentlens

Each discovered capability skill carries an absolute `path`, `level`, and `_source` with `provider`, display `providerName`, absolute source `path`, and `level`. The runtime skill adds `filePath`, `baseDir`, string `source`, `hide`, and optional Agent Plugin `containRoot`, while preserving `_source`. [`SourceMeta`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/capability/types.ts#L93-L119), [`runtime Skill`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/skills.ts#L17-L42), [`runtime mapping`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/skills.ts#L220-L242)

OMP does not attach a durable skill ID, content hash, install timestamp, package version, package repository URL, or complete package identity to the runtime skill. Agent Plugin `containRoot` is a security boundary, not package-version provenance. This is an **inspection finding from the complete pinned `Skill` and `SourceMeta` shapes**, not a forward compatibility guarantee. [`SourceMeta`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/capability/types.ts#L93-L105), [`runtime Skill`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/skills.ts#L17-L42)

Recommended Agentlens installed-skill identity is therefore the tuple `(agent client, active profile/default, cwd, skill name, winning provider, winning absolute path)` plus observation time. This is an **Agentlens modeling recommendation**, not an OMP identity contract. Same-name replacement or a changed `cwd` can change the winner without changing the name.

## Extension API and lifecycle surfaces

### What an extension can observe

| Surface | Supported information | Limitation |
| --- | --- | --- |
| `pi.pi.discoverSkills(...)` | Re-runs public SDK discovery and returns runtime skills plus warnings. [`SDK docs`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/docs/sdk.md#L335-L347) | Must supply the merged skill settings and disabled IDs to mirror the ordinary session; cannot represent an SDK-supplied explicit `skills` list. `_agentDir` is unused. [`implementation`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/sdk.ts#L792-L804) |
| `pi.getCommands()` | Session-bound skill command names, descriptions, and paths when skill commands are enabled. [`projection`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/extensions/get-commands-handler.ts#L24-L66) | Omits provider/level provenance and exposes no skill entries when skill commands are disabled. |
| `session_start` | Runs after interactive runtime configuration and is the earliest practical extension lifecycle point for a session inventory snapshot. [`runtime initialization`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/modes/runtime-init.ts#L136-L148) | Event payload contains only `type`; it does not carry skills. [`SessionStartEvent`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/shared-events.ts#L23-L31) |
| `resources_discover` | Types permit an extension to return `skillPaths`. [`event types`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/extensions/types.ts#L686-L702) | It is not an inventory event, and OMP's own docs say the implemented emitter has no `AgentSession` callsites in this release. Treat it as non-operational for ordinary sessions. [`docs/extensions.md`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/docs/extensions.md#L342-L345) |
| `getActiveSkills()` | Exported process-global snapshot used by `skill://`. [`implementation comment`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/skills.ts#L43-L66) | Not an `ExtensionAPI` inventory method, documented as process-global, and unsafe as a session authority in a process hosting multiple sessions. Do not make it the supported Agentlens boundary. |

### Snapshot timing and refresh

Session creation starts default skill discovery using the active merged settings, then materializes either the explicit SDK list or the discovered result before the `AgentSession` is built. [`startup discovery`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/sdk.ts#L1358-L1367), [`materialization`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/sdk.ts#L1582-L1591)

There is no skills filesystem watcher in the inspected loader/session paths. That is a **negative source inspection finding**. Refresh is explicit:

- `AgentSession.refreshSkills()` resets capability caches, re-runs `loadSkills()` with current `cwd`, skill settings, and disabled IDs, updates the main session's active snapshot, rebuilds prompt metadata, and notifies command metadata listeners. [`refresh implementation`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/session/session-tools.ts#L1176-L1196)
- Interactive `cwd` change invokes skill refresh. [`interactive cwd change`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/modes/interactive-mode.ts#L1538-L1548)
- `/reload-plugins` refreshes plugin discovery, skills, slash commands, and MCP state. [`reload implementation`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/slash-commands/builtin-marketplace.ts#L27-L39), [`command`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/slash-commands/builtin-marketplace.ts#L552-L562)

No extension event announces that the effective skill list changed, and `ExtensionContext` exposes no `refreshSkills()` action. An extension can observe `session_start` and session/cwd lifecycle, but cannot subscribe to an authoritative first-party skill-change event. [`ExtensionContext`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/extensions/types.ts#L455-L540), [`extension event overloads`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/extensions/types.ts#L1218-L1279)

**Agentlens recommendation:** record a snapshot at `session_start`; re-run discovery when Agentlens itself observes a supported session `cwd`/reload boundary; otherwise label the snapshot time and do not claim live completeness. A private filesystem watcher may improve freshness but would be Agentlens behavior, not OMP change detection.

## Unsupported out-of-process and analytics paths

No first-party CLI JSON inventory command, RPC/ACP skill-list endpoint, daemon API, or installed-skill change stream was found in the v18.0.3 public SDK/extension documentation and pinned source. The documented discovery helper is in-process, extensions themselves are loaded as in-process modules, and the extension action list has tools/commands but no `getSkills()` method. [`SDK discovery helpers`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/docs/sdk.md#L335-L347), [`extension architecture`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/docs/extensions.md#L1-L24), [`ExtensionAPI actions`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/extensions/types.ts#L1401-L1451)

A separate Agentlens process can reproduce filesystem discovery, but it cannot prove that a running agent client used the same profile, settings snapshot, explicit SDK skill list, plugin state, provider registry, or reload point. Therefore it must be labeled **reconstructed inventory**, not the effective installed-skill set of a running OMP session. This is an **Agentlens inference** from the session-bound inputs and override paths above.

OMP v18.0.3 also exposes no dedicated `skill_invoked` event or payload carrying canonical skill identity and `initiator`. The extension events include input, messages, tools, and session lifecycle, while skills are read on demand through `skill://` or injected through `/skill:<name>`; neither path emits a terminal skill-specific usage event. [`extension events`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/packages/coding-agent/src/extensibility/extensions/types.ts#L1218-L1279), [`documented runtime behavior`](https://github.com/can1357/oh-my-pi/blob/160ed439ac0df594347e7d7018b813a7ffdb5e81/docs/skills.md#L127-L174)

Consequently, installed-skill discovery does not produce a usage event, initiator, observed invocation count, invocation pattern, or skill adoption measure. Parsing user `/skill:` input would miss model-selected `skill://` reads and cannot establish a complete observed invocation count. Agentlens must report OMP usage coverage as unavailable unless a separate deterministic invocation-signal integration is established. This paragraph is an **Agentlens product conclusion**, not an OMP compatibility claim.

## Relevant source history

These commits are compatibility boundaries present in v18.0.3. “First containing release” is based on first-party commit/changelog/tag history, not a promise about every intermediate build.

| First containing release | Commit | Inventory change |
| --- | --- | --- |
| v15.5.12 | [`bdce9cf068529f82ddf46728f487a40b7fe8657c`](https://github.com/can1357/oh-my-pi/commit/bdce9cf068529f82ddf46728f487a40b7fe8657c) | Enabled installed OMP plugin packages to contribute sibling discovery trees, including skills. |
| v15.9.0 | [`6e49d33b719a36b940726485f9997268a357c990`](https://github.com/can1357/oh-my-pi/commit/6e49d33b719a36b940726485f9997268a357c990) | Recognized Agent Skills `disable-model-invocation` as hidden from model listing but still installed. |
| v15.12.4 | [`8338a8af9412a7da93ba1294b4b2b0b2dd0f1418`](https://github.com/can1357/oh-my-pi/commit/8338a8af9412a7da93ba1294b4b2b0b2dd0f1418) | Added independent user/project toggles for `.agent[s]` skills. |
| v16.3.9 | [`dd3375981fc1f74d953e7aec43710efd6a65a2fc`](https://github.com/can1357/oh-my-pi/commit/dd3375981fc1f74d953e7aec43710efd6a65a2fc) | Made an enabled lower-priority duplicate survive when a higher-priority source is disabled. |
| v17.2.5 | [`1596d9231e7356abf3581f1fcf08feb388dd6fe5`](https://github.com/can1357/oh-my-pi/commit/1596d9231e7356abf3581f1fcf08feb388dd6fe5) | Made configured custom-directory skills replace same-name default-provider skills. |
| v17.2.11 | [`2ad61c7b9236faa927d5343094e95b20e21a370f`](https://github.com/can1357/oh-my-pi/commit/2ad61c7b9236faa927d5343094e95b20e21a370f) | Added Agent Plugins 1.0.0 discovery, containment, and priority-75 provider semantics. |

Agentlens should pin its compatibility fixture to v18.0.3 behavior and treat provider lists, priority, filters, and profile resolution as versioned implementation contracts requiring reinspection on OMP upgrades. OMP's public helper is the supported call boundary; the exact returned semantics remain release-specific because the provider registry and filters have changed repeatedly in the cited history.

## Gaps Agentlens must display

1. **No running-session inventory receipt for extensions.** `session_start` carries no skill list; public `discoverSkills()` reconstructs ordinary discovery, while `getCommands()` is partial.
2. **No supported out-of-process inventory/provenance API.** Filesystem reconstruction is not a running-session authority.
3. **No change event or watcher contract.** Inventory has a snapshot time; explicit refresh boundaries are observable only indirectly.
4. **Incomplete provenance.** Provider, level, and absolute winning path exist; durable ID, package version/origin, install time, and content hash do not.
5. **No normalized exclusion reason.** A missing skill may be invalid, filtered, shadowed, provider-disabled, plugin-disabled, profile-inapplicable, outside the current `cwd`, or simply absent.
6. **No skill invocation/initiator event.** Installed-skill inventory alone cannot yield usage events, observed invocation count, invocation patterns, or skill adoption.
7. **Documentation/source mismatch.** v18.0.3 source registers `agent-plugins` at priority 75 while the same tag's skills document omits it.
8. **SDK override gap.** Sessions created with an explicit `skills` list can differ from `discoverSkills()`; only the SDK owner can read the exact `AgentSession.skills` list.

These gaps are the supported boundary of this report. Agentlens should surface them as unknown or reconstructed states, never silently convert them into complete inventory or usage coverage.
