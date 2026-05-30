# Bit Validity Check

We want to verify that every encrypted slot contains only:

```text
0 or 1
```

## Bit check

Use:

```text
x * (x - 1)
```

because:

```text
0 * (0 - 1) = 0
1 * (1 - 1) = 0
```

but:

```text
2 * (2 - 1) = 2
3 * (3 - 1) = 6
```

Only `0` and `1` produce zero.

---

# Example

Input:

```text
[1, 0, 1]
```

Compute:

```text
[1*(1-1), 0*(0-1), 1*(1-1)]
```

Result:

```text
[0, 0, 0]
```

All valid.

---

# Invalid Example

Input:

```text
[1, 2, 0]
```

Compute:

```text
[1*(1-1), 2*(2-1), 0*(0-1)]
```

Result:

```text
[0, 2, 0]
```

Contains an error.

---

# Sum All Errors

Compute:

```text
E = sum(all slots)
```

Valid:

```text
E = 0
```

Invalid:

```text
E != 0
```

---

# Fermat Little Theorem

For prime modulus `p`:

```text
x^(p-1) = 1 mod p   if x != 0
x^(p-1) = 0         if x == 0
```

So:

```text
E^(p-1)
```

becomes:

```text
0 -> 0
nonzero -> 1
```

This creates an encrypted error flag.

---

# Modulus 65537

```text
65537 - 1 = 65536 = 2^16
```

Compute:

```text
E^65536
```

using repeated squaring.

---

# Modulus 786433

```text
786433 - 1 = 786432 = 3 * 2^18
```

Compute:

```text
E^786432
```

as:

```text
(E^3)^(2^18)
```

---

# Final Validity Mask

Error flag:

```text
is_error =
0 if valid
1 if invalid
```

Compute:

```text
is_ok = 1 - is_error
```

Result:

```text
1 -> valid
0 -> invalid
```

---

# Full Flow

```text
e_i = x_i * (x_i - 1)
```

```text
E = sum(e_i)
```

```text
is_error = E^(p-1)
```

```text
is_ok = 1 - is_error
```
