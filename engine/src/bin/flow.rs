use it_network_eng_sim::flow::cplex::{CplexMetrics, read_solution};
use it_network_eng_sim::flow::{FlowConfig, FlowSnapshot, FlowSolver, gen_lp, lp_filename};
use serde::Serialize;
use std::{
    env, fs,
    path::Path,
    process::{Command, ExitCode},
    time::Instant,
};

#[derive(Serialize)]
struct CplexReport {
    wall_seconds: f64,
    metrics: CplexMetrics,
    solution: String,
    log: String,
}

#[derive(Serialize)]
struct FlowReport {
    solver: &'static str,
    rust_wall_seconds: f64,
    result: FlowSnapshot,
    cplex: Option<CplexReport>,
}

fn option<'a>(args: &'a [String], key: &str) -> Result<Option<&'a str>, String> {
    args.iter()
        .position(|a| a == key)
        .map(|position| {
            args.get(position + 1)
                .filter(|value| !value.starts_with("--"))
                .map(String::as_str)
                .ok_or_else(|| format!("Expected a value after {key}"))
        })
        .transpose()
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    fs::write(path, json).map_err(|error| error.to_string())
}

fn solve(cfg: FlowConfig, work: u32) -> Result<FlowReport, String> {
    let started = Instant::now();
    let mut solver = FlowSolver::new(cfg)?;
    solver.advance(work);
    Ok(FlowReport {
        solver: "rust",
        rust_wall_seconds: started.elapsed().as_secs_f64(),
        result: solver.snapshot(),
        cplex: None,
    })
}

fn run_case(
    cfg: FlowConfig,
    dir: &Path,
    work: u32,
    cplex: Option<&str>,
) -> Result<FlowReport, String> {
    let name = lp_filename(&cfg);
    let stem = name.trim_end_matches(".lp");
    let lp_path = dir.join(&name);
    fs::write(&lp_path, gen_lp(&cfg)?).map_err(|error| error.to_string())?;
    let mut report = solve(cfg, work)?;
    if let Some(executable) = cplex {
        let solution_name = format!("{stem}.sol");
        let solution = dir.join(&solution_name);
        if solution.exists() {
            fs::remove_file(&solution).map_err(|error| error.to_string())?;
        }
        let started = Instant::now();
        let output = Command::new(executable)
            .args([
                "-c",
                &format!("read \"{}\"", lp_path.display()),
                "optimize",
                "display solution variables -",
                &format!("write \"{}\"", solution.display()),
            ])
            .output()
            .map_err(|error| format!("Could not execute CPLEX: {error}"))?;
        let wall_seconds = started.elapsed().as_secs_f64();
        let log = format!("{stem}.cplex.log");
        fs::write(
            dir.join(&log),
            format!(
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
        )
        .map_err(|error| error.to_string())?;
        if !output.status.success() || !solution.is_file() {
            return Err(format!("CPLEX did not produce a solution; inspect {log}"));
        }
        let metrics = read_solution(
            &fs::read_to_string(&solution).map_err(|error| error.to_string())?,
            report.result.config.trans,
        )?;
        report.cplex = Some(CplexReport {
            wall_seconds,
            metrics,
            solution: solution_name,
            log,
        });
    }
    write_json(&dir.join(format!("{stem}.json")), &report)?;
    Ok(report)
}

fn compact_json(value: &impl Serialize) -> Result<String, String> {
    serde_json::to_string(value).map_err(|error| error.to_string())
}

fn benchmark_csv(results: &[FlowReport]) -> Result<String, String> {
    let mut csv = String::from(
        "Y,rust_wall_seconds,rust_status,rust_objective,rust_nonzero_links,rust_max_capacity,rust_transit_loads,rust_highest_links,cplex_wall_seconds,cplex_status,cplex_objective,cplex_nonzero_links,cplex_max_capacity,cplex_transit_loads,cplex_highest_links\n",
    );
    for report in results {
        let result = &report.result;
        let mut fields = vec![
            result.config.trans.to_string(),
            report.rust_wall_seconds.to_string(),
            result.status.to_string(),
            result
                .obj
                .map(|value| value.to_string())
                .unwrap_or_default(),
            result.nonzero_links.to_string(),
            result
                .max_cap
                .map(|value| value.to_string())
                .unwrap_or_default(),
            compact_json(&result.loads)?,
            compact_json(&result.highest_links)?,
        ];
        if let Some(cplex) = &report.cplex {
            let metrics = &cplex.metrics;
            fields.extend([
                cplex.wall_seconds.to_string(),
                metrics.status.clone(),
                metrics.obj.to_string(),
                metrics.nonzero_links.to_string(),
                metrics.max_cap.to_string(),
                compact_json(&metrics.loads)?,
                compact_json(&metrics.highest_links)?,
            ]);
        } else {
            fields.extend(vec![String::new(); 7]);
        }
        csv.push_str(
            &fields
                .iter()
                .map(|field| format!("\"{}\"", field.replace('"', "\"\"")))
                .collect::<Vec<_>>()
                .join(","),
        );
        csv.push('\n');
    }
    Ok(csv)
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "--help") {
        println!(
            "flow X Y Z [--output model.lp] [--solve] [--work 1000000]\nflow --benchmark DIR [--work 1000000] [--cplex /path/to/cplex]\nLP: positive counts, at most 1,000,000 path variables. Rust solving: 1..32 per tier.\nY<3 exports a valid, intentionally infeasible LP."
        );
        return Ok(());
    }
    let work: u32 = option(&args, "--work")?
        .unwrap_or("1000000")
        .parse()
        .map_err(|_| "Invalid --work count")?;
    if let Some(directory) = option(&args, "--benchmark")? {
        let dir = Path::new(directory);
        fs::create_dir_all(dir).map_err(|error| error.to_string())?;
        let cplex = option(&args, "--cplex")?;
        let mut results = Vec::new();
        for y in 3..=7 {
            let report = run_case(
                FlowConfig {
                    srcs: 7,
                    trans: y,
                    dests: 7,
                    seed: 1,
                },
                dir,
                work,
                cplex,
            )?;
            let obj = report
                .result
                .obj
                .map(|value| value.to_string())
                .unwrap_or_else(|| "—".into());
            println!(
                "Y={y}: {} | r={obj} | Rust {:.6}s",
                report.result.status, report.rust_wall_seconds
            );
            results.push(report);
        }
        write_json(&dir.join("benchmark.json"), &results)?;
        fs::write(dir.join("benchmark.csv"), benchmark_csv(&results)?)
            .map_err(|error| error.to_string())?;
    } else {
        if args.len() < 3 {
            return Err("Expected X Y Z; see --help".into());
        }
        let parse = |n: usize| {
            args[n]
                .parse::<usize>()
                .map_err(|_| "Expected three positive integer counts".to_string())
        };
        let cfg = FlowConfig {
            srcs: parse(0)?,
            trans: parse(1)?,
            dests: parse(2)?,
            seed: 1,
        };
        let output = option(&args, "--output")?
            .map(str::to_owned)
            .unwrap_or_else(|| lp_filename(&cfg));
        fs::write(&output, gen_lp(&cfg)?).map_err(|error| error.to_string())?;
        println!("Wrote {output}");
        if args.iter().any(|a| a == "--solve") {
            let report = solve(cfg, work)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
            );
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
