// DCR — Cargo-like C/C++ project manager.
//
// Copyright (C) 2026 Dexoron (Bezotechestvo Vladimir) <main@dexoron.su>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::config::{PROFILE, flags};
use crate::utils::log::warn;

pub struct BuildRunFlags {
    pub profile: String,
    pub target: Option<String>,
    pub workspace: Option<String>,
    pub force: bool,
    pub clean: bool,
    pub verbose: bool,
}

pub fn parse_build_run_flags(args: &[String]) -> Result<BuildRunFlags, i32> {
    let mut profile = PROFILE.to_string();
    let mut target = None;
    let mut workspace = None;
    let mut force = false;
    let mut clean = false;
    let mut verbose = false;
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
        if candidate == "verbose" {
            verbose = true;
            continue;
        }
        if candidate == "workspace" {
            if let Some(w) = iter.next() {
                workspace = Some(w.clone());
            } else {
                warn("--workspace requires a value");
                return Err(1);
            }
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
        workspace,
        force,
        clean,
        verbose,
    })
}
