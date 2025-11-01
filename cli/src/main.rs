use std::fs;
use std::fs::File;
use std::io::Write;
use std::io::Cursor;
use std::path::Path;
use std::process::Command as ProcessCommand;
use std::os::unix::fs::PermissionsExt;
use clap::{Parser, Subcommand};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;
use zip::ZipArchive;
use zip::write::FileOptions;
use zip::CompressionMethod;
use walkdir::WalkDir;

const REGISTRY_URL: &str = "http://mpkg.idiotstudios.co.za/api/";
const LOADER_VERSION: &str = "latest";

#[derive(Parser)]
#[command(name = "mpkg", version, about = "Package manager for Mpkg")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Install { name: String },
    InstallNpm { name: String },
    Init { name: String },
    Run { file: String, args: Vec<String>},
    Package { file: String, output: Option<String> },
    Update
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
        Commands::Install { name } => install_package(&name)?,
        Commands::InstallNpm { name} => install_npm_package(&name)?,
        Commands::Init { name } => init_project(&name)?,
        Commands::Run { file, args } => run_js(&file, &args)?,
        Commands::Package { file , output} => {
            let output_file = output.unwrap_or_else(|| {
                let base = Path::new(&file)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                format!("{}.zip", base)
            });
            create_zip(&file, &output_file)?
        }
        Commands::Update => update_self()?
    }

    Ok(())
}

fn update_self() -> Result<()> {
    use std::time::Duration;
    let repo = "idiotstudios/mpkg";
    let current_version = env!("CARGO_PKG_VERSION");
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("mpkg-updater")
        .build()?;

    println!("🔍 Checking for updates...");
    let resp: serde_json::Value = client
        .get(format!("https://api.github.com/repos/{repo}/releases/latest"))
        .send()?
        .json()?;

    let latest_tag = resp["tag_name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Failed to read release tag"))?;

    if latest_tag == current_version {
        println!("✅ Already up to date ({})", current_version);
        return Ok(());
    }

    println!("⬇️  Found new version: {}", latest_tag);

    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let pattern = format!("mpkg-{os}-{arch}");
    let asset_url = resp["assets"]
        .as_array()
        .and_then(|arr| {
            arr.iter()
                .find_map(|a| a["browser_download_url"].as_str())
                .filter(|url| url.contains(&pattern))
        })
        .ok_or_else(|| anyhow::anyhow!("No asset found for {os}-{arch}"))?;

    println!("📦 Downloading binary for {os}/{arch}...");
    let bytes = client.get(asset_url).send()?.bytes()?;

    let exe_path = std::env::current_exe()?;
    let tmp_path = exe_path.with_extension("update");

    std::fs::write(&tmp_path, &bytes)?;
    #[cfg(unix)]
    {
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&tmp_path, perms)?;
    }

    // Windows can't overwrite a running binary
    if cfg!(windows) {
        println!("♻️  Restarting updater...");
        ProcessCommand::new("cmd")
            .args([
                "/C",
                &format!(
                    "timeout /t 1 >nul & move /Y \"{}\" \"{}\" & start \"\" \"{}\"",
                    tmp_path.display(),
                    exe_path.display(),
                    exe_path.display()
                ),
            ])
            .spawn()?;
    } else {
        fs::rename(&tmp_path, &exe_path)?;
        println!("✅ Updated successfully!");
        println!("🔄 Restarting...");
        ProcessCommand::new(exe_path).spawn()?;
    }

    Ok(())
}

