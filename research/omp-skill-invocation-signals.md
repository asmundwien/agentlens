# OMP skill invocation signals

## Scope and version

This finding covers the installed OMP 17.2.9 runtime. The Homebrew receipt at `/opt/homebrew/Cellar/omp/17.2.9/INSTALL_RECEIPT.json` identifies version `17.2.9`, and the formula at `/opt/homebrew/Cellar/omp/17.2.9/.brew/omp.rb` downloads that version from the `can1357/oh-my-pi` GitHub release. Source links below are pinned to tag `v17.2.9`. ([release source](https://github.com/can1357/oh-my-pi/tree/v17.2.9))

Agentlens defines an invocation as a request that resolves a named skill and applies its instructions to a conversation. With that definition, OMP has two relevant paths: the user-facing `/skill:<name>` dispatcher and a model-issued `read` of the root `skill://<name>` URL. OMP has no event named `skill_invoked` or equivalent in its extension event union. ([installed skills documentation](omp://skills.md), [extension event union](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/extensibility/extensions/types.ts#L961-L993))

## Findings

| Path | Attempt signal | Best current success signal | Initiator | Recommended deduplication key |
| --- | --- | --- | --- | --- |
| `/skill:<name>` | `input` containing a registered skill token | `message_end` where the message has `role: "custom"`, `customType: "skill-prompt"`, `attribution: "user"`, and valid skill details | user | Durable journal ingestion: `(sessionId, custom_message entry id)`. Live-only fallback: recorder-minted ID, because the event has no entry ID. |
| `read({ path: "skill://<name>" })` | `tool_call` for `read` | matching `tool_result` with the same `toolCallId`, root skill URL, and `isError: false` | model | `(sessionId, toolCallId)` |

The two signals are generic message and tool events. They are not a normalized OMP skill event. ([custom skill message construction](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/modes/skill-command.ts#L11-L76), [tool event types](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/extensibility/extensions/types.ts#L824-L928))

### User invocation through `/skill:<name>`

The interactive dispatcher registers each discovered skill as `skill:<name>`. It recognizes a leading `/skill:<name>` and a whitespace-delimited token embedded in prose. The embedded form removes the token and passes the surrounding prose as skill arguments. Drafts beginning with another slash command or a local bash/Python sigil do not use the embedded form. ([registration](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/modes/interactive-mode.ts#L1205-L1215), [parser](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/extensibility/skills.ts#L398-L466))

A known command is resolved through the registered skill map. OMP then reads `skill.filePath`, strips frontmatter, renders the skill body and arguments, and builds a custom message with these fields:

```text
role: "custom"
customType: "skill-prompt"
display: true
attribution: "user"
details: {
  name: string
  path: string
  args?: string
  lineCount: number
}
timestamp: number
```

The file read happens before the message is built. An unknown name returns no built command, and a file read failure rejects the dispatch. ([message builder](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/extensibility/skills.ts#L480-L525), [dispatcher](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/modes/skill-command.ts#L36-L86), [`skill-prompt` and detail types](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/session/messages.ts#L42-L42), [custom message shape](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/session/messages.ts#L426-L438), [message fields](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/session/messages.ts#L885-L897))

`input` runs before built-in and skill dispatch. Its exact payload is `{ type: "input", text, images?, source }`, where `source` is `"interactive" | "rpc" | "extension"`. A handler can mark the input handled or replace its text and images. `input` is therefore an attempt signal, not proof of resolution. ([input order](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/modes/controllers/input-controller.ts#L640-L750), [payload and result](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/extensibility/extensions/types.ts#L794-L800), [dispatch behavior](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/extensibility/extensions/runner.ts#L1222-L1245))

After resolution, `promptCustomMessage()` either starts a turn or queues the message according to `steer` or `followUp`. OMP emits message lifecycle events when the queued message is actually delivered. At `message_end`, it persists the skill as a `custom_message` entry that participates in model context. Each persisted entry has its own generated `id`, `parentId`, and timestamp. ([delivery](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/session/agent-session.ts#L5041-L5105), [message event forwarding](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/session/agent-session.ts#L3335-L3373), [persistence](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/session/agent-session.ts#L2477-L2492), [custom entry creation](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/session/session-manager.ts#L2131-L2157))

**Count without an OMP change.** Count `message_end` only when its `message` matches the exact shape above. Do not count `input`, unknown `/skill:` text, or `message_start`. `message_end` is downstream of exact-name lookup, successful file loading, and delivery into the conversation. Its exact event payload is `{ type: "message_end", message: AgentMessage }`. ([message event payload](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/extensibility/extensions/types.ts#L671-L688), [unknown command rejection](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/modes/skill-command.ts#L27-L34))

**Deduplication limitation.** `message_end` does not contain the persisted session entry ID, and persistence happens after extension fan-out. A journal reader can use `(sessionId, entry.id)` for a durable `custom_message` entry. A live extension must mint and persist its own invocation ID or perform a later branch scan. A key assembled from `message.timestamp` and skill details is only best effort. ([event before persistence](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/session/agent-session.ts#L2390-L2424), [entry ID returned after append](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/session/session-manager.ts#L2131-L2157))

### Model invocation through `skill://<name>`

The model sees discovered skill metadata in the system prompt and can request the skill body with the ordinary `read` tool. The root form `skill://<name>` resolves to that skill's `SKILL.md`; `skill://<name>/<relative-path>` resolves an asset inside its directory. Skill names use exact matching. Unknown names, missing files, absolute paths, traversal, and paths outside the skill directory throw. ([installed skills documentation](omp://skills.md), [skill protocol](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/internal-urls/skill-protocol.ts#L1-L110), [model-facing read input](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/tools/read.ts#L718-L723))

The read tool routes internal URLs through `InternalUrlRouter.resolve()`, passing the session's skill list. A successful result includes read details such as `resolvedPath` and `contentType`; the tool result is then returned to the agent loop. ([read routing](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/tools/read.ts#L2299-L2340), [internal resolution](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/tools/read.ts#L3261-L3335), [read details](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/tools/read.ts#L726-L755))

The relevant extension events and exact payloads are:

```text
tool_call: {
  type: "tool_call"
  toolCallId: string
  toolName: "read"
  input: { path: string }
}

tool_result: {
  type: "tool_result"
  toolCallId: string
  toolName: "read"
  input: Record<string, unknown>
  content: Array<TextContent | ImageContent>
  details: ReadToolDetails | undefined
  isError: boolean
}
```

`tool_call` runs before execution and may block or revise input. `tool_result` runs after the wrapped tool returns or throws. The wrapper sets `isError` from the execution outcome and uses the same `toolCallId` on both events. ([event schemas](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/extensibility/extensions/types.ts#L824-L928), [wrapper call phase](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/extensibility/extensions/wrapper.ts#L190-L229), [wrapper result phase](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/extensibility/extensions/wrapper.ts#L335-L385))

**Count without an OMP change.** Correlate `tool_call` and `tool_result` by `toolCallId`. Count only `toolName === "read"`, a root skill URL, and `isError === false`. Parse the URL rather than using a string prefix. Do not count `skill://name/asset`, because that proves an asset read, not application of the named instruction pack. A line selector can return only part of `SKILL.md`; strict invocation accounting should not treat a partial root read as application of the full skill. ([root and relative semantics](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/internal-urls/skill-protocol.ts#L4-L9), [selector routing](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/tools/read.ts#L2299-L2338))

Use `(sessionId, toolCallId)` as the durable deduplication key. `toolCallId` correlates attempt and result, while the session ID scopes call IDs across conversations. The read-only session manager exposes `getSessionId()`, and OMP creates session IDs with UUIDv7. ([context access](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/extensibility/extensions/types.ts#L415-L449), [read-only session API](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/session/session-manager.ts#L326-L347), [session ID generation and getter](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/session/session-manager.ts#L67-L69), [getter](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/session/session-manager.ts#L1815-L1821))

### Extension and hook coverage

The default CLI runs the extension system. `--hook` aliases `--extension`, discovered JS/TS hook factories load as extension modules, and tools use the extension wrapper. The legacy hook subsystem has generic `tool_call` and `tool_result` events too, but it adds no skill-specific event. ([installed hook documentation](omp://hooks.md), [installed extension documentation](omp://extensions.md))

The useful exact extension event names are:

| Event | Meaning for this research | Payload fields |
| --- | --- | --- |
| `input` | user attempt before skill resolution | `type`, `text`, `images?`, `source` |
| `message_start` | custom skill message began delivery | `type`, `message` |
| `message_end` | custom skill message completed delivery | `type`, `message` |
| `tool_call` | model read attempt | `type`, `toolCallId`, `toolName`, `input` |
| `tool_result` | wrapped read result | `type`, `toolCallId`, `toolName`, `input`, `content`, `details`, `isError` |
| `tool_execution_start` | raw execution observability | `type`, `toolCallId`, `toolName`, `args`, `intent?` |
| `tool_execution_end` | raw execution completion | `type`, `toolCallId`, `toolName`, `result`, `isError` |
| `session_start` | session-local extension initialized | `type` only |
| `agent_start` | an agent loop started | `type` only |

The canonical schemas are in the extension and shared-event types. Neither `session_start` nor `agent_start` carries session or agent identity. ([message, execution, and input schemas](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/extensibility/extensions/types.ts#L660-L800), [tool schemas](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/extensibility/extensions/types.ts#L824-L928), [session and agent schemas](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/extensibility/shared-events.ts#L27-L30), [agent schema](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/extensibility/shared-events.ts#L188-L207))

One caveat matters for the model path. `tool_result` handlers form middleware and may alter content, details, or `isError`. A recorder observes the state at its position in extension order, not a dedicated immutable post-middleware skill outcome. ([installed extension documentation](omp://extensions.md), [result mutation](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/extensibility/extensions/wrapper.ts#L349-L385))

### Main agent, subagents, and identity

Task subagents receive the parent session's skill list. OMP passes extension source paths, not loaded extension instances, into children, so each child loads a new extension instance bound to its own session API, event bus, working directory, and runtime. Restricted-tool subagents do not load ambient extensions. ([skill and extension forwarding](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/task/executor.ts#L2974-L3016), [per-session extension loading](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/sdk.ts#L2013-L2048))

A child extension can therefore observe that child's `message_end` and `tool_result` events and can read `ctx.sessionManager.getSessionId()`. `ExtensionContext` does not contain `agentId`, `agentKind`, or `parentAgentId`. ([context fields](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/extensibility/extensions/types.ts#L415-L455))

OMP's process registry does contain agent identity. The main registry ID is exactly `Main`. Each `AgentRef` has `id`, `displayName`, `kind`, optional `parentId`, `status`, `session`, `sessionFile`, `createdAt`, `lastActivity`, and optional `activity`. Child session creation passes the allocated child ID as `agentId` and the spawner ID as `parentAgentId`. These registry IDs are stable for the registered agent instance but are not a global cross-session identifier. ([registry fields](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/registry/agent-registry.ts#L15-L55), [child identity wiring](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/task/executor.ts#L2918-L2945), [session options](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/task/executor.ts#L3008-L3016))

The parent event bus exposes three exact task channels:

```text
task:subagent:event
task:subagent:progress
task:subagent:lifecycle
```

`task:subagent:event` payload is `{ id: string, event: AgentSessionEvent }`. The lifecycle payload is `{ id, agent, agentSource, description?, status, sessionFile?, parentToolCallId?, index, detached? }`. The raw child event channel carries `tool_execution_start` and `tool_execution_end`, not extension `tool_call` and `tool_result`. ([channel and payload definitions](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/task/types.ts#L58-L103), [forwarding](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/task/executor.ts#L1318-L1324))

**Attribution without an OMP change.** There are two workable choices:

1. Let each child-loaded extension record its own success event with the child session ID. Derive the registry ID by matching the live `AgentRegistry` entry whose `session.sessionManager` is the same manager as `ctx.sessionManager`. This depends on a registry lookup rather than the canonical event contract. ([registry list and live session field](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/registry/agent-registry.ts#L33-L55), [registry listing](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/registry/agent-registry.ts#L185-L197))
2. Subscribe in the parent to `task:subagent:event`. Use payload `id` for child attribution, correlate `tool_execution_start.args.path` and `tool_execution_end` by `toolCallId`, and use lifecycle `sessionFile` when available. This sees model reads but does not expose the normalized `tool_result` details or a child session ID directly. ([execution payloads](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/extensibility/extensions/types.ts#L690-L715), [task payloads](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/task/types.ts#L65-L103))

The first route is closer to the true success signal. The second has explicit child ID attribution but weaker result semantics. Neither is a clean, documented, single-event contract.

### Autoload is not a user or model invocation

An agent definition may list `autoloadSkills`. The task executor builds those with invocation kind `"autoload"` and injects hidden `skill-prompt` messages before the first child prompt. The injected message omits explicit attribution, so `promptCustomMessage` defaults it to `"agent"`. This path is configuration-driven, not a request by the user or model. Agentlens should exclude it from a two-value `user | model` initiator field unless the product adds a third initiator. ([autoload kind](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/extensibility/skills.ts#L480-L513), [task injection](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/task/executor.ts#L3187-L3203), [default attribution](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/session/agent-session.ts#L5083-L5102))

## What Agentlens can count now

A session-scoped OMP extension can count these without a core change:

- successful user invocations from `message_end` and the `skill-prompt` message shape;
- successful model root reads from paired `tool_call` and `tool_result` events where `isError` is false;
- session identity from `ctx.sessionManager.getSessionId()`;
- model-read deduplication from `(sessionId, toolCallId)`;
- child activity when the extension is loaded in children, plus best-effort child registry attribution;
- explicit child registry IDs through `task:subagent:event`, with weaker raw execution semantics.

These capabilities follow from the event, context, session, and task APIs cited above. They require composition across generic APIs and careful exclusion of failed lookups, assets, partial reads, and autoload messages.

## Blind spots that need an OMP core event

A core change is needed for a complete deterministic contract:

- There is no event that normalizes both `/skill:<name>` and root `skill://<name>` success. ([extension event union](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/extensibility/extensions/types.ts#L961-L993))
- User-success message events have no durable invocation ID or session entry ID. ([message event payload](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/extensibility/extensions/types.ts#L671-L688))
- Canonical event payloads and `ExtensionContext` omit `agentId`, `agentKind`, and `parentAgentId`. ([context fields](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/extensibility/extensions/types.ts#L415-L455))
- Model-read events expose a raw path, not normalized `skillName`, `skillPath`, or root-versus-asset classification. ([read and tool payloads](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/extensibility/extensions/types.ts#L824-L928))
- A `tool_result` proves successful resolution and delivery of content to the conversation. It cannot prove that the model followed the instructions. OMP's resolver returns content and the agent loop receives a tool result; semantic compliance is not represented in the event model. ([resolver result](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/internal-urls/skill-protocol.ts#L84-L104), [wrapper result](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/extensibility/extensions/wrapper.ts#L335-L385))
- Autoload does not fit the required user/model initiator enum. ([autoload implementation](https://github.com/can1357/oh-my-pi/blob/v17.2.9/packages/coding-agent/src/task/executor.ts#L3187-L3203))

The clean core contract is a post-success `skill_invoked` event emitted in both main and child sessions:

```ts
interface SkillInvokedEvent {
  type: "skill_invoked";
  invocationId: string;
  initiator: "user" | "model";
  mechanism: "slash" | "skill-url";
  skillName: string;
  skillPath: string;
  sessionId: string;
  agentId: string;
  agentKind: "main" | "sub";
  parentAgentId?: string;
  timestamp: number;
  toolCallId?: string;
  sessionEntryId?: string;
  args?: string;
}
```

Emit it only after exact-name resolution, successful skill-body loading, and insertion into conversation context. `invocationId` should be the universal idempotency key. Keep `toolCallId` for model correlation and `sessionEntryId` for user-message journal correlation. This schema is a recommendation, not an existing OMP API.

## Documented facts versus inference

Everything above that describes event names, payloads, resolution order, persistence, or identity fields comes from the pinned OMP source or installed `omp://` documentation.

The following conclusions are inferences from those facts:

- `message_end` is the best live success signal for `/skill:<name>` because it occurs after resolution and at delivery completion.
- A successful root `read` result is evidence that OMP applied the skill text to the conversation as tool content, but not that the model obeyed it.
- Matching a live registry entry to `ctx.sessionManager` is a workable agent-ID bridge, but it is not a declared extension-context contract.
- Autoload should not be forced into either user or model initiation.
- A dedicated post-success core event is the only way to provide uniform attribution and exactly-once identity across both paths.
