# CoverNode mixing strategy and parameters

The CoverNode in our system design fulfills two tasks.
First, it unlinks[^1] the messages forwarded to the journalists from the original sender, hence ensuring anonymity.
Seconds, it filters out cover messages which make up the vast majority of ingress so that the dead-drops are of reasonable size.

## Operations and mixing strategy

The proposed mixing strategy works as a combination of a threshold and timing mix.
All incoming messages are first decrypted to determine whether they are real or cover messages.
Real messages are added to an in-memory FIFO queue called $queue$.
Cover messages are discarded.

The CoverNode release a new dead-drop when one of the following conditions is fulfilled:

- At least $threshold_{max}$ messages have been received since the last release
- At least $threshold_{min}$ messages have been received since the last release **and** at least $timeout$ time has passed since the last release

For each release up to $output_{size}$ many real messages are added from the $queue$ to the output $buffer$.
If the buffer is smaller than $output_{size}$, then additional cover messages are added so that it always has the same size.
The $buffer$ is shuffled and then transformed into a signed dead-drop.


[^1]: The unlinkability property is described here: https://dud.inf.tu-dresden.de/literatur/Anon_Terminology_v0.28.pdf
[^2]: See the following paper for an overview of mix types and potential attacks: https://apps.dtic.mil/sti/pdfs/ADA465475.pdf

## Parameter choice

For the **production deployment (PROD/AUDIT)** we choose the following parameters.
They allow handling large volumes of cover messages.

- U2J: User to journalist
  - $threshold_{min} = 100,000$ messages
  - $threshold_{max} = 500,000$ messages
  - $timeout = 1$ hour
  - $output_{size} = 500$ messages
- J2U: Journalist to user
  - $threshold_{min} = 50$ messages
  - $threshold_{max} = 100$ messages
  - $timeout = 1$ hour
  - $output_{size} = 20$ messages

For the **testing deployment (CODE/DEMO)** we choose the following parameters.
They allow easy testing as the dead-drops are released frequently.

- U2J: User to journalist
  - $threshold_{min} = 2$ messages
  - $threshold_{max} = 10$ messages
  - $timeout = 15$ minutes
  - $output_{size} = 10$ messages
- J2U: Journalist to user
  - $threshold_{min} = 10$ messages
  - $threshold_{max} = 40$ messages
  - $timeout = 15$ minutes
  - $output_{size} = 5$ messages

Note that for the testing deployment we set higher J2U than U2J values.
This is expected, as we will only have a few users, while the cover traffic service will run continuously.
For the release deployments, there are of course many more users.

The values are set here:
- PROD: [../infra/on-premises/overlays/prod/kustomization.yaml](../infra/on-premises/overlays/prod/kustomization.yaml)
- STAGING: [../infra/on-premises/overlays/staging/kustomization.yaml](../infra/on-premises/overlays/staging/kustomization.yaml)
- DEV: [../infra/on-premises/overlays/dev/kustomization.yaml](../infra/on-premises/overlays/dev/kustomization.yaml)
- Integration tests: [../integration-tests/src/images/covernode.rs](../integration-tests/src/images/covernode.rs)

## Testing with continuous cover traffic

In order to facilitate realistic testing, we run a dedicated Cover Traffic service that generates cover messages for both the U2J and J2U directions.

We set the following values:

- Production deployments (PROD/AUDIT)
  - U2J: 80000 messages per hour
  - J2U: 10 messages per hour
- Testing deployments (CODE/DEMO)
  - U2J: 10 messages per hour
  - J2U: 10 messages per hour

The values are set here: [../cdk/lib/cover-traffic.ts](../cdk/lib/cover-traffic.ts).

## Calculations

