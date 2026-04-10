use crate::cli::build;
use crate::config::{FILE_DCR_TEST_H, FILE_TEST_C, flags};
use crate::core::config::Config;
use crate::utils::fs::find_project_root;
use crate::utils::log::error;
use crate::utils::text::{BOLD_GREEN, BOLD_RED, RESET, colored};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

const BOLD_BLUE: &str = "\x1b[1m\x1b[94m";

pub fn test(args: &[String]) -> i32 {
    let mut init_header = false;
    for arg in args {
        if arg == "--help" {
            println!("USAGE:");
            println!("    dcr test [--init]");
            println!();
            println!("ALIASES:");
            println!("    dcr tests");
            println!();
            println!("DESCRIPTION:");
            println!("    Runs project tests and prints a unified testsuite report.");
            println!();
            println!("OPTIONS:");
            println!("    --init            Create tests/dcr_test.h in current project");
            return 0;
        }
        if arg == "--init" {
            init_header = true;
            continue;
        }
        error("Unknown argument");
        return 1;
    }

    let start_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => {
            error("Failed to determine current directory");
            return 1;
        }
    };
    let root = match find_project_root(&start_dir) {
        Ok(Some(dir)) => dir,
        Ok(None) => {
            error("dcr.toml file not found");
            return 1;
        }
        Err(_) => {
            error("Failed to find project root");
            return 1;
        }
    };

    if init_header {
        match with_dir(&root, ensure_test_header) {
            Ok(()) => {}
            Err(msg) => {
                error(&msg);
                return 1;
            }
        }
    }

    match with_dir(&root, run_testsuite) {
        Ok(code) => code,
        Err(msg) => {
            error(&msg);
            1
        }
    }
}

fn run_testsuite() -> Result<i32, String> {
    if build::build(&[String::from("--release")]) != 0 {
        return Ok(1);
    }

    let config = Config::open("./dcr.toml").map_err(|_| "Failed to read dcr.toml".to_string())?;
    let name = config.get("package.name").and_then(|v| v.as_str()).ok_or("package.name not found in dcr.toml")?;
    let kind = config.get("package.kind").and_then(|v| v.as_str()).unwrap_or("bin");

    let compiler = env::var("DCR_CC").unwrap_or_else(|_| "cc".to_string());
    let profile = "release";
    let flags_vec = flags(profile).unwrap_or(&[]);
    let flags_str = flags_vec.join(" ");

    let test_c = "./tests/test.c";
    if !Path::new(test_c).exists() {
        return Err("tests/test.c not found; run 'dcr test --init' first".to_string());
    }

    // Compile test.c to object
    let obj_path = "./tests/test.o";
    let compile_cmd = format!("{} -c {} -o {} {}", compiler, test_c, obj_path, flags_str);
    let compile_status = Command::new("sh")
        .arg("-c")
        .arg(&compile_cmd)
        .status()
        .map_err(|_| format!("Failed to compile {}", test_c))?;
    if !compile_status.success() {
        return Err(format!("Compilation of {} failed", test_c));
    }

    // Link
    let bin_path = format!("./tests/test{}", std::env::consts::EXE_SUFFIX);
    let link_cmd = if kind == "lib" {
        format!("{} {} -o {} -Ltarget/release -l{}", compiler, obj_path, bin_path, name)
    } else {
        format!("{} {} -o {}", compiler, obj_path, bin_path)
    };
    let link_status = Command::new("sh")
        .arg("-c")
        .arg(&link_cmd)
        .status()
        .map_err(|_| format!("Failed to link {}", bin_path))?;
    if !link_status.success() {
        return Err(format!("Linking of {} failed", bin_path));
    }

    let out = Command::new(&bin_path)
        .output()
        .map_err(|_| format!("Failed to run `{bin_path}`"))?;

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    if !stderr.trim().is_empty() {
        eprint!("{}", stderr);
    }

    let declared = extract_test_names("./tests/test.c");
    let mut pass = 0;
    let mut skip = 0;
    let mut fail = 0;
    let mut parsed_any = false;

    for line in stdout.lines() {
        let line = line.trim();
        if let Some(name) = line.strip_prefix("[PASS] ") {
            println!("{} {}", colored("[PASS]", BOLD_GREEN), name);
            pass += 1;
            parsed_any = true;
            continue;
        }
        if let Some(name) = line.strip_prefix("[SKIP] ") {
            println!("{} {}", colored("[SKIP]", BOLD_BLUE), name);
            skip += 1;
            parsed_any = true;
            continue;
        }
        if let Some(name) = line.strip_prefix("[FAIL] ") {
            println!("{} {}", colored("[FAIL]", BOLD_RED), name);
            fail += 1;
            parsed_any = true;
            continue;
        }
        if let Some((status, name)) = line.split_once('\t') {
            match status {
                "PASS" => {
                    println!("{} {}", colored("[PASS]", BOLD_GREEN), name);
                    pass += 1;
                    parsed_any = true;
                }
                "SKIP" => {
                    println!("{} {}", colored("[SKIP]", BOLD_BLUE), name);
                    skip += 1;
                    parsed_any = true;
                }
                "FAIL" => {
                    println!("{} {}", colored("[FAIL]", BOLD_RED), name);
                    fail += 1;
                    parsed_any = true;
                }
                _ => {}
            }
        }
    }

    if !parsed_any {
        if out.status.success() {
            for name in &declared {
                println!("{} {}", colored("[PASS]", BOLD_GREEN), name);
            }
            pass = declared.len() as i32;
        } else {
            println!("{} testsuite", colored("[FAIL]", BOLD_RED));
            fail = 1;
        }
    }

    let total = if parsed_any || declared.is_empty() {
        pass + skip + fail
    } else {
        declared.len() as i32
    };

    println!();
    println!("{}", colored("=====================", BOLD_GREEN));
    println!("{}", colored("  Testsuite summary  ", BOLD_GREEN));
    println!("{}", colored("=====================", BOLD_GREEN));
    println!("TOTAL: {}", total);
    print_field("PASS", pass, BOLD_GREEN);
    print_field("SKIP", skip, BOLD_BLUE);
    print_field("FAIL", fail, BOLD_RED);
    println!("{}", colored("=====================", BOLD_GREEN));

    if fail > 0 || !out.status.success() {
        return Ok(1);
    }
    Ok(0)
}

fn ensure_test_header() -> Result<(), String> {
    fs::create_dir_all("./tests").map_err(|_| "Failed to create tests/".to_string())?;
    fs::write("./tests/dcr_test.h", FILE_DCR_TEST_H)
        .map_err(|_| "Failed to write tests/dcr_test.h".to_string())?;
    println!("Created tests/dcr_test.h");
    fs::write("./tests/test.c", FILE_TEST_C)
        .map_err(|_| "Failed to write tests/test.c".to_string())?;
    println!("Created tests/test.c");
    Ok(())
}

fn extract_test_names(path: &str) -> Vec<String> {
    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut names = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("TEST_CASE(")
            && let Some(name) = rest.strip_suffix("),")
        {
            names.push(name.to_string());
        }
    }
    names
}

fn print_field(label: &str, value: i32, color: &str) {
    if value == 0 {
        println!("{}:  {}", label, value);
    } else {
        println!("{}{}:  {}{}", color, label, value, RESET);
    }
}

fn with_dir<F, T>(dir: &Path, f: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String>,
{
    let prev = std::env::current_dir().map_err(|_| "Failed to get current dir".to_string())?;
    std::env::set_current_dir(dir).map_err(|_| "Failed to change directory".to_string())?;
    let result = f();
    let _ = std::env::set_current_dir(prev);
    result
}
