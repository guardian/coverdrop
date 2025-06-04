use crate::{coverdrop_service::CoverDropService, dev::watch::status::BuildStatus};

#[derive(Clone, Debug)]
pub enum BuilderSignal {
    /// Begin signal to indicate to the UI that it should do things like
    /// clear the previous build log.
    Begin(CoverDropService),
    /// Builder can tell the UI which stage of the build it's on
    Status(CoverDropService, BuildStatus),
    //// Indicates that a build has run successfully
    Success(CoverDropService),
    //// Indicates that a build has failed
    Failed(CoverDropService),
    /// Build can asynchronously forward a log line from the Docker build
    LogLine(CoverDropService, String),
}
