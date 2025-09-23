#!/bin/sh
#![allow(unused_attributes)] /*
                             OUT=/tmp/tmp && rustc "$0" -o ${OUT} && exec ${OUT} $@ || exit $? #*/

use std::fs;
use std::io::Result;
use std::path::PathBuf;
use std::process::Command;

fn mkdir(dir_name: &str) -> Result<()> {
    fs::create_dir(dir_name)
}

fn pwd() -> Result<PathBuf> {
    std::env::current_dir()
}

fn cd(dir_name: &str) -> Result<()> {
    std::env::set_current_dir(dir_name)
}

fn main() -> Result<()> {
    let _ = mkdir("tmp");

    cd("tmp")?;

    let tmp_path = pwd()?.to_string_lossy().to_string();
    let build_path = format!("{}/ffmpeg_build", tmp_path);
    let branch = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "release/7.1".to_string());
    let target = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "".to_string());
    let num_job = std::thread::available_parallelism().unwrap().get();

    if fs::metadata("ffmpeg").is_err() {
        Command::new("git")
            .arg("clone")
            .arg("--single-branch")
            .arg("--branch")
            .arg(&branch)
            .arg("--depth")
            .arg("1")
            .arg("https://github.com/ffmpeg/ffmpeg")
            .status()?;
    }

    cd("ffmpeg")?;

    Command::new("git")
        .arg("fetch")
        .arg("origin")
        .arg(&branch)
        .arg("--depth")
        .arg("1")
        .status()?;

    Command::new("git")
        .arg("checkout")
        .arg("FETCH_HEAD")
        .status()?;

    let mut configure_cmd = Command::new("./configure");
    configure_cmd
        .arg(format!("--prefix={}", build_path))
        // To workaround `https://github.com/larksuite/rsmpeg/pull/98#issuecomment-1467511193`
        .arg("--disable-decoder=exr,phm")
        .arg("--disable-programs")
        .arg("--disable-autodetect");

    // Configure for musl targets
    if target.contains("musl") {
        if target.contains("x86_64") {
            configure_cmd
                .arg("--target-os=linux")
                .arg("--arch=x86_64")
                .arg("--cc=musl-gcc")
                .arg("--cxx=musl-g++");
        } else if target.contains("aarch64") {
            configure_cmd
                .arg("--target-os=linux")
                .arg("--arch=aarch64")
                .arg("--cc=aarch64-linux-musl-gcc")
                .arg("--cxx=aarch64-linux-musl-g++")
                .arg("--enable-cross-compile");
        }
    }

    configure_cmd.status()?;

    Command::new("make")
        .arg("-j")
        .arg(num_job.to_string())
        .status()?;

    Command::new("make").arg("install").status()?;

    cd("..")?;

    Ok(())
}