fn create_zip<P: AsRef<Path>>(source: P, output: P) -> Result<()> {
    let file = File::create(&output)?;
    let mut zip = zip::ZipWriter::new(file);
    let options: FileOptions<'_ , ()> = FileOptions::default()
        .compression_method(CompressionMethod::Deflated);

    let base_path = source.as_ref();

    for entry in WalkDir::new(base_path) {
        let entry = entry?;
        let path = entry.path();
        let name = path.strip_prefix(base_path)?.to_string_lossy();

        if path.is_file() {
            zip.start_file(name.replace("\\", "/"), options)?;
            std::io::copy(&mut std::fs::File::open(path)?, &mut zip)?;
        } else if !name.is_empty() {
            zip.add_directory(name.replace("\\", "/") + "/", options)?;
        }
    }

    zip.finish()?;
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
    let packages_dir = Path::new("packages");
    fs::create_dir_all(&packages_dir)?;

    let reader = Cursor::new(bytes);
    let mut zip = ZipArchive::new(reader)?;

    let package_dir = packages_dir.join(name);
    fs::create_dir_all(&package_dir)?;
    zip.extract(&package_dir)?;

    println!("✅ Package extracted to {:?}", package_dir);

    update_pkg_jsonc(name)?;
    Ok(())
}

fn install_npm_package(name: &str) -> Result<()> {
    use std::process::Command;
    println!("📦 Installing {} via npm...", name);

    // Make sure node_modules exists
    std::fs::create_dir_all("packages/node_modules")?;

    let status = Command::new("npm")
        .arg("install")
        .arg(name)
        .arg("--prefix")
        .arg("packages/") // installs inside packages/node_modules
        .status()?;

    if !status.success() {
        anyhow::bail!("npm install failed with status: {}", status);
    }

    // Update pkg.jsonc
    let dep_name = format!("npm/{}", name);
    update_pkg_jsonc(&dep_name)?;

    println!("✅ Installed {} via npm", name);

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
    let gitignorepath = std::path::Path::new(".gitignore");

    if gitignorepath.exists() {
        println!(".gitignore already exists, skipping creation...");
    } else {
        let gitignore = "/packages\n";
        let mut gitignore_file = File::create(".gitignore")?;
        gitignore_file.write_all(gitignore.as_bytes())?;
        println!("Created .gitignore");
    }
    if path.exists() {
        println!("pkg.jsonc already exists, skipping creation...");
    } else {
        let pkg = PkgManifest {
            name: project_name.to_string(),
            version: "0.1.0".to_string(),
            dependencies: Some(serde_json::json!({})),
        };
        fs::write(path, serde_json::to_string_pretty(&pkg)?)?;
        println!("Created pkg.jsonc");
    }

    let loader_url1 = format!("{REGISTRY_URL}/loader/{LOADER_VERSION}/1");
    let loader_url2 = format!("{REGISTRY_URL}/loader/{LOADER_VERSION}/2");
    let loader_path1 = Path::new("packages/loader").join("bootstrap.mjs");
    let loader_path2 = Path::new("packages/loader").join("mpkg-loader.mjs");
    fs::create_dir_all("packages/loader")?;
    let resp1 = reqwest::blocking::get(&loader_url1)?;
    if !resp1.status().is_success() {
        anyhow::bail!("Failed to download loader: {}", resp1.status());
    }
    let resp2 = reqwest::blocking::get(&loader_url2)?;
    if !resp2.status().is_success() {
        anyhow::bail!("Failed to download loader: {}", resp2.status());
    }
    let bytes1 = resp1.bytes()?;
    let bytes2 = resp2.bytes()?;
    fs::write(&loader_path1, &bytes1)?;
    fs::write(&loader_path2, &bytes2)?;
    println!("📝 Downloaded mpkg-loader.mjs to {:?}", loader_path2);
    println!("📝 Downloaded bootsrap.mjs to {:?}", loader_path1);
    println!("🎉 Initialized new project: {project_name}");
    Ok(())
}

fn run_js(file: &str, args: &[String]) -> Result<()> {
    use std::process::Command;

    let loader_path = env::current_dir()?.join("packages/loader/bootstrap.mjs");

    if !loader_path.exists() {
        anyhow::bail!("Loader not found at {:?}", loader_path);
    }

    let mut cmd = Command::new("node");
    cmd.arg(loader_path)
       .arg(file)
       .args(args);

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("Node exited with status: {}", status);
    }

    Ok(())
}

