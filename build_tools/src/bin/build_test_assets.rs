use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    let cwd = env::current_dir().expect("Failed to get current working directory");
    let testbin_crate_dir = cwd.join("testbin");

    let targets = vec![
        "aarch64-pc-windows-msvc",
        "arm64ec-pc-windows-msvc",
        "i686-pc-windows-msvc",
        "x86_64-pc-windows-msvc",
    ];
    for target in targets {
        let build_output = Command::new("cargo")
            .arg("build")
            .current_dir(&testbin_crate_dir)
            .arg("--release")
            .arg("--target")
            .arg(target)
            .output()
            .expect("Failed to execute cargo build for testbin");

        if !build_output.status.success() {
            panic!(
                "Failed to build helper-crate. Exit code: {:?}\nOutput:\nstderr: {}\nstdout: {}",
                build_output.status.code(),
                String::from_utf8_lossy(&build_output.stderr),
                String::from_utf8_lossy(&build_output.stdout)
            );
        }

        let artifact_path = testbin_crate_dir
            .join("target")
            .join(target)
            .join("release")
            .join("testbin.exe");

        if !artifact_path.exists() {
            panic!(
                "Helper crate artifact not found at: {}",
                artifact_path.display()
            );
        }
        fs::copy(
            artifact_path,
            cwd.join("test_assets")
                .join(format!("testbin_{}.exe", target)),
        )
        .unwrap();
    }
}
