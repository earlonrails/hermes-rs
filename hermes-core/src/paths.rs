use std::env;
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};

static PROFILE_FALLBACK_WARNED: AtomicBool = AtomicBool::new(false);

/// Return the Hermes home directory (default: ~/.hermes).
pub fn get_hermes_home() -> PathBuf {
    if let Ok(val) = env::var("HERMES_HOME") {
        let trimmed = val.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    // Guard: if a non-default profile is sticky-active, warn once.
    if !PROFILE_FALLBACK_WARNED.load(Ordering::Relaxed) {
        if let Some(home) = dirs::home_dir() {
            let active_path = home.join(".hermes").join("active_profile");
            if let Ok(active) = fs::read_to_string(&active_path) {
                let active = active.trim();
                if !active.is_empty() && active != "default" {
                    PROFILE_FALLBACK_WARNED.store(true, Ordering::Relaxed);
                    eprintln!(
                        "[HERMES_HOME fallback] HERMES_HOME is unset but active \
                        profile is {:?}. Falling back to ~/.hermes, which \
                        is the DEFAULT profile — not {:?}. Any data this \
                        process writes will land in the wrong profile. The \
                        subprocess spawner should pass HERMES_HOME explicitly \
                        (see issue #18594).",
                        active, active
                    );
                }
            }
        }
    }

    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".hermes")
}

pub fn get_default_hermes_root() -> PathBuf {
    let native_home = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".hermes");
        
    let env_home = env::var("HERMES_HOME").unwrap_or_default();
    if env_home.is_empty() {
        return native_home;
    }
    
    let env_path = PathBuf::from(env_home);
    if let Ok(env_resolved) = env_path.canonicalize() {
        if let Ok(native_resolved) = native_home.canonicalize() {
            if env_resolved.starts_with(&native_resolved) {
                return native_home;
            }
        }
    }

    if let Some(parent) = env_path.parent() {
        if parent.file_name().and_then(|n| n.to_str()) == Some("profiles") {
            if let Some(grandparent) = parent.parent() {
                return grandparent.to_path_buf();
            }
        }
    }

    env_path
}

pub fn get_optional_skills_dir(default: Option<PathBuf>) -> PathBuf {
    if let Ok(override_val) = env::var("HERMES_OPTIONAL_SKILLS") {
        let trimmed = override_val.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    if let Some(def) = default {
        return def;
    }
    get_hermes_home().join("optional-skills")
}

pub fn get_hermes_dir(new_subpath: &str, old_name: &str) -> PathBuf {
    let home = get_hermes_home();
    let old_path = home.join(old_name);
    if old_path.exists() {
        old_path
    } else {
        home.join(new_subpath)
    }
}

pub fn display_hermes_home() -> String {
    let home = get_hermes_home();
    if let Some(user_home) = dirs::home_dir() {
        if let Ok(stripped) = home.strip_prefix(&user_home) {
            let mut s = String::from("~/");
            s.push_str(&stripped.to_string_lossy());
            return s;
        }
    }
    home.to_string_lossy().to_string()
}

pub fn get_subprocess_home() -> Option<PathBuf> {
    env::var("HERMES_HOME").ok().and_then(|h| {
        let profile_home = PathBuf::from(h).join("home");
        if profile_home.is_dir() {
            Some(profile_home)
        } else {
            None
        }
    })
}

// ─── Environment Helpers ───────────────────────────────────────────────────

pub fn is_termux() -> bool {
    let prefix = env::var("PREFIX").unwrap_or_default();
    env::var("TERMUX_VERSION").is_ok() || prefix.contains("com.termux/files/usr")
}

pub fn is_wsl() -> bool {
    if let Ok(content) = fs::read_to_string("/proc/version") {
        content.to_lowercase().contains("microsoft")
    } else {
        false
    }
}

pub fn is_container() -> bool {
    if Path::new("/.dockerenv").exists() || Path::new("/run/.containerenv").exists() {
        return true;
    }
    if let Ok(content) = fs::read_to_string("/proc/1/cgroup") {
        if content.contains("docker") || content.contains("podman") || content.contains("/lxc/") {
            return true;
        }
    }
    false
}

// ─── Well-Known Paths ──────────────────────────────────────────────────────

pub fn get_config_path() -> PathBuf {
    get_hermes_home().join("config.yaml")
}

pub fn get_skills_dir() -> PathBuf {
    get_hermes_home().join("skills")
}

pub fn get_env_path() -> PathBuf {
    get_hermes_home().join(".env")
}

pub const OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";
pub const OPENROUTER_MODELS_URL: &str = "https://openrouter.ai/api/v1/models";
pub const AI_GATEWAY_BASE_URL: &str = "https://ai-gateway.vercel.sh/v1";
