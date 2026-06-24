//! Detached spawn of the headless `research run` flow.
//!
//! On approve, the TUI does not execute the research itself — it spawns a
//! detached child of `current_exe` with the generated brief and returns to
//! a confirmation screen. The child survives even if the user quits the
//! TUI immediately afterward (`setsid` on Unix puts it in its own session).

use std::path::{Path, PathBuf};

use crate::brief::TuiBrief;

/// Information about a successful (or fake) detached launch.
#[derive(Clone, Debug)]
pub struct SpawnInfo {
    pub session_id: String,
    pub topic: String,
    pub provider: String,
    pub processor: String,
    pub language: String,
    pub output_path: PathBuf,
    pub log_path: PathBuf,
    pub pid: u32,
    pub mocked: bool,
}

/// Mode of the detached launch.
pub enum SpawnMode<'a> {
    /// Real launch — `current_exe` plus `run` subcommand.
    Real { root: &'a Path },
    /// Fake launch — useful for `--mock` and the screenshot tour.
    Mock { root: &'a Path },
}

/// Launch detached, return info about the spawned child.
pub fn spawn(
    brief: &TuiBrief,
    provider: &str,
    processor: &str,
    mode: SpawnMode<'_>,
) -> Result<SpawnInfo, String> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let short = &session_id[..8];
    let topic = brief.topic.clone();
    let language = brief.language.clone();
    let provider = provider.to_string();
    let processor = processor.to_string();

    let (root, mocked) = match mode {
        SpawnMode::Real { root } => (root.to_path_buf(), false),
        SpawnMode::Mock { root } => (root.to_path_buf(), true),
    };
    let out = root.join("output");
    let stamp = chrono::Local::now().naive_local();
    let org = research_storage::organizer::organizer(&out);
    let folder_name =
        research_storage::organizer::Organized::name(&org, &stamp, &topic, &session_id);
    let output_path = out.join(&folder_name);
    let log_path = out.join(format!("run-{}.log", short));

    if mocked {
        return Ok(SpawnInfo {
            session_id,
            topic,
            provider,
            processor,
            language,
            output_path,
            log_path,
            pid: 0,
            mocked: true,
        });
    }

    let exe = std::env::current_exe().map_err(|e| format!("current_exe failed: {e}"))?;
    let query = brief.render_query();

    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir output: {e}"))?;
    }
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| format!("open log: {e}"))?;
    let log_dup = log_file.try_clone().map_err(|e| format!("dup log: {e}"))?;

    let mut cmd = std::process::Command::new(&exe);
    cmd.arg("run")
        .arg(&topic)
        .arg(&query)
        .arg("--processor")
        .arg(&processor)
        .arg("--language")
        .arg(&language)
        .arg("--provider")
        .arg(&provider)
        .env("RESEARCH_SESSION_ID", &session_id)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::from(log_file))
        .stderr(std::process::Stdio::from(log_dup))
        .current_dir(&root);

    #[cfg(unix)]
    unsafe {
        use std::os::unix::process::CommandExt;
        cmd.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }

    let child = cmd.spawn().map_err(|e| format!("spawn: {e}"))?;
    let pid = child.id();

    Ok(SpawnInfo {
        session_id,
        topic,
        provider,
        processor,
        language,
        output_path,
        log_path,
        pid,
        mocked: false,
    })
}
