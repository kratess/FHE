# FHE VDAF Prototype 1

This crate implements a simpler VDAF-like prototype over OpenFHE BGV.
It is similar in shape to Prio3 because clients submit encrypted reports,
aggregators validate and aggregate them, and the collector combines aggregator
outputs to recover one aggregate result.

Unlike `fhe-vdaf-2`, this protocol does not split each value into shards.
Each client sends the whole encrypted input to exactly one aggregator.

## Protocol Shape

The demo uses:

```rust
const CLIENTS_NUM: usize = 2;
const AGGREGATORS_NUM: usize = 2;
```

Each client can submit any number of input values. For every input:

1. The client checks that the value is at most `MAX_SIZE`.
2. The value is encoded into binary BGV slots.
3. The packed value is encrypted.
4. A fixed-seed RNG chooses one aggregator to receive that encrypted input.

The assignment is random-looking but deterministic for now because the demo
uses a fixed `RNG_SEED`.

## Input Encoding

Each input is represented directly as a bounded binary vector:

```text
[ low binary bits | capped top bit ]
```

The number of value slots is derived from `MAX_SIZE`. Low slots use normal
binary weights. The last slot uses the remaining weight needed to make the
all-ones encoding decode exactly to `MAX_SIZE`.

For example, if `MAX_SIZE = 62`, the slot weights are:

```text
[1, 2, 4, 8, 16, 31]
```

So `62` is encoded as:

```text
[1, 1, 1, 1, 1, 1]
```

That decodes as:

```text
1 + 2 + 4 + 8 + 16 + 31 = 62
```

For example, the value `5` is encoded as:

```text
[1, 0, 1, 0, 0, 0, 0]
```

There is no signature field and no final validity slot in this protocol.

## Validator/Aggregator Work

Each aggregator receives encrypted full inputs. The aggregator acts as a
validator before adding an input to its aggregate.

For each encrypted input, it checks that every slot is binary by computing:

```text
s * (s - 1)
```

This is zero when `s` is `0` or `1`, and nonzero for invalid slot values.
The aggregator sums all slot errors into slot `0` using `EvalSum`:

```text
err_sum = sum(s_i * (s_i - 1))
```

Then it converts the encrypted error sum into an encrypted boolean using
Fermat's little theorem. For prime plaintext modulus `p`:

```text
x^(p - 1) = 0 when x = 0
x^(p - 1) = 1 when x != 0
```

The code computes:

```text
is_error = err_sum^(p - 1)
is_ok = 1 - is_error
```

`is_ok` is then replicated across every slot with rotations and masks.
The original encrypted input is multiplied by this replicated mask:

```text
checked_input = input * is_ok
```

If the input is valid, `is_ok = 1` and the ciphertext stays unchanged.
If the input is invalid, `is_ok = 0` and the whole ciphertext becomes zero.

After validation, the aggregator homomorphically adds all checked inputs it
received and returns one encrypted aggregate share to the collector.

## Collector Work

The collector receives encrypted aggregate shares from the aggregators. It does
not need individual client inputs.

The collector:

1. Homomorphically adds the aggregator aggregate shares.
2. Decrypts the final collector ciphertext.
3. Decodes the binary slots into an integer.
4. Checks that the decoded value equals the expected aggregate in the demo.

In the current demo values:

```text
client 0: 51, 7, 4
client 1: 49, 13
expected total = 124
```

## Similarity To Prio3

This prototype keeps the VDAF-style separation between clients, aggregators,
and collector:

- Clients submit encrypted reports.
- Aggregators validate reports and aggregate locally.
- The collector combines aggregator outputs.
- The final result is an aggregate, not a list of individual reports.

The main difference from Prio3 is that the whole input goes to one aggregator,
not a shard to every aggregator. Privacy in this prototype relies on the input
being encrypted under BGV and on aggregators not having the secret key.

## Current Limitations And Risks

This code is a prototype, not a complete protocol.

- Each full input goes to one aggregator. If that aggregator can decrypt, it
  can recover the full client input.
- The demo creates the secret key in `main` so it can print debug values and
  decode the final result. In a real deployment, validators/aggregators must
  not have access to the secret key.
- Aggregator assignment uses a fixed RNG seed for reproducibility. Real client
  routing would need fresh randomness or a defined routing policy.
- Validation only checks that slots are binary. It does not authenticate the
  client or prove that the report came from an allowed user.
- Invalid reports are silently zeroed before aggregation. This keeps aggregation
  moving, but it means invalid submissions reduce the aggregate instead of
  causing an explicit protocol failure.
- The Fermat validity conversion currently supports the plaintext moduli used
  in the code path, especially `786433`. Unsupported moduli panic.
- This prototype does not provide a security proof against malicious clients,
  malicious aggregators, replay, or report duplication.

## Running

OpenFHE must be installed and visible to `openfhe-bgv-rs`. The wrapper supports
these environment variables:

```text
OPENFHE_DIR
OPENFHE_INCLUDE_DIR
OPENFHE_LIB_DIR
OPENFHE_STATIC
```

Run the demo:

```sh
cargo run
```

Run the tests:

```sh
cargo test
```
