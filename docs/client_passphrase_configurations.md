# Client Password Configuration

For the plausibly-deniable encryption we require the user to memorize a passphrase that is generated from words of the [EFF word list](https://www.eff.org/deeplinks/2016/07/new-wordlists-random-passphrases).
The large version that we use contains 7776 words.
Generating the passphrase for the user avoids the common security risk where the user chooses a weak passphrase.

We use two approaches.
For devices with a Secure Element we use techniques described in the [Sloth paper](https://eprint.iacr.org/2023/1792).
Sloth relies on operations within the Secure Element to effectively rate-limit guesses.
For devices without a Secure Element we use password-based key derivation with the memory-hard function [Argon2](https://en.wikipedia.org/wiki/Argon2).

All supported iOS devices have a Secure Element (called Secure Enclave) and therefore only use the Sloth techniques.
On Android most modern smartphones have a Secure Element.
However, there are also many devices without that we need to support using Argon2.

In all cases we want that an exhaustive search of the passphrase space takes at least 100 years plus reasonable safety margins (usually a factor of 10x).

## Choosing Parameters

For our calculations we assume a resourceful adversary.
They have access to a large compute cluster with 1000 TiB of RAM and many CPUs.
For comparision the [Cambridge HPC](https://www.hpc.cam.ac.uk/high-performance-computing ) has a total of ~500 TiB.
We further assume that each CPU can compute an Argon2 hash in 10ms and the parallelisation factor is limited by the required memory.

## Results

For the Sloth variants we pick:
- passphrase length of 3 words
- an expected derivation runtime of 1 second

Note that the underlying Sloth parameters (l for Android, n for iOS) follow from the expected derivation runtime.

For the Argon2 variant we pick:
- passphrase length of 5 words
- iteration count of 3
- memory requirement of 256 MiB

## Calculations

These calculations are taken from an [internal spreadsheet](https://docs.google.com/spreadsheets/d/1GMtV6nqRbO9KtL8vCnQPHSw9NkLPzMRRiELJG4BJToQ/edit#gid=0).


Direction | Parameter | Value | Unit
-- | -- | -- | --
 &nbsp; | **General parameters** |
IN | Word list length | 7,776 | words
IN | Sloth time | 1.0 | second
  |   |   |  
 &nbsp; | **Adversary assumptions** |   |  
IN | Cluster Argon2 time | 0.01 | seconds
IN | Cluster memory | 1,048,576,000 | MiB
  |   |   |  
  |   |   |  
 &nbsp; | **Sloth-based passphrase authentication** |   |  
IN | Passphrase | 3 | words
OUT | Passphrase combinations | 4.70E+11 |  
OUT | Passphrase entropy | 38.77 | bits
  |   |   |  
OUT | Exhaustive search | 4.70E+11 | seconds
OUT | Exhaustive search | 14,909 | years
  |   |   |  
 &nbsp; | **Argon2-based passphrase authentication** |   |  
IN | Passphrase | 5 | words
OUT | Passphrase combinations | 2.84E+19 |  
OUT | Passphrase entropy | 64.62 | bits
  |   |   |  
IN | Argon2 memory parameter | 256 | MiB
OUT | Adversary parallel factor | 4,096,000 |  
OUT | Adversary tries per second | 409,600,000 | 1/s
  |   |   |  
OUT | Exhaustive search | 6.94E+10 | seconds
OUT | Exhaustive search | 2,201 | years



