use std::{
    path::{Path, PathBuf},
    process::Command,
};

const RUN_ARGS: &[&str] = &["-s", "-serial", "stdio"];


pub fn main() {
    let mut args = std::env::args().skip(1); // skip executable name

    let kernel_binary_path = {
        let path = PathBuf::from(args.next().unwrap());
        path.canonicalize().unwrap()
    };
    let no_boot = if let Some(arg) = args.next() {
        match arg.as_str() {
            "--no-run" => true,
            other => panic!("unexpected argument `{}`", other),
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
    run_cmd.args(RUN_ARGS);

    let exit_status = run_cmd.status().unwrap();
    if !exit_status.success() {
        std::process::exit(exit_status.code().unwrap_or(1));
    }
}

fn build_image(kernel_bin: &Path) -> PathBuf {
    let kernel_manifest = locate_cargo_manifest::locate_manifest().unwrap();
    let bootloader_manifest = bootloader_locator::locate_bootloader("bootloader").unwrap();

    let mut build_cmd = Command::new(env!("CARGO"));
    build_cmd.current_dir(bootloader_manifest.parent().unwrap());
    build_cmd.arg("builder");
    build_cmd
        .arg("--kernel-manifest")
        .arg(&kernel_manifest);
    build_cmd.arg("--kernel-binary").arg(&kernel_bin);
    build_cmd
        .arg("--target-dir")
        .arg(kernel_manifest.parent().unwrap().join("target"));
    build_cmd
        .arg("--out-dir")
        .arg(kernel_bin.parent().unwrap());
    //build_cmd.arg("--quiet");

    

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

