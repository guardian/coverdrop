use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    path::{Path, PathBuf},
};

use crate::{coverdrop_service::CoverDropService, dev::build::cargo::CargoMetadata};

/// Represents a service in the watch tool. E.g. CoverNode, Identity-API, etc.
#[derive(Debug)]
pub struct FsWatchedCrate {
    pub name: String,
    pub path: PathBuf,
    /// *Services*, not crates, we need to rebuild if this crate gets modified
    pub rebuild_on_change: HashSet<CoverDropService>,
}

impl FsWatchedCrate {
    pub fn new(name: &str, path: impl AsRef<Path>) -> Self {
        let mut dependant_services = HashSet::new();

        if let Some(service) = CoverDropService::from_str(name) {
            dependant_services.insert(service);
        }

        FsWatchedCrate {
            name: name.to_string(),
            path: path.as_ref().to_owned(),
            rebuild_on_change: dependant_services,
        }
    }

    /// Using cargo metadata construct a list of watched crates.
    ///
    /// Cargo uses a representation where a package lists it's dependencies.
    /// Because we want to know which crates to rebuild after a change we want to invert
    /// this relationship and find the package's dependents. This makes it simple to
    /// look up what we need to rebuild after a filesystem event is fired.
    ///
    /// Due to the nature of CoverUp only wanting to deal with services deployed into
    /// Kubernetes, we don't store any non-service crates, put another way, we are not
    /// interested in rebuilding libraries separately to their services.
    pub fn from_cargo_metadata(metadata: &CargoMetadata) -> Vec<FsWatchedCrate> {
        let mut crate_name_to_service = HashMap::<String, FsWatchedCrate>::new();

        for package in &metadata.packages {
            tracing::info!("Adding {} to watched crate list", &package.name);
            crate_name_to_service.insert(
                package.name.clone(),
                FsWatchedCrate::new(&package.name, package.path()),
            );
        }

        for package in &metadata.packages {
            if let Some(service) = CoverDropService::from_str(&package.name) {
                for dependency in &package.dependencies {
                    // If it has a path then it's a local dependency...
                    if let Some(_path) = &dependency.path {
                        // If the crate is a service (not some other package)
                        // Add the parent package as a dependant of another crate - inverting the relationship
                        if let Some(dependency_watched_crate) =
                            crate_name_to_service.get_mut(&dependency.name)
                        {
                            dependency_watched_crate.rebuild_on_change.insert(service);
                        } else {
                            tracing::error!(
                                "Could not find package {} in watched crates map, it should be there.",
                                dependency.name
                            );
                        }
                    }
                }
            }
        }

        crate_name_to_service.into_values().collect()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        coverdrop_service::CoverDropService,
        dev::{
            build::cargo::{CargoMetadata, MetadataDependency, MetadataPackage},
            watch::fs::FsWatchedCrate,
        },
    };

    #[test]
    fn test_from_cargo_metadata_dependency_graph_flipping() {
        let common_id = "path+file:///path/to/common#0.1.0".to_string();
        let journalist_vault_id = "path+file:///path/to/journalist-vault#0.1.0".to_string();
        let covernode_id = "path+file:///path/to/covernode#0.1.0".to_string();

        let metadata = CargoMetadata {
            workspace_root: PathBuf::from("."),
            workspace_members: vec![
                common_id.clone(),
                covernode_id.clone(),
                journalist_vault_id.clone(),
            ],
            packages: vec![
                MetadataPackage {
                    id: covernode_id,
                    name: "covernode".to_string(),
                    dependencies: vec![
                        MetadataDependency {
                            name: "common".to_string(),
                            path: Some(PathBuf::from("/path/to/common")),
                        },
                        MetadataDependency {
                            name: "another_unrelated_crate".to_string(),
                            path: None,
                        },
                    ],
                },
                MetadataPackage {
                    id: journalist_vault_id,
                    name: "journalist-vault".to_string(),
                    dependencies: vec![MetadataDependency {
                        name: "common".to_string(),
                        path: Some(PathBuf::from("/path/to/common")),
                    }],
                },
                MetadataPackage {
                    id: common_id,
                    name: "common".to_string(),
                    dependencies: vec![MetadataDependency {
                        name: "yet_another_unrelated_crate".to_string(),
                        path: None,
                    }],
                },
            ],
        };

        let watched_crates = FsWatchedCrate::from_cargo_metadata(&metadata);
        println!("{watched_crates:#?}");

        // Assertions
        assert_eq!(watched_crates.len(), 3);
        assert!(watched_crates.iter().any(|c| c.name == "covernode"));
        assert!(watched_crates.iter().any(|c| c.name == "common"));
        assert!(watched_crates.iter().any(|c| c.name == "journalist-vault"));

        let common = watched_crates.iter().find(|c| c.name == "common").unwrap();
        assert_eq!(common.rebuild_on_change.len(), 1);
        assert!(common
            .rebuild_on_change
            .contains(&CoverDropService::CoverNode));

        let journalist_vault = watched_crates
            .iter()
            .find(|c| c.name == "journalist-vault")
            .unwrap();
        assert_eq!(journalist_vault.rebuild_on_change.len(), 0);

        let covernode = watched_crates
            .iter()
            .find(|c| c.name == "covernode")
            .unwrap();
        assert_eq!(covernode.rebuild_on_change.len(), 1);
        assert!(covernode
            .rebuild_on_change
            .contains(&CoverDropService::CoverNode));
    }
}
