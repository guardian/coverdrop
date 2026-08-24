# covernode

The CoverNode service runs on premises and operates as a threshold mix node, providing anonymity for users towards network adversaries.
It decrypts the outer (U2C) layer of the incoming messages and then adds them to the mixing process.
The CoverNode cannot decrypt the inner (U2J) layer containing the end-to-end encrypted message between the user and the journalist.
This mixing process then yields dead drops which are signed and published via the [API](../api/README.md).
Journalists later download these dead drops to find new messages from users.
The opposite direction, from journalists to users, works analogously.
