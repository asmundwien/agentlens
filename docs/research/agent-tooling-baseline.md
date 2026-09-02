# Authoritative agent tooling baseline

**Snapshot date:** 2026-09-02  
**Host scope:** the current macOS user account and the `agentlens` repository  
**Decision scope:** inventory only. This report does not decide what to retain and does not authorize deletion, upgrades, resets, or credential changes.

## Method and safety boundary

This is a direct, read-only snapshot. Versions came from the installed executable or application bundle; ownership came from symlink targets, Homebrew/npm/Cargo records, package manifests, and repository source. Disk-use figures are rounded filesystem totals and will drift as clients write sessions and caches. A configured or installed component is not necessarily a running process.

Credential values, cookies, session contents, prompts, and conversation contents were never read into this report. Credential-bearing files are named only by owning path, permission mode, and data category. JSON/TOML/YAML inspection emitted object keys, server/plugin names, booleans, paths, and package versions only. The evidence appendix records reproducible safe commands; it intentionally omits commands that would print secret values.

Status terms:

- **Active/configured:** current executable, registry entry, settings reference, or client-managed store is present.
- **Installed, not enabled:** package metadata is present but current settings explicitly disable it or do not enable it.
- **Obsolete:** the owning client explicitly marks that copy obsolete.
- **Cache/history/temp:** retained data, not an active installation.
- **Unresolved:** direct evidence does not establish a single owner or explain a version mismatch.

## Executive inventory

