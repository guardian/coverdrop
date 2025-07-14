use axum::{body::Bytes, extract::State, Json};
use axum_extra::{headers::UserAgent, TypedHeader};
use common::{
    api::models::messages::user_to_covernode_message::EncryptedUserToCoverNodeMessage,
    healthcheck::HealthCheck,
};
use itertools::Itertools as _;

use crate::{errors::AppError, kinesis_client::KinesisClient};

pub async fn get_healthcheck() -> Json<HealthCheck> {
    let result = HealthCheck::new("u2j-appender", "ok");

    Json(result)
}

enum AppendOutcome {
    /// The message was appended to the stream successfully
    Success,
    /// The message could not be parsed
    ParseFailed,
    /// The message was rejected by the kinesis service due to back pressure
    AppendBackPressure,
    /// The message failed to be put on the kinesis stream for another reason
    AppendFailed,
}

impl AppendOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppendOutcome::Success => "success",
            AppendOutcome::ParseFailed => "parse_failed",
            AppendOutcome::AppendBackPressure => "append_back_pressure",
            AppendOutcome::AppendFailed => "append_failed",
        }
    }
}

const METRIC_NAME: &str = "Client";
const UNKNOWN_PLATFORM: &str = "unknown";

fn user_agent_to_platform_and_version(
    user_agent: &TypedHeader<UserAgent>,
) -> (&'static str, String) {
    // User agent strings are like
    // Android beta     GuardianNews/6.172.121288 (android 33; beta)
    // Android          GuardianNews/6.171.21275 (android 34)
    // iPhone           Guardian/23759 CFNetwork/3826.400.120 Darwin/24.3.0

    let user_agent = user_agent.as_str();

    let platform = if user_agent.contains("android") {
        "android"
    } else if user_agent.contains("Darwin") {
        "ios"
    } else {
        return (
            UNKNOWN_PLATFORM,
            "could_not_find_platform_name_in_user_agent".to_string(),
        );
    };

    let version_str = user_agent
        .split("/")
        .nth(1) // Jump over the app name to the version
        .and_then(|rest| rest.split(" ").next()) // Cut out the OS version
        .unwrap_or("unknown_version"); // If we fail to get any of that then just return unknown

    let version = if platform == "android" {
        version_str.split(".").take(2).join(".")
    } else {
        version_str.to_string()
    };

    (platform, version)
}

fn put_metrics(user_agent: &Option<TypedHeader<UserAgent>>, outcome: AppendOutcome) {
    // find device type
    match user_agent {
        Some(user_agent) => {
            let (platform, version) = user_agent_to_platform_and_version(user_agent);

            metrics::counter!(METRIC_NAME,
                "platform" => platform,
                "version" => version,
                "outcome" => outcome.as_str()
            )
            .increment(1);
        }
        None => {
            metrics::counter!(METRIC_NAME,
                "platform" => UNKNOWN_PLATFORM,
                "version" => "no_user_agent",
                "outcome" => outcome.as_str()
            )
            .increment(1);
        }
    };
}

#[axum::debug_handler]
pub async fn post_u2j_message(
    user_agent: Option<TypedHeader<UserAgent>>,
    State(kinesis_client): State<KinesisClient>,
    body: Bytes,
) -> Result<(), AppError> {
    let u2j_message = serde_json::from_slice::<EncryptedUserToCoverNodeMessage>(&body)
        .inspect_err(|_| {
            put_metrics(&user_agent, AppendOutcome::ParseFailed);
        })?;

    kinesis_client
        .encode_and_put_u2j_message(u2j_message)
        .await
        .map_err(|e| match e.as_service_error() {
            Some(se) if se.is_provisioned_throughput_exceeded_exception() => {
                put_metrics(&user_agent, AppendOutcome::AppendBackPressure);
                AppError::AppendBackPressure(e)
            }
            _ => {
                put_metrics(&user_agent, AppendOutcome::AppendFailed);
                AppError::AppendFailed(e)
            }
        })?;

    put_metrics(&user_agent, AppendOutcome::Success);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_extra::headers::UserAgent;

    #[test]
    fn test_user_agent_parsing() {
        // Android beta
        let android_beta = TypedHeader(UserAgent::from_static(
            "GuardianNews/6.172.121288 (android 33; beta)",
        ));
        let (platform, version) = user_agent_to_platform_and_version(&android_beta);
        assert_eq!(platform, "android");
        assert_eq!(version, "6.172");

        // Android
        let android = TypedHeader(UserAgent::from_static(
            "GuardianNews/6.171.21275 (android 34)",
        ));
        let (platform, version) = user_agent_to_platform_and_version(&android);
        assert_eq!(platform, "android");
        assert_eq!(version, "6.171");

        // iPhone
        let iphone = TypedHeader(UserAgent::from_static(
            "Guardian/23759 CFNetwork/3826.400.120 Darwin/24.3.0",
        ));
        let (platform, version) = user_agent_to_platform_and_version(&iphone);
        assert_eq!(platform, "ios");
        assert_eq!(version, "23759");

        // Unknown
        let unknown = TypedHeader(UserAgent::from_static("Unknown/1.0"));
        let (platform, version) = user_agent_to_platform_and_version(&unknown);
        assert_eq!(platform, "unknown");
        assert_eq!(version, "could_not_find_platform_name_in_user_agent");

        // Malformed version number
        let malformed = TypedHeader(UserAgent::from_static("GuardianNews (android 34)"));
        let (platform, version) = user_agent_to_platform_and_version(&malformed);
        assert_eq!(platform, "android");
        assert_eq!(version, "unknown_version");

        // Missing app name and version number
        let malformed = TypedHeader(UserAgent::from_static("(android 34)"));
        let (platform, version) = user_agent_to_platform_and_version(&malformed);
        assert_eq!(platform, "android");
        assert_eq!(version, "unknown_version");

        // Missing app name
        let malformed = TypedHeader(UserAgent::from_static("123 (android 34)"));
        let (platform, version) = user_agent_to_platform_and_version(&malformed);
        assert_eq!(platform, "android");
        assert_eq!(version, "unknown_version");
    }
}
