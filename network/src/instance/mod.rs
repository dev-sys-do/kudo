pub mod request;
pub mod response;

use crate::error::KudoNetworkError;
use crate::utils::{
    bridge_name, default_interface_name, namespace_name, run_command, veth_in_name, veth_out_name,
};

use request::{CleanInstanceRequest, SetupInstanceRequest};
use response::SetupInstanceResponse;

/// Create a network namespace and interfaces, and configure routes to isolate instances
pub fn setup_instance(
    request: SetupInstanceRequest,
) -> Result<SetupInstanceResponse, KudoNetworkError> {
    let default_interface = default_interface_name()?;
    let bridge = bridge_name(request.node_id.clone());
    let namespace = namespace_name(request.instance_id.clone());
    let veth_out = veth_out_name(request.instance_id.clone());
    let veth_in = veth_in_name(request.instance_id.clone());

    // Create a network namespace
    run_command("ip", &["netns", "add", &namespace])?;

    // Create veth pairs to communicate between inside and outside the namespace
    run_command(
        "ip",
        &[
            "link", "add", "dev", &veth_out, "type", "veth", "peer", "name", &veth_in,
        ],
    )?;

    // Move veth_in inside the namespace
    run_command("ip", &["link", "set", "dev", &veth_in, "netns", &namespace])?;

    // Attribute an IP address to veth_in (the instance ip address)
    run_command(
        "ip",
        &[
            "netns",
            "exec",
            &namespace,
            "ip",
            "address",
            "add",
            &request.instance_ip_cidr.to_string(),
            "dev",
            &veth_in,
        ],
    )?;

    // Set veth_out, lo (inside the newly created namespace) and veth_in up
    run_command("ip", &["link", "set", "dev", &veth_out, "up"])?;
    run_command(
        "ip",
        &[
            "netns", "exec", &namespace, "ip", "link", "set", "dev", "lo", "up",
        ],
    )?;
    run_command(
        "ip",
        &[
            "netns", "exec", &namespace, "ip", "link", "set", "dev", &veth_in, "up",
        ],
    )?;

    // link veth_out to the node network interface
    run_command("ip", &["link", "set", "dev", &veth_out, "master", &bridge])?;

    // Add default route to be able to get out the newly created namespace
    run_command(
        "ip",
        &[
            "netns",
            "exec",
            &namespace,
            "ip",
            "route",
            "add",
            "default",
            "via",
            &request.node_ip_addr.to_string(),
        ],
    )?;

    // Iptables rules to expose ports
    for port in request.ports.iter() {
        run_command(
            "iptables",
            &[
                "-t",
                "nat",
                "-A",
                "PREROUTING",
                "-i",
                &default_interface,
                "-p",
                "tcp",
                "--dport",
                &port.source.to_string(),
                "-j",
                "DNAT",
                "--to",
                &format!(
                    "{}:{}",
                    request.instance_ip_cidr.address(),
                    port.destination
                ),
            ],
        )?;
        run_command(
            "iptables",
            &[
                "-t",
                "nat",
                "-A",
                "PREROUTING",
                "-i",
                &default_interface,
                "-p",
                "udp",
                "--dport",
                &port.source.to_string(),
                "-j",
                "DNAT",
                "--to",
                &format!(
                    "{}:{}",
                    request.instance_ip_cidr.address(),
                    port.destination
                ),
            ],
        )?;
        run_command(
            "iptables",
            &[
                "-t",
                "nat",
                "-A",
                "OUTPUT",
                "-o",
                "lo",
                "-p",
                "tcp",
                "-m",
                "tcp",
                "--dport",
                &port.source.to_string(),
                "-j",
                "DNAT",
                "--to-destination",
                &format!(
                    "{}:{}",
                    request.instance_ip_cidr.address(),
                    port.destination
                ),
            ],
        )?;
        run_command(
            "iptables",
            &[
                "-t",
                "nat",
                "-A",
                "OUTPUT",
                "-o",
                "lo",
                "-p",
                "udp",
                "-m",
                "udp",
                "--dport",
                &port.source.to_string(),
                "-j",
                "DNAT",
                "--to-destination",
                &format!(
                    "{}:{}",
                    request.instance_ip_cidr.address(),
                    port.destination
                ),
            ],
        )?;
    }

    Ok(SetupInstanceResponse::new(veth_out, namespace))
}

/// Remove instance network namespace and interfaces created for it
pub fn clean_instance(request: CleanInstanceRequest) -> Result<(), KudoNetworkError> {
    let default_interface = default_interface_name()?;
    let namespace = namespace_name(request.instance_id.clone());
    let veth_out = veth_out_name(request.instance_id.clone());

    run_command("ip", &["netns", "del", &namespace])?;
    run_command("ip", &["link", "del", &veth_out])?;

    for port in request.ports.iter() {
        run_command(
            "iptables",
            &[
                "-t",
                "nat",
                "-D",
                "PREROUTING",
                "-i",
                &default_interface,
                "-p",
                "tcp",
                "--dport",
                &port.source.to_string(),
                "-j",
                "DNAT",
                "--to",
                &format!(
                    "{}:{}",
                    request.instance_ip_cidr.address(),
                    port.destination
                ),
            ],
        )?;

        run_command(
            "iptables",
            &[
                "-t",
                "nat",
                "-D",
                "PREROUTING",
                "-i",
                &default_interface,
                "-p",
                "udp",
                "--dport",
                &port.source.to_string(),
                "-j",
                "DNAT",
                "--to",
                &format!(
                    "{}:{}",
                    request.instance_ip_cidr.address(),
                    port.destination
                ),
            ],
        )?;

        run_command(
            "iptables",
            &[
                "-t",
                "nat",
                "-D",
                "OUTPUT",
                "-o",
                "lo",
                "-p",
                "tcp",
                "-m",
                "tcp",
                "--dport",
                &port.source.to_string(),
                "-j",
                "DNAT",
                "--to-destination",
                &format!(
                    "{}:{}",
                    request.instance_ip_cidr.address(),
                    port.destination
                ),
            ],
        )?;

        run_command(
            "iptables",
            &[
                "-t",
                "nat",
                "-D",
                "OUTPUT",
                "-o",
                "lo",
                "-p",
                "udp",
                "-m",
                "udp",
                "--dport",
                &port.source.to_string(),
                "-j",
                "DNAT",
                "--to-destination",
                &format!(
                    "{}:{}",
                    request.instance_ip_cidr.address(),
                    port.destination
                ),
            ],
        )?;
    }
    Ok(())
}
