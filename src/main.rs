use std::io::{self, Read};
use std::process::ExitCode;

use agentlens::claude_hook;
use agentlens::model::{Client, Initiator, UsageEvent};
use agentlens::report::{Query, Report};
use agentlens::storage::Store;
use chrono::Utc;
use clap::{Args, Parser, Subcommand, ValueEnum};

const MAX_HOOK_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Parser)]
#[command(
    name = "agentlens",
    about = "Collect and report observed skill invocations"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Record one normalized skill invocation. Intended for trusted client adapters.
    Collect(CollectArgs),
    /// Read and normalize one Claude Code hook payload from standard input.
    ClaudeHook,
    /// Show observed skill invocations by skill, agent client, and initiator.
    #[command(
        after_help = "Examples:\n  agentlens report\n  agentlens report --since 7d\n  agentlens report --since 2026-08-01 --until 2026-09-01\n  agentlens report --format json | jq '.skills[] | select(.observed_invocation_count > 5)'\n\nDates are UTC day boundaries. Relative --since bounds are anchored once at command start.\nThe selected interval is [since, until)."
    )]
    Report(ReportArgs),
}

#[derive(Debug, Args)]
struct CollectArgs {
    /// Agent client that emitted the normalized signal.
    #[arg(long, value_enum)]
    client: Client,
    /// Exact, most-qualified client-reported skill name.
    #[arg(long)]
    skill: String,
    /// Immediate party that requested the skill.
    #[arg(long, value_enum)]
    initiator: Initiator,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Table,
    Json,
}

#[derive(Debug, Args)]
struct ReportArgs {
    /// Inclusive lower bound: 7d, 2026-08-01, or RFC 3339.
    #[arg(long)]
    since: Option<String>,
    /// Exclusive upper bound: 2026-08-22 or RFC 3339.
    #[arg(long)]
    until: Option<String>,
    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    format: OutputFormat,
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Collect(args) => collect(args),
        Command::ClaudeHook => collect_claude_hook(),
        Command::Report(args) => report(args),
    }
}

fn collect(args: CollectArgs) -> ExitCode {
    let Some(event) = UsageEvent::observed(args.client, args.skill, args.initiator) else {
        eprintln!("agentlens: collection dropped: skill name is empty");
        return ExitCode::SUCCESS;
    };
    persist_fail_open(&event);
    ExitCode::SUCCESS
}

fn collect_claude_hook() -> ExitCode {
    let mut input = Vec::new();
    if io::stdin()
        .lock()
        .take(MAX_HOOK_BYTES + 1)
        .read_to_end(&mut input)
        .is_err()
    {
        eprintln!("agentlens: collection dropped: hook input is unreadable");
        return ExitCode::SUCCESS;
    }
    if input.len() as u64 > MAX_HOOK_BYTES {
        eprintln!("agentlens: collection dropped: hook input exceeds 1 MiB");
        return ExitCode::SUCCESS;
    }

    match claude_hook::normalize(&input) {
        Ok(Some(event)) => persist_fail_open(&event),
        Ok(None) => {}
        Err(_) => eprintln!("agentlens: collection dropped: invalid hook JSON"),
    }
    ExitCode::SUCCESS
}

fn persist_fail_open(event: &UsageEvent) {
    let result = Store::open_default().and_then(|mut store| store.insert(event));
    if result.is_err() {
        eprintln!("agentlens: collection dropped: storage unavailable");
    }
}

fn report(args: ReportArgs) -> ExitCode {
    let query = match Query::parse(args.since.as_deref(), args.until.as_deref(), Utc::now()) {
        Ok(query) => query,
        Err(error) => {
            eprintln!("agentlens: {error}");
            return ExitCode::FAILURE;
        }
    };
    let store = match Store::open_default() {
        Ok(store) => store,
        Err(error) => {
            eprintln!("agentlens: {error}");
            return ExitCode::FAILURE;
        }
    };
    let report = match Report::load(&store, &query) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("agentlens: {error}");
            return ExitCode::FAILURE;
        }
    };

    match args.format {
        OutputFormat::Table => println!("{}", report.render_table()),
        OutputFormat::Json => match serde_json::to_string_pretty(&report) {
            Ok(json) => println!("{json}"),
            Err(_) => {
                eprintln!("agentlens: failed to serialize report");
                return ExitCode::FAILURE;
            }
        },
    }
    ExitCode::SUCCESS
}
