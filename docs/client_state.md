# Client Data structures and state management

This doc outlines the data structures and state we need to store in the native apps, for each piece of data we want to know:
- where it should be stored - either **disk** or in **memory**, 
- if it is available in the logged in 🔐 or logged out 🔓 state (or both)
- other requirements like when it should be fetched etc

We also organise our data in groups depending on its access availability

## Public
Data in the public group is alway available to the user

### 🔓 PublicKeys data / Message recipient
- This data contains all the public keys required to send messages towards the journalists via the CoverNode, this needs to be requested in a deniable way ie not on user interaction. 
- These need to be requested on app startup. 
- This should be cached locally on **disk** and at every app start we can check if its older than e.g. 24h and re-fetched.
- This should be available in **memory**to the view
- This is available both logged in and logged out of a CoverDrop session

### 🔓 Downloaded Dead Drops
- The dead drops are downloaded by the app in the background (e.g. initiated during app start)
- They are stored in a database and the download logic makes sure that the last 7 days are available
- This is available both logged in and logged out of a CoverDrop session, but only messages to the user are available unencrypted in memory when the user in logged in

### 🔐 Session state
- The user enters a secure session after they have entered their passphrase, we need to store this session state i.e. logged in or not.
- This is available both logged in and logged out of a CoverDrop session
- This is stored in **memory** 

## Secret
Data in the secret group is only available when the user is logged in.
All data in this group is stored using [Encrypted Storage](client_data_structures_and_algorithms.md#encrypted-storage-with-plausible-deniability) on disk with the help of a key in the secure element

### 🔐 Message Mailbox
- The messages the user composes, this state needs to be stored locally, but also encrypted in the Mailbox storage.
- The Mailbox needs to be stored locally on **disk** , but read into memory, and written back to disk when another message is sent or a new message is received
- It’s ok to store the message unencrypted in **memory** as its being composed (ie the UI bound state)
- The sent message is stored encrypted using the [Encrypted Storage](client_data_structures_and_algorithms.md#encrypted-storage-with-plausible-deniability) on disk with the help of a key in the secure element
- The sent messages are copied into the Sending Queue and also stored in the Mailbox
- Messages in the message mailbox expiry after 7 days and are deleted from disk.
- This is only available logged in to a CoverDrop session

### 🔐Passphrase / User Key `k_user`
- This is generated for the user when they try and start a CoverDrop session
- A User Key is derived from the Passphrase
- This User Key in kept in **memory** during the CoverDrop session
- This is only available logged in to a CoverDrop session


### 🔐 Sending queue key 
- This is the key that is used to encrypt the sending queue
- The key is only available when the user in logged into the CoverDrop session


## Secure Enclave
Data in the Secure Enclave group is stored on the devices Secure Enclave / Secret Element

### 🔐Secure Element Key `k_se`
- This is generated /  stored in the Secure Enclave.
- https://developer.apple.com/documentation/security/certificate_key_and_trust_services/keys/protecting_keys_with_the_secure_enclave
- The passphrase is used in combination with the stored Secure Element Key to encrypt the inner covernode message.
- This is only available logged in to a CoverDrop session

## Private Sending Queue
Data in the Private Sending Queue can be dequeued (ie a message is removed from the queue for sending) when the user is logged out from the CoverDrop session and this happens as a backgroud process. 
Real messages are only added within the CoverDrop session and we use the Sending Queue key to add the message at the front most position of the queue. See [Client Data structures](client_data_structures_and_algorithms.md#private-sending-queue) for more details


### 🔓 [Sending queue](client_data_structures_and_algorithms.md#private-sending-queue)
- Contains the Real and Fake CoverNode Messages encrypted to be sent out
- This is stored on **disk** 
- Has a fixed size and filled-up with dummy data
- See https://github.com/guardian/CoverDrop/pull/145 
- This is only available logged in to a CoverDrop session
- The internal state can be inspected with a secret that is available only within the CoverDrop session

## Session state transitions
 
### 🔐 CoverDrop Session start
- When a user enters their passphrase they start a CoverDrop session
- The user has access to Message Mailbox which contains sent messages (this is separate from the Sending queue), we scan the Downloaded Dead Drops for received messages.
- The User Key is also kept in memory for the duration of the CoverDrop session
- We could store the start and expiry time of the session, if we want time based session access (ie  my session lasts 3 hours until I need to re-enter my passphrase)
- We retrieve the secret for the Private Sending Queue
- We set the session state to logged in
 
## 🔐 CoverDrop Session end
- We remove the Passphrase / User Key from memory
- We remove the message Mailbox from memory
- We set the session state to logged out
- The session ends when the user navigates away from the private views
- The session ends after the expiry time has been reached
- The session ends when the user presses the exit button
- The session ends when the user backgrounds the app
