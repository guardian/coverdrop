mod inventory;

pub use inventory::{
    AnsibleInventory, InventoryChildren, InventoryCluster, InventoryServer, InventoryVars,
};

// Call into ansible directly from the shell to perform early stages of bring up
// mainly configures the cluster to run k3s and adds some hardening setup.
#[allow(dead_code)]
pub fn apply_ansible() -> anyhow::Result<()> {
    //pipenv run ansible-playbook --key-file "${SSH_KEY_LOCATION}" "playbook/${stage}.yml" -i "${INVENTORY_LOCATION}" $ASK_BECOME_PASS_ARG
    todo!()
}
