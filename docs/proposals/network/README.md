# Network

## How it works

### Introduction

The objective of the network service is to make several instances communicate with each other as if they were all connected to a single private network, no matter if they are on the same machine or not.

It also brings a network isolation layer between instances.

### Suggested approach

To try to solve this problem, the agent service has to use a network library to setup network interfaces, namespaces, routing tables and iptables rules for each new node or new instance.

Each instance run in its own network namespace. Its default network interface is a veth one, paired with another veth interface outside the instance namespace.

All instance's veth interfaces (outside their instance's namespace) running on the same node are connected to a single bridge interface called Container Network Interface (CNI).

The CNI is also connected to the default node's interface to allow communications from and to outside the node.

Bellow is a representation of the network interfaces and namespaces in a Kudo node:

![Node example](schema.png)

## API examples

### Setup node

Use `setup_node` function from `node` module each time you want to create a CNI and configure
iptables rules for a new node.

```rust
let node_id = "node";
let node_ip_addr = Ipv4Addr::from_str("10.0.0.1").unwrap();
let node_ip_cidr = Ipv4Inet::new(node_ip_addr, 24).unwrap();
let nodes_ips = Vec::new();

let request = SetupNodeRequest::new(node_id.to_string(), node_ip_cidr, nodes_ips);
let response = setup_node(request).unwrap();

println!("CNI name: {}", response.interface_name);
```

After each node reboot, you need to reconfigure iptables running `setup_iptables` and
`add_other_nodes` functions from `node` module.

```rust
let node_id = "node";
let request = SetupIptablesRequest::new(node_id.to_string());

let other_node_id = "node2";
let other_node_cluster_ip = Ipv4Inet::new(Ipv4Addr::from_str("10.0.1.1").unwrap(), 24).unwrap();
let other_node_external_ip = Ipv4Addr::from_str("22.22.22.22").unwrap();
let new_node_request = NodeRequest::new(
    other_node_id.to_string(),
    NodeRequest::new(other_node_cluster_ip, other_node_external_ip),
)
let nodes_ips = vec![new_node_request];

setup_iptables(request).unwrap();
add_other_nodes(nodes_ips).unwrap();
```

### Setup instance

Before running a new instance, please call `setup_instance` function from `instance` module to setup
network namespace, interfaces and routing tables.

```rust
let node_id = "node";
let node_ip_addr = Ipv4Addr::from_str("10.0.0.1").unwrap();
let instance_id = "instance";
let instance_ip_addr = Ipv4Addr::from_str("10.0.0.2").unwrap();
let instance_ip_cidr = Ipv4Inet::new(instance_ip_addr, 24).unwrap();
let ports = vec![Port::new(80, 8080)];

let request = SetupInstanceRequest::new(
    node_id.to_string(),
    node_ip_addr,
    instance_id.to_string(),
    instance_ip_cidr,
    ports,
);
let response = setup_instance(request).unwrap();
println!("Instance default interface: {}", response.interface_name);
println!("Network namespace: {}", response.namespace_name);
```

You can also get the namespace's name of a given instance with `get_namespace_name` from `utils`
module.

```rust
let instance_id = "instance";
let namespace_name = get_namespace_name(instance_id.to_string());
println!("Namespace of {}: {}", instance_id, namespace_name);
```

### Add other nodes

When a new node join the cluster, you need to call `new_node_in_cluster` function from `node`
module.

```rust
let node_id = "node";
let node_cluster_ip = Ipv4Inet::new(Ipv4Addr::from_str("10.0.1.1").unwrap(), 24).unwrap();
let node_external_ip = Ipv4Addr::from_str("22.22.22.22").unwrap();
let new_node_request = NodeRequest::new(
    node_id.to_string(),
    NodeIp::new(node_cluster_ip, node_external_ip),
);

new_node_in_cluster(new_node_request).unwrap();
```

And you have to run `delete_node_in_cluster` function from `node` module when a node leave the
cluster.

```rust
let node_id = "node";
let node_cluster_ip = Ipv4Inet::new(Ipv4Addr::from_str("10.0.1.1").unwrap(), 24).unwrap();
let node_external_ip = Ipv4Addr::from_str("22.22.22.22").unwrap();
let delete_node_request = NodeRequest::new(
    node_id.to_string(),
    NodeIp::new(node_cluster_ip, node_external_ip),
);

delete_node_in_cluster(delete_node_request).unwrap();
```

### Clean up

To delete CNI and iptables rules of a specific node, use `clean_node` function from `node` module.

```rust
let node_id = "node";
let request = CleanNodeRequest::new(node_id.to_string());
clean_node(request).unwrap();
```

Run `clean_instance` function from `instance` module to delete network namespace and interfaces of a
specific instance.

```rust
let instance_id = "instance";
let instance_ip_addr = Ipv4Addr::from_str("10.0.0.2").unwrap();
let instance_ip_cidr = Ipv4Inet::new(instance_ip_addr, 24).unwrap();
let ports = vec![Port::new(80, 8080)];
let request = CleanInstanceRequest::new(
    instance_id.to_string(),
    ports,
    instance_ip_cidr,
);
clean_instance(request).unwrap();
```
