use std::{
    fs::{self},
    io,
    path::{Path, PathBuf},
    process::{Child, Command},
};

use config::Config;

mod config;

const RUN_ARGS: &[&str] = &["-s", "-serial", "stdio", "-m", "256M", "-hdb", "fat.img"];
const DEBUG_ARGS: &[&str] = &["-s", "-S", "-monitor", "stdio", "-hdb", "initrd.img"];
pub fn main() {
    let mut args = std::env::args().skip(1); // skip executable name

    let kernel_binary_path = {
        let path = PathBuf::from(args.next().unwrap());
        path.canonicalize().unwrap()
    };

    let manifest = {
        let mut path = PathBuf::from(args.next().unwrap());
        path = path.canonicalize().unwrap();

        match fs::read_to_string(&path) {
            Ok(text) => Some(text),
            Err(err) => {
                println!("Failed To Open File '{}' ({})", (&path).display(), err);
                None
            }
        }
    };

    let no_boot = if let Some(arg) = args.next() {
        match arg.as_str() {
            "--no-run" => true,
            _ => false,
        }
    } else {
        false
    };

    let debug = if let Some(arg) = args.next() {
        match arg.as_str() {
            "--gdb" => true,
            "--debug" => true,
            _ => false,
        }
    } else {
        false
    };

    let bios = build_image(&kernel_binary_path);

    if no_boot {
        println!("Created disk image at `{}`", bios.display());
        return;
    }

    let mut run_cmd = Command::new("qemu-system-x86_64");
    run_cmd
        .arg("-drive")
        .arg(format!("format=raw,file={}", bios.display()));

    if !debug {
        run_cmd.args(RUN_ARGS);
    } else {
        run_cmd.args(DEBUG_ARGS);
    }

    if let Some(cfg) = manifest {
        run_from_cfg(&cfg);
    } else {
        run_cmd.status().expect("Failed To Launch Qemu...");
    }
}

fn build_image(kernel_bin: &Path) -> PathBuf {
    let kernel_manifest = locate_cargo_manifest::locate_manifest().unwrap();
    let bootloader_manifest = bootloader_locator::locate_bootloader("bootloader").unwrap();

    let mut build_cmd = Command::new(env!("CARGO"));
    build_cmd.current_dir(bootloader_manifest.parent().unwrap());
    build_cmd.arg("builder");
    build_cmd.arg("--kernel-manifest").arg(&kernel_manifest);
    build_cmd.arg("--kernel-binary").arg(&kernel_bin);
    build_cmd
        .arg("--target-dir")
        .arg(kernel_manifest.parent().unwrap().join("target"));
    build_cmd.arg("--out-dir").arg(kernel_bin.parent().unwrap());
    build_cmd.arg("--quiet");

    if !build_cmd.status().unwrap().success() {
        panic!("build failed");
    } else {
        println!("Built {}", kernel_bin.display())
    }

    let kernel_binary_name = kernel_bin.file_name().unwrap().to_str().unwrap();
    let disk_image = kernel_bin
        .parent()
        .unwrap()
        .join(format!("boot-bios-{}.img", kernel_binary_name));
    if !disk_image.exists() {
        panic!(
            "Disk image does not exist at {} after bootloader build",
            disk_image.display()
        );
    }
    disk_image
}


pub fn run_from_cfg(cfg: &str) -> Option<io::Result<Child>> {
    if let Ok(config) = &Config::from(cfg) {
        let mut cmd = Command::new(config.runner.clone());
        cmd.args(config.to_args());

        cmd.status().expect("Failed To Run...");
    }
    None
}
