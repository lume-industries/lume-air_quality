use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use reminders_slide::RemindersPayload;
use vzglyd_sidecar::{channel_active, channel_push, env_var, sleep_secs};

fn read_updated_payload(
    path: &Path,
    last_modified: &mut Option<SystemTime>,
) -> Result<Option<Vec<u8>>, String> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => return Ok(None),
    };
    let modified = metadata
        .modified()
        .map_err(|error| format!("read mtime for '{}': {error}", path.display()))?;
    if last_modified.as_ref().is_some_and(|previous| *previous >= modified) {
        return Ok(None);
    }

    let bytes =
        fs::read(path).map_err(|error| format!("read bridge payload '{}': {error}", path.display()))?;
    serde_json::from_slice::<RemindersPayload>(&bytes)
        .map_err(|error| format!("invalid reminders JSON '{}': {error}", path.display()))?;
    *last_modified = Some(modified);
    Ok(Some(bytes))
}

#[cfg(target_arch = "wasm32")]
fn main() {
    let path = PathBuf::from(env_var("REMINDERS_PATH").unwrap_or_else(|| "/data/reminders.json".to_string()));
    let mut last_modified = None;

    loop {
        if !channel_active() {
            sleep_secs(1);
            continue;
        }

        match read_updated_payload(&path, &mut last_modified) {
            Ok(Some(bytes)) => { channel_push(&bytes); },
            Ok(None) => {}
            Err(error) => eprintln!("reminders-sidecar: {error}"),
        }
        sleep_secs(5);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("reminders-sidecar is intended for wasm32-wasip1");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> PathBuf {
        let unique = format!(
            "lume-reminders-sidecar-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn sample_payload() -> Vec<u8> {
        serde_json::to_vec(&RemindersPayload {
            fetched_at: "2026-03-29T10:30:00Z".to_string(),
            reminders: vec![],
        })
        .unwrap()
    }

    #[test]
    fn reads_new_payload_once_until_file_changes() {
        let dir = temp_dir();
        let path = dir.join("reminders.json");
        fs::write(&path, sample_payload()).unwrap();

        let mut last_modified = None;
        let first = read_updated_payload(&path, &mut last_modified).unwrap();
        let second = read_updated_payload(&path, &mut last_modified).unwrap();

        assert!(first.is_some());
        assert!(second.is_none());
    }

    #[test]
    fn rejects_invalid_json() {
        let dir = temp_dir();
        let path = dir.join("reminders.json");
        fs::write(&path, b"not json").unwrap();

        let mut last_modified = None;
        let error = read_updated_payload(&path, &mut last_modified).unwrap_err();
        assert!(error.contains("invalid reminders JSON"));
    }
}
