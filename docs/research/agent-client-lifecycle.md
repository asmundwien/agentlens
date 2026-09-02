# Supported agent-client lifecycle and cleanup controls

Research date: 2026-09-02  
Scope: the installed Claude, Codex, GitHub Copilot, OMP, Pi, ChatGPT, Visual Studio Code, and Agentlens components on this macOS workstation.

## Safety and interpretation

No install, update, uninstall, logout, reset, prune, garbage-collection, cache-cleanup, plugin-removal, worktree-removal, or data-deletion command in this report was run. Destructive commands are recorded as reference material only. Read-only version, package inventory, help, and directory-name inspections were used; no credential value was read or recorded.

The controls below belong to different layers and must not be conflated:

1. **Application/package removal** removes manager-owned executable or app artifacts and its package record.
2. **Zap/clean reset** additionally removes named user-data paths.
3. **Logout/revocation** acts on authentication, not necessarily settings or sessions.
4. **Session deletion** can have separate local and cloud copies.
5. **Plugin/extension removal** is narrower than application removal.
6. **Cache/GC** may archive or evict only explicitly named data.
7. **Account deletion** is a remote data lifecycle and is not app uninstall.

`Unknown` means the reviewed first-party product docs and source do not define the behavior. A package-manager path list is evidence that the manager targets a path, but not evidence of what every file in the path means.

## Local installation baseline

Direct, read-only inspection used `which`, each client's version/help command, `realpath`, `npm ls -g --depth=0 --json`, `brew list --versions`, `brew info --json=v2`, `cargo install --list`, application `Info.plist` version reads, and `code --list-extensions --show-versions`.

| Component | Observed installation and version | Package/state observation |
|---|---|---|
| Claude Code CLI | `2.1.241`, `/opt/homebrew/bin/claude` resolving into global npm package `@anthropic-ai/claude-code` | npm owns the executable/package; user state is under `~/.claude` and macOS Keychain/fallback credential storage. |
| Claude Desktop | `/Applications/Claude.app`, `1.22209.3`; Homebrew cask record exists | Homebrew's recorded installed cask version is older than the app bundle, consistent with an app-managed update. |
| Claude Code VS Code extension | `anthropic.claude-code@2.1.258` | Managed by VS Code, while substantial shared Claude state remains under `~/.claude`. |
| Codex CLI | `codex-cli 0.144.1`, Homebrew cask binary | `~/.codex` contains configuration, credentials category, sessions, databases, logs, caches, plugins, and Computer Use support data. No values were opened. |
| GitHub Copilot CLI | CLI reports `1.0.59`, Homebrew cask record `1.0.3` | The executable can self-update independently of the Homebrew receipt. `~/.copilot` contains settings, authentication category, sessions, plugins, caches, and IDE state. |
| OMP | `omp/18.1.2`, Homebrew formula `can1357/tap/omp` | `~/.omp` contains settings/auth databases, history/models, sessions/blobs, plugins, caches, logs, browser assets, and managed native artifacts. |
| Pi | CLI/package content reports `0.80.2`; Homebrew formula receipt/keg reports `0.78.0` | The mismatch is observed, not explained. `~/.pi/agent` contains settings, authentication category, sessions, package payloads, caches, and logs. |
| ChatGPT | `/Applications/ChatGPT.app`, `26.825.32147`, Homebrew cask record exists | The current app includes Chat, Work, and Codex. Local Codex and app state spans `~/.codex` and app containers/support paths. |
| ChatGPT VS Code extension | `openai.chatgpt@26.825.51511` | Managed by VS Code; Codex CLI and IDE share cached login details. |
| Visual Studio Code | `/Applications/Visual Studio Code.app`, `1.134.0`; Homebrew cask record `1.108.2` | The app has self-updated beyond the cask receipt. User settings, extension payloads, storage, sessions, and caches exist under the documented VS Code user-data roots. |
| Agentlens | Cargo install record `agentlens v0.1.0`, executable at `~/.local/bin/agentlens` | Durable data is `~/Library/Application Support/Agentlens/agentlens.sqlite3`; client integrations are separate OMP and Claude configuration. |

A stale package receipt does not prove the live bundle/binary version. The Copilot, Pi, VS Code, and desktop-app differences above are why lifecycle work should identify the **owning updater currently in use** before making changes.

## Package-manager controls shared by several clients

