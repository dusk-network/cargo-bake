use clap::{App, AppSettings, Arg, SubCommand};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use bytehash::{Blake2b, ByteHash};
use dusk_abi::H256;

const COMMAND_NAME: &str = "bake";
const COMMAND_DESCRIPTION: &str =
    "A third-party cargo extension which bake your Rusk Smart Contracts made with love & cake!";
const COMMAND_AUTHOR: &str = "zer0 <matteo@dusk-network.com>";

fn get_target_path(manifest_path: &str, is_debug_build: bool) -> PathBuf {
    let mut path = PathBuf::from("target/wasm32-unknown-unknown");

    if is_debug_build {
        path.push("debug");
    } else {
        path.push("release");
    }

    let mut cmd_meta = cargo_metadata::MetadataCommand::new();

    cmd_meta.manifest_path(manifest_path);
    let _metadata = cmd_meta.no_deps().exec().unwrap();

    let package_name = &_metadata.packages[0].name;

    path.push(package_name.to_owned().replace("-", "_") + ".wasm");

    assert_eq!(
        path.exists(),
        true,
        "{} does not exists",
        path.to_string_lossy()
    );
    path
}

fn hash<H: ByteHash>(contract_path: &Path) -> H256 {
    let mut file = File::open(contract_path).unwrap();
    let mut contract_code = vec![];
    file.read_to_end(&mut contract_code).unwrap();

    H256::from_bytes(H::hash(&contract_code).as_ref())
}

fn main() {
    let matches = App::new(format!("cargo-{}", COMMAND_NAME))
        .about(COMMAND_DESCRIPTION)
        .author(COMMAND_AUTHOR)
        .version(clap::crate_version!())
        // We have to lie about our binary name since this will be a third party
        // subcommand for cargo, this trick learned from cargo-outdated
        .bin_name("cargo")
        // We use a subcommand because parsed after `cargo` is sent to the third party plugin
        // which will be interpreted as a subcommand/positional arg by clap
        .subcommand(
            SubCommand::with_name(COMMAND_NAME)
                .about(COMMAND_DESCRIPTION)
                .arg(
                    Arg::with_name("debug")
                        .long("debug")
                        .help("Bakes in debug mode"),
                )
                .arg(
                    Arg::with_name("manifest-path")
                        .long("manifest-path")
                        .value_name("PATH")
                        .takes_value(true)
                        .help("Path to Cargo.toml"),
                )
                .arg(
                    Arg::with_name("color")
                        .long("color")
                        .value_name("WHEN")
                        .takes_value(true)
                        .possible_values(&["auto", "always", "never"])
                        .default_value("auto")
                        .hide_possible_values(true)
                        .help("Coloring: auto, always, never"),
                ),
        )
        .settings(&[AppSettings::SubcommandRequired])
        .get_matches();

    let matches = matches.subcommand_matches(COMMAND_NAME).unwrap();

    let manifest_path = matches.value_of("manifest-path").unwrap_or("./Cargo.toml");
    let is_debug_build = matches.is_present("debug");

    let mut color = matches.value_of("color").unwrap();
    if color == "auto" {
        color = if atty::is(atty::Stream::Stdout) {
            "always"
        } else {
            "never"
        }
    }

    let mut cmd = Command::new("cargo");
    cmd.arg("rustc");

    if !is_debug_build {
        cmd.arg("--release");
    }

    cmd.args(&["--color", color])
        .args(&["--manifest-path", manifest_path])
        .args(&["--target", "wasm32-unknown-unknown"])
        .arg("--")
        .args(&["-C", "link-args=-s"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let mut proc = cmd.spawn().expect("failed to execute process");
    let status = proc.wait();

    let target_path = get_target_path(manifest_path, is_debug_build);
    println!("Exited with status {:?}", status);
    println!("target: {:?}", target_path);
    println!(
        "contract hash: {:?}",
        hash::<Blake2b>(target_path.as_path())
    );
}
