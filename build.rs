use std::process::Command;

fn main() {
	let git_output = Command::new("git").args(&["rev-parse", "--short", "HEAD"]).output().unwrap();
	let git_hash = String::from_utf8(git_output.stdout).unwrap();
	println!("cargo:rustc-env=GIT_HASH={git_hash}");

	let build_ts = chrono::Utc::now();
	println!("cargo:rustc-env=BUILD_TS={build_ts:?}");
}
