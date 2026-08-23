# Agentlens

Agentlens records observable skill-invocation attempts from OMP and Claude Code so a developer can see which skills are requested and how often.

## Language

**Agent client**:
A coding-agent application that can load and apply skills. OMP and Claude Code are the first supported clients.
_Avoid_: Provider, runtime, platform

**Skill**:
A named instruction pack that an agent client can load into a conversation on demand.
_Avoid_: Command, prompt, plugin

**Skill invocation**:
An explicit request by a user or model to apply one complete named skill to a conversation. It exists when the agent client emits a deterministic request signal; later resolution or application may fail. Each observable request counts separately. Preloading a skill, reading only selected lines, or reading a skill asset does not count.
_Avoid_: File read, activation, trigger

**Initiator**:
The immediate party that issued the skill request. An initiator is either the user or the model; a model remains the initiator when it acts on a user's instruction.
_Avoid_: Source, actor

**Usage event**:
A durable record of one observed skill-invocation signal, including its agent client, initiator, skill name, and observation time. It carries no conversation, session, or agent-instance identity. The first feature records signals independently and does not deduplicate repeated delivery.
_Avoid_: Metric, log entry, hit

**Observed invocation count**:
The number of usage events within a report's query scope. It reflects only deterministic signals exposed by agent clients and does not imply successful skill application or complete usage accounting.
_Avoid_: Successful uses, Total usage
