# FHE VDAF 1

This is the simplest prototype in this repo.

Think of it like this:

- Clients have numbers.
- Clients encrypt each number.
- Each encrypted number is sent to one aggregator.
- The aggregator checks that the encrypted bits look valid.
- The collector adds the aggregator outputs and gets only the final sum.

This is "VDAF-like" because there are still three roles:

- client
- aggregator
- collector

## Running The Split Actors

This crate now exposes separate Rust binaries for each role.

From [`prio3/`](../):

```bash
cargo run -p fhe-vdaf-1 --bin setup
cargo run -p fhe-vdaf-1 --bin client
cargo run -p fhe-vdaf-1 --bin aggregator
cargo run -p fhe-vdaf-1 --bin collector
```

By default, each binary reads or writes artifacts under `runtime/` inside the crate directory.
You can override that by passing a different runtime path as the first argument.

But it is not a full VDAF specification. It is a small demo built on OpenFHE BGV.

## What The Code Does

In [`src/main.rs`](./src/main.rs):

- `MAX_SIZE = 100`
- `AGGREGATORS_NUM = 2`
- each client can send multiple values

The demo clients are:

```text
client 0: 51, 7, 4
client 1: 49, 13
```

So the expected total is:

```text
51 + 7 + 4 + 49 + 13 = 124
```

## Simple Flow

For each value:

1. The client checks `value <= MAX_SIZE`.
2. The client turns the value into bits.
3. The client encrypts those bits.
4. One aggregator is chosen.
5. That aggregator validates the encrypted bits and adds them to its local sum.
6. The collector adds the aggregator sums and decrypts the final result.

## How A Value Is Encoded

The value is stored as binary slots.

With `MAX_SIZE = 100`, the code uses 7 slots with weights:

```text
[1, 2, 4, 8, 16, 32, 37]
```

Why is the last weight `37`?

Because the code wants the all-ones vector to decode to exactly `100`:

```text
1 + 2 + 4 + 8 + 16 + 32 + 37 = 100
```

Example:

```text
value 5 -> [1, 0, 1, 0, 0, 0, 0]
```

because:

```text
1 + 4 = 5
```

Example:

```text
value 100 -> [1, 1, 1, 1, 1, 1, 1]
```

## How The Aggregator Checks Validity

The aggregator cannot read the plaintext value, but it can still check whether each slot is a bit.

For each encrypted slot `x`, it computes:

```text
x * (x - 1)
```

This gives:

```text
0 -> 0
1 -> 0
2 -> 2
3 -> 6
```

So only `0` and `1` pass.

Then the aggregator adds all slot errors:

```text
E = sum(x_i * (x_i - 1))
```

- `E = 0` means valid
- `E != 0` means invalid

The code then turns that into an encrypted yes/no flag and multiplies the whole ciphertext by that flag:

- valid input stays the same
- invalid input becomes all zeros

So bad inputs are dropped from the sum.

## Small Example

Suppose a client sends:

```text
7 -> [1, 1, 1, 0, 0, 0, 0]
```

Every slot is `0` or `1`, so the input is valid.

If somebody somehow sent:

```text
[1, 2, 1, 0, 0, 0, 0]
```

then the slot with `2` fails the bit check, and the aggregator turns the whole encrypted input into zero before aggregation.

## What Makes `fhe-vdaf-1` Different

This version does not split one value across all aggregators.

Instead:

- one encrypted value
- goes to one aggregator

So it is simpler than `fhe-vdaf-2`, but also weaker from a protocol point of view.

## Protocol Problems

1. One aggregator gets the whole encrypted report.
If that aggregator ever gets the secret key, or if decryption leaks somewhere else, it can recover the client's full value because the value was not split across aggregators.

2. Invalid reports are silently zeroed instead of causing a hard failure.
That keeps the demo simple, but a real protocol usually needs stronger handling for bad reports, replayed reports, or malicious clients.

3. Malicious validators may inject arbitrary data and mark it as valid. Consequently, the collector does not effectively validate the information and instead aggregates potentially malicious data from untrusted aggregators.
