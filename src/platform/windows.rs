pub fn bin_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    match target_dir {
        Some(dir) => format!("{}/{}.exe", dir.trim_end_matches('/'), name),
        None => format!("./target/{profile}/{name}.exe"),
    }
}

pub fn lib_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    match target_dir {
        Some(dir) => format!("{}/{}.lib", dir.trim_end_matches('/'), name),
        None => format!("./target/{profile}/{name}.lib"),
    }
}

pub fn elf_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    match target_dir {
        Some(dir) => format!("{}/{}.exe", dir.trim_end_matches('/'), name),
        None => format!("./target/{profile}/{name}.exe"),
    }
}

pub fn efi_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    match target_dir {
        Some(dir) => format!("{}/{}.efi", dir.trim_end_matches('/'), name),
        None => format!("./target/{profile}/{name}.efi"),
    }
}

pub fn shared_lib_path(profile: &str, name: &str, target_dir: Option<&str>) -> String {
    match target_dir {
        Some(dir) => format!("{}/{}.dll", dir.trim_end_matches('/'), name),
        None => format!("./target/{profile}/{name}.dll"),
    }
}
