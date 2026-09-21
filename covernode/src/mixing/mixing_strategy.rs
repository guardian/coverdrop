use crate::mixing::mixing_message_types::{
    MixingInputMessage, MixingOutputMessage, SeenMessageHashes,
};
use chrono::{DateTime, Duration, Utc};
use covernode_database::{MessageHash, MessageHashExpiry, MessageHashesWithExpiries};
use std::cmp::min;
use std::collections::hash_map::Entry;
use std::marker::PhantomData;

#[derive(Debug, PartialEq)]
pub struct MixingOutput<Output> {
    pub messages: Vec<Output>,
    pub message_hashes: MessageHashesWithExpiries,
}

pub trait MixingStrategy<Input, Output> {
    fn consume_and_check_for_new_output(
        &mut self,
        message: Input,
        now: DateTime<Utc>,
        messaging_key_expiry: MessageHashExpiry,
    ) -> Option<MixingOutput<Output>>;
}

#[derive(Clone, Copy, Debug)]
pub struct MixingStrategyConfiguration {
    pub threshold_min: usize,
    pub threshold_max: usize,

    pub metrics_name: &'static str,
    pub metrics_threshold_min: &'static str,
    pub metrics_threshold_max: &'static str,

    pub timeout: Duration,
    pub output_size: usize,
}

impl MixingStrategyConfiguration {
    pub fn new(
        threshold_min: usize,
        threshold_max: usize,
        metrics_name: &'static str,
        timeout: Duration,
        output_size: usize,
    ) -> Self {
        let metrics_threshold_min = threshold_min.to_string().leak();
        let metrics_threshold_max = threshold_max.to_string().leak();

        Self {
            threshold_min,
            threshold_max,
            metrics_name,
            metrics_threshold_min,
            metrics_threshold_max,
            timeout,
            output_size,
        }
    }
}

struct MixingStrategyState<Output> {
    seen_messages: usize,
    last_output_timestamp: DateTime<Utc>,
    buffer: Vec<(Output, MessageHash, MessageHashExpiry)>,
    seen_message_hashes: SeenMessageHashes,
}

impl<Output> MixingStrategyState<Output> {
    pub fn new(now: DateTime<Utc>, seen_message_hashes: SeenMessageHashes) -> Self {
        Self {
            seen_messages: 0,
            buffer: Vec::new(),
            last_output_timestamp: now,
            seen_message_hashes,
        }
    }

    pub fn reset(&mut self, now: DateTime<Utc>) {
        self.seen_messages = 0;
        self.last_output_timestamp = now;
        self.seen_message_hashes
            .retain(|_, expiry| !expiry.is_expired_at(now));
    }
}

pub struct CoverDropMixingStrategy<Input, Output> {
    config: MixingStrategyConfiguration,
    state: MixingStrategyState<Output>,
    marker: PhantomData<Input>,
}

/// The `CoverDropMixingStrategy` fires if either
/// - the number of input images since the last output exceeds `threshold_max`
/// - OR the number of input images since the last output exceeds `threshold_min` AND at least
///   `timeout` much time passed since the last output
impl<Input, Output> CoverDropMixingStrategy<Input, Output>
where
    Input: MixingInputMessage<Output>,
    Output: MixingOutputMessage,
{
    pub fn new(
        config: MixingStrategyConfiguration,
        now: DateTime<Utc>,
        seen_message_hashes: SeenMessageHashes,
    ) -> Self {
        let state = MixingStrategyState::new(now, seen_message_hashes);
        Self {
            config,
            state,
            marker: PhantomData,
        }
    }

    fn consume(&mut self, message: Input, messaging_key_expiry: MessageHashExpiry) {
        // increase total number of messages we have seen
        self.state.seen_messages += 1;

        metrics::counter!(
            self.config.metrics_name,
            "threshold_min" => self.config.metrics_threshold_min,
            "threshold_max" => self.config.metrics_threshold_max,
        )
        .absolute(self.state.seen_messages as u64);

        // If the message is real
        if let Some(real_message_payload) = message.to_payload_if_real() {
            // Check if the hash is in the hash map, if not the message is new.
            // If it's new, we add it to the hash map and put it on to the buffer.
            if let Entry::Vacant(entry) = self
                .state
                .seen_message_hashes
                .entry(real_message_payload.to_hash())
            {
                self.state
                    .buffer
                    .push((real_message_payload, *entry.key(), messaging_key_expiry));
                entry.insert(messaging_key_expiry);
            }
        }
    }

    fn maybe_next_output(&mut self, now: DateTime<Utc>) -> Option<MixingOutput<Output>> {
        // if we are not "ready" yet, return early with `None`
        if !self.should_create_output(now) {
            return None;
        }

        // collect the oldest real messages from the buffer
        let cut = min(self.config.output_size, self.state.buffer.len());

        let mut message_hashes = Vec::<(MessageHash, MessageHashExpiry)>::new();
        let mut output_messages = Vec::with_capacity(self.config.output_size);

        for (message, message_hash, message_hash_expiry) in self.state.buffer.drain(..cut) {
            message_hashes.push((message_hash, message_hash_expiry));
            output_messages.push(message);
        }

        // fill up with cover messages if necessary
        while output_messages.len() < self.config.output_size {
            output_messages.push(Output::generate_new_random_message());
        }

        // reset the current number of seen messages and last recorded timestamp
        // also drop hashes from messages encrypted with expired keys.
        self.state.reset(now);

        Some(MixingOutput {
            messages: output_messages,
            message_hashes,
        })
    }

    fn should_create_output(&self, now: DateTime<Utc>) -> bool {
        // Case 1: number of seen messages meets the maximum threshold
        if self.state.seen_messages >= self.config.threshold_max {
            return true;
        }

        // Case 2: number of seen messages meets the minimum threshold AND enough time has passed
        let since_last_output = now - self.state.last_output_timestamp;
        if (self.state.seen_messages >= self.config.threshold_min)
            && (since_last_output >= self.config.timeout)
        {
            return true;
        }

        // Otherwise:
        false
    }
}

