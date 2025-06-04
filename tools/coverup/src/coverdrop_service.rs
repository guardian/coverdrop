use clap::ValueEnum;

use crate::kube_client::CoverDropNamespace;

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CoverDropService {
    Api,
    IdentityApi,
    #[clap(name = "covernode")]
    CoverNode,
}

impl CoverDropService {
    pub fn port(&self) -> u16 {
        match self {
            CoverDropService::Api => api::DEFAULT_PORT,
            CoverDropService::IdentityApi => identity_api::DEFAULT_PORT,
            CoverDropService::CoverNode => covernode::DEFAULT_PORT,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "api" => Some(CoverDropService::Api),
            "identity-api" => Some(CoverDropService::IdentityApi),
            "covernode" => Some(CoverDropService::CoverNode),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CoverDropService::Api => "api",
            CoverDropService::IdentityApi => "identity-api",
            CoverDropService::CoverNode => "covernode",
        }
    }

    pub fn as_deployment_str(&self) -> &'static str {
        match self {
            CoverDropService::Api => "api-deployment",
            CoverDropService::IdentityApi => "identity-api-deployment",
            CoverDropService::CoverNode => "covernode-deployment",
        }
    }

    pub fn to_namespace(self) -> CoverDropNamespace {
        match self {
            CoverDropService::Api => CoverDropNamespace::Cloud,
            CoverDropService::IdentityApi => CoverDropNamespace::OnPremises,
            CoverDropService::CoverNode => CoverDropNamespace::OnPremises,
        }
    }

    pub fn all() -> &'static [CoverDropService] {
        &[
            CoverDropService::Api,
            CoverDropService::IdentityApi,
            CoverDropService::CoverNode,
        ]
    }

    pub fn as_pvc_name(&self) -> &'static str {
        match self {
            CoverDropService::Api => "api-persistentvolumeclaim",
            CoverDropService::IdentityApi => "identity-api-persistentvolumeclaim",
            CoverDropService::CoverNode => "covernode-persistentvolumeclaim",
        }
    }
}
