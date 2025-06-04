# Client Data Structures and Algorithms

## Encrypted Storage with Plausible Deniability

The encrypted storage with plausible deniability is implemented using the [Sloth library](https://github.com/lambdapioneer/sloth).
The choice of our parameters is described in [client_passphrase_configurations.md](client_passphrase_configurations.md).

#### Implementations

- [x] [Android](../android/core/src/main/java/com/theguardian/coverdrop/core/encryptedstorage/EncryptedStorageImpl.kt)
- [x] [iOS](../ios/reference/CoverDropCore/Sources/CoverDropCore/Crypto/EncryptedStorage.swift)


## Private Sending Queue

A Private Sending Queue is a data structure to store a mix of real and cover items. An adversary
cannot tell from a single snapshot how many real and how many cover items are included. However,
a caller that uses a consistent `secret` will be able to tell how many real messages are
currently stored. Also, it ensures that real messages that are `enqueue`d are placed before all
cover messages.

It has the following operations:

- `create(n)` which initializes a new private sending queue.
- `dequeue()` which removes and returns the front-most item and fills up the queue with a new cover message.
- `get_level(secret)` which returns the current number of real messages.
- `enqueue(secret, msg)` which enqueues a real message.

This is a novel construction that is not yet described in the paper.

#### Algorithm

The following is a sample implementation of a private sending queue in Python (pseudo-)code.
It requires that the HMAC variant used generates digests of 32 byte length that are indistinguishable from random bytes.

```python3
class PrivateSendingQueue:
    def __init__(self, n, item_size):
        """Creates a new PrivateSendingQueue that is initially filled with `size` cover items.
        """
        self.storage = [new_cover_message() for _ in range(n)]
        self.hints = [random_bytes(32) for _ in range(n)]
        self.n = n
        self.item_size = item_size

        self._assert_invariants()

    def dequeue(self):
        """Returns the front-most item of the queue. If there were any real messages in the buffer,
        they would be at the front and returned before any cover messages. Afterwards the buffer
        is filled up to `self.size` again.
        """
        item = self.storage.pop(0)
        self.storage.append(new_cover_message())

        self.hints.pop(0)
        self.hints.append(random_bytes(32))

        self._assert_invariants()
        return item

    def fill_level(self, secret):
        """Returns the current number of real messages. This requires that the same `secret` is used
        for both `fill_level` and `enqueue`.
        """
        fill = 0
        for msg, hint in zip(self.storage, self.hints):
            if hmac(secret, msg) != hint:
                break
            fill += 1
        return fill

    def enqueue(self, secret, msg):
        """Enqueues a new message. If the same `secret` is used for all calls to [enqueue], it
        guarantees that: (a) the real messages are returned FIFO and (b) they are returned before
        any cover messages.

        However, if different `secret` values are used, existing real messages are not detected and
        will be overwritten.
        """
        curr_fill = self.fill_level(secret)
        if curr_fill == len(self.storage):
            raise RuntimeError("no space")
        self.storage[curr_fill] = msg
        self.hints[curr_fill] = hmac(secret, msg)

        self._assert_invariants()

    def _assert_invariants(self):
        assert (len(self.storage) == self.n)
        assert (len(self.hints) == self.n)
        assert (all([len(m) == self.item_size for m in self.storage]))
```

#### Implementations

- [x] [Android](../android/core/src/main/java/com/theguardian/coverdrop/core/crypto/PrivateSendingQueue.kt)
- [x] [iOS](../ios/reference/CoverDropCore/Sources/CoverDropCore/Crypto/PrivateSendingQueue/PrivateSendingQueue.swift)
