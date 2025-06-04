use crate::{coverdrop_service::CoverDropService, dev::watch::status::ServicePodStatus};

/// A signal emitted from the k8s watcher
#[derive(Clone)]
pub enum K8sSignal {
    /// All the pods for a particular service
    PodsStatus(CoverDropService, Vec<ServicePodStatus>),
}