- Homebrew `upgrade` updates outdated, unpinned packages. `uninstall` removes the installed formula/cask. `uninstall --cask --zap` additionally removes all files declared by the cask's `zap` stanza and may remove files shared by applications. `brew cleanup` removes stale locks/outdated downloads, downloads older than its age threshold, and old versions of installed formulae; it is not application-state cleanup. See the [Homebrew man page](https://docs.brew.sh/Manpage#uninstall-remove-rm-options-installed_formulacask-) and its [`cleanup` definition](https://docs.brew.sh/Manpage#cleanup-options-formulacask-).
- A global npm uninstall removes everything npm installed for that package from the global prefix. It does not claim to remove application-owned state elsewhere. See [`npm uninstall`](https://docs.npmjs.com/cli/v11/commands/npm-uninstall/).
- Cargo installs binaries in the selected install root and normally tracks packages in install-root metadata. `cargo uninstall` removes a package installed by Cargo, by default all of its binaries. See [`cargo install`](https://doc.rust-lang.org/cargo/commands/cargo-install.html) and [`cargo uninstall`](https://doc.rust-lang.org/cargo/commands/cargo-uninstall.html).

## Claude

### Claude Code CLI

**Install and update**

- Anthropic recommends the native installer and also documents Homebrew. Native installs auto-update in the background; `claude update` applies an update immediately. Homebrew installs require `brew upgrade claude-code` or `brew upgrade claude-code@latest`, unless package-manager auto-update is explicitly enabled. `DISABLE_AUTOUPDATER=1` disables background checks but not manual update; `DISABLE_UPDATES` blocks both. Homebrew keeps old formula versions until `brew cleanup`. [Anthropic setup and update documentation](https://code.claude.com/docs/en/setup#update-claude-code)
- The installed copy here is the legacy global npm package. The documented manager action to replace/reinstall it is npm's global install; the documented removal is `npm uninstall -g @anthropic-ai/claude-code`. npm removes package-owned files and its global package record, not `~/.claude` or a macOS Keychain item. [Anthropic uninstall documentation](https://code.claude.com/docs/en/setup#uninstall-claude-code), [npm uninstall semantics](https://docs.npmjs.com/cli/v11/commands/npm-uninstall/)

**Uninstall and reset**

- Anthropic keeps executable removal separate from state removal. Native removal deletes `~/.local/bin/claude` and `~/.local/share/claude`; Homebrew removes its cask; npm removes its global package. None of those product instructions says it removes user configuration. [Claude Code uninstall methods](https://code.claude.com/docs/en/setup#uninstall-claude-code)
- The destructive state reset is separately documented as removing `~/.claude`, `~/.claude.json`, and, per project, `.claude` and `.mcp.json`. Anthropic warns this deletes settings, allowed tools, MCP server configuration, and session history. Desktop and IDE clients also write `~/.claude`, so they can recreate it. [Configuration removal](https://code.claude.com/docs/en/setup#remove-configuration-files)
- On macOS, Claude normally stores credentials in Keychain and falls back to `~/.claude/.credentials.json` with mode `0600` if Keychain cannot accept the write. Filesystem reset does not have a documented guarantee that it removes the Keychain entry. [Claude authentication storage](https://code.claude.com/docs/en/authentication#credential-management)

**Authentication and sessions**

- `/logout` removes/revokes the credential created by supported sign-in flows and resets first-launch setup. It does not document removal of transcripts, settings, or plugins. Environment credentials and cloud-provider credentials are separate. [Claude authentication](https://code.claude.com/docs/en/authentication#log-in-to-claude-code)
- `claude project purge PATH --dry-run` previews a project purge. The applied operation removes that project's transcripts, auto-memory, matching task/debug/file-history entries, prompt history, and its `~/.claude.json` project entry. `--all` expands the scope. Shell snapshots and configuration backups are not project-scoped purge targets; backups may retain old project entries until rotation. [Claude application-data cleanup](https://code.claude.com/docs/en/claude-directory#delete-data-for-one-project)
- `/clear` resets conversational context but is not a durable transcript/settings/auth reset. [Claude commands](https://code.claude.com/docs/en/commands)

**Garbage collection and caches**

- Claude Code runs an age-based sweep with `cleanupPeriodDays` (default 30, minimum 1) across transcripts and named derived artifacts including tool/subagent data, file history, plans, debug data, paste/image/upload caches, session environment, tasks, crash shell snapshots, feedback drafts/bundles, usage cache, and legacy directories. Running-session markers are removed on normal exit and crash leftovers on the next launch. [Authoritative retention inventory](https://code.claude.com/docs/en/claude-directory#application-data)
- Auto-memory contents are excluded from this age sweep. Claude Desktop/Cowork transcripts are kept indefinitely by default unless `desktopSessionCleanupPeriodDays` or managed policy supplies a limit. `history.jsonl`, usage totals, changelog cache, and named policy caches are not generally age-swept; logout deletes only the documented remote-settings and policy-limit caches. [Retention exceptions](https://code.claude.com/docs/en/claude-directory#kept-until-you-delete-them)
- Anthropic publishes a manual safe-delete table. It explicitly says not to delete `~/.claude.json`, `~/.claude/settings.json`, or `~/.claude/plugins/` merely to clear local data because they contain auth/preferences/plugins. [Manual cleanup table](https://code.claude.com/docs/en/claude-directory#clear-local-data)

**VS Code extension cleanup**

- VS Code's Extensions view can uninstall Claude Code. To prevent CLI-driven reinstallation, disable the auto-install setting or `CLAUDE_CODE_IDE_SKIP_AUTO_INSTALL=1`. Anthropic documents deleting `~/Library/Application Support/Code/User/globalStorage/anthropic.claude-code` as the extension-specific data/settings reset on macOS. This is not documented as deleting shared `~/.claude` state or Keychain credentials. [Claude Code for VS Code](https://code.claude.com/docs/en/vs-code#uninstall-the-extension)

### Claude Desktop

- Anthropic documents the official download/enterprise `.pkg` or `.dmg` installation and automatic updates. [Desktop install](https://support.claude.com/en/articles/10065433-install-claude-desktop), [macOS enterprise deployment](https://support.claude.com/en/articles/12611117-deploy-claude-desktop-for-macos)
- Anthropic provides no first-party Desktop uninstall, factory reset, local cache purge, GC, or file-by-file preservation contract found in the reviewed docs: **unknown**.
- Homebrew's cask removes the app in ordinary uninstall. Its separate `zap` stanza targets Claude Application Support, recent-document metadata, caches and ShipIt cache, HTTP storage, logs, preferences, and saved state. The cask does not classify which contain auth/settings/sessions and does not target the Keychain. [Homebrew Claude cask](https://github.com/Homebrew/homebrew-cask/blob/HEAD/Casks/c/claude.rb)
- Desktop extensions can be installed and updated, and sensitive fields use OS secure storage, but Anthropic does not document end-user extension uninstall or secure-credential cleanup: **unknown**. [Claude Desktop extensions](https://support.claude.com/en/articles/10949351-getting-started-with-local-mcp-servers-on-claude-desktop)

## Codex CLI

**Install, update, and package removal**

- OpenAI supports the standalone installer, npm, and Homebrew. Rerun the standalone installer or npm install to update; Homebrew uses `brew upgrade --cask codex`. [Codex CLI install/update](https://learn.chatgpt.com/docs/codex/cli#getting-started), [official repository README](https://github.com/openai/codex#installing-and-running-codex-cli)
- OpenAI does not publish a general uninstall section. Its installer source identifies manager-specific conflict-removal commands, including `brew uninstall --cask codex` and `npm uninstall -g @openai/codex`, but no standalone-uninstall action. Standalone removal and shell-profile cleanup are therefore **unknown**. [OpenAI standalone installer](https://chatgpt.com/codex/install.sh)
- The installed Homebrew cask's normal uninstall removes the `codex` binary and Homebrew cask record. Its `zap` stanza only declares `rmdir ~/.codex`; `rmdir` does not recursively erase a populated tree. [Homebrew Codex cask](https://github.com/Homebrew/homebrew-cask/blob/HEAD/Casks/c/codex.rb)

**Settings, authentication, and sessions**

- User config lives at `~/.codex/config.toml`; project overrides use trusted project `.codex/config.toml` files. [Codex configuration](https://learn.chatgpt.com/docs/config-file/config-basic)
- `codex logout` removes saved credentials for API-key and ChatGPT authentication. The CLI and IDE extension share cached login, so logout from either requires sign-in on the next use of both. Credentials are in `$CODEX_HOME/auth.json` or the OS credential store according to `cli_auth_credentials_store=file|keyring|auto`. Logout does not document deletion of config, transcripts, history, caches, plugins, or external environment credentials. [Codex authentication and credential storage](https://learn.chatgpt.com/docs/auth#credential-storage), [CLI reference](https://learn.chatgpt.com/docs/developer-commands?surface=cli#codex-logout)
- `codex archive SESSION` hides a session from the active picker but preserves the transcript; `unarchive` restores it. `codex delete SESSION` permanently deletes a saved interactive transcript. Live and archived transcripts are under `$CODEX_HOME/sessions` and `$CODEX_HOME/archived_sessions`. [Codex session commands](https://learn.chatgpt.com/docs/developer-commands?surface=cli#codex-archive-and-codex-unarchive), [app troubleshooting paths](https://learn.chatgpt.com/docs/reference/troubleshooting#feedback-and-logs)
- OpenAI documents no bulk local retention TTL, factory reset, or general cache-clean command: **unknown**. Deleting `$CODEX_HOME` should not be presented as supported reset guidance.

**Plugin/extension cleanup**

- `codex plugin remove` removes an installed plugin; connector authorization may remain connected in ChatGPT and must be managed separately. Marketplace removal, plugin removal, and connector disconnection are distinct. [Codex plugin commands](https://learn.chatgpt.com/docs/developer-commands?surface=cli#codex-plugin)
- OpenAI's IDE docs do not publish extension-uninstall or extension-storage reset semantics. VS Code can generically uninstall `openai.chatgpt`, but whether that removes extension settings/session state is **unknown**. CLI/IDE shared authentication remains governed by `codex logout`.

**Update garbage collection**

- The standalone installer removes interrupted staging/current temporary paths and replaces incomplete or same-version release directories. It has no documented user-facing old-release or application-cache GC control. Whether completed older standalone releases are retained is **unknown**. [Installer source](https://chatgpt.com/codex/install.sh)

## GitHub Copilot CLI

**Install, update, and uninstall**

- GitHub supports Homebrew, npm, an install script, and direct downloads. `copilot update` downloads/installs the latest version; `copilot version` checks. [Copilot CLI installation](https://docs.github.com/en/copilot/how-tos/copilot-cli/set-up-copilot-cli/install-copilot-cli), [command reference](https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-command-reference)
- For this Homebrew cask installation, ordinary uninstall removes the binary/cask record and does not invoke the cask's state zap. `brew uninstall --cask --zap copilot-cli` additionally deletes `~/.copilot`. The cask does not list a Keychain item, so complete OAuth credential cleanup through zap is **unknown**. [Homebrew Copilot CLI cask](https://github.com/Homebrew/homebrew-cask/blob/HEAD/Casks/c/copilot-cli.rb), [Homebrew zap semantics](https://docs.brew.sh/Manpage#uninstall-remove-rm-options-installed_formulacask-)
- GitHub publishes no uninstall procedure for script/direct-download installs: **unknown**. npm's supported global removal is `npm uninstall -g @github/copilot`, which removes npm-owned package files rather than `~/.copilot`.

**State and cache inventory**

- GitHub documents `~/.copilot` as the root for configuration, auth application state, complete session files, session database, command history, logs, permissions, agents/instructions/skills/hooks/extensions, installed plugins/plugin data, MCP/LSP config, and MCP secret/OAuth fallback storage. Its separate macOS cache root is `~/Library/Caches/copilot` for marketplace caches, update packages, and other ephemeral data. [Copilot configuration-directory reference](https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-config-dir-reference)
- There is no single supported factory-reset command or general cache purge. GitHub's safe-delete table permits logs and plugin data; deleting session state loses resumability, deleting settings resets preferences, and deleting MCP OAuth fallback state can require reauthentication. Installed-plugin payloads should be removed through plugin commands to keep metadata consistent. [Safe-delete table](https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-config-dir-reference#what-you-can-safely-delete)
- Disabling MCP tool snapshots leaves existing snapshot files untouched; it is not cache cleanup. [MCP tool snapshot caching](https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-command-reference#tool-snapshot-caching)

**Authentication and sessions**

- `/logout` removes the locally stored Copilot token but does not revoke it on GitHub. Server revocation is separate under GitHub account authorized applications. Environment tokens and a `gh` credential fallback are independent and are not removed by Copilot logout. [Copilot authentication](https://docs.github.com/en/copilot/how-tos/copilot-cli/set-up-copilot-cli/authenticate-copilot-cli)
- `/permissions reset` clears in-memory approvals for the current session. A saved project approval can be reset by removing the relevant `permissions-config.json` entry, but GitHub warns not to edit it while a session is running. `/clear`, `/new`, and `/reset` start a new conversation rather than resetting durable client state. [Copilot command reference](https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-command-reference)
- `/session delete`, `/session delete SESSION-ID`, `/session delete-all`, and `/session prune --older-than DAYS` are supported local session cleanup controls; prune supports `--dry-run`. Single-session deletion can optionally delete its synced copy. Delete-all/prune affect local sessions only and skip in-use sessions; synced copies require GitHub.com cleanup. [Copilot session data](https://docs.github.com/en/copilot/concepts/agents/copilot-cli/chronicle#managing-your-session-data)

**Plugin/extension cleanup**

- `copilot plugin uninstall NAME` removes a plugin; removing a marketplace is refused while its plugins remain unless `--force` also uninstalls them. Skill removal by name deletes copied skill files, while removing a registered custom-directory source leaves that source directory on disk. [Copilot plugin reference](https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-plugin-reference)

No GitHub Copilot VS Code extension is currently active in `code --list-extensions`; only an old cached VSIX name was observed. If reinstalled, VS Code's generic extension lifecycle below applies, but first-party docs do not promise that extension uninstall removes Copilot settings, chat sessions, or authentication.

## OMP / Oh My Pi

**Install, update, and uninstall gap**

- OMP documents curl, Homebrew, Bun, Nix, and mise installation. This workstation's executable is Homebrew formula `can1357/tap/omp`. [OMP README install section](https://github.com/can1357/oh-my-pi/blob/984a4f2dc9e50f6645b8fe04a91570876f8d3c83/README.md#L44-L91)
- `omp update` detects the owning manager. `--check` is read-only, `--force` reinstalls, `--canary`/`--stable` switch channels, and `--plugins` updates installed plugins. Homebrew, mise, Bun/npm, and standalone paths update their own installation surface; Nix tells the user to update/rebuild through Nix. The updater's Bun path also best-effort prunes stale Bun package cache entries. [OMP updater source](https://github.com/can1357/oh-my-pi/blob/984a4f2dc9e50f6645b8fe04a91570876f8d3c83/packages/coding-agent/src/cli/update-cli.ts#L1948-L2078)
- OMP has no product-level uninstall command or end-to-end removal contract. Homebrew supports `brew uninstall can1357/tap/omp`, which removes the keg/link/package record. The OMP formula has no state-removal hook, but OMP itself does not publish a preservation guarantee for `~/.omp`: exact whole-application uninstall semantics are **unknown**. The curl standalone installer has no documented uninstaller. [OMP installer source](https://github.com/can1357/oh-my-pi/blob/984a4f2dc9e50f6645b8fe04a91570876f8d3c83/scripts/install.sh#L264-L304)

**Settings, authentication, and sessions**

- Global state/settings default to `~/.omp`; the active agent directory is normally `~/.omp/agent`, relocatable with `PI_CODING_AGENT_DIR`. `omp config reset KEY` writes that one setting's schema default; it does not delete the key or reset project settings/auth/sessions/plugins/caches. There is no documented all-settings/factory reset. [OMP settings lifecycle](https://github.com/can1357/oh-my-pi/blob/984a4f2dc9e50f6645b8fe04a91570876f8d3c83/docs/settings.md#L20-L76)
- `/logout PROVIDER` soft-deletes the stored credentials for that provider, clears its in-memory/provider-assignment cache, and leaves other providers and other state intact. Environment-provided credentials are outside the database and remain. No all-provider wipe is documented. [OMP authentication storage](https://github.com/can1357/oh-my-pi/blob/984a4f2dc9e50f6645b8fe04a91570876f8d3c83/packages/ai/src/auth-storage.ts#L2694-L2725)
- Sessions, content-addressed blobs, and terminal breadcrumbs have separate roots under the agent directory. `/clear` adds a logical context boundary; full-history export still includes prior entries. It is not deletion. No first-party session-delete/bulk-purge action was found. [OMP session layout and clear boundary](https://github.com/can1357/oh-my-pi/blob/984a4f2dc9e50f6645b8fe04a91570876f8d3c83/docs/session.md#L25-L52)

**Supported GC and worktree cleanup**

- `omp gc` is dry-run by default; mutation requires `--apply`. It can independently sweep unreferenced blobs, archive cold sessions, and checkpoint/truncate history/model database WALs. [GC command source](https://github.com/can1357/oh-my-pi/blob/984a4f2dc9e50f6645b8fe04a91570876f8d3c83/packages/coding-agent/src/commands/gc.ts)
- Blob GC scans active and archived session files for `blob:sha256` references and deletes only unreferenced blob files older than a five-minute write grace. Session archive GC skips active/pending/interrupted/unknown sessions, gzip-archives eligible cold JSONL sessions and moves related artifacts, cleans corresponding search/history/stat index rows, and preserves configurable newest-session floors. Defaults are archive after 30 days, retain newest 20 globally and 10 per working directory, with blobs/archive/WAL all enabled. [GC implementation](https://github.com/can1357/oh-my-pi/blob/984a4f2dc9e50f6645b8fe04a91570876f8d3c83/packages/coding-agent/src/cli/gc-cli.ts), [GC setting defaults](https://github.com/can1357/oh-my-pi/blob/984a4f2dc9e50f6645b8fe04a91570876f8d3c83/packages/coding-agent/src/config/settings-schema.ts#L6017-L6027)
- Archiving is not transcript deletion: session JSONL is moved/compressed under the archive root. WAL checkpointing reclaims WAL space without deleting settings/auth records. The command does not purge every OMP cache, log, browser asset, model cache, or plugin cache.
- `omp worktree clear --dry-run` previews cleanup of agent-managed worktrees under `~/.omp/wt`; applied clear mutates worktrees, and `--all` includes live PR-checkout worktrees. The command says nothing about general settings/auth/session cleanup. This command was not run. [OMP worktree command source](https://github.com/can1357/oh-my-pi/tree/984a4f2dc9e50f6645b8fe04a91570876f8d3c83/packages/coding-agent/src/commands)

**Plugin cleanup**

- `omp plugin uninstall` removes the selected scope's plugin registry entry, runtime link, plugin/settings runtime records, and unreferenced install/cache paths; paths referenced by another scope are preserved. npm plugin removal delegates to Bun package removal and then deletes that plugin's runtime records. Marketplace removal deletes marketplace registration/catalog cache but intentionally does not imply plugin uninstall. [OMP marketplace docs](https://github.com/can1357/oh-my-pi/blob/984a4f2dc9e50f6645b8fe04a91570876f8d3c83/docs/marketplace.md#L192-L231), [marketplace uninstall implementation](https://github.com/can1357/oh-my-pi/blob/984a4f2dc9e50f6645b8fe04a91570876f8d3c83/packages/coding-agent/src/extensibility/plugins/marketplace/manager.ts#L451-L516)

**VM data**

- OMP documents browser assets, agent-managed worktrees, runtimes, and caches, but no product-owned VM image/disk lifecycle was found. No VM cleanup claim is supported. Browser assets and VM disks must not be conflated.

## Pi coding agent

The current first-party upstream is [`badlogic/pi-mono`](https://github.com/badlogic/pi-mono). The installed package metadata still names the Earendil package/fork. The lifecycle behavior below is the current documented Pi contract; version-specific differences for the observed `0.80.2` payload are unknown unless also visible in its local `--help`.

**Install, update, and uninstall**

- Current Pi documents global npm installation and a curl installer. `pi update` updates Pi; `--extensions` updates installed packages; `--all` updates both; a source argument updates one package. The installed `0.80.2` help exposes these actions. [Pi quickstart](https://github.com/badlogic/pi-mono/blob/e266507b606b9552fa277252644054afd4384b11/packages/coding-agent/docs/quickstart.md#L5-L34), [Pi usage reference](https://github.com/badlogic/pi-mono/blob/e266507b606b9552fa277252644054afd4384b11/packages/coding-agent/docs/usage.md#L133-L158)
- Pi's documented application uninstall uses the original npm/pnpm/Yarn/Bun manager and explicitly preserves **settings, credentials, sessions, and installed Pi packages** in `~/.pi/agent`. [Pi uninstall contract](https://github.com/badlogic/pi-mono/blob/e266507b606b9552fa277252644054afd4384b11/packages/coding-agent/docs/quickstart.md#L22-L34)
- This copy is instead under a Homebrew `pi-coding-agent` keg. Homebrew uninstall removes the keg/link/receipt; the formula does not define user-state cleanup. The current package's explicit preservation contract supports treating `~/.pi/agent` as separate, but the installed formula/content version mismatch means update ownership should be resolved before acting. [Homebrew Pi formula](https://github.com/Homebrew/homebrew-core/blob/HEAD/Formula/p/pi-coding-agent.rb)

**State, authentication, and sessions**

- There is no product-wide factory reset or all-settings reset. `/new` starts a new session and preserves old session files; `--no-session` makes only that run ephemeral. Settings are global `~/.pi/agent/settings.json` plus project `.pi/settings.json`. [Pi settings](https://github.com/badlogic/pi-mono/blob/e266507b606b9552fa277252644054afd4384b11/packages/coding-agent/docs/settings.md#L1-L10), [Pi sessions](https://github.com/badlogic/pi-mono/blob/e266507b606b9552fa277252644054afd4384b11/packages/coding-agent/docs/sessions.md#L5-L42)
- `/logout` removes only the selected provider's entry from `~/.pi/agent/auth.json`; other provider entries and environment/ambient credentials remain. Sessions/settings/packages are not removed. [Pi providers/auth docs](https://github.com/badlogic/pi-mono/blob/e266507b606b9552fa277252644054afd4384b11/packages/coding-agent/docs/providers.md#L7-L35), [auth deletion source](https://github.com/badlogic/pi-mono/blob/e266507b606b9552fa277252644054afd4384b11/packages/coding-agent/src/core/auth-storage.ts#L479-L489)
- The resume picker supports deleting a selected session with confirmation and uses the platform trash CLI when available. No bulk session retention/GC is documented. Exported/shared copies are not documented as deleted. [Pi session deletion](https://github.com/badlogic/pi-mono/blob/e266507b606b9552fa277252644054afd4384b11/packages/coding-agent/docs/sessions.md#L5-L42)

**Package/extension cleanup and caches**

- `pi remove SOURCE` (`pi uninstall` alias) removes a package and its settings source record. For managed npm/git sources it removes the managed payload; for a local path it unregisters the reference but leaves source files. `pi config` enable/disable preserves package records and payloads. [Pi package lifecycle](https://github.com/badlogic/pi-mono/blob/e266507b606b9552fa277252644054afd4384b11/packages/coding-agent/docs/packages.md#L18-L91), [removal implementation](https://github.com/badlogic/pi-mono/blob/e266507b606b9552fa277252644054afd4384b11/packages/coding-agent/src/core/package-manager.ts#L1040-L1063)
- The managed installer best-effort deletes stale `staging/update-*` directories while holding its update lock. Pi has no documented general cache, model-catalog, auth-cache, or session-GC command. Deletion/retention semantics for `models-store.json` are **unknown**. [Installer staging GC](https://github.com/badlogic/pi-mono/blob/e266507b606b9552fa277252644054afd4384b11/packages/coding-agent/src/package-manager-cli.ts#L143-L174)

**VM data**

- No Pi-owned VM image/disk mechanism or cleanup control is documented. Package sandboxes, sessions, and managed package checkouts are not VM data.

## ChatGPT for macOS

**Install and update**

- The current app is downloaded from ChatGPT and includes Chat, Work, and Codex. OpenAI documents checking for updates/restarting and migration from the former separate Codex app. Existing Codex chats/projects should remain, but preservation of local preferences, worktrees, caches, credentials, and macOS permissions is not specified. [ChatGPT macOS download](https://help.openai.com/en/articles/9275200-downloading-the-chatgpt-macos-app), [desktop-app migration](https://help.openai.com/en/articles/20001276-moving-to-the-new-chatgpt-desktop-app)
- Homebrew's `chatgpt` cask marks the app as self-updating. Ordinary cask uninstall removes the app/cask record. OpenAI provides no first-party macOS uninstall procedure or preservation contract: **unknown**. [Homebrew ChatGPT cask](https://github.com/Homebrew/homebrew-cask/blob/HEAD/Casks/c/chatgpt.rb)
- The cask's separate zap targets Codex/ChatGPT Application Support, caches, HTTP storage, logs, preferences, saved state, group containers, Computer Use authorization support, and `~/.codex`. The metadata does not classify settings/auth/sessions and does not prove account/cloud deletion. Zap may remove shared Codex CLI/IDE state because `~/.codex` is shared.

**Logout, local state, and cloud retention**

- App profile **Log out** clears current app credentials. ChatGPT **Active sessions** can end one first-party session or all first-party sessions/devices. These controls expressly do not manage Codex CLI sessions or third-party-only sign-ins. Their effect on app caches/settings/local transcripts is not documented. [Active sessions](https://help.openai.com/en/articles/20001257-managing-active-sessions-in-chatgpt), [log out all devices](https://help.openai.com/en/articles/9243857-how-do-i-log-out-of-all-of-my-devices)
- OpenAI provides no native macOS factory reset, cache-clear, local credential-purge, or local app-GC action found in the reviewed docs: **unknown**.
- App uninstall is not account deletion and does not cancel a subscription. Chats remain with the account until separately deleted; deleted chats are removed from the account view immediately and normally scheduled for permanent deletion within 30 days, subject to documented exceptions. macOS uploads are cloud/account data. [Subscription cancellation boundary](https://help.openai.com/en/articles/7232927-canceling-your-chatgpt-subscription), [chat/file retention](https://help.openai.com/en/articles/8983778-chat-and-file-retention-policies-in-chatgpt), [macOS data retention](https://help.openai.com/en/articles/9268871-how-is-data-retained-in-the-macos-app)

**Computer Use, Computer History, extensions, and VM data**

- Computer Use plugin disablement and removal of always-allowed apps are in app settings. macOS Screen Recording and Accessibility permissions are separately revoked in macOS Privacy & Security. The docs do not say app uninstall revokes these permissions or removes the locked-use authorization plugin. [Computer Use controls](https://learn.chatgpt.com/docs/computer-use)
- Computer History supports deleting one timeline item or clearing the last 10 minutes/hour/day/all history. Clearing deletes matching local interaction events and generated memories. Temporary events live in the ChatGPT App Group and age out after up to 48 hours; generated Markdown memories remain under `$CODEX_HOME/memories/extensions/skysight/` until deleted/cleared. [Computer History cleanup and retention](https://learn.chatgpt.com/docs/customization/computer-history#review-and-clear-history)
- On macOS, Computer Use drives allowed host apps. OpenAI mentions a VM only as a user-provided Windows isolation option; it documents no ChatGPT-managed macOS VM disk or VM cleanup. The Homebrew CUA paths must not be labeled VM data without evidence.
- Work with Apps can be disabled in ChatGPT settings and its macOS Accessibility permission revoked. OpenAI does not document removal/state cleanup for its VS Code helper extension. VS Code can generically uninstall the observed `openai.chatgpt` extension, but what extension-specific data remains is **unknown**. [Work with Apps](https://help.openai.com/en/articles/10119604-work-with-apps-on-macos)

## Visual Studio Code

**Install, update, and uninstall**

- On macOS, Microsoft documents installing the app in Applications and moving it to Trash to uninstall. VS Code updates itself and prompts when a new version is ready. [macOS setup](https://code.visualstudio.com/docs/setup/mac), [uninstall](https://code.visualstudio.com/docs/setup/uninstall)
- For this cask-tracked app, Homebrew ordinary uninstall removes the app, command-line symlinks, and cask receipt. Its zap stanza targets `~/.vscode`, VS Code Application Support, caches/ShipIt, HTTP storage, preferences, recent documents, and saved state. [Homebrew VS Code cask](https://github.com/Homebrew/homebrew-cask/blob/HEAD/Casks/v/visual-studio-code.rb)
- Microsoft says the comprehensive local reset is deleting `$HOME/Library/Application Support/Code`, `~/.vscode-shared`, and `~/.vscode`, which removes all user data and returns VS Code to pre-install state. This can reset settings without uninstalling the app. It does not document Keychain or Settings Sync cloud deletion. [VS Code clean uninstall/reset](https://code.visualstudio.com/docs/setup/uninstall#_clean-uninstall)

**Settings, profiles, sync, caches, and sessions**

- Individual settings can be reset; deleting entries from user `settings.json` resets all changed user settings but not extensions/sessions/auth. User settings are under `$HOME/Library/Application Support/Code/User/settings.json`. [VS Code settings and reset](https://code.visualstudio.com/docs/configure/settings#_reset-settings)
- Deleting a profile is supported, but Microsoft does not enumerate file, shared-resource, Keychain, or cloud propagation semantics: **unknown**. An Empty Profile is a non-destructive diagnostic isolation that disables custom extensions/modified settings while preserving the existing profile. [VS Code profiles](https://code.visualstudio.com/docs/configure/profiles)
- Settings Sync cloud deletion is separate: turn off sync and select clearing of cloud data. Re-enabling then acts as first sign-in. Local sync backups age out after 30 days and remote backups retain the documented latest versions; clean local uninstall is not documented as clearing the cloud copy. [Settings Sync](https://code.visualstudio.com/docs/configure/settings-sync)
- Microsoft documents no selective macOS Cache/CachedData/workspace-storage cleanup action. The supported broad reset above is not evidence that ad hoc subdirectory deletion is safe.
- Agent-session controls distinguish New, Close, Archive/Done, and permanent Delete. Closing hides a chat; archive is reversible. Delete is irreversible and can remove associated session worktrees; worktree-only uncommitted files can be lost. Extension or app uninstall is not documented as deleting these sessions. [Manage agent sessions](https://code.visualstudio.com/docs/agents/run/sessions/manage-sessions)

**Extension and authentication cleanup**

- Extensions can be disabled without uninstalling; disabled state persists. Uninstall is Extensions view → Manage → Uninstall → Restart Extensions or `code --uninstall-extension PUBLISHER.ID`; `--profile` scopes it. Auto-update normally updates enabled extensions, and manual update/check commands exist. [Extension Marketplace lifecycle](https://code.visualstudio.com/docs/configure/extensions/extension-marketplace#_manage-extensions), [extension CLI](https://code.visualstudio.com/docs/configure/command-line#_working-with-extensions)
- Generic extension uninstall does not promise deletion of an extension's settings, global/workspace storage, credentials, caches, or sessions: **unknown per extension**. This applies to the observed `anthropic.claude-code` and `openai.chatgpt` extensions except where their vendor docs define additional cleanup.
- VS Code account sign-out and remote OAuth revocation are separate. Settings Sync authentication uses the OS keychain; Microsoft provides no supported selective keychain purge and does not say clean uninstall removes it. [VS Code Copilot setup/sign-out](https://code.visualstudio.com/docs/setup/copilot), [GitHub authorization revocation](https://docs.github.com/en/copilot/how-tos/configure-personal-settings/configure-in-ide#revoking-github-copilot-authorization)

**VM data**

- VS Code profiles, extension hosts, agent worktrees, and user-data directories are not VM disks. No VS Code-owned VM cleanup action is documented here. Remote/dev-container VM or Docker data belongs to the remote/container provider and is not documented as removed by VS Code uninstall or reset.

## Agentlens

**Install, update, and package record**

- Agentlens `0.1.0` supports source installation with `cargo install --locked --path .`. Cargo owns the installed executable and install-root package metadata. Rerunning a path install rebuilds/reinstalls, but Agentlens has no explicit release/update workflow. [Agentlens install instructions](https://github.com/asmundwien/agentlens/blob/1eca3cf08320e3a4c66df15d2c52b9db9533be77/README.md#install-the-executable), [Cargo path-install behavior](https://doc.rust-lang.org/cargo/commands/cargo-install.html)
- `cargo uninstall agentlens` (with the original custom `--root` if applicable) removes the Cargo-owned binary/package record. Agentlens explicitly says this does not remove either client integration and never deletes durable data. [Agentlens executable removal](https://github.com/asmundwien/agentlens/blob/1eca3cf08320e3a4c66df15d2c52b9db9533be77/README.md#remove-the-executable)

**Integrations and durable data**

- OMP cleanup removes only the `~/.omp/agent/extensions/agentlens.ts` symlink when it is actually a symlink, then restarts OMP. It preserves Claude integration, executable, and data. Claude cleanup removes only the two exact Agentlens hook handlers and now-empty containers from `~/.claude/settings.json`, preserving unrelated settings/hooks and OMP/data. [Agentlens integration removal](https://github.com/asmundwien/agentlens/blob/1eca3cf08320e3a4c66df15d2c52b9db9533be77/README.md#remove-agentlens)
- Durable state is a local SQLite database/WAL under `~/Library/Application Support/Agentlens`. After both integrations and executable are removed, deleting that entire Agentlens directory is the documented irreversible data reset and removes all observed invocation history. [Agentlens data deletion](https://github.com/asmundwien/agentlens/blob/1eca3cf08320e3a4c66df15d2c52b9db9533be77/README.md#delete-collected-data-separately), [storage path/source](https://github.com/asmundwien/agentlens/blob/1eca3cf08320e3a4c66df15d2c52b9db9533be77/src/storage.rs#L24-L30)
- Agentlens itself has no account/authentication, sessions, VM data, runtime cache, cache-clear, selective event delete, archive, retention, vacuum, or GC command. Build artifacts in the source checkout are Cargo artifacts rather than Agentlens runtime state.

## Documentation gaps and cleanup boundaries

1. **Claude Desktop:** no vendor uninstall/reset/cache/GC contract; extension uninstall and secure-credential cleanup unknown.
2. **Claude Code:** filesystem reset does not document Keychain-item removal; native old-version pruning details are incomplete.
3. **Codex CLI:** no standalone uninstall, factory reset, general cache cleanup, or session-retention GC; IDE extension cleanup unknown.
4. **Copilot CLI:** no script/direct-download uninstall, factory reset, or general cache GC; Homebrew zap omits Keychain semantics.
5. **OMP:** no product-level uninstall or factory reset, no complete cache purge, no durable session deletion control; package-manager removal semantics for user state are not promised by OMP. `omp gc` is scoped storage maintenance, not full cleanup.
6. **Pi:** no factory reset or broad cache/session GC; application uninstall explicitly preserves the whole Pi user-data root.
7. **ChatGPT macOS:** no vendor uninstall, factory reset, local cache purge, credential purge, or app-state preservation inventory; no product-owned macOS VM data lifecycle is documented.
8. **VS Code:** extension-specific residual state, selective cache cleanup, Keychain cleanup, and profile-delete propagation are not fully documented; Settings Sync cloud state is separate.
9. **Agentlens:** no updater, selective deletion, retention/GC, cache, auth, session, or VM lifecycle. Its executable, two integrations, and durable database are intentionally independent.

The evidence supports documenting available controls, not choosing a retention or deletion policy. Any cleanup decision must separately choose the intended layer—package, settings, credentials, local sessions, synced/cloud sessions, plugins/extensions, caches/archives, worktrees, OS permissions, or account data—and must preserve `unknown` where first-party semantics stop.
