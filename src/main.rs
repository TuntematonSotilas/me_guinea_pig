use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    if should_launch_android(&args) {
        if let Err(err) = run_android_debug() {
            eprintln!("failed to launch Android debug build: {err}");
            std::process::exit(1);
        }
        return;
    }

    me_guinea_pig::main();
}

fn should_launch_android(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "android")
}

fn run_android_debug() -> Result<(), String> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let script_path = PathBuf::from(manifest_dir).join("scripts/run-android.sh");

    let status = Command::new("bash")
        .arg(&script_path)
        .current_dir(manifest_dir)
        .status()
        .map_err(|err| format!("failed to execute {}: {err}", script_path.display()))?;

    if !status.success() {
        return Err(format!("Android debug launcher exited with status {status}"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::should_launch_android;

    #[test]
    fn detects_android_launch_argument() {
        assert!(should_launch_android(&["me_guinea_pig".to_string(), "android".to_string()]));
        assert!(!should_launch_android(&["me_guinea_pig".to_string()]));
    }
}
