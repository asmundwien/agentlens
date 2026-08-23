import { homedir } from "node:os";
import { resolve } from "node:path";
import type { ExtensionAPI } from "@oh-my-pi/pi-coding-agent";

export type Initiator = "user" | "model";

export interface UsageObservation {
	readonly skill: string;
	readonly initiator: Initiator;
}

type UnknownRecord = Record<string, unknown>;
type Collector = (observation: UsageObservation) => void;

interface PendingSkillRead {
	readonly path: string;
	readonly skill: string;
}

const READ_SELECTOR_PART = /^(?:raw|conflicts|-?\d+(?:[-+]\d+)?(?:,\d+(?:[-+]\d+)?)*)$/i;
const ROOT_SKILL_URL = /^skill:\/\/([^/?#]+)\/?$/i;

function isRecord(value: unknown): value is UnknownRecord {
	return typeof value === "object" && value !== null && !Array.isArray(value);
}

function hasReadSelector(path: string): boolean {
	const schemeEnd = path.indexOf("://") + 3;
	const lastColon = path.lastIndexOf(":");
	return lastColon >= schemeEnd && READ_SELECTOR_PART.test(path.slice(lastColon + 1));
}

export function parseRootSkillRead(path: unknown): string | undefined {
	if (typeof path !== "string" || hasReadSelector(path)) return undefined;

	const match = ROOT_SKILL_URL.exec(path);
	if (!match) return undefined;

	try {
		const skill = decodeURIComponent(match[1]);
		return skill.trim().length > 0 ? skill : undefined;
	} catch {
		return undefined;
	}
}

function parseUserObservation(event: unknown): UsageObservation | undefined {
	if (!isRecord(event) || event.type !== "message_end" || !isRecord(event.message)) return undefined;

	const message = event.message;
	if (
		message.role !== "custom" ||
		message.customType !== "skill-prompt" ||
		message.display !== true ||
		message.attribution !== "user" ||
		(typeof message.content !== "string" && !Array.isArray(message.content)) ||
		typeof message.timestamp !== "number" ||
		!Number.isFinite(message.timestamp) ||
		!isRecord(message.details)
	) {
		return undefined;
	}

	const details = message.details;
	if (
		typeof details.name !== "string" ||
		details.name.trim().length === 0 ||
		typeof details.path !== "string" ||
		details.path.length === 0 ||
		typeof details.lineCount !== "number" ||
		!Number.isInteger(details.lineCount) ||
		details.lineCount < 0 ||
		(details.args !== undefined && typeof details.args !== "string")
	) {
		return undefined;
	}

	return { skill: details.name, initiator: "user" };
}

export function createObservationHandlers(collect: Collector) {
	const pendingReads = new Map<string, PendingSkillRead>();

	function emit(observation: UsageObservation | undefined): void {
		if (!observation) return;
		try {
			collect(observation);
		} catch {
			// Collection must never disrupt OMP.
		}
	}

	return {
		messageEnd(event: unknown): void {
			emit(parseUserObservation(event));
		},

		toolCall(event: unknown): void {
			if (!isRecord(event) || typeof event.toolCallId !== "string") return;

			pendingReads.delete(event.toolCallId);
			if (event.toolName !== "read" || !isRecord(event.input)) return;

			const path = event.input.path;
			const skill = parseRootSkillRead(path);
			if (typeof path === "string" && skill) {
				pendingReads.set(event.toolCallId, { path, skill });
			}
		},

		toolResult(event: unknown): void {
			if (!isRecord(event) || typeof event.toolCallId !== "string") return;

			const pending = pendingReads.get(event.toolCallId);
			pendingReads.delete(event.toolCallId);
			if (
				!pending ||
				event.toolName !== "read" ||
				event.isError !== false ||
				!isRecord(event.input) ||
				event.input.path !== pending.path ||
				parseRootSkillRead(event.input.path) !== pending.skill
			) {
				return;
			}

			emit({ skill: pending.skill, initiator: "model" });
		},

		reset(): void {
			pendingReads.clear();
		},
	};
}

export function agentlensExecutablePath(
	environment: Readonly<Record<string, string | undefined>> = process.env,
): string {
	const installRoot = environment.CARGO_INSTALL_ROOT ?? environment.CARGO_HOME ?? resolve(homedir(), ".cargo");
	return resolve(installRoot, "bin", "agentlens");
}

export function collectorCommand(
	observation: UsageObservation,
	executable = agentlensExecutablePath(),
): readonly string[] {
	return [
		executable,
		"collect",
		"--client",
		"omp",
		"--skill",
		observation.skill,
		"--initiator",
		observation.initiator,
	];
}

function collectWithAgentlens(observation: UsageObservation): void {
	try {
		const process = Bun.spawn({
			cmd: collectorCommand(observation),
			stdin: "ignore",
			stdout: "ignore",
			stderr: "ignore",
		});
		process.unref();
	} catch {
		// Missing executables and process failures are deliberately fail-open.
	}
}

export default function agentlensExtension(pi: ExtensionAPI): void {
	const handlers = createObservationHandlers(collectWithAgentlens);

	pi.on("message_end", event => handlers.messageEnd(event));
	pi.on("tool_call", event => handlers.toolCall(event));
	pi.on("tool_result", event => handlers.toolResult(event));
	pi.on("session_start", handlers.reset);
	pi.on("session_switch", handlers.reset);
	pi.on("session_branch", handlers.reset);
	pi.on("session_tree", handlers.reset);
	pi.on("session_shutdown", handlers.reset);
}
