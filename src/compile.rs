use std::ffi::OsStr;
use std::process::ExitStatus;
use std::{fs, io};
use std::{path::PathBuf, process::Command};

use crate::args::Action;

pub fn handle_compile(args: &Action) -> Result<(), String> {
    let Action::Compile {
        bpf_source_file,
        binary_file,
        out_dir,
    } = args
    else {
        return Err(format!("Invalid subcommand args: {:?}", args));
    };
    let message = "Compiling for Femto-Containers requires header files that \
                   are included in RIOT. Because of this, the compilation \
                   process needs to use the Makefile setup used by RIOT. \
                   You need to ensure that the file {file-name} \
                   you are trying to compile is located inside of a directory \
                   which contains a Makefile that points to RIOT base directory. \
                   See bpf/femto-container directory for an example";

    let formatted_message = message.replace("{file-name}", bpf_source_file);
    println!("[WARNING]\n{}", formatted_message);

    let source_path = PathBuf::from(bpf_source_file);
    let source_directory = source_path.parent().unwrap();
    let file_name = source_path.components().last().unwrap().as_os_str();

    let source_dir_name = source_directory.to_str().unwrap();

    let Ok(_) = compile_with_riot_build_system(file_name, source_dir_name) else {
        return Err("Failed to compile the eBPF bytecode.".to_string());
    };

    // Users can specify an optional name of the target output binary. If it is
    // set we rename the binary created in the previous step
    if let Some(file_name) = binary_file {
        let Ok(_) = rename_generated_binary(
            source_path.to_str().unwrap(),
            source_directory.to_str().unwrap(),
            file_name,
        ) else {
            return Err("Failed to rename the binary file.".to_string());
        };
    }

    // Now we clean up the sources directory by moving all generated binaries
    // and intermediate object files into the configured out/ directory.
    let Ok(()) = clean_up_sources_dir(source_dir_name, out_dir) else {
        return Err("Failed to clean up the object files.".to_string());
    };
    Ok(())
}

fn compile_with_riot_build_system(
    source_name: &OsStr,
    source_directory: &str,
) -> io::Result<ExitStatus> {
    Command::new("make")
        .env("RBPF_SOURCES", source_name)
        .arg("-C")
        .arg(source_directory)
        .arg("clean")
        .arg("all")
        .spawn()
        .expect("Failed to compile the eBPF bytecode.")
        .wait()
}

fn rename_generated_binary(
    source_name: &str,
    source_directory: &str,
    output_name: &str,
) -> io::Result<ExitStatus> {
    let base_name = source_name
        .split("/")
        .last()
        .unwrap()
        .split(".")
        .nth(0)
        .expect("You need to provide the .c source file")
        .to_string();

    Command::new("mv")
        .arg(format!("{}/{}.bin", source_directory, base_name))
        .arg(output_name)
        .spawn()
        .expect("Failed to copy the binary file.")
        .wait()
}

fn clean_up_sources_dir(source_directory: &str, out_dir: &str) -> Result<(), String> {
    // Make sure the out directory exists
    if !PathBuf::from(out_dir).exists() {
        std::fs::create_dir(out_dir).expect("Failed to create the object file directory.");
    }

    let read_dir = fs::read_dir(source_directory);
    for entry in read_dir.unwrap() {
        let path = &entry.unwrap().path();
        let path_str = path.to_str().unwrap();
        let extension = path.extension().and_then(OsStr::to_str);
        if Some("o") == extension || Some("bin") == extension {
            let Ok(_) = Command::new("mv")
                .arg(path_str)
                .arg(out_dir)
                .spawn()
                .expect("Failed to copy the binary file.")
                .wait()
            else {
                return Err(format!("Failed to move the file: {}.", path_str));
            };
        }
    }
    Ok(())
}
