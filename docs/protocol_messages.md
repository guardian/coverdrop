# Protocol Messages

In the CoverDrop protocol messages are exchanged between users and journalists via the CoverNode.
For this purpose the inner messages are wrapped into an outer message that's decrypted inside the CoverNode.

We call a message from the user to the journalist `UserToJournalistMessage` (U2J).
These are wrapped in a `UserToCoverNodeMessage` (U2C) for the first leg.
Then the CoverNode decrypts it and, if real, forwards it to the Journalist wrapped in a `CoverNodeToJournalistMessage`.
Similarly, for the other direction we have `JournalistToUserMessage` (J2U) and `JournalistToCoverNodeMessage` (J2C).
In this direction there no extra wrapping is needed after the CoverNode.

This schema gives an overview of the flow of messages:

```
                         MAB                                        TPB
+--------+     +-------------------- +     +--------+     +---------------------+     +--------+
|        |     |             AB      |     |        |     |     AB              |     |        |
|        |     |          +-------+  |     |        |     |  +-------+          |     |        |
|  USER  | +-> | U2C MSG  |U2J MSG|  | +-> | COVER  | +-> |  |U2J MSG|  C2J MSG | +-> | JOURNO |
|        |     |          +-------+  |     | NODE   |     |  +-------+          |     |        |
|        |     |                     |     |        |     |                     |     |        |
|        |     +-------------------- +     |        |     +---------------------+     |        |
|        |                                 |        |                                 |        |
|        |                                 |        |               MAB               |        |
|        |                                 |        |     +---------------------+     |        |
|        |                     TPB         |        |     |             TPB     |     |        |
|        |                  +-------+      |        |     |          +-------+  |     |        |
|        | <-+              |J2U MSG| <-+  |        | <-+ | J2C MSG  |J2U MSG|  | <-+ |        |
|        |                  +-------+      |        |     |          +-------+  |     |        |
|        |                                 |        |     |                     |     |        |
+--------+                                 +--------+     +---------------------+     +--------+
```

### Key

#### Cryptographic Primitives

AB = [Anonymous Box](cryptography.md#anonymous-box)

MAB = [Multi Anonymous Box](cryptography.md#multi-anonymous-box)

TPB = [Two Party Box](cryptography.md#two-party-box)

## User to Journalist Message

For the messages from the user to the journalist we (1) do not require the journalist to know the user's public key
beforehand and (2) we need to include the user's public key inside the ciphertext.
The `UserToJournalistMessage` is an [Anonymous Box](cryptography.md#anonymous-box) around an inner struct containing the
user's public key, and a [Padded Compressed String](cryptography.md#padded-compressed-string).

Between the public key and padded compressed string there is a reserved byte which can serve as a flag if we ever need to
add different kinds of user to journalist messages.

The padding length is set to 512 Bytes.

The encoded representation looks like the following:

```
                             +---------------------+
                             | User Message        |
                             +---------------------+

                             +----------+----------+
                                        v

            +------------+---+--------------------------+
            | Public Key | R | Padded Compressed String |
            +------------+---+--------------------------+

            +----------------+--------------------------+
                             v

+-----+-----+-------------------------------------------+
| epk | tag |               Ciphertext                  |   Anonymous Box
+-----+-----+-------------------------------------------+
```

## User to CoverNode Message

For sending messages via the CoverNode, we wrap it with an outer message that also caries a 4 byte long tag that
identifies the recipient or indicates that is a cover message.
The `UserToCoverNodeMessage` is a [Multi Anonymous Box](cryptography.md#multi-anonymous-box) around an inner struct containing a
flag and the inner `UserToJournalistMessage`.

The recipient tag is either `[0x00,0x00,0x00,0x00]` for cover messages or the SHA-256 hash of
the recipient's identifier truncated to the first 4 bytes.

The encoded representation looks like the following:

```
                        +---------------+---------------------------------------+
                        | recipient_tag |       User to Journalist Message      |
                        +---------------+---------------------------------------+

                        +---------------------------+---------------------------+
                                                    v

+------+---+------+-----+-------------------------------------------------------+
| ab_1 |...| ab_r | tag |                     Ciphertext                        |   Multi Anonymous Box
+------+---+------+-----+-------------------------------------------------------+
```

## CoverNode to Journalist Message

For sending a message from the CoverNode to the journalist, we wrap it with an outer message.
Multiple such messages are collected into a dead drop as explained in [API Messages](api_messages#journalist-dead-drop).
The additional layer of encryption controlled by the CoverNode prevents users from observing their own messages in the
journalist dead drop and make active attacks more difficult.
The `CoverNodeToJournalistMessage` is a [Two Party Box](cryptography.md#two-party-box) around the
inner `UserToJournalistMessage`.

The encoded representation looks like the following:

```
      +---------------------------------------------------+
      |           User to Journalist Message              |
      +---------------------------------------------------+

      +-----------------------+---------------------------+
                              v

+-----+---------------------------------------------------+-------+
| tag |      Ciphertext                                   | nonce |   Two Party Box
+-----+---------------------------------------------------+-------+
```

## Journalist to User Message

The `JournalistToUserMessage` is a [Two Party Box](cryptography.md#two-party-box) which can contain different types of message.
The type of message is distinguished using a byte, where `0` indicates a normal text message, `1` indicates a hand over message, etc.
The message body is always the same size, regardless of the message type.

The currently available message types are:

-   Message: containing a 512 byte [Padded Compressed String](cryptography.md#padded-compressed-string).
-   Handover: containing the ID of the target journalist that the user is being handed over to. Unused bytes are set to `0x00`, padding out the payload to 512 bytes to match the other message types.

The encoded representation of a plain message looks like the following:

```
          +---------------------+
          | Journalist Message  |
          +---------------------+

          +----------+----------+
                     v

      +---+--------------------------+
      | 0 | Padded Compressed String |
      +---+--------------------------+

      +-------------+----------------+
                    v

+-----+------------------------------+-------+
| tag |          Ciphertext          | nonce |   Two Party Box
+-----+------------------------------+-------+
```

## Journalist to CoverNode Message

For sending messages via the CoverNode, we wrap it with an outer message that can also indicate whether it is a real or
fake message.
The `JournalistToCoverNodeMessage` is a [Multi Anonymous Box](cryptography.md#multi-anonymous-box) around an inner struct
containing a flag and the
inner `JournalistToJournalistMessage`.
The flag is `0x01` for real messages and `0x00` for cover messages.

The encoded representation looks like the following:

```
                        +------+---------------------------------------------------+
                        | flag |           Journalist to User Message              |
                        +------+---------------------------------------------------+

                        +---------------------------+------------------------------+
                                                    v

+------+---+------+-----+----------------------------------------------------------+
| ab_1 |...| ab_r | tag |                     Ciphertext                           |   Multi Anonymous Box
+------+---+------+-----+----------------------------------------------------------+
```
