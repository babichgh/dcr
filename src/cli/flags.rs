use crate::config::{PROFILE, flags};
use crate::utils::log::warn;

pub struct BuildRunFlags {
    pub profile: String,
    pub target: Option<String>,
    pub force: bool,
    pub clean: bool,
}

pub fn parse_build_run_flags(args: &[String]) -> Result<BuildRunFlags, i32> {
    let mut profile = PROFILE.to_string();
    let mut target = None;
    let mut force = false;
    let mut clean = false;
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        if !arg.starts_with("--") {
            warn("Unknown argument");
            return Err(1);
        }
        let candidate = arg.trim_start_matches("--");
        if candidate == "force" {
            force = true;
            continue;
        }
        if candidate == "clean" {
            clean = true;
            continue;
        }
        if candidate == "target" {
            if let Some(t) = iter.next() {
                target = Some(t.clone());
            } else {
                warn("--target requires a value");
                return Err(1);
            }
            continue;
        }
        if flags(candidate).is_some() {
            if profile != PROFILE {
                warn("Duplicate profile flag");
                return Err(1);
            }
            profile = candidate.to_string();
            continue;
        }
        warn("Unknown build flag");
        return Err(1);
    }

    Ok(BuildRunFlags {
        profile,
        target,
        force,
        clean,
    })
}
