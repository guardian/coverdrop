import { Message } from "../model/bindings/Message.ts";
import {
  EuiButton,
  EuiButtonEmpty,
  EuiCallOut,
  EuiDatePicker,
  EuiModal,
  EuiModalBody,
  EuiModalFooter,
  EuiModalHeader,
  EuiModalHeaderTitle,
  EuiRadioGroup,
} from "@elastic/eui";
import { Fragment, useCallback, useEffect, useMemo, useState } from "react";
import moment, { Moment } from "moment";
import { getChats, setCustomExpiry } from "../commands/chats.ts";
import { useMessageStore } from "../state/messages.ts";

interface CustomExpiryModalProps {
  close: () => void;
  maybeMessageToSetCustomExpiryFor: Message | null;
}

const DATETIME_FORMAT = "YYYY-MM-DD h:mmA";

const roundUpToNextHour = (date: Moment): Moment =>
  date.clone().add(1, "hour").startOf("hour");

export const CustomExpiryModal = ({
  close,
  maybeMessageToSetCustomExpiryFor,
}: CustomExpiryModalProps) => {
  if (!maybeMessageToSetCustomExpiryFor) {
    return null;
  }
  const message = maybeMessageToSetCustomExpiryFor;

  const messageStore = useMessageStore();

  const setMaybeCustomExpiry = useCallback(
    async (maybeNewExpiryDate: string | null) => {
      await setCustomExpiry(
        maybeMessageToSetCustomExpiryFor,
        maybeNewExpiryDate,
      );

      messageStore.setMessages(
        await getChats(), // reload messages from vault after mutation
      );

      close(); // close the modal after setting the expiry
    },
    [maybeMessageToSetCustomExpiryFor],
  );

  const nowMoment = useMemo(() => moment(), [maybeMessageToSetCustomExpiryFor]);
  const maxExpiryMoment = nowMoment.clone().add(90, "days");

  const normalExpiryMoment = moment(message.normalExpiry);

  const maybeExistingCustomExpiry = message.customExpiry
    ? moment(message.customExpiry)
    : null;

  const [maybeNewExpiryMoment, setMaybeNewExpiryMoment] =
    useState<Moment | null>(
      roundUpToNextHour(
        maybeExistingCustomExpiry || nowMoment.clone().add(7, "days"),
      ),
    );
  const safelySetMaybeNewExpiryMoment = (newDate: Moment) => {
    if (newDate.isBefore(nowMoment)) {
      setMaybeNewExpiryMoment(roundUpToNextHour(nowMoment));
    } else if (newDate.isAfter(maxExpiryMoment)) {
      setMaybeNewExpiryMoment(roundUpToNextHour(maxExpiryMoment));
    } else {
      setMaybeNewExpiryMoment(newDate);
    }
  };

  const choices = {
    ...(maybeExistingCustomExpiry
      ? {
          reset: {
            label: `Reset to normal expiry (${normalExpiryMoment.format(DATETIME_FORMAT)})`,
            onSelected: () => setMaybeNewExpiryMoment(null),
          },
        }
      : {}),
    threeDays: {
      label: `3 days from now`,
      onSelected: () =>
        safelySetMaybeNewExpiryMoment(
          roundUpToNextHour(nowMoment.clone().add(3, "days")),
        ),
    },
    sevenDays: {
      label: "7 days from now",
      onSelected: () =>
        safelySetMaybeNewExpiryMoment(
          roundUpToNextHour(nowMoment.clone().add(7, "days")),
        ),
    },
    fourteenDays: {
      label: "14 days from now",
      onSelected: () =>
        safelySetMaybeNewExpiryMoment(
          roundUpToNextHour(nowMoment.clone().add(14, "days")),
        ),
    },
    custom: {
      label: (
        <EuiDatePicker
          fullWidth
          selected={maybeNewExpiryMoment}
          onChange={safelySetMaybeNewExpiryMoment}
          onFocus={() => setSelectedChoiceId("custom")}
          dateFormat={DATETIME_FORMAT}
          showTimeSelect
          timeIntervals={60}
          minDate={nowMoment}
          maxDate={maxExpiryMoment}
          minTime={
            maybeNewExpiryMoment?.isAfter(nowMoment.clone().endOf("day"))
              ? moment().startOf("day")
              : nowMoment
          }
          maxTime={moment().endOf("day")}
        />
      ),
      onSelected: () => {},
    },
  } as const;

  const [selectedChoiceId, setSelectedChoiceId] =
    useState<keyof typeof choices>("sevenDays");

  useEffect(() => {
    choices[selectedChoiceId]?.onSelected();
  }, [selectedChoiceId]);

  // TODO consider adding the 'expiring soon' as a warning banner here

  const shouldExpireImmediatelyIfExtensionCleared =
    !maybeNewExpiryMoment && nowMoment.isAfter(normalExpiryMoment);

  return (
    <EuiModal onClose={close}>
      <EuiModalHeader>
        <EuiModalHeaderTitle>Custom expiry for message</EuiModalHeaderTitle>
      </EuiModalHeader>
      <EuiModalBody>
        {maybeExistingCustomExpiry ? (
          <div>
            <EuiCallOut
              size="s"
              title={`This message already has a custom expiry (${maybeExistingCustomExpiry.format(DATETIME_FORMAT)})`}
              iconType="clockCounter"
            />
            <br />
          </div>
        ) : (
          <div>
            <EuiCallOut
              size="s"
              title={`This message is due to expire at the normal time of ${normalExpiryMoment.format(DATETIME_FORMAT)}`}
              iconType="clock"
            />
            <br />
          </div>
        )}
        <EuiRadioGroup
          options={Object.entries(choices).map(([id, { label }]) => ({
            id,
            label,
          }))}
          idSelected={selectedChoiceId}
          onChange={(id) => setSelectedChoiceId(id as keyof typeof choices)}
        />

        <br />
        <EuiCallOut title="Important" color="warning" iconType="warning">
          Setting a custom expiry only affects your copy of the message.
          <br />
          <br />
          {message.type === "userToJournalistMessage" ? (
            <>
              The source&apos;s copy of this message will expire at its normal
              expiry time (some time before{" "}
              {normalExpiryMoment.format(DATETIME_FORMAT)}).
            </>
          ) : (
            <>
              The source&apos;s copy of this message will expire at its normal
              time, which will be 14 days after whenever they received it (some
              time after {normalExpiryMoment.format(DATETIME_FORMAT)}).
            </>
          )}
        </EuiCallOut>
        <br />
        <div>
          If you click Save below this message will then expire at{" "}
          {maybeNewExpiryMoment?.format(DATETIME_FORMAT) ||
            normalExpiryMoment.format(DATETIME_FORMAT)}
          .
        </div>
        {shouldExpireImmediatelyIfExtensionCleared && (
          <Fragment>
            <br />
            <EuiCallOut
              size="s"
              iconType="warning"
              color="danger"
              title="IMPORTANT: this means the message will expire immediately"
            />
          </Fragment>
        )}
      </EuiModalBody>
      <EuiModalFooter>
        <EuiButtonEmpty onClick={close}>Cancel</EuiButtonEmpty>
        <EuiButton
          fill
          color={
            shouldExpireImmediatelyIfExtensionCleared ? "danger" : "primary"
          }
          onClick={() =>
            setMaybeCustomExpiry(
              maybeNewExpiryMoment && maybeNewExpiryMoment.toISOString(),
            )
          }
        >
          Save
        </EuiButton>
      </EuiModalFooter>
    </EuiModal>
  );
};
