# Slide 1: costo operativo della linear regression FHE

## Parametri

- `d` = numero di feature reali
- `F = nextPow2(d)` = numero di feature effettive nel circuito dopo padding
- `S` = numero di sample
- `ringDim` = dimensione dell'anello CKKS
- `batchSize = ringDim / 2`

Vincolo di packing:

`S * F <= batchSize`

Quindi il costo reale del circuito dipende da `F`, non direttamente da `d`.

## Relazione con il gradient descent classico

Per ogni epoch, il gradient descent standard fa:

1. `y_hat = XW + b`
2. `E = y_hat - y`
3. `grad_W = (eta / S) * X^T E`
4. `W = W - grad_W`
5. `grad_b = (eta / S) * sum(E)`
6. `b = b - grad_b`

In FHE la logica e' la stessa, ma servono operazioni extra per:

- espandere i dati nei blocchi packed
- ruotare gli slot
- sommare tra slot
- replicare gradiente e bias su tutti i blocchi

## Operazioni FHE per epoch

Costo esplicito per epoch:

- moltiplicazioni: `6`
- add/sub: `F + 3*ceil(log2(S)) + 3`
- rotazioni: `(F - 1) + 3*ceil(log2(S))`
- somme strutturate:
  - `1 x EvalSumCols`
  - `1 x EvalSumRows`

Stima semplice del costo interno delle somme strutturate:

- `EvalSumCols(..., F, ...)` circa `log2(F)` rotazioni + `log2(F)` add
- `EvalSumRows(..., S, ...)` circa `ceil(log2(S))` rotazioni + `ceil(log2(S))` add

## Formula complessiva di massima

Per ogni epoch:

- moltiplicazioni: `6`
- add/sub totali stimati: `F + 4*ceil(log2(S)) + 3 + log2(F)`
- rotazioni totali stimate: `F - 1 + 4*ceil(log2(S)) + log2(F)`

## Lettura intuitiva

- se aumentano le feature, cresce `F = nextPow2(d)`
- se aumentano i sample, cresce soprattutto `log2(S)`
- il collo di bottiglia pratico non e' solo il numero di moltiplicazioni, ma soprattutto:
  - rotazioni
  - somme strutturate
  - vincolo `S * F <= batchSize`