impl<Input, Output> MixingStrategy<Input, Output> for CoverDropMixingStrategy<Input, Output>
where
    Input: MixingInputMessage<Output>,
    Output: MixingOutputMessage,
{
    fn consume_and_check_for_new_output(
        &mut self,
        message: Input,
        now: DateTime<Utc>,
        messaging_key_expiry: MessageHashExpiry,
    ) -> Option<MixingOutput<Output>> {
        self.consume(message, messaging_key_expiry);
        self.maybe_next_output(now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::time;
    use rand::random;
    use sha2::Digest;

    fn get_test_config() -> MixingStrategyConfiguration {
        MixingStrategyConfiguration {
            threshold_min: 2,
            threshold_max: 4,

            metrics_name: "test_name",
            metrics_threshold_min: "2",
            metrics_threshold_max: "4",
            output_size: 2,
            timeout: Duration::seconds(60),
        }
    }

    /// Test implementation of our MixingOutputMessage trait for simpler testing
    #[derive(Debug, Clone, PartialEq)]
    pub struct TestMixingOutputMessage {
        pub(crate) content: [u8; 8],
    }

    impl MixingOutputMessage for TestMixingOutputMessage {
        fn generate_new_random_message() -> Self {
            TestMixingOutputMessage { content: random() }
        }

        fn to_hash(&self) -> MessageHash {
            let mut hasher = sha2::Sha256::new();
            hasher.update(self.content);
            MessageHash::new(hasher.finalize().into())
        }
    }

    /// Test implementation of our MixingInputMessage trait for simpler testing
    #[derive(Debug, Clone, PartialEq)]
    pub struct TestMixingInputMessage {
        pub(crate) inner: Option<TestMixingOutputMessage>,
    }

    impl TestMixingInputMessage {
        pub fn new_with_random_inner() -> Self {
            Self {
                inner: Some(MixingOutputMessage::generate_new_random_message()),
            }
        }
        pub fn new_empty() -> Self {
            Self { inner: None }
        }
    }

    impl MixingInputMessage<TestMixingOutputMessage> for TestMixingInputMessage {
        fn to_payload_if_real(self) -> Option<TestMixingOutputMessage> {
            self.inner
        }
    }

    #[test]
    fn test_max_threshold_firing() {
        let now = time::now();
        let messaging_key_expiry = MessageHashExpiry::new(now + Duration::days(1));
        let mut mixer =
            CoverDropMixingStrategy::new(get_test_config(), now, SeenMessageHashes::new());

        let in1 = TestMixingInputMessage::new_with_random_inner();
        assert_eq!(
            mixer.consume_and_check_for_new_output(in1.clone(), now, messaging_key_expiry),
            None
        );

        let in2 = TestMixingInputMessage::new_with_random_inner();
        assert_eq!(
            mixer.consume_and_check_for_new_output(in2.clone(), now, messaging_key_expiry),
            None
        );

        let in3 = TestMixingInputMessage::new_empty();
        assert_eq!(
            mixer.consume_and_check_for_new_output(in3.clone(), now, messaging_key_expiry),
            None
        );

        // The fourth message will hit the max causing the oldest two messages to be released
        let in4 = TestMixingInputMessage::new_with_random_inner();
        let output = mixer
            .consume_and_check_for_new_output(in4.clone(), now, messaging_key_expiry)
            .unwrap();
        assert_eq!(
            output.messages,
            vec![in1.inner.unwrap(), in2.inner.unwrap()]
        );

        assert_eq!(output.message_hashes.len(), 2);

        // At this point only the fourth message is in the buffer; adding more empty ones will then
        // cause a new output
        let in5 = TestMixingInputMessage::new_empty();
        assert_eq!(
            mixer.consume_and_check_for_new_output(in5.clone(), now, messaging_key_expiry),
            None
        );
        let in6 = TestMixingInputMessage::new_empty();
        assert_eq!(
            mixer.consume_and_check_for_new_output(in6.clone(), now, messaging_key_expiry),
            None
        );
        let in7 = TestMixingInputMessage::new_empty();
        assert_eq!(
            mixer.consume_and_check_for_new_output(in7.clone(), now, messaging_key_expiry),
            None
        );

        let in8 = TestMixingInputMessage::new_empty();
        let output = mixer.consume_and_check_for_new_output(in8.clone(), now, messaging_key_expiry);

        // The output should have our oldest real message at the start and then padded with a
        // random one
        let output = output.unwrap();
        assert_eq!(&output.messages[0], &in4.inner.unwrap());
        assert_ne!(&output.messages[1], &output.messages[0]);
        assert_eq!(output.message_hashes.len(), 1)
    }

    #[test]
    fn test_min_threshold_and_timeout_firing() {
        let mut now = time::now();
        let messaging_key_expiry = MessageHashExpiry::new(now + Duration::days(1));
        let test_config = get_test_config();
        let mut mixer = CoverDropMixingStrategy::new(test_config, now, SeenMessageHashes::new());

        let in1 = TestMixingInputMessage::new_with_random_inner();
        assert_eq!(
            mixer.consume_and_check_for_new_output(in1.clone(), now, messaging_key_expiry),
            None
        );

        // Exceeding the threshold_min, but not the timeout
        let in2 = TestMixingInputMessage::new_with_random_inner();
        assert_eq!(
            mixer.consume_and_check_for_new_output(in2.clone(), now, messaging_key_expiry),
            None
        );

        // Exceeding the threshold_min AND the timeout
        now += test_config.timeout;
        let in3 = TestMixingInputMessage::new_empty();
        let output = mixer
            .consume_and_check_for_new_output(in3.clone(), now, messaging_key_expiry)
            .unwrap();
        assert_eq!(
            output.messages,
            vec![in1.inner.unwrap(), in2.inner.unwrap()]
        );

        // As a result of the output, the internal state's counter and last timestamp get reset

        // Exceeding the timeout, but not the threshold_min
        now += test_config.timeout;
        let in4 = TestMixingInputMessage::new_empty();
        assert_eq!(
            mixer.consume_and_check_for_new_output(in4.clone(), now, messaging_key_expiry),
            None
        );

        // Meeting the threshold_min
        let in5 = TestMixingInputMessage::new_with_random_inner();
        let output = mixer
            .consume_and_check_for_new_output(in5.clone(), now, messaging_key_expiry)
            .unwrap();

        // The output should have our oldest real message at the start and then padded with a
        // random one
        assert_eq!(&output.messages[0], &in5.inner.unwrap());
        assert_ne!(&output.messages[1], &output.messages[0]);
    }

    #[test]
    fn test_only_cover_messages() {
        let now = time::now();
        let messaging_key_expiry = MessageHashExpiry::new(now + Duration::days(1));
        let mut mixer =
            CoverDropMixingStrategy::new(get_test_config(), now, SeenMessageHashes::new());

        // send threshold_max cover messages
        let in1 = TestMixingInputMessage::new_empty();
        assert_eq!(
            mixer.consume_and_check_for_new_output(in1.clone(), now, messaging_key_expiry),
            None
        );

        let in2 = TestMixingInputMessage::new_empty();
        assert_eq!(
            mixer.consume_and_check_for_new_output(in2.clone(), now, messaging_key_expiry),
            None
        );

        let in3 = TestMixingInputMessage::new_empty();
        assert_eq!(
            mixer.consume_and_check_for_new_output(in3.clone(), now, messaging_key_expiry),
            None
        );

        // The fourth message will hit the max causing the oldest two messages to be released
        let in4 = TestMixingInputMessage::new_empty();
        let output = mixer
            .consume_and_check_for_new_output(in4, now, messaging_key_expiry)
            .unwrap();

        // The output should have two random messages
        assert_eq!(output.messages.len(), 2);
        assert_ne!(&output.messages[0], &output.messages[1]);
        assert!(output.message_hashes.is_empty());
    }

    #[test]
    fn test_remove_duplicate_messages() {
        let now = time::now();
        let messaging_key_expiry = MessageHashExpiry::new(now + Duration::days(1));
        let config = MixingStrategyConfiguration {
            threshold_max: 500,
            ..get_test_config()
        };
        let mut mixer = CoverDropMixingStrategy::new(config, now, SeenMessageHashes::new());

        let duplicate_message = TestMixingInputMessage::new_with_random_inner();

        // Duplicate messages should be deduped
        for _i in 0..100 {
            mixer.consume_and_check_for_new_output(
                duplicate_message.clone(),
                now,
                messaging_key_expiry,
            );
        }

        // Cover messages should be ignored
        for _i in 0..100 {
            mixer.consume_and_check_for_new_output(
                TestMixingInputMessage::new_empty(),
                now,
                messaging_key_expiry,
            );
        }

        assert_eq!(mixer.state.seen_message_hashes.len(), 1);
        assert_eq!(mixer.state.buffer.len(), 1);
        assert_eq!(mixer.state.seen_messages, 200);
    }

    #[test]
    fn test_keep_unique_messages() {
        let now = time::now();
        let messaging_key_expiry = MessageHashExpiry::new(now + Duration::days(1));
        let config = MixingStrategyConfiguration {
            threshold_max: 500,
            ..get_test_config()
        };
        let mut mixer = CoverDropMixingStrategy::new(config, now, SeenMessageHashes::new());

        for _i in 0..100 {
            mixer.consume_and_check_for_new_output(
                TestMixingInputMessage::new_with_random_inner(),
                now,
                messaging_key_expiry,
            );
        }

        assert_eq!(mixer.state.seen_message_hashes.len(), 100);
        assert_eq!(mixer.state.buffer.len(), 100);
        assert_eq!(mixer.state.seen_messages, 100);
    }

    #[test]
    fn test_expire_old_seen_message_hashes() {
        let now = time::now();

        // Messaging key expired yesterday
        let messaging_key_expiry = MessageHashExpiry::new(now - Duration::days(1));

        let config = get_test_config();
        let mut mixer = CoverDropMixingStrategy::new(config, now, SeenMessageHashes::new());

        for _i in 0..3 {
            mixer.consume_and_check_for_new_output(
                TestMixingInputMessage::new_with_random_inner(),
                now,
                messaging_key_expiry,
            );
        }

        // Messaging key will expire in five minutes
        let messaging_key_expiry = MessageHashExpiry::new(now + Duration::minutes(5));

        let test_non_expired_message = TestMixingInputMessage::new_with_random_inner();

        // Two weeks elapse
        let now = now + Duration::weeks(2);

        mixer.consume_and_check_for_new_output(
            test_non_expired_message.clone(),
            now,
            messaging_key_expiry,
        );

        // 4 messages creates a dead drop and triggers a state reset
        assert_eq!(mixer.state.seen_messages, 0);
        // 3 messages with an expired messaging key and 1 without
        assert_eq!(mixer.state.seen_message_hashes.len(), 1);
        // Confirm the message is the one which doesn't have an expired key
        assert!(mixer
            .state
            .seen_message_hashes
            .contains_key(&test_non_expired_message.inner.unwrap().to_hash()))
    }

    #[test]
    fn test_deduplicate_seen_messages_using_passed_hash_map() {
        let config = MixingStrategyConfiguration {
            threshold_max: 500,
            ..get_test_config()
        };
        let now = time::now();
        let test_non_expired_message = TestMixingInputMessage::new_with_random_inner();
        let test_non_expired_message_hash =
            test_non_expired_message.inner.as_ref().unwrap().to_hash();

        let messaging_key_expiry = MessageHashExpiry::new(now + Duration::days(1));

        let mut seen_hashes = SeenMessageHashes::new();
        seen_hashes.insert(test_non_expired_message_hash, messaging_key_expiry);

        let mut mixer = CoverDropMixingStrategy::new(config, now, seen_hashes);

        for _i in 0..4 {
            mixer.consume_and_check_for_new_output(
                test_non_expired_message.clone(),
                now,
                messaging_key_expiry,
            );
        }

        // 4 messages all of which are duplicates of the previously seen hash
        assert!(mixer.state.buffer.is_empty());
        assert_eq!(mixer.state.seen_messages, 4);
        assert_eq!(mixer.state.seen_message_hashes.len(), 1);
        // Confirm the message hash is the one we started with
        assert!(mixer
            .state
            .seen_message_hashes
            .contains_key(&test_non_expired_message_hash));
    }
}
