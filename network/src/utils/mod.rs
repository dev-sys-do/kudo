use std::{
    cmp::min,
    io::ErrorKind,
    process::{Command, Output},
};

use crate::error::KudoNetworkError;

const IFACE_MAX_SIZE: usize = 12;

pub(crate) fn bridge_name(node_id: String) -> String {
    format!("kbr{}", &node_id[..min(IFACE_MAX_SIZE, node_id.len())])
}

pub(crate) fn run_command(cmd: &str, args: &[&str]) -> Result<String, KudoNetworkError> {
    let output = Command::new(cmd).args(args).output();
    wrap_command_output(output, cmd, args)
}

fn wrap_command_output(
    output: Result<Output, std::io::Error>,
    cmd: &str,
    args: &[&str],
) -> Result<String, KudoNetworkError> {
    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(KudoNetworkError::CommandFailed(format!(
                    "{} {} : {}",
                    cmd,
                    args.join(" "),
                    String::from_utf8_lossy(&output.stderr)
                )))
            }
        }
        Err(e) if e.kind() == ErrorKind::PermissionDenied => {
            eprintln!("Not enough permission to run the command. Are you running as root?");
            Err(KudoNetworkError::CommandError(Box::new(e)))
        }
        Err(e) => Err(KudoNetworkError::CommandError(Box::new(e))),
    }
}
