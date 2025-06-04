use std::{path::PathBuf, str::FromStr};

use crate::coverdrop_service::CoverDropService;

#[derive(Clone, Debug)]
pub struct PvcPath {
    pub pvc: String,
    pub path: PathBuf,
}

impl FromStr for PvcPath {
    type Err = String;

    /// Parse a PVC path from a string reference which has the format "$PVC_NAME:$PATH".
    /// If the part before the colon matches a CoverDrop service name this will automatically be expended
    /// to the service's full persistent volume claim name.
    ///
    /// For example: "covernode" becomes "covernode-persistentvolumeclaim".
    ///
    /// If you wish to copy to a PVC which is not associated with a CoverDrop service then you can use the
    /// full PVC name. For example "foo-persistentvolumeclaim:/data/example.txt"
    ///
    /// If the PVC is named exactly as a CoverDrop service name (e.g. "covernode" or "identity-api")
    /// then that's too bad, give your PVC a better name, one containing "-persistentvolumeclaim" at the end for example.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(2, ':').collect();

        if parts.len() == 2 {
            let pvc = if let Some(service) = CoverDropService::from_str(parts[0]) {
                service.as_pvc_name().to_string()
            } else {
                parts[0].to_string()
            };

            let path = PathBuf::from(parts[1]);

            Ok(PvcPath { pvc, path })
        } else {
            Err(format!("{} is not in the format $PVC_NAME:$PATH", s))
        }
    }
}

#[derive(Clone, Debug)]
pub enum LocalOrPvcPath {
    /// A path on a PVC, e.g foo-persistentvolumeclaim:/data/example.txt
    /// If the part before the colon matches a CoverDrop service name then this will internally
    /// be expanded to the full persistent volume claim name,
    Pvc(PvcPath),
    /// A path on the host machine which is running coverup
    Local { path: PathBuf },
}

impl LocalOrPvcPath {
    pub fn _new_local(path: PathBuf) -> Self {
        LocalOrPvcPath::Local { path }
    }

    pub fn _new_pvc(pvc: String, path: PathBuf) -> Self {
        LocalOrPvcPath::Pvc(PvcPath { pvc, path })
    }
}

impl FromStr for LocalOrPvcPath {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Providing an absolute path to a local file is an escape hatch if your path has a colon in
        // it for some reason.
        if s.starts_with('/') {
            Ok(LocalOrPvcPath::Local {
                path: PathBuf::from(s),
            })
        } else if let Ok(pvc_path) = PvcPath::from_str(s) {
            Ok(LocalOrPvcPath::Pvc(pvc_path))
        } else {
            Ok(LocalOrPvcPath::Local {
                path: PathBuf::from(s),
            })
        }
    }
}
