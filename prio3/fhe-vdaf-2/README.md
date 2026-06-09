# FHE VDAF 2

This prototype is closer to a real VDAF shape than `fhe-vdaf-1`.

The main idea is:

- a client has one value
- the client splits it into one encrypted shard per aggregator
- each aggregator checks and sums its own shard
- the collector combines the aggregator sums to recover only the final total

So no single aggregator receives the client's value directly.

## Running The Split Actors

This crate now exposes separate Rust binaries for each role.

From [`prio3/`](../):

```bash
cargo run -p fhe-vdaf-2 --bin setup
cargo run -p fhe-vdaf-2 --bin client
cargo run -p fhe-vdaf-2 --bin aggregator
cargo run -p fhe-vdaf-2 --bin collector
```

By default, each binary reads or writes artifacts under `runtime/` inside the crate directory.
You can override that by passing a different runtime path as the first argument.

## What The Code Does

In [`src/main.rs`](./src/main.rs):

- `MAX_SIZE = 100`
- `AGGREGATORS_NUM = 2`
- each client value is scaled by `2`
- that scaled value is split into 2 bounded parts

The demo values are:

```text
51 and 49
```

So the expected final result is:

```text
51 + 49 = 100
```

## Simple Flow

For each client value:

1. The client checks `value <= MAX_SIZE`.
2. The client computes `scaled = value * AGGREGATORS_NUM`.
3. The scaled value is split into one part for each aggregator.
4. Each part is encoded into bits.
5. The client appends extra signature bits and one final validity slot.
6. Each packed shard is encrypted and sent to its aggregator.
7. Each aggregator validates its encrypted shards and sums them.
8. The collector adds the aggregator outputs and divides by `AGGREGATORS_NUM`.

## Why The Value Is Scaled

With 2 aggregators, the code first computes:

```text
scaled = value * 2
```

Example:

```text
value = 51
scaled = 102
```

Then the client chooses two parts whose sum is `102`.

Example:

```text
part A = 40
part B = 62
```

because:

```text
40 + 62 = 102
```

Each aggregator gets only one encrypted part.

At the end, the collector adds all parts and divides by `2`, so:

```text
102 / 2 = 51
```

## What One Encrypted Shard Looks Like

Each shard is packed like this:

```text
[ value bits | signature bits | validity slot ]
```

The value bits use the same kind of bounded binary encoding as `fhe-vdaf-1`.

With `MAX_SIZE = 100`, the value slot weights are:

```text
[1, 2, 4, 8, 16, 32, 37]
```

So for example:

```text
62 -> [0, 1, 1, 1, 1, 1, 0]
```

because:

```text
2 + 4 + 8 + 16 + 32 = 62
```

## How The Aggregator Checks Validity

Each aggregator checks that every packed slot is a bit.

For each slot `x`, it computes:

```text
x * (x - 1)
```

Only `0` and `1` make this equal to zero.

Then it sums all errors:

```text
E = sum(x_i * (x_i - 1))
```

- `E = 0` means the shard is valid
- `E != 0` means the shard is invalid

The code converts that into an encrypted flag:

```text
is_ok = 1 - is_error
```

Then:

- if the shard is valid, it stays
- if the shard is invalid, it is zeroed out

The aggregator also writes the validity result into the final reserved slot.

## What The Signature Bits Are For

The client appends 64 deterministic random-looking bits based on the original value.

The code uses them as a simple consistency marker so different aggregator packs can be checked against each other in tests.

This is not a full authentication system. It is just part of the prototype structure.

## Small Example

Suppose the client value is:

```text
49
```

The code scales it:

```text
49 * 2 = 98
```

Then it might split that into:

```text
36 and 62
```

Aggregator 0 gets one encrypted shard.
Aggregator 1 gets the other encrypted shard.

Neither shard is the original `49`.

The collector adds the two aggregated outputs and rescales:

```text
(36 + 62) / 2 = 49
```

## What Makes `fhe-vdaf-2` Better Than `fhe-vdaf-1`

`fhe-vdaf-1` sends the whole encrypted value to one aggregator.

`fhe-vdaf-2` sends one encrypted shard to each aggregator.

That means one aggregator sees only part of the value flow, not the full report.

## Protocol Problems

1. A shard still leaks something if it is ever decrypted.
One shard is not the original value, but it is still a bounded piece of the value, so this is not the same as having a full malicious-secure VDAF proof.

2. Bad shards create a hard protocol question.
If one shard is invalid, one option is to zero only that shard and keep the other shards. But then the report is only partially counted. The bigger problem is malicious shards with inconsistent signature bits or wrong validity information: after aggregation, the signatures may no longer match and the validity counts may no longer add up to the exact expected number. Then it is unclear what the collector should do. Should it drop the whole bucket of aggregated shards? If attackers spam many bad shards, they could keep poisoning buckets and push the protocol toward denial of service.
