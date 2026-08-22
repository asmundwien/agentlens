# Claude Code skill invocation signals

Research date: 2026-08-22. Official documentation was read on that date. The installed behavior checks used Anthropic's `@anthropic-ai/claude-code` 2.1.235 native binary; its package metadata is at `/opt/homebrew/lib/node_modules/@anthropic-ai/claude-code/package.json`.

## Finding

Claude Code has a deterministic success signal for model-selected skills: `PostToolUse` with matcher `Skill`. Pair it with `PreToolUse` to observe attempts. The successful payload names the resolved skill in `tool_response.commandName`, reports `tool_response.success: true`, and repeats the call's `tool_use_id` in the installed 2.1.235 binary. Anthropic documents the event timing and generic fields, but not the `Skill` input or response schema. The concrete `skill`, `args`, `commandName`, and `success` properties below are installed behavior, not a published compatibility contract. [Official hook reference: `PreToolUse`](https://code.claude.com/docs/en/hooks#pretooluse), [official hook reference: `PostToolUse`](https://code.claude.com/docs/en/hooks#posttooluse), [local probe M1](#m1-model-selected-success-in-the-main-agent).

Claude Code does not expose an equivalent deterministic success signal for a direct user-selected `/skill`. `UserPromptExpansion` runs before expansion and can block it. A direct command bypasses `PreToolUse` for `Skill`. No documented post-expansion event confirms that Claude Code resolved the skill body and applied it to the conversation. Under Agentlens's definition, `UserPromptExpansion` proves an attempt, not a successful skill invocation. [Official hook reference: `UserPromptExpansion`](https://code.claude.com/docs/en/hooks#userpromptexpansion), [official skill failure behavior](https://code.claude.com/docs/en/skills#when-an-injected-command-fails), [local probe U2](#u2-direct-expansion-can-precede-failure).

