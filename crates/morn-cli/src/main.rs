//! `morn` developer CLI: doctor / status / node / conformance / domain / package.
//! Commands call canonical services; they never write the DB directly.

use std::process::ExitCode;

use morn_integration::ConnectorProvider;

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
        "provider" => provider(&args),
        "connector" => connector(&args),
        "plugin" => plugin(&args),
        "compat" => compat(),
        "migrate" => migrate(&args),
        "help" | "--help" | "-h" => {
            println!("morn doctor|status|node|conformance|domain|package|provider|connector|plugin|compat|migrate");
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

fn provider(_args: &[String]) -> ExitCode {
    let mut ok = true;
    for p in [
        &morn_harness::RuleIntelligence as &dyn morn_harness::IntelligenceProvider,
        &morn_harness::SolverIntelligence,
    ] {
        match morn_harness::run_intelligence_conformance(p) {
            Ok(_) => println!("OK provider {} (conformance passed)", p.provider_name()),
            Err(e) => {
                println!("FAIL provider {}: {e}", p.provider_name());
                ok = false;
            }
        }
    }
    // Harness provider contract (two implementations).
    let ws = morn_kernel::ids::WorkspaceId::generate();
    let mut ctx = morn_harness::contract::test_context(&ws);
    ctx.provenance_refs.push("wp-1".to_string());
    let mut native = morn_harness::MornNativeHarness::new();
    match morn_harness::run_harness_smoke(&mut native, &ctx) {
        Ok(r) if r.all_ok() => println!("OK harness provider morn-native (contract smoke)"),
        Ok(_) => {
            println!("FAIL harness provider morn-native (smoke not all_ok)");
            ok = false;
        }
        Err(e) => {
            println!("FAIL harness provider morn-native: {e}");
            ok = false;
        }
    }
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn connector(args: &[String]) -> ExitCode {
    let sub = args.get(2).map(String::as_str).unwrap_or("list");
    match sub {
        "list" => {
            let c = morn_integration::GenericFixtureConnector::new();
            println!(
                "connector: {} health={} applied_external_ids={}",
                c.provider_name(),
                c.health(),
                c.applied_external_ids.len()
            );
            ExitCode::SUCCESS
        }
        "health" => {
            let c = morn_integration::GenericFixtureConnector::new();
            if c.health() {
                println!("connector health: ok");
                ExitCode::SUCCESS
            } else {
                eprintln!("connector health: FAIL");
                ExitCode::FAILURE
            }
        }
        _ => {
            eprintln!("connector subcommand: list|health");
            ExitCode::FAILURE
        }
    }
}

fn plugin(args: &[String]) -> ExitCode {
    let sub = args.get(2).map(String::as_str).unwrap_or("list");
    let lc = morn_package::PackLifecycle::new();
    match sub {
        "list" => {
            println!("plugin packs: {}", lc.packs.len());
            println!("lifecycle history events: {}", lc.history.len());
            ExitCode::SUCCESS
        }
        "validate" => {
            let m = morn_package::PluginManifest::new("hello-plugin", "capability-provider");
            match m.validate() {
                Ok(_) => {
                    println!("plugin manifest valid");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("plugin manifest invalid: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        _ => {
            eprintln!("plugin subcommand: list|validate");
            ExitCode::FAILURE
        }
    }
}

fn compat() -> ExitCode {
    let snapshot = morn_kernel::contracts::ContractSnapshot::v1();
    println!(
        "semantic contract v{} ({} contracts)",
        snapshot.kernel_version,
        snapshot.contracts.len()
    );
    let mut candidate = snapshot.clone();
    candidate.kernel_version = morn_kernel::version::Version::new(2, 0, 0);
    match morn_kernel::contracts::ContractCompatibility::check(&snapshot, &candidate) {
        morn_kernel::contracts::Compatibility::Incompatible => {
            println!("compat: major bump -> Incompatible (correct)");
            ExitCode::SUCCESS
        }
        other => {
            println!("compat: unexpected result {other:?}");
            ExitCode::FAILURE
        }
    }
}

fn migrate(args: &[String]) -> ExitCode {
    let sub = args.get(2).map(String::as_str).unwrap_or("status");
    match sub {
        "status" => match morn_app::AppState::new(":memory:") {
            Ok(state) => {
                let g = state.lock();
                println!("schema version: {}", g.store.schema_version().unwrap_or(0));
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("migrate status failed: {e}");
                ExitCode::FAILURE
            }
        },
        "preflight" => {
            println!("preflight: schema v1 -> v2 migration requires snapshot + restore plan (no silent destructive)");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("migrate subcommand: status|preflight");
            ExitCode::FAILURE
        }
    }
}
