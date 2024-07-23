use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process::Command;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    toggle_feature: bool,
    volume: u8,
    brightness: u8,
    color: String,
    threshold: u8,
    max_threshold: u8,
}

pub fn spawn_config_terminal(config_path: &str) {
    // Open the new terminal window and run the config script
    let command = format!(
        "cmd.exe /C start cmd.exe /K \"rustc config_interactive.rs && config_interactive.exe {}\"",
        config_path
    );
    Command::new("cmd")
        .args(["/C", "start", "cmd.exe", "/K", command.as_str()])
        .spawn()
        .expect("Failed to open new terminal");
}