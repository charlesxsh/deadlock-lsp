use std::{process::{Command, Stdio, ExitStatus}, fs, ffi::OsString, env};

use serde::Deserialize;

/// Copied from Miri
/// Returns the "default sysroot" if no `--sysroot` flag is set.
/// Should be a compile-time constant.
pub fn compile_time_sysroot() -> Option<String> {
    if option_env!("RUSTC_STAGE").is_some() {
        // This is being built as part of rustc, and gets shipped with rustup.
        // We can rely on the sysroot computation in librustc.
        return None;
    }
    // For builds outside rustc, we need to ensure that we got a sysroot
    // that gets used as a default.  The sysroot computation in librustc would
    // end up somewhere in the build dir.
    // Taken from PR <https://github.com/Manishearth/rust-clippy/pull/911>.
    let home = option_env!("RUSTUP_HOME").or(option_env!("MULTIRUST_HOME"));
    let toolchain = option_env!("RUSTUP_TOOLCHAIN").or(option_env!("MULTIRUST_TOOLCHAIN"));
    Some(match (home, toolchain) {
        (Some(home), Some(toolchain)) => format!("{}/toolchains/{}", home, toolchain),
        _ => option_env!("RUST_SYSROOT")
            .expect("To build Miri without rustup, set the `RUST_SYSROOT` env var at build time")
            .to_owned(),
    })
}


#[derive(Debug, Deserialize)]
struct Cargo {
    package: CargoPackage,
}

#[derive(Debug, Deserialize)]
struct CargoPackage {
    name: String
}

fn cargo() -> Command {
    Command::new(env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo")))
}


pub fn get_analysis_cmd(dir: &str, out: &str) -> Command {
    let ws_dir = std::path::Path::new(dir);
    eprintln!("running analysis in directory: {}", dir);
    let mut crate_name = None;
    let cargo_toml_fp = ws_dir.join("Cargo.toml");
    match fs::read_to_string(&cargo_toml_fp) {
        Ok(toml_str) => {
            let decoded: Cargo = toml::from_str(&toml_str).unwrap();
            crate_name = Some(decoded.package.name)
        },
        Err(err) => {
            eprintln!("read {:?}: {}", cargo_toml_fp, err)
        },
    }

    let mut cmd = cargo();

    let dl_rustc = env::var_os("__DL_RUSTC").unwrap_or(OsString::from(""));
    cmd.env("RUSTC_WRAPPER", dl_rustc);
    cmd.env("__DL_CRATE", crate_name.unwrap_or("".to_string()));
    cmd.env("__DL_OUT", out);
    cmd.arg("build");
    cmd.current_dir(ws_dir);
    cmd.stdout(Stdio::null());
    eprintln!("{:?} in {:?}", cmd, ws_dir);

    cmd
}

pub fn run_analysis_in_dir(dir: &str, out: &str) -> ExitStatus {
    let mut cmd = get_analysis_cmd(dir, out);
    let exit_status = cmd
        .spawn()
        .expect("could not run cargo")
        .wait()
        .expect("failed to wait for cargo?");
    if !exit_status.success() {
        eprintln!("cargo error code: {}", exit_status.code().unwrap_or(-1));
    };

    exit_status
}

pub fn cargo_clean(cwd: &str) -> ExitStatus {
    let mut cmd = cargo();
    cmd.arg("clean");
    cmd.current_dir(cwd);
    let exit_status = cmd
        .spawn()
        .expect("could not run cargo")
        .wait()
        .expect("failed to wait for cargo?");
    if !exit_status.success() {
        eprintln!("cargo error code: {}", exit_status.code().unwrap_or(-1));
    };

    exit_status
}

#[cfg(test)]
mod tests {

    use std::error::Error;

    use super::*;

    #[test]
    fn test_compile_time_sysroot() {
        let res = compile_time_sysroot();
        assert!(res.is_some());
    }


    #[test]
    fn test_get_analysis_cmd() {
        let cmd = get_analysis_cmd("123", "345");

        assert_eq!(cmd.get_current_dir().unwrap().to_str().unwrap(), "123");
        let res = cmd.get_envs().find(|x| x.0 == "__DL_OUT");
        assert_eq!(res.unwrap().1.unwrap().to_str().unwrap(), "345");

    }


    #[test]
    fn test_run_analysis_in_dir() -> Result<(),Box<dyn Error>> {
        let repo = ".tmp/fake_repo";
        fs::create_dir_all(repo)?;

        let res = run_analysis_in_dir(repo, "not existed output");
        assert!(res.success());

        Ok(())
    }


    

}