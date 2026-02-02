use std::env;
use polkadot_sdk::substrate_build_script_utils::{generate_cargo_keys, rerun_if_git_head_changed};

fn main() {

    // Force the path if we know where it is and the build is failing
    if env::var("LIBCLANG_PATH").is_err() {
        let clang_14 = "/usr/lib/llvm-14/lib";
        if std::path::Path::new(clang_14).exists() {
            println!("cargo:rustc-env=LIBCLANG_PATH={}", clang_14);
            // This line is crucial: it tells cargo to tell linkers where to look
            println!("cargo:rustc-link-search=native={}", clang_14);
        }
    }

    generate_cargo_keys();

    rerun_if_git_head_changed();
}
