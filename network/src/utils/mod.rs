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

pub(crate) fn default_interface_name() -> Result<String, KudoNetworkError> {
    default_net::get_default_interface()
        .map(|interface| interface.name)
        .map_err(|err| {
            KudoNetworkError::DefaultNetworkInterfaceError(format!(
                "Could not find default interface : {}",
                err
            ))
        })
}

pub fn namespace_name(instance_id: String) -> String {
    // Linux network interface names are size limited!
    format!(
        "kns{}",
        &instance_id[..min(IFACE_MAX_SIZE, instance_id.len())]
    )
}

pub(crate) fn veth_out_name(instance_id: String) -> String {
    format!(
        "kvo{}",
        &instance_id[..min(IFACE_MAX_SIZE, instance_id.len())]
    )
}

pub(crate) fn veth_in_name(instance_id: String) -> String {
    format!(
        "kvi{}",
        &instance_id[..min(IFACE_MAX_SIZE, instance_id.len())]
    )
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
