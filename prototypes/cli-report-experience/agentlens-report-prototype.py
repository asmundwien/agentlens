#!/usr/bin/env python3
"""PROTOTYPE — three aggregate-report variants, selected by A/B/C or all.

This is a presentation prototype over fixed sample counts. It is not Agentlens code.
"""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from typing import Callable


BUCKETS = (
    ("omp", "user"),
    ("omp", "model"),
    ("claude_code", "user"),
    ("claude_code", "model"),
)


@dataclass(frozen=True)
class SkillCount:
    skill: str
    counts: tuple[int, int, int, int]

    @property
    def total(self) -> int:
        return sum(self.counts)


SAMPLE = (
    SkillCount("tdd", (2, 6, 0, 4)),
    SkillCount("grilling", (5, 0, 2, 0)),
    SkillCount("research", (0, 2, 0, 3)),
    SkillCount("wayfinder", (1, 1, 1, 0)),
)


def command(text: str) -> str:
    return f"\033[2m$ {text}\033[0m"


def heading(label: str, title: str, tradeoff: str) -> str:
    return f"\n\033[1;36m{label} — {title}\033[0m\n{tradeoff}\n"


def totals(skills: tuple[SkillCount, ...]) -> tuple[int, int]:
    return sum(skill.total for skill in skills), len(skills)


def render_matrix(skills: tuple[SkillCount, ...]) -> str:
    total, skill_count = totals(skills)
    lines = [
        f"{total} observed invocations · {skill_count} skills",
        "Scope: all retained events",
        "",
        "SKILL       OMP USER  OMP MODEL  CLAUDE CODE USER  CLAUDE CODE MODEL  TOTAL",
        "──────────  ────────  ─────────  ────────────────  ─────────────────  ─────",
    ]
    if not skills:
        lines.append("(no observed invocations)")
        return "\n".join(lines)
    for skill in skills:
        ou, om, cu, cm = skill.counts
        lines.append(
            f"{skill.skill:<10}  {ou:>8}  {om:>9}  {cu:>16}  {cm:>17}  {skill.total:>5}"
        )
    lines.append("──────────  ────────  ─────────  ────────────────  ─────────────────  ─────")
    bucket_totals = [sum(skill.counts[index] for skill in skills) for index in range(4)]
    lines.append(
        f"TOTAL       {bucket_totals[0]:>8}  {bucket_totals[1]:>9}  "
        f"{bucket_totals[2]:>16}  {bucket_totals[3]:>17}  {total:>5}"
    )
    return "\n".join(lines)


def matrix_json(skills: tuple[SkillCount, ...]) -> str:
    total, skill_count = totals(skills)
    payload = {
        "query": {"since": None, "until": None},
        "observed_invocation_count": total,
        "skill_count": skill_count,
        "skills": [
            {
                "skill": skill.skill,
                "observed_invocation_count": skill.total,
                "by_client_and_initiator": [
                    {
                        "client": client,
                        "initiator": initiator,
                        "observed_invocation_count": count,
                    }
                    for (client, initiator), count in zip(BUCKETS, skill.counts)
                ],
            }
            for skill in skills
        ],
    }
    return json.dumps(payload, indent=2)


def render_grouped(skills: tuple[SkillCount, ...]) -> str:
    total, skill_count = totals(skills)
    lines = ["ALL RETAINED EVENTS", f"{total} observed invocations across {skill_count} skills", ""]
    if not skills:
        lines.append("No observed invocations in this scope.")
        return "\n".join(lines)
    for skill in skills:
        ou, om, cu, cm = skill.counts
        lines.extend(
            [
                f"{skill.skill}  {skill.total}",
                f"  OMP          user {ou or '—':>2}   model {om or '—':>2}",
                f"  Claude Code  user {cu or '—':>2}   model {cm or '—':>2}",
                "",
            ]
        )
    return "\n".join(lines).rstrip()


def grouped_json(skills: tuple[SkillCount, ...]) -> str:
    total, _ = totals(skills)
    payload = {
        "scope": "all",
        "total": total,
        "skills": {
            skill.skill: {
                "total": skill.total,
                "omp": {"user": skill.counts[0], "model": skill.counts[1]},
                "claude_code": {"user": skill.counts[2], "model": skill.counts[3]},
            }
            for skill in skills
        },
    }
    return json.dumps(payload, indent=2)


def render_long(skills: tuple[SkillCount, ...]) -> str:
    total, skill_count = totals(skills)
    rows: list[str] = []
    omitted = 0
    for skill in skills:
        for (client, initiator), count in zip(BUCKETS, skill.counts):
            if count == 0:
                omitted += 1
                continue
            rows.append(f"{skill.skill:<10}  {client:<11}  {initiator:<9}  {count:>5}")
    lines = [
        "SKILL       CLIENT       INITIATOR  COUNT",
        "──────────  ───────────  ─────────  ─────",
        *(rows or ["(no observed invocations)"]),
        "",
        f"{total} observed invocations · {skill_count} skills · {omitted} zero buckets omitted",
    ]
    return "\n".join(lines)


