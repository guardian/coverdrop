//! Utility conditions to wait for various states in the kubernetes cluster.

use k8s_openapi::api::apps::v1::Deployment;
use kube::runtime::wait::Condition;

pub fn is_rollout_restart_complete() -> impl Condition<Deployment> {
    |obj: Option<&Deployment>| {
        let Some(deployment) = &obj else {
            return false;
        };

        // Get current generation
        let Some(generation) = deployment.metadata.generation else {
            return false;
        };

        // Get observed generation
        let Some(status) = &deployment.status else {
            return false;
        };

        let Some(observed_generation) = status.observed_generation else {
            return false;
        };

        let generation_matched_observed = generation == observed_generation;
        let no_unavailable_replicas = status.unavailable_replicas.is_none()
            || status.unavailable_replicas.is_some_and(|c| c == 0);

        generation_matched_observed && no_unavailable_replicas
    }
}
