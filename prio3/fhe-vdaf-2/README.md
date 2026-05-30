# FHE VDAF Prototype

This crate is a prototype VDAF-like aggregation flow built on OpenFHE BGV.
It is similar in spirit to Prio3: clients split an input into verifier shares,
aggregators process their shares independently, and the collector combines the
aggregated outputs to recover only the aggregate.

The difference from a normal Prio3-style implementation is that the shares are
encrypted with BGV. Aggregators operate on ciphertexts, so they do not need to
decrypt individual client submissions in order to aggregate them.

## Protocol Shape

The demo currently uses two aggregators:

```rust
const AGGREGATORS_NUM: usize = 2;
```

For each client value:

1. The client checks that the value is at most `MAX_SIZE`.
2. The value is scaled by the number of aggregators.
3. The scaled value is split into bounded additive shares.
4. Each share is bit-decomposed into packed BGV plaintext slots.
5. A deterministic signature-like bit string is appended to every share.
6. A final slot is reserved for the aggregator validity result.
7. Each packed share is encrypted and sent to one aggregator.

The collector later adds the encrypted aggregate shares and rescales by
`AGGREGATORS_NUM` to recover the original aggregate.

## Sharding

The client does not send the input directly to any aggregator. It sends one
encrypted shard per aggregator.

For a value `x`, the code computes:

```text
scaled = x * AGGREGATORS_NUM
```

Then it chooses bounded shard values whose sum is `scaled`. With two
aggregators, the two shard values add up to `2x`. The collector later divides
the final sum by `2`.

This means that even if an aggregator could decrypt one shard, that shard is
not the original input. It is only one additive component of the scaled input.
The individual shard still leaks some information if decrypted, especially
because it is bounded, but it is not enough by itself to reconstruct the exact
client value without the other shard.

In the intended FHE setting, aggregators do not receive the secret key and
therefore cannot decrypt their shard ciphertexts at all. They only evaluate BGV
operations over encrypted slots.

## Packed Shard Layout

Each encrypted shard is packed into BGV SIMD slots:

```text
[ value bits | signature bits | validity slot ]
```

The value portion uses `MAX_BITS` slots derived from
`MAX_PER_AGGREGATORS_UNITS`. Low slots store normal binary bits. The last value
slot uses the remaining weight needed to make the all-ones value encoding
decode exactly to `MAX_PER_AGGREGATORS_UNITS`.

For example, if `MAX_PER_AGGREGATORS_UNITS = 62`, the value-slot weights are:

```text
[1, 2, 4, 8, 16, 31]
```

So shard value `62` is encoded as:

```text
[1, 1, 1, 1, 1, 1]
```

The signature portion has `RANDOM_BITS_LEN` slots. These bits are used by the
collector to check that all aggregators are aggregating matching client packs.

The final slot is reserved for validity. Aggregators write `1` when the shard
passes the encrypted bit check and `0` when it fails.

## Validity Calculation

The aggregator computes validity directly over the encrypted shard. It never
needs to inspect plaintext slots.

First, it checks that every packed slot is binary. For each encrypted slot `s`,
the aggregator evaluates:

```text
s * (s - 1)
```

This expression is `0` when `s` is `0` or `1`, and nonzero for other values.
The aggregator computes this for all packed slots, then uses `EvalSum` to add
the per-slot errors into slot `0`.

The result is an encrypted error sum:

```text
err_sum = sum(s_i * (s_i - 1))
```

If `err_sum` is zero, the shard passed the bit check. If it is nonzero, at
least one slot was invalid.

To convert this encrypted value into a boolean flag, the code uses Fermat's
little theorem over the plaintext field. For prime plaintext modulus `p`:

```text
x^(p - 1) = 0 when x = 0
x^(p - 1) = 1 when x != 0
```

So the aggregator computes:

```text
is_error = err_sum^(p - 1)
is_ok = 1 - is_error
```

The implementation has optimized exponent paths for the supported plaintext
moduli:

- `65537`, where `p - 1 = 2^16`
- `786433`, where `p - 1 = 3 * 2^18`

After computing `is_ok`, the aggregator uses plaintext masks and rotations to
apply it to the shard:

- A slot-0 mask keeps only `is_ok` in slot `0`.
- Rotations replicate `is_ok` across all shard slots.
- A validity-slot mask places `is_ok` only into the final validity slot.
- A clear-last-slot mask removes any client-provided value from the reserved
  validity slot before the aggregator writes its own value.

Finally, the shard is multiplied by replicated `is_ok`. If `is_ok = 1`, the
encrypted shard remains unchanged except for the validity slot. If `is_ok = 0`,
the encrypted shard is masked to zero and the validity slot becomes `0`.

## Aggregator Work

Aggregators keep a list of encrypted shards and aggregate them homomorphically.
For each shard, the aggregator first calls `append_bit_check`.

The bit check verifies encrypted slots are binary by computing:

```text
slot * (slot - 1)
```

This is zero exactly when a slot is `0` or `1`. The code sums those per-slot
errors and uses Fermat-style exponentiation over the BGV plaintext field to
turn a nonzero error into `1`. Then it computes:

```text
is_ok = 1 - is_error
```

If the shard is valid, the aggregator keeps the shard and writes `1` into the
validity slot. If the shard is invalid, the aggregator masks the shard to zero
and writes `0` into the validity slot.

This is intentionally different from a VDAF that returns a hard validation
error. Here invalid shards are truncated/masked out of the encrypted aggregate
instead of immediately aborting at the aggregator.

## Collector Work

The collector receives one encrypted aggregate share per aggregator. In the
current runtime path it does not decrypt those per-aggregator shares. It first
homomorphically adds the encrypted shares into one final collector ciphertext.

Only after that final collector aggregation does the demo decrypt and view the
resulting slot vector. The collector then decodes the value bits and rescales
the result by `AGGREGATORS_NUM`.

The code still contains a test-only helper for checking that decoded aggregate
share packs agree on the validity count and signature bit pack. That check is
useful for testing invalid-shard behavior, but it is not part of the current
main runtime flow.

## Similarity To Prio3

The high-level shape is close to Prio3:

- Clients split an input into multiple aggregator shares.
- Aggregators do local validity processing.
- The collector combines aggregator outputs.
- A single aggregator should not learn the client input.

The prototype differs from standard Prio3 in important ways:

- Validation is done with homomorphic arithmetic over encrypted BGV slots.
- Invalid shards are masked to zero instead of causing an immediate aggregator
  error.
- The collector performs a consistency check after aggregation.
- This is not a complete VDAF specification or production-ready protocol.

## Current Limitations And Risks

This code is a prototype and has several protocol caveats.

- A decrypted shard is not the input, but it can still leak partial information
  because the shard range is bounded.
- The prototype does not handle malicious aggregators
  completely. It demonstrates the encrypted aggregation path and some local
  validity masking, not a full security proof.
