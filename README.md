# Agentlens

Agentlens records observable skill-invocation attempts from OMP and Claude Code in a local SQLite database, then reports observed counts by skill, agent client, and initiator.

The first release supports source installation on macOS Apple Silicon. Collection is user-wide, local, metadata-only, and fail-open: an unavailable collector does not stop either agent client.

## Prerequisites

- Rust and Cargo
- OMP 18.0.0, the verified version; compatibility with earlier releases is unknown
- Claude Code with `UserPromptExpansion` hook support
- A local clone of this repository that will remain at a stable absolute path

## Install the executable

From the repository root:

```sh
cargo install --locked --path .
AGENTLENS_BIN="$(command -v agentlens)"
test -n "$AGENTLENS_BIN" && test -x "$AGENTLENS_BIN"
printf 'Agentlens executable: %s\n' "$AGENTLENS_BIN"
"$AGENTLENS_BIN" --help
```

`AGENTLENS_BIN` must print an absolute path. The normal Cargo location is `/Users/YOU/.cargo/bin/agentlens`. If `command -v` prints nothing, add Cargo's bin directory to `PATH` and run the check again.

The OMP extension resolves the executable from `CARGO_INSTALL_ROOT`, then `CARGO_HOME`, then `~/.cargo`. If Cargo uses a non-default install root, that environment variable must also be present in OMP's environment. Claude Code uses the literal absolute path added to its settings below.

## Install the OMP integration

The default OMP profile loads extensions from `~/.omp/agent/extensions`. Create an absolute symlink to the tracked extension:

```sh
AGENTLENS_REPO="$(pwd -P)"
OMP_EXTENSION_DIR="$HOME/.omp/agent/extensions"
OMP_EXTENSION="$OMP_EXTENSION_DIR/agentlens.ts"

mkdir -p "$OMP_EXTENSION_DIR"
test ! -e "$OMP_EXTENSION" && test ! -L "$OMP_EXTENSION"
ln -s "$AGENTLENS_REPO/integrations/omp/agentlens.ts" "$OMP_EXTENSION"
readlink "$OMP_EXTENSION"
```

The final command must print the absolute path ending in `/integrations/omp/agentlens.ts`. Do not replace an existing `agentlens.ts`; inspect it first. Moving or deleting the repository breaks this symlink.

Restart OMP after adding the extension. This installs Agentlens only for the default profile. Named profiles and alternate agent directories need their own extension entry.

## Install the Claude Code integration

Edit `~/.claude/settings.json`. Preserve every existing top-level setting, hook event, matcher group, and hook handler. Under the existing top-level `hooks` object, append these two matcher groups to the corresponding arrays; create only the missing objects or arrays.

Replace `/Users/YOU/.cargo/bin/agentlens` with the absolute path printed as `AGENTLENS_BIN` above:

```json
{
  "hooks": {
    "UserPromptExpansion": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "/Users/YOU/.cargo/bin/agentlens",
            "args": ["claude-hook"]
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Skill",
        "hooks": [
          {
            "type": "command",
            "command": "/Users/YOU/.cargo/bin/agentlens",
            "args": ["claude-hook"]
          }
        ]
      }
    ]
  }
}
```

This is a complete file only when no other Claude Code settings or hooks exist. Otherwise, merge the two matcher groups into the existing arrays; do not replace the file with the example.

Validate the merged file before starting Claude Code:

```sh
plutil -convert xml1 -o /dev/null -- "$HOME/.claude/settings.json" && \
  printf 'Claude Code settings JSON is valid\n'
```

Claude Code normally reloads user settings without a restart. Start a fresh session for the smoke check so the tested configuration is unambiguous.

## Smoke checks

### Executable and storage without retaining test data

This exercises both normalized collection and Claude Code hook normalization against a disposable home directory:

```sh
SMOKE_HOME="$(mktemp -d)"
HOME="$SMOKE_HOME" "$AGENTLENS_BIN" collect \
  --client omp --skill agentlens-omp-smoke --initiator user
printf '%s\n' '{"hook_event_name":"UserPromptExpansion","expansion_type":"slash_command","command_name":"agentlens-claude-smoke"}' \
  | HOME="$SMOKE_HOME" "$AGENTLENS_BIN" claude-hook
HOME="$SMOKE_HOME" "$AGENTLENS_BIN" report --format json
rm -rf "$SMOKE_HOME"
```

