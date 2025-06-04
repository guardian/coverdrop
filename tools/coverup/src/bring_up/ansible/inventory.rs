// k3s_cluster:
//     children:
//         server:
//             hosts:
//                 $NODE1_IP:
//                 $NODE2_IP:
//                 $NODE3_IP:

//     # Required Vars
//     vars:
//         ansible_port: 22
//         ansible_user: ubuntu
//         k3s_version: v1.26.9+k3s1
//         token: "mytoken" # Use ansible vault if you want to keep it secret
//         api_endpoint: "{{ hostvars[groups['server'][0]]['ansible_host'] | default(groups['server'][0]) }}"
//         extra_server_args: ""
//         extra_agent_args: ""

use std::collections::HashMap;

#[allow(dead_code)]
pub struct InventoryVars {
    pub ansible_port: u16,
    pub anisble_user: &'static str,
    pub k3s_version: &'static str,
    pub token: String,
    pub api_endpoint: &'static str,
    pub extra_server_args: &'static str,
    pub extra_agent_args: &'static str,
}

#[allow(dead_code)]
pub struct InventoryServer {
    // Slightly weird construct... The host IPs are empty maps in our inventory.yaml
    // file which means to serialize it we need this hashmap to nothing.
    pub hosts: HashMap<String, ()>,
}

#[allow(dead_code)]
pub struct InventoryChildren {
    pub server: InventoryServer,
}

#[allow(dead_code)]
pub struct InventoryCluster {
    pub children: InventoryChildren,
    pub vars: InventoryVars,
}

#[allow(dead_code)]
pub struct AnsibleInventory {
    pub k3s_cluster: InventoryCluster,
}