| Surface | Installed/current version | Active owner and primary paths | State |
|---|---:|---|---|
| Claude Code CLI | 2.1.241 | npm `@anthropic-ai/claude-code`; `/opt/homebrew/bin/claude` → `/opt/homebrew/lib/node_modules/@anthropic-ai/claude-code/bin/claude.exe`; user state `~/.claude` | Active/configured ([E1](#e1-clients-apps-and-package-owners), [E4](#e4-client-state-and-disk-use)) |
| Claude desktop | 1.22209.3 | `/Applications/Claude.app`; `~/Library/Application Support/Claude` | Installed; app is newer than its stale Homebrew receipt ([E1](#e1-clients-apps-and-package-owners), [E4](#e4-client-state-and-disk-use)) |
| Claude VS Code extension | 2.1.258 | `~/.vscode/extensions/anthropic.claude-code-2.1.258-darwin-arm64` | Active registry entry; 10 older copies explicitly obsolete ([E2](#e2-vs-code-and-editor-integrations)) |
| Codex CLI | 0.144.1 | Homebrew cask; `/opt/homebrew/bin/codex` → `/opt/homebrew/Caskroom/codex/0.144.1/codex-aarch64-apple-darwin`; state `~/.codex` | Active/configured ([E1](#e1-clients-apps-and-package-owners), [E4](#e4-client-state-and-disk-use)) |
| GitHub Copilot CLI | executable reports 1.0.59 | Homebrew cask `copilot-cli`; `/opt/homebrew/bin/copilot`; state `~/.copilot` | Active install; Caskroom directory says 1.0.3, so upstream/cask version mapping is unresolved ([E1](#e1-clients-apps-and-package-owners), [E4](#e4-client-state-and-disk-use)) |
| GitHub Copilot in VS Code | bundled `copilot-chat` 0.62.0 | `/Applications/Visual Studio Code.app/Contents/Resources/app/extensions/copilot`; `~/Library/Application Support/Code/User/globalStorage/github.copilot-chat` | Bundled and configured; no separate current user-extension registry entry ([E2](#e2-vs-code-and-editor-integrations)) |
| OMP | 18.1.2 | Homebrew formula `can1357/tap/omp`; `/opt/homebrew/bin/omp` → Cellar 18.1.2; state `~/.omp` | Active/configured ([E1](#e1-clients-apps-and-package-owners), [E4](#e4-client-state-and-disk-use)) |
| Pi | executable reports 0.80.2 | Homebrew formula `pi-coding-agent`; `/opt/homebrew/bin/pi` → linked Cellar 0.78.0; state `~/.pi` | Active install; binary/receipt versions disagree and remain unresolved ([E1](#e1-clients-apps-and-package-owners), [E4](#e4-client-state-and-disk-use)) |
| ChatGPT desktop | 26.825.32147 | `/Applications/ChatGPT.app`; native data `~/Library/Application Support/com.openai.chat` | Installed; no `chatgpt` CLI on `PATH`; app is newer than its stale Homebrew receipt ([E1](#e1-clients-apps-and-package-owners), [E4](#e4-client-state-and-disk-use)) |
| ChatGPT VS Code extension | 26.825.51511 | `~/.vscode/extensions/openai.chatgpt-26.825.51511-darwin-arm64` | Active registry entry; 9 older copies explicitly obsolete ([E2](#e2-vs-code-and-editor-integrations)) |
| VS Code | 1.134.0 | `/Applications/Visual Studio Code.app`; `/opt/homebrew/bin/code`; data `~/Library/Application Support/Code` | Active install; app is newer than its stale Homebrew receipt ([E1](#e1-clients-apps-and-package-owners), [E2](#e2-vs-code-and-editor-integrations)) |
| Agentlens | crate 0.1.0 | Cargo install record for `~/.cargo/bin/agentlens`; second distinct binary at `~/.local/bin/agentlens`; data `~/Library/Application Support/Agentlens/agentlens.sqlite3` | Both client integrations select the Cargo binary; ownership of the different `.local` copy is unresolved ([E6](#e6-agentlens-and-repository-local-functions)) |
| Docker cagent | 1.23.0 | `/usr/local/bin/cagent` → Docker Desktop; state `~/.cagent` | Additional installed agent CLI, outside the named core set ([E1](#e1-clients-apps-and-package-owners), [E4](#e4-client-state-and-disk-use)) |

Negative evidence at snapshot time: no `chatgpt` CLI, standalone `Codex.app`, `Pi.app`, or GitHub Copilot desktop app was found. No Cursor app/CLI was found. This proves only the inspected `PATH`, `/Applications`, and `~/Applications`, not every possible volume or browser profile. ([E1](#e1-clients-apps-and-package-owners))

## Client configuration, integrations, and retained data

### Claude

- `~/.claude/settings.json` is active user configuration. It selects a model/effort/TUI policy, enables voice and memory-related preferences, enables `mattpocock-skills@claude-plugins-official`, and installs Agentlens handlers for `UserPromptExpansion` and `PostToolUse`/`Skill`. Both handlers execute `~/.cargo/bin/agentlens claude-hook`. ([E3](#e3-plugins-skills-and-client-configuration), [E6](#e6-agentlens-and-repository-local-functions))
- `~/.claude/plugins/installed_plugins.json` records `mattpocock-skills@claude-plugins-official` 1.2.3 and `frontend-design@claude-plugins-official` with version `unknown`. Only the former appears in `enabledPlugins`; therefore the frontend-design copy is **installed, not enabled by current settings**. The plugin store is about 30 MiB. ([E3](#e3-plugins-skills-and-client-configuration), [E4](#e4-client-state-and-disk-use))
- `~/.claude` is about 879 MiB and owns CLI histories and working state: `history.jsonl`, `projects/`, `sessions/`, `file-history/`, `tasks/`, `jobs/`, `paste-cache/`, shell/session environments, backups, telemetry, plugin caches, and a small general cache. These are classified as history/cache, not separate client installs. ([E4](#e4-client-state-and-disk-use))
- `~/Library/Application Support/Claude` is about 13 GiB and owns desktop state, including a roughly 9.6 GiB `vm_bundles/claudevm.bundle`, bundled Claude Code runtimes, local-agent sessions, Chromium-style cookies/local storage/cache, and `claude_desktop_config.json`. That config has no `mcpServers` map. ([E4](#e4-client-state-and-disk-use), [E5](#e5-mcp-topology))
- Desktop cache and logs also exist at `~/Library/Caches/com.anthropic.claudefordesktop` and `~/Library/Logs/Claude`. ([E4](#e4-client-state-and-disk-use))

### Codex

- `~/.codex/config.toml` is the active CLI configuration. It enables the installed plugins `sites`, `browser`, `visualize`, `documents`, `pdf`, `spreadsheets`, `presentations`, `template-creator`, and `codex-app-tools`. It configures `node_repl`; `computer-use` is explicitly disabled. ([E3](#e3-plugins-skills-and-client-configuration), [E5](#e5-mcp-topology))
- `~/.codex` is about 1.6 GiB: current sessions about 946 MiB; archived sessions about 47 MiB; plugin runtimes about 319 MiB; `.tmp` about 194 MiB; cache about 21 MiB; and computer-use assets about 68 MiB. It also owns state/log/memory/queue/history SQLite files, shell snapshots, plugin staging, and vendor-import caches. ([E4](#e4-client-state-and-disk-use))
- `~/.cache/codex-runtimes/codex-primary-runtime` is a separate roughly 1.5 GiB runtime/plugin dependency cache. ([E4](#e4-client-state-and-disk-use))
- `~/Library/Application Support/Codex` is about 159 MiB of Chromium-style support/cache/login data. There is no standalone Codex app; the installed ChatGPT bundle identifies as `com.openai.codex`, so this directory is **likely shared/owned by that bundle**, but the precise product boundary remains unresolved. ([E1](#e1-clients-apps-and-package-owners), [E4](#e4-client-state-and-disk-use))

### GitHub Copilot

- `~/.copilot` is about 339 MiB and owns Copilot CLI `config.json`, `settings.json`, `mcp-config.json`, OAuth configuration, command history, session state, IDE locks/logs, and a large `pkg/` runtime cache. The configuration files are mode `0600`. ([E4](#e4-client-state-and-disk-use), [E7](#e7-credential-and-authentication-stores))
- VS Code 1.134.0 bundles GitHub Copilot (`copilot-chat` 0.62.0); user settings explicitly enable `github.copilot.nextEditSuggestions.enabled`. `globalStorage/github.copilot-chat` owns session and embeddings caches. An old `github.copilot-1.388.0` VSIX remains in `CachedExtensionVSIXs`, but it is a cache, not a current user extension. ([E2](#e2-vs-code-and-editor-integrations))
- The GitHub CLI reports its GitHub authentication is in the macOS keyring. That shared keyring is recorded as an unresolved credential owner rather than inspected. No token value is present here. ([E7](#e7-credential-and-authentication-stores))

### OMP

- `~/.omp` is about 1.2 GiB. Main owners are `agent/` (configuration, sessions, histories, models, blobs and SQLite), `puppeteer/` browser assets, `natives/18.1.2`, `run/`, `logs/`, and GitHub/legacy-extension caches. ([E4](#e4-client-state-and-disk-use))
- `~/.omp/agent/extensions/agentlens.ts` is an active absolute symlink to this repository's `integrations/omp/agentlens.ts`. The installed file and tracked source were byte-identical at snapshot time. The extension resolves Agentlens from `CARGO_INSTALL_ROOT`, then `CARGO_HOME`, then `~/.cargo/bin`. ([E6](#e6-agentlens-and-repository-local-functions))
- `~/.omp/agent/mcp.json` configures `confluence-local` over stdio with implicit enablement and `hdir_confluence` over HTTP with `enabled: false`. Generated MCP and GitHub caches live under `~/.omp/cache` and are not authoritative configuration. ([E5](#e5-mcp-topology))
- `~/.omp/agent/history.db`, `models.db`, and `agent.db` are active local databases; `history.db` held 962 history rows and Agentlens held 536 usage events at inspection time. These counts are volatile. ([E4](#e4-client-state-and-disk-use), [E6](#e6-agentlens-and-repository-local-functions))

### Pi

- `~/.pi/agent/settings.json` configures two local source packages (`source/swarm/pi` and the handbook Pi integration) plus npm packages `pi-subagents`, `@juicesharp/rpiv-web-tools`, `context-mode`, `@asmundwien/pi-kit`, `pi-schedule-prompt`, and `pi-mcp-adapter`. ([E3](#e3-plugins-skills-and-client-configuration))
- `~/.pi/agent/npm/package-lock.json` is the authoritative Pi-local npm resolution: `@asmundwien/pi-kit` 0.5.0, `@juicesharp/rpiv-web-tools` 1.20.0, `context-mode` 1.0.166, `pi-mcp-adapter` 2.10.0, `pi-schedule-prompt` 0.4.1, and `pi-subagents` 0.31.0. Global npm contains older/different copies of several of these packages, so the Pi-local store, not the global copy, owns the configured extensions. ([E1](#e1-clients-apps-and-package-owners), [E3](#e3-plugins-skills-and-client-configuration))
- `~/.pi` is about 511 MiB. It owns per-project sessions, `run-history.jsonl`, MCP discovery/npm caches, crash logs, a roughly 262 MiB extension npm tree, and context-mode session/content state. ([E4](#e4-client-state-and-disk-use))
- `~/.pi/agent/mcp-cache.json` names `confluence-hdir`; because it is a cache rather than the settings source, it is classified as **discovered/cached ownership**, not proof the server is currently enabled. ([E5](#e5-mcp-topology))

### ChatGPT and VS Code

- `/Applications/ChatGPT.app` is about 1.3 GiB. `~/Library/Application Support/com.openai.chat` owns native conversation/project/model/draft data and VS Code pairing records; its size was about 7.6 MiB. `~/Library/Caches/com.openai.chat` is a separate cache. ([E4](#e4-client-state-and-disk-use))
- VS Code user agent extensions are `anthropic.claude-code@2.1.258` and `openai.chatgpt@26.825.51511`. `~/.vscode/extensions/.obsolete` explicitly marks 10 older Claude directories and 9 older ChatGPT directories obsolete, but those directories remain on disk. The entire extension tree is about 8.0 GiB. ([E2](#e2-vs-code-and-editor-integrations), [E4](#e4-client-state-and-disk-use))
- `~/Library/Application Support/Code` is about 1.3 GiB and owns editor history/workspace/global storage, agent session data, an `agent-host` cache, and about 387 MiB of cached extension VSIXs. These stores can contain prompts, workspaces, sessions, and credentials; contents were not inspected. ([E2](#e2-vs-code-and-editor-integrations), [E4](#e4-client-state-and-disk-use))

## Skills and plugins

| Owning path | Installed names | Classification |
|---|---|---|
| `~/.agents/skills` (156 KiB) | `cli-for-agents`, `deslop`, `how`, `principle-subtract-before-you-add`, `principle-type-system-discipline`, `typescript-best-practices`, `unslop`, `why` | Shared user skill store with `.skill-lock.json`; active ownership depends on each client's discovery rules ([E3](#e3-plugins-skills-and-client-configuration)) |
| `~/.claude/skills` (88 KiB) | `caveman`, `find-skills`, `grill-me`, `grill-with-docs`, `improve-codebase-architecture`, `tdd`, plus `unslop` | Claude-specific; `unslop` symlinks to `../../.agents/skills/unslop` ([E3](#e3-plugins-skills-and-client-configuration)) |
| `~/.codex/skills` (548 KiB) | system-managed `imagegen`, `openai-docs`, `plugin-creator`, `review-agent`, `skill-creator`, `skill-installer`; user `figma-implement-design` | Codex system/user skill store ([E3](#e3-plugins-skills-and-client-configuration)) |
| `~/.codex/vendor_imports/skills` (8.7 MiB) | curated/imported skill cache | Cache, not an independently enabled skill set ([E3](#e3-plugins-skills-and-client-configuration)) |
| `~/.claude/plugins` (30 MiB) | frontend-design, mattpocock-skills | Installed plugin store; only mattpocock-skills is enabled ([E3](#e3-plugins-skills-and-client-configuration)) |
| `~/.codex/plugins` (319 MiB) | bundled/primary-runtime plugin applications and caches | Configured plugin runtime/store; enabled list is in `~/.codex/config.toml` ([E3](#e3-plugins-skills-and-client-configuration)) |
| `~/.pi/agent/npm` (262 MiB) | six resolved npm extension packages | Active Pi-managed extension store ([E3](#e3-plugins-skills-and-client-configuration)) |
| `~/.omp/agent/extensions/agentlens.ts` | Agentlens | Active symlinked OMP extension; repository owns source ([E6](#e6-agentlens-and-repository-local-functions)) |

## MCP topology

| Client/config owner | Server/component | State |
|---|---|---|
| `~/.omp/agent/mcp.json` | `confluence-local` (stdio) | Configured, implicit enabled ([E5](#e5-mcp-topology)) |
| `~/.omp/agent/mcp.json` | `hdir_confluence` (Atlassian HTTP) | Installed/configured but explicitly disabled ([E5](#e5-mcp-topology)) |
| `~/.copilot/mcp-config.json` | `notion-sentinel` (HTTP) | Configured; file mode `0600` ([E5](#e5-mcp-topology), [E7](#e7-credential-and-authentication-stores)) |
| `~/Library/Application Support/Code/User/mcp.json` | `notion` (HTTP) | Configured for VS Code ([E5](#e5-mcp-topology)) |
| `~/.codex/config.toml` | `node_repl` | Configured ([E5](#e5-mcp-topology)) |
| `~/.codex/config.toml` | `computer-use` | Explicitly disabled; supporting assets remain under `~/.codex/computer-use` ([E5](#e5-mcp-topology)) |
| `~/.pi/agent/mcp-cache.json` | `confluence-hdir` | Cached discovery only; active source unresolved ([E5](#e5-mcp-topology)) |
| `~/.claude/.claude.json` and Claude desktop config | project/desktop MCP maps | Agentlens project map exists but is empty; desktop config has no `mcpServers` ([E5](#e5-mcp-topology)) |
| `~/.mcp-auth/mcp-remote-0.1.37` | MCP OAuth material | Authentication support residue; client/server owner unresolved ([E7](#e7-credential-and-authentication-stores)) |

Server URLs, OAuth client data, headers, and token values are deliberately omitted. Configuration presence does not prove a server was reachable at snapshot time.

## Authentication and configuration stores

| Owning path | Data category | Mode / ownership status |
|---|---|---|
| `~/.codex/auth.json` | Codex auth mode plus API-key/token categories | `0600`; values not read ([E7](#e7-credential-and-authentication-stores)) |
| `~/.pi/agent/auth.json` | Pi provider credentials for `openai-codex` and `github-copilot` | `0600`; values not read ([E7](#e7-credential-and-authentication-stores)) |
| `~/.omp/agent/agent.db` table `auth_credentials` | OMP provider credential records (3 rows at snapshot) | SQLite owner confirmed; record data not read ([E7](#e7-credential-and-authentication-stores)) |
| `~/.omp/agent/.env` | OMP environment/API-key category | **`0644`** at snapshot; value not read. This is an observed permission fact, not a remediation decision. ([E7](#e7-credential-and-authentication-stores)) |
| `~/.copilot/config.json`, `mcp-config.json`, `mcp-oauth-config/` | Copilot account, trust, MCP and OAuth state | files `0600`, OAuth directory client-owned; values not read ([E7](#e7-credential-and-authentication-stores)) |
| `~/Library/Application Support/Claude/Cookies` and related local/session storage | Claude desktop web/session authentication state | cookie file `0600`; contents not read ([E7](#e7-credential-and-authentication-stores)) |
| `~/Library/Application Support/Codex/Default/{Cookies,Login Data}` | Chromium-style app authentication state | file ownership observed; exact product boundary unresolved ([E4](#e4-client-state-and-disk-use), [E7](#e7-credential-and-authentication-stores)) |
| `~/Library/Application Support/Code/User/globalStorage/state.vscdb` | VS Code extension global state, potentially including account references | shared editor owner; values not read ([E2](#e2-vs-code-and-editor-integrations), [E7](#e7-credential-and-authentication-stores)) |
| `~/Library/Keychains/login.keychain-db` | shared macOS keyring used by GitHub CLI and potentially app clients | shared owner intentionally unresolved; contents not inspected ([E7](#e7-credential-and-authentication-stores)) |
| `~/.mcp-auth/mcp-remote-0.1.37` | MCP OAuth tokens/verifier/client metadata | 16 KiB; values not read; initiating client unresolved ([E7](#e7-credential-and-authentication-stores)) |

## Histories, caches, temporary data, and obsolete copies

| Path | Approximate disk use | Classification |
|---|---:|---|
| `~/.claude` | 879 MiB | Active config plus CLI sessions, project transcripts, file history, tasks/jobs, paste and plugin caches ([E4](#e4-client-state-and-disk-use)) |
| `~/Library/Application Support/Claude` | 13 GiB | Active desktop support plus bundled VM/runtime and browser caches ([E4](#e4-client-state-and-disk-use)) |
| `~/.codex` | 1.6 GiB | Active config plus sessions/history/plugins/cache/temp ([E4](#e4-client-state-and-disk-use)) |
| `~/.cache/codex-runtimes` | 1.5 GiB | Codex runtime cache ([E4](#e4-client-state-and-disk-use)) |
| `~/.copilot` | 339 MiB | Active config plus CLI package/session/history cache ([E4](#e4-client-state-and-disk-use)) |
| `~/.omp` | 1.2 GiB | Active agent state plus browser/native/cache/log data ([E4](#e4-client-state-and-disk-use)) |
| `~/.pi` | 511 MiB | Active Pi state plus npm/MCP/session/context caches ([E4](#e4-client-state-and-disk-use)) |
| `~/.vscode/extensions` | 8.0 GiB | Current and retained extension copies; `.obsolete` is authoritative for the 19 old Claude/ChatGPT versions ([E2](#e2-vs-code-and-editor-integrations), [E4](#e4-client-state-and-disk-use)) |
| `~/Library/Application Support/Code/CachedExtensionVSIXs` | 387 MiB | Download cache, including current/old agent VSIXs and legacy Copilot VSIXs ([E2](#e2-vs-code-and-editor-integrations)) |
| `~/Library/Application Support/Agentlens` | 128 KiB | Active local SQLite usage-event history ([E6](#e6-agentlens-and-repository-local-functions)) |
| `/var/folders/k0/cvbttqs57rsft5k0t_ty5ky00000gp/T` | volatile | macOS temp root retained hundreds of Codex-named entries plus MCP, OMP, Pi, Confluence, and VS Code residue. Classification is name/location only; no item is declared safe to delete. ([E8](#e8-temporary-residue)) |

## Package-manager ownership and unresolved duplicates

- **Homebrew formulas:** OMP 18.1.2 and `pi-coding-agent` Cellar 0.78.0 are linked installs. Pi's executable reports 0.80.2, so the receipt and runtime disagree. ([E1](#e1-clients-apps-and-package-owners))
- **Homebrew casks:** Codex is owned at 0.144.1; Copilot CLI is stored under Caskroom 1.0.3 but reports 1.0.59. Claude, ChatGPT, and VS Code have Homebrew receipts older than their self-updated/current application bundles. ([E1](#e1-clients-apps-and-package-owners))
- **Global npm (`/opt/homebrew/lib`):** owns Claude Code 2.1.241 and older/global copies of `@asmundwien/pi-kit`, `@juicesharp/rpiv-web-tools`, `context-mode`, `pi-schedule-prompt`, and `pi-subagents`. Pi's configured dependency owner is instead `~/.pi/agent/npm`. ([E1](#e1-clients-apps-and-package-owners), [E3](#e3-plugins-skills-and-client-configuration))
- **Cargo:** `cargo install --list` records Agentlens 0.1.0 from `/Users/asmund.wien/source/private/agentlens` and owns `~/.cargo/bin/agentlens`. The different 3.0 MiB `~/.local/bin/agentlens` wins normal `PATH` resolution but has no observed package-manager receipt; it is an unresolved duplicate. ([E6](#e6-agentlens-and-repository-local-functions))
- **Client-managed stores:** Claude plugins, Codex plugins/runtime, Pi npm extensions, VS Code extensions, and Copilot's `pkg/` are owned by their respective clients rather than by the global package-manager record. ([E2](#e2-vs-code-and-editor-integrations), [E3](#e3-plugins-skills-and-client-configuration))

## Agentlens repository-local functions

This repository contains no tracked `AGENTS.md`, `CLAUDE.md`, `.mcp.json`, `.claude/`, `.codex/`, `.vscode/`, or project-local client settings at the inspected commit. Its local agent-tooling functions are code, not configuration: ([E6](#e6-agentlens-and-repository-local-functions))

- `integrations/omp/agentlens.ts` observes qualifying OMP skill signals and spawns `~/.cargo/bin/agentlens collect ...` fail-open. The installed OMP extension is a symlink to this file. ([E6](#e6-agentlens-and-repository-local-functions))
- `src/claude_hook.rs` normalizes Claude Code `UserPromptExpansion` slash commands and successful `PostToolUse`/`Skill` results without retaining raw hook payload fields. ([E6](#e6-agentlens-and-repository-local-functions))
- `src/main.rs` exposes `collect`, `claude-hook`, and `report`; `src/storage.rs` owns the default local SQLite store at `~/Library/Application Support/Agentlens/agentlens.sqlite3`. ([E6](#e6-agentlens-and-repository-local-functions))
- `CONTEXT.md` defines Agentlens' domain language and explicitly names OMP and Claude Code as the initial supported clients. ([E6](#e6-agentlens-and-repository-local-functions))

## Remaining uncertainty

1. Homebrew receipts do not explain the newer runtime versions reported by Pi, Copilot CLI, Claude.app, ChatGPT.app, and VS Code. This report records both sides and does not label either corrupt. ([E1](#e1-clients-apps-and-package-owners))
2. The product boundary of `~/Library/Application Support/Codex` is not definitive because no standalone Codex app exists while ChatGPT's installed bundle identifier is `com.openai.codex`. ([E1](#e1-clients-apps-and-package-owners), [E4](#e4-client-state-and-disk-use))
3. Shared macOS Keychain records were not enumerated, so per-client keychain ownership remains unresolved by design. ([E7](#e7-credential-and-authentication-stores))
4. Pi's cached `confluence-hdir` server and `~/.mcp-auth/mcp-remote-0.1.37` do not reveal their authoritative initiating configuration/client without inspecting sensitive or transient state. ([E5](#e5-mcp-topology), [E7](#e7-credential-and-authentication-stores))
5. Negative browser-extension evidence was not exhaustive across every browser profile; this baseline makes no machine-wide absence claim for browser integrations.

## Evidence

All paths below use `~` for `/Users/asmund.wien`. Commands were run from a read-only inventory session; output summaries omit secrets.

### E1. Clients, apps, and package owners

```sh
which claude codex copilot omp pi code agentlens chatgpt
claude --version                 # 2.1.241 (Claude Code)
codex --version                  # codex-cli 0.144.1
copilot --version                # GitHub Copilot CLI 1.0.59
omp --version                    # omp/18.1.2
pi --version                     # 0.80.2
code --version                   # 1.134.0, arm64
readlink /opt/homebrew/bin/{claude,codex,copilot,omp,pi,code}
brew info omp
brew info pi-coding-agent
brew info --cask codex
brew info --cask copilot-cli
npm list --global --depth=0
plutil -extract CFBundleShortVersionString raw APP/Contents/Info.plist
```

Direct bundle inspection returned Claude 1.22209.3, ChatGPT 26.825.32147, and VS Code 1.134.0. Homebrew's first-party metadata linked OMP to [omp.sh](https://omp.sh/), Pi to [pi.dev](https://pi.dev/), Codex to the [OpenAI Codex repository](https://github.com/openai/codex), and Copilot CLI to [GitHub's Copilot CLI documentation](https://docs.github.com/en/copilot/concepts/agents/about-copilot-cli).

### E2. VS Code and editor integrations

```sh
code --list-extensions --show-versions
# anthropic.claude-code@2.1.258
# openai.chatgpt@26.825.51511

du -sh ~/.vscode/extensions \
  ~/Library/Application\ Support/Code/CachedExtensionVSIXs \
  ~/Library/Application\ Support/Code/User/{globalStorage,workspaceStorage}
```

Direct reads of `~/.vscode/extensions/extensions.json` and `.obsolete` confirmed the two current registry entries and 19 obsolete Claude/ChatGPT entries. `/Applications/Visual Studio Code.app/Contents/Resources/app/extensions/copilot/package.json` identifies bundled `copilot-chat` 0.62.0. `~/Library/Application Support/Code/User/settings.json` has `github.copilot.nextEditSuggestions.enabled: true`; `globalStorage/github.copilot-chat` contains Copilot-owned session/embeddings stores.

### E3. Plugins, skills, and client configuration

```sh
du -sh ~/.agents/skills ~/.claude/skills ~/.codex/skills \
  ~/.codex/vendor_imports/skills ~/.claude/plugins ~/.codex/plugins \
  ~/.pi/agent/npm
readlink ~/.claude/skills/unslop
npm list --global --depth=0
```

Names, enabled booleans, and package versions came from redacted structural reads of `~/.claude/settings.json`, `~/.claude/plugins/installed_plugins.json`, `~/.codex/config.toml`, `~/.pi/agent/settings.json`, `~/.pi/agent/npm/package.json`, and `package-lock.json`. No scalar credential value was emitted.

### E4. Client state and disk use

```sh
du -sh ~/.claude ~/.codex ~/.copilot ~/.omp ~/.pi ~/.agents \
  ~/.cache/codex-runtimes ~/.vscode/extensions \
  /Applications/{Claude.app,ChatGPT.app,Visual\ Studio\ Code.app} \
  ~/Library/Application\ Support/{Claude,Codex,Code,Agentlens,com.openai.chat}
du -sk ~/.codex/{sessions,archived_sessions,plugins,.tmp,cache,computer-use}
du -sk ~/Library/Application\ Support/Claude/{vm_bundles/claudevm.bundle,claude-code,claude-code-vm,local-agent-mode-sessions}
```

Directory listings classified histories, SQLite databases, caches, runtime stores, project/session data, and app-support subtrees by path/name only. SQLite schema/table inspection counted OMP history rows without selecting conversation rows.

### E5. MCP topology

Redacted structural reads (server names, transport, `enabled` booleans only) covered:

```text
~/.omp/agent/mcp.json
~/.copilot/mcp-config.json
~/Library/Application Support/Code/User/mcp.json
~/.codex/config.toml
~/.pi/agent/mcp-cache.json
~/.claude/.claude.json
~/Library/Application Support/Claude/claude_desktop_config.json
```

The transport/config classifications align with the clients' first-party config surfaces, including OpenAI's [Codex configuration reference](https://developers.openai.com/codex/config-reference/). URLs, headers, command arguments that could contain credentials, and OAuth values were omitted.

### E6. Agentlens and repository-local functions

```sh
cargo install --list
cargo metadata --no-deps --format-version 1
readlink ~/.omp/agent/extensions/agentlens.ts
cmp ~/.omp/agent/extensions/agentlens.ts integrations/omp/agentlens.ts
cmp ~/.local/bin/agentlens ~/.cargo/bin/agentlens   # different
```

Source evidence: `Cargo.toml` (crate 0.1.0), `integrations/omp/agentlens.ts:136-184`, `src/claude_hook.rs:5-53`, `src/main.rs:13-149`, `src/storage.rs`, `README.md:26-47,49-94,140-144`, and `CONTEXT.md:1-28`. `git ls-files` established the absence of tracked project-local client configuration at the inspected commit.

### E7. Credential and authentication stores

```sh
stat -f '%Sp %N' ~/.codex/auth.json ~/.pi/agent/auth.json \
  ~/.omp/agent/.env ~/.omp/agent/config.yml \
  ~/.copilot/config.json ~/.copilot/mcp-config.json \
  ~/Library/Application\ Support/Claude/Cookies
sqlite3 ~/.omp/agent/agent.db '.schema auth_credentials'
sqlite3 ~/.omp/agent/agent.db 'select count(*) from auth_credentials;'
gh auth status   # account/keyring status only; masked token output discarded
```

Only top-level field/provider names were read from Codex and Pi auth JSON. OMP's schema established that `auth_credentials.data` is credential-bearing; the column was never selected. Keychain, cookie, OAuth, and token contents were not inspected.

### E8. Temporary residue

A filename-only `os.walk` under `/var/folders/k0/cvbttqs57rsft5k0t_ty5ky00000gp/T` counted entries containing `.com.openai.codex`, `mcp-`, `omp`, `pi`, `ultra-confluence`, or `vscode` and summed regular-file sizes without opening contents. Counts and sizes are deliberately treated as volatile discovery evidence, not package ownership or deletion guidance.
