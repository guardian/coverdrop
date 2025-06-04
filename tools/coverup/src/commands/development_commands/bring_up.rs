use std::{collections::HashSet, net::Ipv4Addr, num::NonZeroU8};

use crate::{
    bring_up::{
        ansible::{
            AnsibleInventory, InventoryChildren, InventoryCluster, InventoryServer, InventoryVars,
        },
        k3s,
    },
    development_image_source::BringUpImageSource,
    multipass::{delete_node, ensure_nodes_running, list_coverdrop_nodes},
};

pub async fn bring_up(
    _image_source: &BringUpImageSource,
    node_count: NonZeroU8,
    cpus_per_node: NonZeroU8,
    ram_gb_per_node: NonZeroU8,
    storage_gb_per_node: NonZeroU8,
) -> anyhow::Result<()> {
    //
    // Set up multipass
    //
    let nodes = list_coverdrop_nodes()?;

    // We want to initialize our cluster while it's in a clean state
    // with all nodes up so let's delete any non-runnning nodes
    let non_running_nodes = nodes.iter().filter(|node| node.state != "Running");

    for node in non_running_nodes {
        delete_node(node)?;
    }

    ensure_nodes_running(
        &nodes,
        node_count,
        cpus_per_node,
        ram_gb_per_node,
        storage_gb_per_node,
    )?;

    // Our nodes should be running now, let's check.
    let nodes = list_coverdrop_nodes()?;

    let unique_ips = nodes
        .iter()
        .flat_map(|node| node.local_ip())
        .collect::<HashSet<&Ipv4Addr>>();

    if unique_ips.len() != node_count.get() as usize {
        anyhow::bail!("Multipass created two nodes with the same IP address, manually delete them using `multipass delete` and try again")
    }

    // Time to build the inventory file for ansible
    let token = k3s::generate_random_token();
    let hosts = unique_ips.iter().map(|ip| (ip.to_string(), ())).collect();

    let _inventory = AnsibleInventory {
        k3s_cluster: InventoryCluster {
            children: InventoryChildren {
                server: InventoryServer {
                    hosts
                },
            },
            vars: InventoryVars {
                ansible_port: 22,
                anisble_user: "ubuntu",
                k3s_version: "v1.26.9+k3s1",
                token,
                api_endpoint: "{{ hostvars[groups['server'][0]]['ansible_host'] | default(groups['server'][0]) }}",
                extra_server_args: "",
                extra_agent_args: ""
            }
        }
    };

    println!("thats all folks!");

    // for each host ssh -i

    Ok(())
}