These calculations are taken from an [internal spreadsheet](https://docs.google.com/spreadsheets/d/19NbZTlYuZkjAzXuxck5lemCxEVlGwX22DBib6uUuaH0/edit#gid=0).

### Production

&nbsp;  | &nbsp;  | U2J | U2J | J2U | J2U
-- | -- | -- | -- | - | --
Parameter | Unit | Low | High | Low | High
  |   |   |   |   |  
**[IN] General parameters** |   |   |   |   |  
Active senders |   | 500,000 | 5,000,000 | 20 | 50
Messages per sender | messages/hour | 0.17 | 0.17 | 1.00 | 2.00
Real messages total | messages/hour | 50.00 | 100.00 | 5.00 | 15.00
Message size | bytes | 800 Bytes | 800 Bytes | 600 Bytes | 600 Bytes
  |   |   |   |   |  
**[IN] Threshold parameters** |   |   |   |   |  
$threshold_{min}$ | messages | 100,000 | 100,000 | 50 | 50
$threshold_{max}$ | messages | 500,000 | 500,000 | 100 | 100
$timeout$ | hour | 1.00 | 1.00 | 1.00 | 1.00
$output_{size}$ | messages | 500 | 500 | 20 | 20
  |   |   |   |   |  
**[OUT] Calculations** |   |   |   |   |  
Expected input rate | messages/hour | 83,333 | 833,333 | 20 | 100
Ratio real messages in input | percent | 0.06% | 0.01% | 25.00% | 15.00%
Firing rate | 1/hour | 0.83 | 1.67 | 0.40 | 1.00
Mean message delay | hour | 0.42 | 0.83 | 0.20 | 0.50
Expected output rate | messages/hour | 416.67 | 833.33 | 8.00 | 20.00
Ratio real messages in output | percent | 12.00% | 12.00% | 50.00% | 75.00%
  |   |   |   |   |  
**[OUT] Performance numbers** |   |   |   |   |  
Ingress | MiB/hour | 63.58 MiB/h | 635.78 MiB/h | 0.01 MiB/h | 0.06 MiB/h
Egress | MiB/hour | 0.32 MiB/h | 0.64 MiB/h | 0.00 MiB/h | 0.01 MiB/h
Receiver daily download burden | KiB/day | 7,813 KiB | 15,625 KiB | 113 KiB | 281 KiB

### Testing


&nbsp;  | &nbsp;  | U2J | U2J | J2U | J2U
-- | -- | -- | -- | -- | --
Parameter | Unit | Low | High | Low | High
  |   |   |   |   |  
**[IN] General parameters** |   |   |   |   |  
Active senders |   | 3 | 20 | 5 | 20
Messages per sender | messages/hour | 1.00 | 1.00 | 1.00 | 1.00
Real messages total | messages/hour | 2.00 | 2.00 | 2.00 | 2.00
Message size | bytes | 800 Bytes | 800 Bytes | 600 Bytes | 600 Bytes
  |   |   |   |   |  
**[IN] Threshold parameters** |   |   |   |   |  
$threshold_{min}$ | messages | 2 | 2 | 10 | 10
$threshold_{max}$ | messages | 10 | 10 | 40 | 40
$timeout$ | hour | 0.25 | 0.25 | 0.25 | 0.25
$output_{size}$ | messages | 10 | 10 | 5 | 5
  |   |   |   |   |  
**[OUT] Calculations** |   |   |   |   |  
Expected input rate | messages/hour | 3 | 20 | 5 | 20
Ratio real messages in input | percent | 66.67% | 10.00% | 40.00% | 10.00%
Firing rate | 1/hour | 1.50 | 4.00 | 0.50 | 2.00
Mean message delay | hour | 0.75 | 2.00 | 0.25 | 1.00
Expected output rate | messages/hour | 30.00 | 80.00 | 2.50 | 10.00
Ratio real messages in output | percent | 6.67% | 2.50% | 80.00% | 20.00%
  |   |   |   |   |  
**[OUT] Performance numbers** |   |   |   |   |  
Ingress | MiB/hour | 0.00 MiB/h | 0.02 MiB/h | 0.00 MiB/h | 0.01 MiB/h
Egress | MiB/hour | 0.02 MiB/h | 0.06 MiB/h | 0.00 MiB/h | 0.01 MiB/h
Receiver daily download burden | KiB/day | 563 KiB | 1,500 KiB | 35 KiB | 141 KiB
