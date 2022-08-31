use proto::scheduler::Port;

pub struct PortParser {}

impl PortParser {
    /// `ports` is a vector of `Port`s, and we want to convert it into a vector of `proto::agent::Port`s
    ///
    /// Arguments:
    ///
    /// * `ports`: Vec<Port>
    ///
    /// Returns:
    ///
    /// A vector of proto::agent::Port
    pub fn to_agent_ports(ports: Vec<Port>) -> Vec<proto::agent::Port> {
        ports
            .into_iter()
            .map(|port| proto::agent::Port {
                source: port.source,
                destination: port.destination,
            })
            .collect()
    }

    /// `from_agent_ports` takes a vector of `proto::agent::Port`s and returns a vector of `Port`s
    ///
    /// Arguments:
    ///
    /// * `ports`: Vec<proto::agent::Port>
    ///
    /// Returns:
    ///
    /// A vector of Port structs.
    pub fn from_agent_ports(ports: Vec<proto::agent::Port>) -> Vec<Port> {
        ports
            .into_iter()
            .map(|port| Port {
                source: port.source,
                destination: port.destination,
            })
            .collect()
    }
}