This leaves user-initiated accounting incomplete without a Claude Code core change. Parsing transcripts would be timing-sensitive because Anthropic says `transcript_path` is written asynchronously and may lag the in-memory conversation. [Official common hook fields](https://code.claude.com/docs/en/hooks#common-input-fields).

## Hook configuration

The collector should accept one JSON object on standard input. An empty matcher on `UserPromptExpansion` captures every prompt-producing command so the collector can check `expansion_type` and the resolved `command_name`. The `Skill` matcher captures model tool calls. Session-wide hooks from settings also run for tools called inside subagents. [Official hook configuration](https://code.claude.com/docs/en/hooks#configuration), [official matcher rules](https://code.claude.com/docs/en/hooks#matcher-patterns), [official subagent hook behavior](https://code.claude.com/docs/en/sub-agents#define-hooks-for-subagents).

```json
{
  "hooks": {
    "UserPromptExpansion": [{
      "matcher": "",
      "hooks": [{ "type": "command", "command": "/absolute/path/to/agentlens-hook" }]
    }],
    "PreToolUse": [{
      "matcher": "Skill",
      "hooks": [{ "type": "command", "command": "/absolute/path/to/agentlens-hook" }]
    }],
    "PostToolUse": [{
      "matcher": "Skill",
      "hooks": [{ "type": "command", "command": "/absolute/path/to/agentlens-hook" }]
    }],
    "PostToolUseFailure": [{
      "matcher": "Skill",
      "hooks": [{ "type": "command", "command": "/absolute/path/to/agentlens-hook" }]
    }],
    "SubagentStart": [{
      "matcher": "",
      "hooks": [{ "type": "command", "command": "/absolute/path/to/agentlens-hook" }]
    }],
    "SubagentStop": [{
      "matcher": "",
      "hooks": [{ "type": "command", "command": "/absolute/path/to/agentlens-hook" }]
    }]
  }
}
```

`SubagentStart` and `SubagentStop` are not needed to capture a subagent's `Skill` call. They are useful for an agent-instance table and for checking lifecycle consistency. `SubagentStart` cannot block creation. [Official `SubagentStart` and `SubagentStop` reference](https://code.claude.com/docs/en/hooks#subagentstart).

## Model-selected path

`PreToolUse` runs after Claude forms the tool parameters and before Claude Code processes the call. It is an attempt event and can deny the call. `PostToolUse` runs only after the tool completes successfully. `PostToolUseFailure` runs after a tool that began execution fails, but excludes invalid tool names, schema or tool-specific validation failures, and permission denials. [Official `PreToolUse`](https://code.claude.com/docs/en/hooks#pretooluse), [official `PostToolUse`](https://code.claude.com/docs/en/hooks#posttooluse), [official `PostToolUseFailure`](https://code.claude.com/docs/en/hooks#posttoolusefailure).

The installed 2.1.235 success payloads were:

```json
{
  "session_id": "<session UUID>",
  "prompt_id": "<prompt UUID>",
  "transcript_path": "<main transcript path>",
  "cwd": "<current directory>",
  "permission_mode": "default",
  "effort": { "level": "high" },
  "hook_event_name": "PreToolUse",
  "tool_name": "Skill",
  "tool_input": { "skill": "model-probe", "args": "sample" },
  "tool_use_id": "toolu_..."
}
```

```json
{
  "session_id": "<same session UUID>",
  "prompt_id": "<same prompt UUID>",
  "transcript_path": "<same main transcript path>",
  "cwd": "<same current directory>",
  "permission_mode": "default",
  "effort": { "level": "high" },
  "hook_event_name": "PostToolUse",
  "tool_name": "Skill",
  "tool_input": { "skill": "model-probe", "args": "sample" },
  "tool_response": { "commandName": "model-probe", "success": true },
  "tool_use_id": "<same toolu_...>",
  "duration_ms": 3
}
```

These objects are a sanitized transcription of [local probe M1](#m1-model-selected-success-in-the-main-agent). The official schema guarantees the common envelope, `tool_name`, `tool_input`, `tool_response`, `tool_use_id`, and optional `duration_ms`; it does not specify the object under `tool_input` or `tool_response` for `Skill`. [Official common hook fields](https://code.claude.com/docs/en/hooks#common-input-fields), [official `PostToolUse` input](https://code.claude.com/docs/en/hooks#posttooluse-input).

For Agentlens, emit the usage event on the successful `PostToolUse`, not on `PreToolUse`. Set `initiator` to `model`. Read the skill name from `tool_response.commandName` on 2.1.235 and retain `tool_input.skill` for diagnostics. Because those properties are undocumented, a production parser should reject or quarantine a success payload when they disagree or are absent rather than invent a name. This paragraph is a recommendation based on [local probe M1](#m1-model-selected-success-in-the-main-agent), not an Anthropic compatibility guarantee.

`PostToolUseFailure` is not complete failure coverage. A 2.1.235 probe whose skill resolution ran a failing injected command emitted `PreToolUse` but neither `PostToolUse` nor `PostToolUseFailure`. The official reference separately warns that pre-execution validation and permission rejection do not fire `PostToolUseFailure`. Agentlens therefore cannot assume every attempt receives one terminal tool hook. [Official `PostToolUseFailure` exclusions](https://code.claude.com/docs/en/hooks#posttoolusefailure), [local probe M2](#m2-model-selected-resolution-failure).

## Direct user-selected path

A direct `/skill-name` bypasses `PreToolUse` for `Skill`. `UserPromptExpansion` runs before the user-typed command reaches Claude and matches on `command_name`. For skills and custom commands, `expansion_type` is `slash_command`; MCP prompts use `mcp_prompt`. The payload also includes `command_args`, `command_source`, and the original `prompt`. [Official `UserPromptExpansion`](https://code.claude.com/docs/en/hooks#userpromptexpansion).

The installed 2.1.235 payload for a project skill was:

```json
{
  "session_id": "<session UUID>",
  "prompt_id": "<prompt UUID>",
  "transcript_path": "<main transcript path>",
  "cwd": "<current directory>",
  "permission_mode": "default",
  "hook_event_name": "UserPromptExpansion",
  "expansion_type": "slash_command",
  "command_name": "probe-skill",
  "command_args": "<arguments>",
  "command_source": "projectSettings",
  "prompt": "<submitted command>"
}
```

This object is a sanitized transcription of [local probe U1](#u1-direct-project-skill). Anthropic documents the fields but gives only a `plugin` example for `command_source`; `projectSettings` is installed behavior. [Official `UserPromptExpansion` input](https://code.claude.com/docs/en/hooks#userpromptexpansion-input).

The hook can return `decision: "block"`, so receipt cannot mean success. There is no documented `PostUserPromptExpansion`, skill-resolution result, `tool_use_id`, or direct-command invocation ID. A local skill whose injected command exited nonzero still emitted `UserPromptExpansion`, then completed with no model turn. Record this signal as a user attempt only. [Official decision control](https://code.claude.com/docs/en/hooks#userpromptexpansion-decision-control), [official injected-command failure behavior](https://code.claude.com/docs/en/skills#when-an-injected-command-fails), [local probe U2](#u2-direct-expansion-can-precede-failure).

### Stacked skills

Claude Code supports a first inline skill plus as many as five more at the start of one message. It stops when it reaches a token that is not an inline user-invocable skill. All expanded skills receive the trailing argument text. This behavior was added in 2.1.199. [Official stacked-skill behavior](https://code.claude.com/docs/en/skills#pass-arguments-to-skills).

The `UserPromptExpansion` schema is singular and has no stack index or group identifier. In 2.1.235, a two-name stacked probe emitted one event for the first name; the second name remained in that event's `command_args`. The event stream did not identify the second name as a separate resolved skill. Parsing `command_args` cannot repair this because command resolution, user-invocable checks, forked skills, and the documented stopping rules live in Claude Code. Exact per-skill usage accounting for stacked direct invocation needs a core event or payload change. [Official stacked-skill stopping rules](https://code.claude.com/docs/en/skills#pass-arguments-to-skills), [local probe U3](#u3-stacked-direct-skills).

## Agent attribution

Settings, managed-policy, and plugin hooks run inside subagents. Tool-hook input inside a subagent adds `agent_id` and `agent_type`; `agent_id` is documented as the unique subagent identifier. Its absence distinguishes main-thread hook calls. `agent_type` is also present for a main session started with `--agent`, so `agent_type` alone does not prove a subagent. [Official hook locations and subagent behavior](https://code.claude.com/docs/en/hooks#hook-locations), [official common subagent fields](https://code.claude.com/docs/en/hooks#common-input-fields).

A 2.1.235 custom subagent invocation reused the main `session_id`, parent `prompt_id`, `transcript_path`, `agent_id`, and `agent_type` across `SubagentStart`, `PreToolUse Skill`, `PostToolUse Skill`, and `SubagentStop`. `SubagentStop.agent_transcript_path` pointed to the subagent's nested transcript, while `transcript_path` remained the main transcript. The successful skill payload otherwise matched the main-agent form. [Official subagent schemas](https://code.claude.com/docs/en/hooks#subagentstart), [local probe S1](#s1-model-selected-skill-inside-a-subagent).

```json
{
  "session_id": "<parent session UUID>",
  "prompt_id": "<parent prompt UUID>",
  "transcript_path": "<main transcript path>",
  "cwd": "<current directory>",
  "permission_mode": "default",
  "hook_event_name": "PostToolUse",
  "agent_id": "a58b9e117b8a1b583",
  "agent_type": "skill-invoker-822",
  "tool_name": "Skill",
  "tool_input": { "skill": "model-probe", "args": "subagent" },
  "tool_response": { "commandName": "model-probe", "success": true },
  "tool_use_id": "toolu_...",
  "duration_ms": 4
}
```

Agentlens should use a main-agent sentinel when `agent_id` is absent and use `agent_id` as the agent-instance key when present. Preserve `agent_type` as a label. This is a design recommendation grounded in Anthropic's documented meaning for those fields. [Official common subagent fields](https://code.claude.com/docs/en/hooks#common-input-fields).

A skill with `context: fork` creates a subagent, and `SubagentStart`/`SubagentStop` describe that agent. Their documented payloads do not carry the originating skill name or parent Skill `tool_use_id`. The regular `PostToolUse Skill` event still establishes the invocation in the calling agent; linking it to the forked `agent_id` is not a documented contract. [Official forked-skill behavior](https://code.claude.com/docs/en/skills#run-skills-in-a-subagent), [official subagent hook schemas](https://code.claude.com/docs/en/hooks#subagentstart).

When a user opens a subagent's transcript, Anthropic says follow-up messages and skills go to that agent. Whether a direct `UserPromptExpansion` fired from that UI carries the viewed subagent's `agent_id` is not specified. Treat attribution for that direct path as unknown until Anthropic documents it or an interactive installed-behavior probe confirms it. [Official subagent transcript interaction](https://code.claude.com/docs/en/sub-agents#resume-subagents).

## Identifiers and deduplication

| Purpose | Available fields | Finding |
| --- | --- | --- |
| Session | `session_id` | Documented as the current session identifier. No stronger uniqueness or resume/fork guarantee is stated. [Official common fields](https://code.claude.com/docs/en/hooks#common-input-fields) |
| Prompt grouping | `prompt_id` | A UUID for the user prompt, shared with the `prompt.id` OpenTelemetry attribute and absent before first user input. [Official common fields](https://code.claude.com/docs/en/hooks#common-input-fields) |
| Model-selected call | `tool_use_id` | Present on tool events. In 2.1.235 it was unchanged from `PreToolUse` to `PostToolUse`. Anthropic does not state a uniqueness scope or cross-phase stability guarantee. [Official tool fields](https://code.claude.com/docs/en/hooks#common-input-fields), [local probe M1](#m1-model-selected-success-in-the-main-agent) |
| Subagent | `agent_id` | Documented as unique for the subagent and repeated on hooks inside it and its lifecycle events. [Official common subagent fields](https://code.claude.com/docs/en/hooks#common-input-fields), [official `SubagentStart`](https://code.claude.com/docs/en/hooks#subagentstart) |
| Direct command | none | `UserPromptExpansion` has no invocation identifier. `prompt_id` groups a whole submitted prompt, not each stacked skill. [Official `UserPromptExpansion` input](https://code.claude.com/docs/en/hooks#userpromptexpansion-input), [official `prompt_id`](https://code.claude.com/docs/en/hooks#common-input-fields) |

For model-selected success, `(session_id, tool_use_id)` is the strongest observed deduplication key. This is an inference from the 2.1.235 payload pair, not a documented guarantee. Keep `agent_id` in the event record for attribution, but do not need it to distinguish the same tool call's two phases. [Local probe M1](#m1-model-selected-success-in-the-main-agent), [local probe S1](#s1-model-selected-skill-inside-a-subagent).

For an unstacked direct attempt, `(session_id, prompt_id, command_name, command_source)` is a workable provisional key. It cannot identify each member of a stack, prove success, or rely on a documented `command_source` enum. It must not be presented as a stable Claude Code invocation ID. [Official `UserPromptExpansion` schema](https://code.claude.com/docs/en/hooks#userpromptexpansion-input), [local probe U3](#u3-stacked-direct-skills).

## Blind spots that require Claude Code changes

1. Add a post-expansion event for direct skills with explicit success or failure after skill resolution and dynamic injection.
2. Add one opaque invocation ID to both direct and model paths, and document its uniqueness and resume/fork behavior.
3. Include `initiator: "user" | "model"`, canonical resolved skill name, source, and version or path in the terminal event.
4. Emit one terminal event per stacked skill with a stack group ID and index.
5. Link `context: fork` lifecycle events to the originating invocation ID and, for model calls, the parent `tool_use_id`.
6. Publish the `Skill` tool's `tool_input` and `tool_response` schemas and terminal-hook coverage for every rejection and failure class.
7. Specify agent attribution for direct skills submitted while viewing a subagent.

These are requirements derived from the documented and observed gaps above, not existing Claude Code features. [Official direct-path schema](https://code.claude.com/docs/en/hooks#userpromptexpansion-input), [official failure exclusions](https://code.claude.com/docs/en/hooks#posttoolusefailure), [official forked-skill behavior](https://code.claude.com/docs/en/skills#run-skills-in-a-subagent).

## Installed primary-source probes

The probes used an isolated project under `/tmp`, project settings only, a command hook that appended its stdin JSON unchanged, and `--no-session-persistence`. No transcript was read. The binary reported `2.1.235`; the installed Anthropic package metadata at `/opt/homebrew/lib/node_modules/@anthropic-ai/claude-code/package.json` also declares version `2.1.235` and the package name `@anthropic-ai/claude-code`.

### M1 model-selected success in the main agent

A model-only project skill was invoked with one argument while only the `Skill` tool was exposed. The hook log contained one `PreToolUse` and one `PostToolUse`. Both had the same `session_id`, `prompt_id`, and `tool_use_id`. The input was `{"skill":"model-probe","args":"sample"}`. The response was `{"commandName":"model-probe","success":true}`.

### M2 model-selected resolution failure

A model-only skill contained an injected `/usr/bin/false` command allowed by its frontmatter. Invocation produced `PreToolUse Skill`, then the client reported a failed Skill tool result to the model. The hook log contained no `PostToolUse` and no `PostToolUseFailure` for that `tool_use_id`.

### U1 direct project skill

A direct project skill with arguments emitted one `UserPromptExpansion`. It reported `expansion_type: "slash_command"`, the resolved `command_name`, argument text in `command_args`, and `command_source: "projectSettings"`. It emitted no `Skill` tool hook.

### U2 direct expansion can precede failure

A user-only skill contained an allowed injected `/usr/bin/false`. It emitted `UserPromptExpansion`. Claude Code's JSON result reported zero model turns and an empty result. Thus the expansion event occurred without the skill instructions reaching a model turn.

### U3 stacked direct skills

A submitted command began with two unique user-only project skill names. The hook log contained one `UserPromptExpansion` for the first name. Its `command_args` began with the second name. No event separately identified the second skill.

### S1 model-selected skill inside a subagent

A custom `skill-invoker-822` subagent with `Skill` access invoked `model-probe`. The log order was `SubagentStart`, `PreToolUse Skill`, `PostToolUse Skill`, and `SubagentStop`. All four records carried the same `session_id`, `prompt_id`, `agent_id`, and `agent_type`. The tool records had the same `tool_use_id`; the successful response was `{"commandName":"model-probe","success":true}`.
