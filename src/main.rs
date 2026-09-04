use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    if should_launch_android(&args) {
        let result = if args.iter().any(|arg| arg == "android-build") {
            run_android_build()
        } else {
            run_android_debug()
        };

        if let Err(err) = result {
            eprintln!("failed to build Android application: {err}");
            std::process::exit(1);
        }
        return;
    }

    me_guinea_pig::main();
}

fn should_launch_android(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "android" || arg == "android-build")
}

fn run_android_debug() -> Result<(), String> {
    run_android_script(&[])
}

fn run_android_build() -> Result<(), String> {
    run_android_script(&["--release"])
}

fn run_android_script(arguments: &[&str]) -> Result<(), String> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let script_path = PathBuf::from(manifest_dir).join("scripts/run-android.sh");

    let status = Command::new("bash")
        .arg(&script_path)
        .args(arguments)
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
        assert!(should_launch_android(&[
            "me_guinea_pig".to_string(),
            "android-build".to_string(),
        ]));
        assert!(!should_launch_android(&["me_guinea_pig".to_string()]));
    }
}
