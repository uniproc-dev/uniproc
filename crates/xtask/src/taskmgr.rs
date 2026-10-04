use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use windows_core::{HSTRING, PWSTR};

use crate::bindings::{
    FreeLibrary, HINSTANCE, HMODULE, LOAD_LIBRARY_AS_DATAFILE, LOAD_LIBRARY_AS_IMAGE_RESOURCE,
    LoadLibraryExW, LoadStringW,
};

const MUI: &str = "Taskmgr.exe.mui";
const REFERENCE_LOCALE: &str = "en-US";
const OUTPUT_FILE: &str = "taskmgr.ftl";

struct Entry {
    key: &'static str,
    id: u32,
    english: &'static str,
}

const fn entry(key: &'static str, id: u32, english: &'static str) -> Entry {
    Entry { key, id, english }
}

const ENTRIES: &[Entry] = &[
    entry("shell-nav-processes", 32403, "Processes"),
    entry("shell-nav-services", 32401, "Services"),
    entry("processes-title", 32403, "Processes"),
    entry("processes-end-task", 38001, "End task"),
    entry("processes-col-name", 37001, "Name"),
    entry("processes-col-cpu", 37008, "CPU"),
    entry("processes-col-memory", 37009, "Memory"),
    entry("processes-col-disk", 37010, "Disk"),
    entry("processes-col-net", 37011, "Network"),
    entry("processes-col-pid", 37002, "PID"),
    entry("processes-col-status", 37004, "Status"),
    entry("processes-col-publisher", 37005, "Publisher"),
    entry("processes-col-process-name", 37006, "Process name"),
    entry("processes-col-command-line", 37007, "Command line"),
    entry("processes-col-user", 32005, "User name"),
    entry("processes-col-image-path", 32032, "Image path name"),
    entry("processes-col-platform", 32034, "Platform"),
    entry("processes-col-elevated", 32036, "Elevated"),
    entry("processes-col-gpu", 41017, "GPU"),
    entry("processes-col-gpu-engine", 41018, "GPU engine"),
    entry("processes-col-gpu-memory", 41034, "Dedicated GPU memory"),
    entry("processes-col-isolation", 48368, "Isolation"),
    entry("processes-status-suspended", 36005, "Suspended"),
    entry("processes-status-efficiency", 36016, "Efficiency mode"),
    entry("processes-elevated-yes", 32280, "Yes"),
    entry("processes-elevated-no", 32281, "No"),
    entry("processes-isolation-app-container", 48384, "AppContainer"),
    entry("processes-isolation-uwp", 48375, "UWP"),
    entry("processes-isolation-silo", 48374, "Silo"),
    entry("processes-category-app", 37153, "Apps"),
    entry("processes-category-background-third-party", 37154, "Background processes"),
    entry("services-title", 32401, "Services"),
    entry("services-restart", 38002, "Restart"),
    entry("services-col-name", 32046, "Name"),
    entry("services-col-pid", 32004, "PID"),
    entry("services-col-status", 32035, "Status"),
    entry("services-col-description", 32038, "Description"),
    entry("services-col-group", 32047, "Group"),
    entry("services-state-continue-pending", 32200, "Continuing"),
    entry("services-state-paused", 32201, "Paused"),
    entry("services-state-pause-pending", 32202, "Pausing"),
    entry("services-state-running", 32203, "Running"),
    entry("services-state-start-pending", 32204, "Starting"),
    entry("services-state-stopped", 32205, "Stopped"),
    entry("services-state-stop-pending", 32206, "Stopping"),
];

struct StringTable(HMODULE);

impl StringTable {
    fn open(path: &Path) -> anyhow::Result<Self> {
        let wide = HSTRING::from(path.as_os_str());
        let module = unsafe {
            LoadLibraryExW(
                &wide,
                None,
                (LOAD_LIBRARY_AS_DATAFILE | LOAD_LIBRARY_AS_IMAGE_RESOURCE) as u32,
            )
        };
        if module.0.is_null() {
            return Err(std::io::Error::last_os_error())
                .with_context(|| format!("cannot load {}", path.display()));
        }
        Ok(Self(module))
    }

    fn get(&self, id: u32) -> Option<String> {
        let mut buffer = [0u16; 1024];
        let len = unsafe {
            LoadStringW(
                Some(HINSTANCE(self.0.0)),
                id,
                PWSTR(buffer.as_mut_ptr()),
                buffer.len() as i32,
            )
        };
        (len > 0).then(|| strip_accelerator(&String::from_utf16_lossy(&buffer[..len as usize])))
    }
}

impl Drop for StringTable {
    fn drop(&mut self) {
        unsafe {
            let _ = FreeLibrary(self.0);
        }
    }
}

fn strip_accelerator(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '&' {
            if chars.peek() == Some(&'&') {
                out.push('&');
                chars.next();
            }
            continue;
        }
        out.push(c);
    }
    out.trim().to_string()
}

