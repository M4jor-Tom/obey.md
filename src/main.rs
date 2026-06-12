use std::path::Path;

use clap::Parser;
use log::info;

use obey::config::configure_logging;
use obey::orchestrator::run_full_loop;

#[derive(Parser)]
#[command(name = "obey", about = "GLTF Scene Orchestrator - LLM-driven multi-character animation")]
struct Cli {
    #[arg(short = 'i', long = "input", required = true)]
    input: String,

    #[arg(short = 'o', long = "output", required = true)]
    output: String,

    #[arg(long = "agent-url")]
    agent_url: Option<String>,

    #[arg(long = "agent-name")]
    agent_name: Option<String>,

    #[arg(long = "agent-key")]
    agent_key: Option<String>,

    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    #[arg(long = "log-file")]
    log_file: Option<String>,
}

fn main() {
    let args = Cli::parse();

    configure_logging(
        if args.verbose { "DEBUG" } else { "INFO" },
        args.log_file.as_deref(),
    );

    if let Some(url) = &args.agent_url {
        if std::env::var("OPENAI_BASE_URL").is_err() {
            std::env::set_var("OPENAI_BASE_URL", url);
        }
    }
    if let Some(name) = &args.agent_name {
        if std::env::var("OPENAI_MODEL").is_err() {
            std::env::set_var("OPENAI_MODEL", name);
        }
    }
    if let Some(key) = &args.agent_key {
        if std::env::var("OPENAI_API_KEY").is_err() {
            std::env::set_var("OPENAI_API_KEY", key);
        }
    }

    let manifest_path = Path::new(&args.input).join("manifest.json");
    let manifest_path = manifest_path.to_string_lossy().to_string();

    match run_full_loop(&manifest_path, &args.input, &args.output) {
        Ok(scene) => {
            let scene_path = Path::new(&args.output).join("scene.gltf");
            let scene_json = serde_json::to_string_pretty(&scene)
                .unwrap_or_default();
            std::fs::write(&scene_path, &scene_json)
                .unwrap_or_else(|e| {
                    eprintln!("Failed to write scene: {}", e);
                });
            info!("Done. Scene written to {}", scene_path.display());
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
