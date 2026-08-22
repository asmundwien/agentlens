# Agentlens

Agentlens records how coding agents use installed skills so a developer can inspect adoption and behavior across agent clients.

## Language

**Agent client**:
A coding-agent application that can load and apply skills. OMP and Claude Code are the first supported clients.
_Avoid_: Provider, runtime, platform

**Skill**:
A named instruction pack that an agent client can load into a conversation on demand.
_Avoid_: Command, prompt, plugin

**Skill invocation**:
A request that successfully resolves a skill and applies its instructions to a conversation. Repeated requests count as separate invocations, even when the client reuses content already present in context.
_Avoid_: File read, activation, trigger

**Initiator**:
The party that requested a skill invocation. An initiator is either the user or the model.
_Avoid_: Source, actor

**Agent instance**:
The main agent or subagent in which a skill invocation occurs.
_Avoid_: Process, thread, session

**Usage event**:
The durable record of one skill invocation, including its agent client, initiator, skill name, time, session, and agent instance when available.
_Avoid_: Metric, log entry, hit
