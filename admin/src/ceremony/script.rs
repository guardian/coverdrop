//! This module contains the script used by the ceremony tool

pub const START_CEREMONY: &str =
    "You're about to start the ceremony to generate the necessary key material to set up CoverDrop.
You will need 6 different removable devices to store the material.
Make sure all devices are appropriately labelled as 'A', 'B', 'C', 'D', 'E, 'F'.
Do you have all removable devices at hand?";

pub const PRE_ANCHOR_ORG_KEY_PAIR: &str =
    "The anchor organization key pair is about to be created.
This is the most sensitive bundle and needs to be stored as securely as possible (instructions
will follow). If the secret key is ever leaked, the whole system's integrity is compromised a new ceremony
will have to be started.
Do you understand?";

pub const POST_ANCHOR_ORG_KEY_PAIR: &str = "The anchor organization key pair has been created.
Make sure the following steps are completed:
- Write the key pair down on a piece of paper and store it in a safe
- Move the key pair to removable device 'A'
Have you completed both steps?";

pub const SKIP_CREATE_ANCHOR_ORG_KEY_PAIR: &str = "Using provided organization key pair.
Make sure the following steps are completed:
- Write the key pair down on a piece of paper and store it in a safe
- Move the key pair to removable device 'A'
Have you completed both steps?";

pub const JOURNALIST_PROVISIONING_KEY_PAIR: &str =
    "The journalist provisioning key pair has been created.
Make sure the following steps are completed:
- Move the journalist provisioning key pair to removable device 'B'. This will be used by the identity API.
- Move the journalist provisioning key pair to removable device 'C'. This will be used by the editorial staff creating journalists.
Have you completed both steps?";

pub const COVERNODE_PROVISIONING_KEY_PAIR: &str =
    "The CoverNode provisioning key pair has been created.
Make sure the following step is completed:
- Move the CoverNode provisioning key pair to removable device 'B'. This will be used by the identity API.
Have you completed this step?";

pub const COVERNODE_DB: &str = "The CoverNode database(s) has been created.
Make sure the following step is completed:
- Move the CoverNode database(s) to removable device 'D'. This will be used by the CoverNode.
Have you completed this step?";

// should we be calling this admin key pair?
pub const ADMIN_KEY_PAIR: &str = "The admin key pair has been created.
Make sure the following step is completed:
- Move the admin key pair to removable device 'E'.
Have you completed this step?";

pub const SET_SYSTEM_STATUS_BUNDLE: &str =
    "The set system status bundle has been created. This will be used in the post-ceremony to send a request to the API to mark CoverDrop as available.
Make sure the following step is completed:
- Move the set system status bundle to removable device 'F'.
Have you completed this step?";

pub const ANCHOR_ORG_PK_BUNDLE: &str =
    "The anchor organization public key bundle has been created. This will be used in the post-ceremony.
Make sure the following step is completed:
- Move trusted organization public key bundle to removable device 'F'.
Have you completed this step?";

pub const PUBLIC_KEY_FORMS_BUNDLE: &str =
    "The public keys bundle has been created. This will be used in the post-ceremony to bootstrap the public key infrastructure.
Make sure the following step is completed:
- Move trusted organization public key bundle to removable device 'F'.
Have you completed this step?";

pub const DELETE_KEY_MATERIAL: &str =
    "The key ceremony is complete. All the key material and forms generated during the ceremony must now be deleted.
Make sure you have transferred the bundles to the appropriate devices.
WARNING: Failure to delete the key material may result in leaks.
Have you deleted all the bundles?";
