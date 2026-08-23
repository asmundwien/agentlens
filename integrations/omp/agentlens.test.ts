import { describe, expect, test } from "bun:test";
import {
	agentlensExecutablePath,
	collectorCommand,
	createObservationHandlers,
	parseRootSkillRead,
	type UsageObservation,
} from "./agentlens";

function userMessage(overrides: Record<string, unknown> = {}): Record<string, unknown> {
	return {
		type: "message_end",
		message: {
			role: "custom",
			customType: "skill-prompt",
			content: "skill body",
			display: true,
			attribution: "user",
			details: {
				name: "Plugin:Review",
				path: "/skills/review/SKILL.md",
				args: "carefully",
				lineCount: 20,
			},
			timestamp: 1,
			...overrides,
		},
	};
}

function rootRead(toolCallId: string, path = "skill://Plugin:Review"): Record<string, unknown> {
	return {
		type: "tool_call",
		toolCallId,
		toolName: "read",
		input: { path },
	};
}

function successfulRead(toolCallId: string, path = "skill://Plugin:Review"): Record<string, unknown> {
	return {
		type: "tool_result",
		toolCallId,
		toolName: "read",
		input: { path },
		content: [{ type: "text", text: "skill body" }],
		details: { kind: "file" },
		isError: false,
	};
}

describe("parseRootSkillRead", () => {
	test("preserves the decoded, case-sensitive, qualified skill name", () => {
		expect(parseRootSkillRead("skill://Plugin%3AReview")).toBe("Plugin:Review");
		expect(parseRootSkillRead("SKILL://Case-Sensitive/")).toBe("Case-Sensitive");
	});

	test("rejects selected reads, assets, and URL decorations", () => {
		for (const path of [
			"skill://review:1-20",
			"skill://review:raw",
			"skill://review:raw:2-4",
			"skill://review/assets/example.md",
			"skill://review?part=1",
			"skill://review#section",
		]) {
			expect(parseRootSkillRead(path)).toBeUndefined();
		}
	});
});

describe("OMP observation", () => {
	test("records a completed user skill prompt with normalized fields only", () => {
		const observations: UsageObservation[] = [];
		const handlers = createObservationHandlers(observation => observations.push(observation));

		handlers.messageEnd(userMessage());

		expect(observations).toEqual([{ skill: "Plugin:Review", initiator: "user" }]);
	});

	test("rejects autoloads and malformed skill prompt messages", () => {
		const observations: UsageObservation[] = [];
		const handlers = createObservationHandlers(observation => observations.push(observation));

		handlers.messageEnd(userMessage({ attribution: "agent" }));
		handlers.messageEnd(userMessage({ details: { name: "review", path: "/skill", lineCount: -1 } }));
		handlers.messageEnd(userMessage({ customType: "other" }));
		handlers.messageEnd(userMessage({ display: false }));
		handlers.messageEnd(userMessage({ timestamp: Number.NaN }));

		expect(observations).toEqual([]);
	});

	test("records one model observation for a matched successful root read", () => {
		const observations: UsageObservation[] = [];
		const handlers = createObservationHandlers(observation => observations.push(observation));

		handlers.toolCall(rootRead("call-1"));
		handlers.toolResult(successfulRead("call-1"));
		handlers.toolResult(successfulRead("call-1"));

		expect(observations).toEqual([{ skill: "Plugin:Review", initiator: "model" }]);
	});

	test("rejects selected, asset, unmatched, changed, and failed model reads", () => {
		const observations: UsageObservation[] = [];
		const handlers = createObservationHandlers(observation => observations.push(observation));

		handlers.toolCall(rootRead("selected", "skill://review:4-8"));
		handlers.toolResult(successfulRead("selected", "skill://review:4-8"));

		handlers.toolCall(rootRead("asset", "skill://review/assets/example.md"));
		handlers.toolResult(successfulRead("asset", "skill://review/assets/example.md"));

		handlers.toolResult(successfulRead("unmatched", "skill://review"));

		handlers.toolCall(rootRead("changed", "skill://review"));
		handlers.toolResult(successfulRead("changed", "skill://other"));

		handlers.toolCall(rootRead("failed", "skill://review"));
		handlers.toolResult({ ...successfulRead("failed", "skill://review"), isError: true });

		expect(observations).toEqual([]);
	});

	test("clears pending correlations at session boundaries", () => {
		const observations: UsageObservation[] = [];
		const handlers = createObservationHandlers(observation => observations.push(observation));

		handlers.toolCall(rootRead("old-session"));
		handlers.reset();
		handlers.toolResult(successfulRead("old-session"));

		expect(observations).toEqual([]);
	});

	test("fails open when collection throws", () => {
		const handlers = createObservationHandlers(() => {
			throw new Error("collector unavailable");
		});

		expect(() => handlers.messageEnd(userMessage())).not.toThrow();
		expect(() => {
			handlers.toolCall(rootRead("call-1"));
			handlers.toolResult(successfulRead("call-1"));
		}).not.toThrow();
	});
});

describe("collector invocation", () => {
	test("uses the absolute Cargo install path", () => {
		expect(agentlensExecutablePath({ CARGO_INSTALL_ROOT: "/opt/agentlens" })).toBe(
			"/opt/agentlens/bin/agentlens",
		);
		expect(agentlensExecutablePath({ CARGO_HOME: "/Users/test/.cargo" })).toBe(
			"/Users/test/.cargo/bin/agentlens",
		);
	});

	test("passes only normalized event fields", () => {
		expect(
			collectorCommand({ skill: "Plugin:Review", initiator: "model" }, "/opt/agentlens/bin/agentlens"),
		).toEqual([
			"/opt/agentlens/bin/agentlens",
			"collect",
			"--client",
			"omp",
			"--skill",
			"Plugin:Review",
			"--initiator",
			"model",
		]);
	});
});
