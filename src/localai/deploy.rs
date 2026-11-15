//! Download a model from HuggingFace and launch it with llama-server.

use crate::hardware::HardwareInfo;
use crate::localai::catalog::{ModelSpec, recommend};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use thiserror::Error;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Error)]
pub enum DeployError {
    #[error("no suitable model for this hardware")]
    NoModel,
    #[error("llama-server not found on PATH; install llama.cpp first")]
    NoLlamaServer,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("model failed to start: {0}")]
    StartFailed(String),
    #[error("timed out waiting for llama-server to become ready")]
    Timeout,
}

/// Result of a successful deployment.
#[derive(Debug, Clone)]
pub struct LocalDeployment {
    pub model: String,
    pub model_path: PathBuf,
    pub endpoint: String,
    pub port: u16,
}

/// Manages a running llama-server child process.
pub struct LocalServer {
    pub child: Child,
    pub endpoint: String,
    pub port: u16,
}

impl Drop for LocalServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

/// Default directory for downloaded models.
pub fn models_dir() -> PathBuf {
    let base = std::env::var("NATALIE_MODELS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            PathBuf::from(home).join(".local/share/natalie/models")
        });
    base
}

/// Download a model if not already present. Returns the local path.
pub async fn download_model(spec: &ModelSpec, dest_dir: &Path) -> Result<PathBuf, DeployError> {
    tokio::fs::create_dir_all(dest_dir).await?;
    let target = dest_dir.join(spec.filename);
    if target.exists() {
        tracing::info!(path = %target.display(), "model already downloaded");
        return Ok(target);
    }

    let url = format!(
        "https://huggingface.co/{}/resolve/main/{}",
        spec.repo, spec.filename
    );
    tracing::info!(%url, "downloading model");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3600))
        .build()
        .map_err(|e| DeployError::Http(e))?;

    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(DeployError::StartFailed(format!(
            "download failed: HTTP {}",
            resp.status()
        )));
    }

    let total = resp.content_length();
    let mut file = tokio::fs::File::create(&target).await?;
    let mut stream = resp.bytes_stream();
    use futures::StreamExt;
    let mut downloaded: u64 = 0;
    let mut last_report = std::time::Instant::now();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        if last_report.elapsed().as_secs() >= 2 {
            if let Some(t) = total {
                let pct = (downloaded as f64 / t as f64 * 100.0) as u64;
                eprintln!("downloading {} … {}% ({} / {} MB)", spec.name, pct, downloaded / 1_048_576, t / 1_048_576);
            } else {
                eprintln!("downloaded {} MB", downloaded / 1_048_576);
            }
            last_report = std::time::Instant::now();
        }
    }
    file.flush().await?;
    eprintln!("download complete: {}", target.display());
    Ok(target)
}


fn path_lookup(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let cand = dir.join(name);
        if cand.is_file() {
            return Some(cand);
        }
    }
    None
}

/// Check that llama-server is available.
pub fn find_llama_server() -> Option<PathBuf> {
    // PATH lookup
    if let Some(path) = path_lookup("llama-server") {
        return Some(path);
    }
    // Common locations
    for cand in [
        "/usr/bin/llama-server",
        "/usr/local/bin/llama-server",
        "/opt/llama.cpp/llama-server",
    ] {
        let p = PathBuf::from(cand);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// Deploy: detect hardware, recommend, download, launch. Returns deployment info.
pub async fn deploy(preferred: Option<&str>) -> Result<(LocalDeployment, LocalServer), DeployError> {
    let hw = HardwareInfo::detect();
    tracing::info!(?hw, "detected hardware");

    let spec: &ModelSpec = if let Some(name) = preferred {
        crate::localai::catalog::CATALOG
            .iter()
            .find(|m| m.name == name || m.filename == name)
            .ok_or(DeployError::NoModel)?
    } else {
        recommend(&hw).ok_or(DeployError::NoModel)?
    };

    eprintln!(
        "Selected model: {} ({} MB) — {}",
        spec.name, spec.size_mb, spec.description
    );

    let server_bin = find_llama_server().ok_or(DeployError::NoLlamaServer)?;
    let model_path = download_model(spec, &models_dir()).await?;

    // Pick a port.
    let port: u16 = std::env::var("NATALIE_LOCAL_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8090);

    let mut child = Command::new(&server_bin)
        .arg("--model")
        .arg(&model_path)
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(port.to_string())
        .arg("--ctx-size")
        .arg("4096")
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()?;

    // Wait for readiness.
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| DeployError::Http(e))?;
    let url = format!("http://127.0.0.1:{port}");
    let health = format!("{url}/health");
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(300);
    loop {
        if start.elapsed() > timeout {
            let _ = child.kill();
            return Err(DeployError::Timeout);
        }
        if let Ok(resp) = client.get(&health).send().await {
            if resp.status().is_success() {
                break;
            }
        }
        // Check process liveness.
        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(DeployError::StartFailed(format!(
                    "llama-server exited early: {status}"
                )));
            }
            _ => {}
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    let deployment = LocalDeployment {
        model: spec.name.to_string(),
        model_path,
        endpoint: url.clone(),
        port,
    };

    let server = LocalServer {
        child,
        endpoint: url,
        port,
    };

    Ok((deployment, server))
}
