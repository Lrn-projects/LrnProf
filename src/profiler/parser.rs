use std::path::Path;

use crate::{logs, utils};

pub fn parse_bin(pid: i32) {
    let output = utils::get_bin_path(pid);
    if !Path::new(&output).exists() {
        logs::error_log("Cannot find the binary of the process".to_string());
    }
    logs::info_log("Binary found".to_string());
}
