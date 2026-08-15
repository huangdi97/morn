//! `morn` developer CLI: doctor / status / node / conformance / domain / package.
//! Commands call canonical services; they never write the DB directly.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("help");
    match cmd {
        "doctor" => doctor(),
        "status" => status(),
        "node" => node(&args),
        "conformance" => conformance(),
        "domain" => domain(&args),
        "package" => package(&args),
        "help" | "--help" | "-h" => {
            println!("morn doctor|status|node|conformance|domain|package");
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("unknown command: {other}");
            ExitCode::FAILURE
        }
    }
}

fn doctor() -> ExitCode {
    println!("morn doctor");
    let mut ok = true;
    for (name, probe) in [
        (
            "rustc",
            std::process::Command::new("rustc")
                .arg("--version")
                .output(),
        ),
        (
            "node",
            std::process::Command::new("node").arg("--version").output(),
        ),
        (
            "sqlite",
            std::process::Command::new("sqlite3")
                .arg("--version")
                .output(),
        ),
    ] {
        match probe {
            Ok(o) if o.status.success() => println!("  OK {name}"),
            _ => {
                println!("  MISSING {name}");
                ok = false;
            }
        }
    }
    let state = morn_app::AppState::new(":memory:");
    match state {
        Ok(s) => {
            let g = s.lock();
            println!(
                "  OK store schema v{}",
                g.store.schema_version().unwrap_or(0)
            );
            println!("  OK canonical ledger path");
        }
        Err(e) => {
            println!("  FAIL store: {e}");
            ok = false;
        }
    }
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn status() -> ExitCode {
    match morn_app::AppState::new(":memory:") {
        Ok(state) => {
            let g = state.lock();
            println!("workspace: {}", g.workspace.name);
            println!("world objects: {}", g.world.objects().len());
            println!("work packages: {}", g.work.work_packages().len());
            println!("artifact versions: {}", g.artifacts.all_version_count());
            println!(
                "certified capabilities: {}",
                g.certification.capabilities.len()
            );
            println!("managed deliveries: {}", g.managed.runs.len());
            println!("rollback receipts: {}", g.rollback.receipts.len());
            println!("predictors: {}", g.predictors.predictors.len());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("status failed: {e}");
            ExitCode::FAILURE
        }
    }
}

fn node(args: &[String]) -> ExitCode {
    let sub = args.get(2).map(String::as_str).unwrap_or("list");
    let mut rt = morn_node::DistributedRuntime::new();
    rt.register_node(morn_node::MornNode::new(
        "node-a",
        morn_node::NodeType::Desktop,
    ));
    rt.register_node(morn_node::MornNode::new(
        "node-b",
        morn_node::NodeType::Worker,
    ));
    match sub {
        "list" => {
            for n in &rt.nodes {
                println!("{} type={:?} healthy={}", n.name, n.node_type, n.healthy);
            }
            ExitCode::SUCCESS
        }
        "doctor" => {
            let healthy = rt.nodes.iter().all(|n| n.healthy);
            println!("nodes: {} healthy={}", rt.nodes.len(), healthy);
            if healthy {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        _ => {
            eprintln!("node subcommand: list|doctor");
            ExitCode::FAILURE
        }
    }
}

fn conformance() -> ExitCode {
    let mut ok = true;
    // Intelligence provider conformance (two fixtures).
    for p in [
        &morn_harness::RuleIntelligence as &dyn morn_harness::IntelligenceProvider,
        &morn_harness::SolverIntelligence,
    ] {
        if let Err(e) = morn_harness::run_intelligence_conformance(p) {
            println!("  FAIL intelligence {}: {e}", p.provider_name());
            ok = false;
        } else {
            println!("  OK intelligence conformance {}", p.provider_name());
        }
    }
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn domain(args: &[String]) -> ExitCode {
    let sub = args.get(2).map(String::as_str).unwrap_or("list");
    let reg = morn_domain_sdk::DomainRegistry::new();
    match sub {
        "list" => {
            println!("domain packs installed: {}", reg.definitions.len());
            println!("enabled: {}", reg.enabled.len());
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("domain subcommand: list");
            ExitCode::FAILURE
        }
    }
}

fn package(args: &[String]) -> ExitCode {
    let sub = args.get(2).map(String::as_str).unwrap_or("inspect");
    let mut lc = morn_package::PackLifecycle::new();
    match sub {
        "inspect" => {
            println!("packs: {}", lc.packs.len());
            println!("lifecycle history events: {}", lc.history.len());
            ExitCode::SUCCESS
        }
        "init" => {
            let m = morn_package::PackManifest::new(
                "hello-domain",
                "domain-pack",
                morn_package::PackVersion::v1(),
            );
            match lc.init(m) {
                Ok(id) => {
                    println!("initialized pack {id}");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("init failed: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        _ => {
            eprintln!("package subcommand: inspect|init");
            ExitCode::FAILURE
        }
    }
}
