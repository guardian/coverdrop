use clap::ValueEnum;

/// During development bring up where should coverup
/// look for images.
#[derive(Debug, ValueEnum, Clone)]
pub enum BringUpImageSource {
    /// Get the containers from the container image
    /// repository using the tag `:main`
    Repository,
    /// Get locally built images tagged `:dev`
    Local,
}
