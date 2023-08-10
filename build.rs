use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs, io};

#[cfg(feature = "bundled")]
fn init_submodule(path: &Path) {
    if !path.join("Makefile.am").exists() {
        let r = Command::new("git")
            .args(&["clone", "https://github.com/open-mpi/hwloc"])
            .current_dir(path.clone())
            .output();
        match r {
            Ok(m) => {}
            Err(e) => println!("Error {}", e),
        }
    }
}

#[cfg(feature = "bundled")]
fn get_os_from_triple(triple: &str) -> Option<&str> {
    triple.splitn(3, "-").nth(2)
}

#[cfg(feature = "bundled")]
fn compile_hwloc2(build_path: &Path, target_os: &str) -> PathBuf {
    let mut config = autotools::Config::new("hwloc");
    config.reconf("-ivf").make_target("install").build()
}

fn main() {
    let required_version = if cfg!(feature = "hwloc-2_8_0") {
        "2.8.0"
    } else if cfg!(feature = "hwloc-2_5_0") {
        "2.5.0"
    } else if cfg!(feature = "hwloc-2_4_0") {
        "2.4.0"
    } else if cfg!(feature = "hwloc-2_3_0") {
        "2.3.0"
    } else if cfg!(feature = "hwloc-2_2_0") {
        "2.2.0"
    } else if cfg!(feature = "hwloc-2_1_0") {
        "2.1.0"
    } else if cfg!(feature = "hwloc-2_0_4") {
        "2.0.4"
    } else {
        "2.0.0"
    };
    #[cfg(feature = "bundled")]
    {
        let target = env::var("TARGET").expect("Cargo build scripts always have TARGET");
        let target_os = get_os_from_triple(target.as_str()).unwrap();
        let hwloc2_source_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        init_submodule(hwloc2_source_path.as_path());
        let mut hwloc2_compiled_path: PathBuf =
            compile_hwloc2(hwloc2_source_path.as_path(), target_os);
        hwloc2_compiled_path.push("lib");
        println!(
            "dummy: {}",
            hwloc2_compiled_path
                .to_str()
                .expect("Link path is not an UTF-8 string")
        );
        println!(
            "cargo:rustc-link-arg=-Wl,-rpath,{}",
            hwloc2_compiled_path
                .to_str()
                .expect("Link path is not an UTF-8 string")
        );
        println!(
            "cargo:rustc-link-search={}",
            hwloc2_compiled_path
                .to_str()
                .expect("Link path is not an UTF-8 string")
        );
        println!("cargo:rustc-link-lib=static=hwloc");
        println!("cargo:rustc-link-lib=xml2");
        println!("cargo:rustc-link-lib=pciaccess");
        println!("cargo:rustc-link-lib=udev");
    }
    #[cfg(not(feature = "bundled"))]
    {
        let lib = pkg_config::Config::new()
            .atleast_version(required_version)
            .statik(true)
            .probe("hwloc")
            .expect("Could not find a suitable version of hwloc");
        if cfg!(target_family = "unix") {
            for link_path in lib.link_paths {
                println!(
                    "cargo:rustc-link-arg=-Wl,-rpath,{}",
                    link_path
                        .to_str()
                        .expect("Link path is not an UTF-8 string")
                );
            }
        }
    }
}
