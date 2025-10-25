use std::fs;
use std::path::Path;
use clap::{Parser, Subcommand};
use anyhow::Result;
use serde::{Deserialize, Serialize};

const REGISTRY_URL: &str = "http://127.0.0.1:8080";

#[derive(Parser)]
#[command(name = "mpkg", version, about = "Package manager for Mpkg")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Download a package from the registry
    Install {
        /// Package name
        name: String,
    },

    Init {
        /// Project Name
        name: String,
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct PkgManifest {
    name: String,
    version: String,
    dependencies: Option<serde_json::Value>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install { name } => {
            install_package(&name)?;
        }
        Commands::Init { name } => {
            init_project(&name)?;
        }
    }

    Ok(())
}

fn install_package(name: &str) -> Result<()> {
    let url = format!("{REGISTRY_URL}/download/{name}");
    println!("⬇️  Downloading {name} from {url}...");

    let response = reqwest::blocking::get(&url)?;
    if !response.status().is_success() {
        anyhow::bail!("Failed to download package: {}", response.status());
    }

    let bytes = response.bytes()?;
    let file_path = Path::new("packages").join(format!("{name}.zip"));
    fs::create_dir_all("packages")?;
    fs::write(&file_path, &bytes)?;
    println!("✅ Saved to {:?}", file_path);

    update_pkg_jsonc(name)?;
    Ok(())
}

fn update_pkg_jsonc(pkg_name: &str) -> Result<()> {
    let path = Path::new("pkg.jsonc");

    let mut pkg: PkgManifest = if path.exists() {
        let text = fs::read_to_string(path)?;
        serde_json::from_str(&text)?
    } else {
        PkgManifest {
            name: "my_project".into(),
            version: "0.1.0".into(),
            dependencies: Some(serde_json::json!({})),
        }
    };

    let mut deps = pkg.dependencies.take().unwrap_or(serde_json::json!({}));
    if let Some(obj) = deps.as_object_mut() {
        obj.insert(pkg_name.to_string(), serde_json::json!("latest"));
    }
    pkg.dependencies = Some(deps);

    fs::write(path, serde_json::to_string_pretty(&pkg)?)?;
    println!("📝 Updated pkg.jsonc with dependency: {pkg_name}");
    Ok(())
}

fn init_project(project_name: &str) -> Result<()> {
    let path = std::path::Path::new("pkg.jsonc");

    if path.exists() {
        println!("⚠️ pkg.jsonc already exists, skipping creation.");
        return Ok(());
    }

    let pkg = PkgManifest {
        name: project_name.to_string(),
        version: "0.1.0".to_string(),
        dependencies: Some(serde_json::json!({})),
    };

    fs::write(path, serde_json::to_string_pretty(&pkg)?)?;
    println!("🎉 Initialized new project: {project_name}");
    Ok(())
}