def long_json(skills: tuple[SkillCount, ...]) -> str:
    total, skill_count = totals(skills)
    payload = {
        "window": {"start": None, "end": None},
        "summary": {"observed_invocations": total, "skills": skill_count},
        "counts": [
            {
                "skill": skill.skill,
                "client": client,
                "initiator": initiator,
                "count": count,
            }
            for skill in skills
            for (client, initiator), count in zip(BUCKETS, skill.counts)
        ],
    }
    return json.dumps(payload, indent=2)


HELP = {
    "A": """Usage: agentlens report [OPTIONS]\n\nShow observed skill invocations by skill, agent client, and initiator.\n\nOptions:\n  --since <BOUND>    Inclusive lower bound: 7d, 2026-08-01, or RFC 3339\n  --until <BOUND>    Exclusive upper bound: 2026-08-22 or RFC 3339\n  --format <FORMAT>  Output format: table or json [default: table]\n  -h, --help         Print help\n\nExamples:\n  agentlens report\n  agentlens report --since 7d\n  agentlens report --since 2026-08-01 --until 2026-09-01\n  agentlens report --format json | jq '.skills[] | select(.observed_invocation_count > 5)'\n\nDates are UTC day boundaries. Relative bounds are anchored once at command start.\nThe selected interval is [since, until).""",
    "B": """Usage: agentlens usage [OPTIONS]\n\nSummarize observed skill usage.\n\nOptions:\n  --last <DURATION>  Look back from command start, for example 24h or 7d\n  --from <TIME>      Inclusive ISO date or RFC 3339 timestamp\n  --before <TIME>    Exclusive ISO date or RFC 3339 timestamp\n  --json             Emit JSON instead of the grouped display\n  -h, --help         Print help\n\nExamples:\n  agentlens usage\n  agentlens usage --last 7d\n  agentlens usage --from 2026-08-01 --before 2026-09-01\n  agentlens usage --json | jq '.skills.tdd'\n\n--last cannot be combined with --from or --before. Dates use UTC.""",
    "C": """Usage: agentlens adoption [OPTIONS]\n\nPrint aggregate skill-adoption counts.\n\nOptions:\n  --time <RANGE>     Half-open range START..END; either side may be omitted\n  --output <FORMAT>  Output format: table or json [default: table]\n  -h, --help         Print help\n\nExamples:\n  agentlens adoption\n  agentlens adoption --time '7d..now'\n  agentlens adoption --time '2026-08-01..2026-09-01'\n  agentlens adoption --output json | jq '.counts[] | select(.count == 0)'\n\nDates are UTC day boundaries. Relative starts use command start as now.""",
}


@dataclass(frozen=True)
class Variant:
    title: str
    tradeoff: str
    proposed_command: str
    table: Callable[[tuple[SkillCount, ...]], str]
    structured: Callable[[tuple[SkillCount, ...]], str]
    structured_flag: str


VARIANTS = {
    "A": Variant(
        "Report matrix",
        "Fastest cross-client comparison; widest terminal output. Explicit zeros keep every bucket visible.",
        "agentlens report",
        render_matrix,
        matrix_json,
        "--format json",
    ),
    "B": Variant(
        "Grouped usage cards",
        "Easiest per-skill reading; weakest for scanning and automation. Em dashes render zero buckets.",
        "agentlens usage",
        render_grouped,
        grouped_json,
        "--json",
    ),
    "C": Variant(
        "Normalized adoption rows",
        "Narrow and grep-friendly; repeats skill names and hides table zeros. JSON keeps explicit zero rows.",
        "agentlens adoption",
        render_long,
        long_json,
        "--output json",
    ),
}


def show(label: str, scenario: str) -> None:
    variant = VARIANTS[label]
    print(heading(label, variant.title, variant.tradeoff))
    skills = () if scenario == "empty" else SAMPLE
    if scenario == "help":
        print(command(f"{variant.proposed_command} --help"))
        print(HELP[label])
    elif scenario == "json":
        print(command(f"{variant.proposed_command} {variant.structured_flag}"))
        print(variant.structured(skills))
    else:
        print(command(variant.proposed_command))
        print(variant.table(skills))


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Compare throwaway Agentlens aggregate-report presentations."
    )
    parser.add_argument("variant", choices=("A", "B", "C", "all"), nargs="?", default="all")
    parser.add_argument(
        "scenario", choices=("table", "empty", "help", "json"), nargs="?", default="table"
    )
    args = parser.parse_args()
    labels = VARIANTS if args.variant == "all" else (args.variant,)
    for label in labels:
        show(label, args.scenario)


if __name__ == "__main__":
    main()