fn fluent_value(text: &str) -> String {
    text.replace('{', "{\"{\"}").replace('}', "{\"}\"}")
}

fn merge(existing: &str, generated: &[(&str, String)]) -> String {
    let mut written = vec![false; generated.len()];
    let mut out = String::new();
    let mut replacing = false;
    for line in existing.lines() {
        if replacing && line.starts_with(char::is_whitespace) && !line.trim().is_empty() {
            continue;
        }
        replacing = false;
        let key = line.split_once('=').map(|(key, _)| key.trim());
        match key.and_then(|key| generated.iter().position(|(k, _)| *k == key)) {
            Some(index) => {
                let (key, value) = &generated[index];
                out.push_str(&format!("{key} = {value}\n"));
                written[index] = true;
                replacing = true;
            }
            None => {
                out.push_str(line);
                out.push('\n');
            }
        }
    }
    for ((key, value), written) in generated.iter().zip(written) {
        if !written {
            out.push_str(&format!("{key} = {value}\n"));
        }
    }
    out
}

fn locale_dir(windows_locale: &str) -> &str {
    if windows_locale == REFERENCE_LOCALE {
        "en"
    } else {
        windows_locale
    }
}

fn mui_locales(root: &Path) -> anyhow::Result<Vec<String>> {
    let mut locales: Vec<String> = fs::read_dir(root)
        .with_context(|| format!("cannot list {}", root.display()))?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().join(MUI).is_file())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();
    locales.sort();
    Ok(locales)
}

fn verify_reference(root: &Path) -> anyhow::Result<()> {
    let reference = StringTable::open(&root.join(REFERENCE_LOCALE).join(MUI))?;
    let mismatched: Vec<String> = ENTRIES
        .iter()
        .filter_map(|entry| {
            let found = reference.get(entry.id);
            (found.as_deref() != Some(entry.english)).then(|| {
                format!(
                    "{} (id {}): expected {:?}, Taskmgr has {:?}",
                    entry.key, entry.id, entry.english, found
                )
            })
        })
        .collect();
    if !mismatched.is_empty() {
        bail!(
            "Taskmgr string ids moved in this Windows build:\n  {}",
            mismatched.join("\n  ")
        );
    }
    Ok(())
}

pub fn run(args: &[String], workspace_root: &Path) -> anyhow::Result<()> {
    let root = match args.first() {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(std::env::var("SystemRoot").context("SystemRoot not set")?)
            .join("System32"),
    };

    verify_reference(&root)?;

    let locales = mui_locales(&root)?;
    if locales.is_empty() {
        bail!("no <locale>\\{MUI} under {}", root.display());
    }

    for locale in &locales {
        let table = StringTable::open(&root.join(locale).join(MUI))?;
        let mut generated = Vec::new();
        let mut missing = Vec::new();
        for entry in ENTRIES {
            match table.get(entry.id) {
                Some(text) => generated.push((entry.key, fluent_value(&text))),
                None => missing.push(entry.key),
            }
        }

        let dir = workspace_root.join("locales").join(locale_dir(locale));
        fs::create_dir_all(&dir)?;
        let target = dir.join(OUTPUT_FILE);
        let existing = match fs::read_to_string(&target) {
            Ok(text) => text,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(err) => return Err(err).with_context(|| format!("cannot read {}", target.display())),
        };
        fs::write(&target, merge(&existing, &generated))
            .with_context(|| format!("cannot write {}", target.display()))?;

        if missing.is_empty() {
            println!("{locale}: {} strings -> {}", ENTRIES.len(), target.display());
        } else {
            println!(
                "{locale}: {} strings -> {}, missing: {}",
                ENTRIES.len() - missing.len(),
                target.display(),
                missing.join(", ")
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_written_by_hand_survive_a_regeneration() {
        let existing = "processes-title = Processes\nprocesses-run-new-task = Run new task\nprocesses-end-task = End task\n";
        let generated = [
            ("processes-title", "Процессы".to_string()),
            ("processes-end-task", "Снять задачу".to_string()),
        ];

        assert_eq!(
            merge(existing, &generated),
            "processes-title = Процессы\nprocesses-run-new-task = Run new task\nprocesses-end-task = Снять задачу\n"
        );
    }

    #[test]
    fn a_new_generated_key_goes_to_the_end() {
        let generated = [("services-title", "Services".to_string())];

        assert_eq!(
            merge("processes-col-pid = PID\n", &generated),
            "processes-col-pid = PID\nservices-title = Services\n"
        );
    }

    #[test]
    fn a_replaced_multiline_value_leaves_no_tail() {
        let existing = "services-title =\n    Old\n    value\nservices-col-pid = PID\n";
        let generated = [("services-title", "Services".to_string())];

        assert_eq!(
            merge(existing, &generated),
            "services-title = Services\nservices-col-pid = PID\n"
        );
    }
}
