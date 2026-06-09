# FHE VDAF 1 Histogram

This crate keeps the same simple protocol shape as `fhe-vdaf-1` but changes the measurement type to a histogram bucket, similar to `Prio3Histogram` in `draft-irtf-cfrg-vdaf-19`.

The roles are still:

- client
- aggregator
- collector

Each client report is one bucket index. The client encodes that report as a one-hot vector, encrypts it, and sends the ciphertext to one aggregator. The aggregator checks that the encrypted vector is binary and sums to exactly one. The collector adds all aggregate shares and decrypts the final bucket counts.

## Running

From [`prio3/`](../):

```bash
cargo run -p fhe-vdaf-1-histogram --bin setup
cargo run -p fhe-vdaf-1-histogram --bin client
cargo run -p fhe-vdaf-1-histogram --bin aggregator
cargo run -p fhe-vdaf-1-histogram --bin collector
```

By default the binaries use a `runtime/` directory relative to the current working directory. Pass a different runtime path as the first argument to override it.

## Demo Parameters

- `BUCKETS = 5`
- `AGGREGATORS_NUM = 2`
- sample reports:
  - client 0: `0, 2, 2`
  - client 1: `1, 4`

Expected histogram:

```text
[1, 1, 2, 0, 1]
```

## Encoding

Each measurement is a bucket index in `[0, 4]`.

The client converts it to a one-hot vector of length 5:

```text
bucket 2 -> [0, 0, 1, 0, 0]
bucket 4 -> [0, 0, 0, 0, 1]
```

## Validation

For each encrypted slot `x`, the aggregator checks bitness with:

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

So only `0` and `1` pass the bit check.

Then the aggregator adds all slot errors:

```text
E_bits = sum(x_i * (x_i - 1))
```

If `E_bits = 0`, every slot looks like a bit.

But that is not enough for a histogram bucket. The vector must also be one-hot, meaning exactly one slot is `1`.

So the aggregator also computes the slot sum:

```text
S = sum(x_i)
```

and checks that:

```text
S = 1
```

If either condition fails:

- not all slots are bits, or
- the slots do not sum to exactly one

then the whole encrypted report is treated as invalid and multiplied by an encrypted zero mask before aggregation.

### Valid Example

```text
[0, 0, 1, 0, 0]
```

- every slot is `0` or `1`
- the sum is `1`

So this is valid.

### Counter Example

```text
[0, 1, 1, 0, 0]
```

This passes the bit check, because every slot is still `0` or `1`.

So:

```text
E_bits = 0
```

But it still fails the one-hot check, because:

```text
S = 0 + 1 + 1 + 0 + 0 = 2
```

and not:

```text
S = 1
```

So `[0, 1, 1, 0, 0]` is invalid and the aggregator zeroes the whole ciphertext before adding it to the aggregate.

## Plotting

After running the protocol, you can render the result as a bar chart:

```bash
python3 fhe-vdaf-1-histogram/scripts/plot_histogram.py
```

To save an image instead of opening a window:

```bash
python3 fhe-vdaf-1-histogram/scripts/plot_histogram.py \
  runtime/collector/result.txt \
  --output histogram.png
```
