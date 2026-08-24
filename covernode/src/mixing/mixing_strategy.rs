use crate::mixing::mixing_message_types::{MixingInputMessage, MixingOutputMessage};
use chrono::{DateTime, Duration, Utc};
use std::cmp::min;
use std::marker::PhantomData;

#[derive(Debug, PartialEq)]
pub struct MixingOutput<Output> {
    pub messages: Vec<Output>,
}

pub trait MixingStrategy<Input, Output> {
    fn consume_and_check_for_new_output(
        &mut self,
        message: Input,
        now: DateTime<Utc>,
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
    buffer: Vec<Output>,
}

impl<Output> MixingStrategyState<Output> {
    pub fn new(now: DateTime<Utc>) -> Self {
        Self {
            seen_messages: 0,
            buffer: Vec::new(),
            last_output_timestamp: now,
        }
    }

    pub fn reset(&mut self, now: DateTime<Utc>) {
        self.seen_messages = 0;
        self.last_output_timestamp = now;
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
    pub fn new(config: MixingStrategyConfiguration, now: DateTime<Utc>) -> Self {
        let state = MixingStrategyState::new(now);
        Self {
            config,
            state,
            marker: PhantomData,
        }
    }

    fn consume(&mut self, message: Input) {
        // increase total number of messages we have seen
        self.state.seen_messages += 1;

        metrics::counter!(
            self.config.metrics_name,
            "threshold_min" => self.config.metrics_threshold_min,
            "threshold_max" => self.config.metrics_threshold_max,
        )
        .absolute(self.state.seen_messages as u64);

        // if it is a real one, we keep it in our buffer
        if let Some(real_message_payload) = message.to_payload_if_real() {
            self.state.buffer.push(real_message_payload);
        }
    }

    fn maybe_next_output(&mut self, now: DateTime<Utc>) -> Option<MixingOutput<Output>> {
        // if we are not "ready" yet, return early with `None`
        if !self.should_create_output(now) {
            return None;
        }

        // collect the oldest real messages from the buffer
        let cut = min(self.config.output_size, self.state.buffer.len());
        let mut output_messages: Vec<Output> = self.state.buffer.drain(..cut).collect();

        // fill up with cover messages if necessary
        while output_messages.len() < self.config.output_size {
            output_messages.push(Output::generate_new_random_message());
        }

        // reset the current number of seen messages and last recorded timestamp
        self.state.reset(now);

        Some(MixingOutput {
            messages: output_messages,
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
    ) -> Option<MixingOutput<Output>> {
        self.consume(message);
        self.maybe_next_output(now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::time;
    use rand::random;

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
        let mut mixer = CoverDropMixingStrategy::new(get_test_config(), now);

        let in1 = TestMixingInputMessage::new_with_random_inner();
        assert_eq!(
            mixer.consume_and_check_for_new_output(in1.clone(), now),
            None
        );

        let in2 = TestMixingInputMessage::new_with_random_inner();
        assert_eq!(
            mixer.consume_and_check_for_new_output(in2.clone(), now),
            None
        );

        let in3 = TestMixingInputMessage::new_empty();
        assert_eq!(
            mixer.consume_and_check_for_new_output(in3.clone(), now),
            None
        );

        // The fourth message will hit the max causing the oldest two messages to be released
        let in4 = TestMixingInputMessage::new_with_random_inner();
        let output = mixer
            .consume_and_check_for_new_output(in4.clone(), now)
            .unwrap();
        assert_eq!(
            output.messages,
            vec![in1.inner.unwrap(), in2.inner.unwrap()]
        );

        // At this point only the fourth message is in the buffer; adding more empty ones will then
        // cause a new output
        let in5 = TestMixingInputMessage::new_empty();
        assert_eq!(
            mixer.consume_and_check_for_new_output(in5.clone(), now),
            None
        );
        let in6 = TestMixingInputMessage::new_empty();
        assert_eq!(
            mixer.consume_and_check_for_new_output(in6.clone(), now),
            None
        );
        let in7 = TestMixingInputMessage::new_empty();
        assert_eq!(
            mixer.consume_and_check_for_new_output(in7.clone(), now),
            None
        );

        let in8 = TestMixingInputMessage::new_empty();
        let output = mixer.consume_and_check_for_new_output(in8.clone(), now);

        // The output should have our oldest real message at the start and then padded with a
        // random one
        let output = output.unwrap();
        assert_eq!(&output.messages[0], &in4.inner.unwrap());
        assert_ne!(&output.messages[1], &output.messages[0]);
    }

    #[test]
    fn test_min_threshold_and_timeout_firing() {
        let mut now = time::now();
        let test_config = get_test_config();
        let mut mixer = CoverDropMixingStrategy::new(test_config, now);

        let in1 = TestMixingInputMessage::new_with_random_inner();
        assert_eq!(
            mixer.consume_and_check_for_new_output(in1.clone(), now),
            None
        );

        // Exceeding the threshold_min, but not the timeout
        let in2 = TestMixingInputMessage::new_with_random_inner();
        assert_eq!(
            mixer.consume_and_check_for_new_output(in2.clone(), now),
            None
        );

        // Exceeding the threshold_min AND the timeout
        now += test_config.timeout;
        let in3 = TestMixingInputMessage::new_empty();
        let output = mixer
            .consume_and_check_for_new_output(in3.clone(), now)
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
            mixer.consume_and_check_for_new_output(in4.clone(), now),
            None
        );

        // Meeting the threshold_min
        let in5 = TestMixingInputMessage::new_with_random_inner();
        let output = mixer
            .consume_and_check_for_new_output(in5.clone(), now)
            .unwrap();

        // The output should have our oldest real message at the start and then padded with a
        // random one
        assert_eq!(&output.messages[0], &in5.inner.unwrap());
        assert_ne!(&output.messages[1], &output.messages[0]);
    }

    #[test]
    fn test_only_cover_messages() {
        let now = time::now();
        let mut mixer = CoverDropMixingStrategy::new(get_test_config(), now);

        // send threshold_max cover messages
        let in1 = TestMixingInputMessage::new_empty();
        assert_eq!(
            mixer.consume_and_check_for_new_output(in1.clone(), now),
            None
        );

        let in2 = TestMixingInputMessage::new_empty();
        assert_eq!(
            mixer.consume_and_check_for_new_output(in2.clone(), now),
            None
        );

        let in3 = TestMixingInputMessage::new_empty();
        assert_eq!(
            mixer.consume_and_check_for_new_output(in3.clone(), now),
            None
        );

        // The fourth message will hit the max causing the oldest two messages to be released
        let in4 = TestMixingInputMessage::new_empty();
        let output = mixer.consume_and_check_for_new_output(in4, now).unwrap();

        // The output should have two random messages
        assert_eq!(output.messages.len(), 2);
        assert_ne!(&output.messages[0], &output.messages[1]);
    }
}
