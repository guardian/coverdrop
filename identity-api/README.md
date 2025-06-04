# Identity API

The identity API exists to support rotating CoverNode and journalist identity keys.

This action requires access to provisioning secret keys which are very sensitive. As a result
the identity API runs on-premises.

## Rotating CoverNode keys

Since the CoverNode is within the same Kubernetes cluster as the identity API we can allow the
CoverNode to make direct requests to endpoints.

When it comes time for a CoverNode to rotate it's key it calls `public-keys/covernode/me/rotate-id-key`
with a signed form requesting a new key. The identity API then confirms the
signing key is from a valid CoverNode, submits this key to the API using a form signed
by the provisioning key and sends back the key and epoch value to the CoverNode.

## Rotating journalist keys

Since the journalists are not within the Kubernetes cluster and we don't want to
allow any inbound connections whatsoever the process for rotating a journalist key
is a little bit more complex.

First the journalist creates a key rotation request form, similar to the kind
that the CoverNode creates in it's direct request to the identity API. It then wraps
this in another form which is submitted to the normal API. Here it enters a queue
of rotation request forms.

While the form is still in the queue the journalist
will not use this new key pair for signing, but will store it as a candidate key pair.
This key pair can be used for verification (checking signatures) if the need arises.

The identity API periodically polls this queue, rotates any valid forms within the queue
and submits the newly signed identity public keys to the API.

The journalist client periodically polls the API to check if it's key has been rotated
and to get an epoch value for that key. Once it has an epoch for this key it can begin to
use the candidate identity key pair for signing.