The report must show two observed invocations: `agentlens-omp-smoke` in the OMP/user bucket and `agentlens-claude-smoke` in the Claude Code/user bucket.

### Live OMP integration

Start a fresh OMP session and invoke one harmless installed skill as a user:

```text
/skill:<installed-skill-name>
```

Then, in another terminal:

```sh
"$AGENTLENS_BIN" report --since 10m
```

The skill must appear in the OMP/user bucket. A model-initiated OMP observation appears only when the model successfully reads the complete root URI `skill://<name>`; asset reads, selected ranges, failed reads, and preload do not count.

### Live Claude Code integration

Start a fresh Claude Code session and invoke one harmless installed skill directly. Then ask Claude to invoke a harmless skill through its `Skill` tool. Check both paths:

```sh
"$AGENTLENS_BIN" report --since 10m
```

The direct command must appear in the Claude Code/user bucket. A successful `Skill` tool result must appear in the Claude Code/model bucket. These live checks intentionally create durable usage events.

The database is created on first collection or report at:

```text
~/Library/Application Support/Agentlens/agentlens.sqlite3
```

## Remove Agentlens

### Remove the OMP integration

Remove only the symlink created above:

```sh
OMP_EXTENSION="$HOME/.omp/agent/extensions/agentlens.ts"
if test -L "$OMP_EXTENSION"; then
  rm "$OMP_EXTENSION"
else
  printf 'Not removed: %s is not a symlink\n' "$OMP_EXTENSION" >&2
fi
```

Restart OMP. This does not affect Claude Code, the executable, or collected data.

After removing the OMP integration, invoke the same harmless skill in a fresh OMP session and rerun `agentlens report --since 10m`. Previous events must remain reportable, while the removed OMP integration adds no new matching observation.

### Remove the Claude Code integration

Edit `~/.claude/settings.json` and remove only the two Agentlens hook handlers whose:

- `command` equals the absolute Agentlens executable path; and
- `args` equals `["claude-hook"]`.

Remove their now-empty matcher groups. Remove the `UserPromptExpansion` or `PostToolUse` array only if that array is then empty, and remove `hooks` only if the object is then empty. Preserve all unrelated handlers, matcher groups, hook events, and top-level settings.

Start a fresh Claude Code session. This does not affect OMP, the executable, or collected data.

Validate the edited file:

```sh
plutil -convert xml1 -o /dev/null -- "$HOME/.claude/settings.json" && \
  printf 'Claude Code settings JSON is valid\n'
```

Then invoke a harmless skill in a fresh Claude Code session and rerun the same `agentlens report --since 10m` command. Its previous events must remain reportable, while the removed Claude Code integration adds no new matching observation.

### Remove the executable

```sh
cargo uninstall agentlens
```

If installation used an explicit custom root, pass the same root to `cargo uninstall --root <ROOT> agentlens`.

Removing the executable does not remove either client configuration. Remove the client integrations first so they do not continue attempting the fail-open collector command.

### Delete collected data separately

Uninstalling Agentlens never deletes durable data. If data deletion is explicitly wanted after removing both integrations and the executable:

```sh
rm -rf "$HOME/Library/Application Support/Agentlens"
```

This is irreversible and deletes the database, WAL files, and all observed invocation history.

## Expected limitations

- Counts are observed invocation signals, not proof that a skill was applied successfully or that the surrounding task succeeded.
- Collection is fail-open. Missing executables, malformed payloads, storage failures, and SQLite contention can drop observations without failing agent work.
- Agentlens does not deduplicate client delivery; repeated qualifying signals produce repeated events.
- OMP 18.0.0 has no dedicated skill-invocation event. User observations occur only after a skill prompt is constructed; model attribution is inferred from a successful complete `skill://<name>` read.
- The OMP symlink above covers only the default profile with ambient extension discovery enabled. Named profiles, alternate agent directories, and sessions with extension discovery disabled are not covered.
- Claude Code `UserPromptExpansion` identifies slash and custom commands but does not prove that every command is backed by a skill. Agentlens records qualifying slash-command names as reported.
- Claude Code model observations require a successful `PostToolUse` result for the `Skill` tool. Failed tool calls do not count. Hidden later skills in stacked commands are not reconstructed.
- User settings do not cover Claude Code cloud sessions. Managed policy with `allowManagedHooksOnly` can suppress these user hooks.
- The first release does not support other operating systems, prebuilt binaries, project-local setup, other agent clients, a daemon, remote synchronization, or historical transcript import.
