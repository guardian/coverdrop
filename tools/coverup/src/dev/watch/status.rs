use std::fmt;

use super::log_ring_buffer::LogRingBuffer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildStatus {
    Idle,
    Building,
    CopyingImageToClusterNodes,
    Deploying,
    Restarting,
}

impl fmt::Display for BuildStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildStatus::Idle => Ok(()),
            BuildStatus::Building => write!(f, "Building"),
            BuildStatus::CopyingImageToClusterNodes => write!(f, "Copying Image to Cluster Nodes"),
            BuildStatus::Deploying => write!(f, "Deploying"),
            BuildStatus::Restarting => write!(f, "Restarting"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ServicePodStatus {
    pub name: String,
    pub phase: String,
    pub is_being_deleted: bool,
}

pub struct ServiceStatus {
    pub dirty: bool,
    pub build_status: BuildStatus,
    pub build_log: LogRingBuffer,
    pub pods: Vec<ServicePodStatus>,
}

impl Default for ServiceStatus {
    fn default() -> Self {
        Self {
            dirty: false,
            build_status: BuildStatus::Idle,
            build_log: LogRingBuffer::default(),
            pods: vec![],
        }
    }
}
