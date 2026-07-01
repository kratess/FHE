# Slide 2: caso medico a 6 feature

## Dataset e packing

Dataset: [`hospital_data_prob.csv`](/home/kratess/fhe/linear_regression/data/hospital_data_prob.csv:1)

Feature:

- `age`
- `cci`
- `num_procedures`
- `systolic`
- `diastolic`
- `bmi`

Target:

- `days_in_hospital`

Parametri del caso:

- feature reali: `d = 6`
- feature circuitali: `F = nextPow2(6) = 8`
- sample: `S = 512`
- `ringDim = 8192`
- `batchSize = 4096`

Controllo packing:

- slot richiesti: `S * F = 512 * 8 = 4096`
- slot disponibili: `4096`

Il dataset entra esattamente in un ciphertext packed.

## Operazioni per epoch nel caso reale

Con `F = 8` e `S = 512`:

- moltiplicazioni: `6`
- rotazioni esplicite: `34`
- add/sub esplicite: `38`
- piu':
  - `1 x EvalSumCols`
  - `1 x EvalSumRows`

Stima includendo anche il costo interno delle somme strutturate:

- rotazioni totali circa: `46`
- add/sub totali circa: `50`

## Tempi dai benchmark CKKS

Riferimento usato: `ringDim = 8192`, `depth = 83`

Nota:

- il training C++ usa una profondita' iniziale effettiva di circa `83`
- il benchmark CKKS non ha un punto esatto a `depth = 83`
- i valori sotto sono quindi stimati per interpolazione lineare tra i benchmark a `depth = 64` e `depth = 128`

- `EvalMult`: `43.5473 ms`
- `EvalAdd`: `0.9666 ms`
- `EvalRotate`: `32.9456 ms`
- `Bootstrap`: `886.483 ms`

Stima microbenchmark per epoch:

- `6 * EvalMult = 261.28 ms`
- `50 * EvalAdd = 48.33 ms`
- `46 * EvalRotate = 1515.50 ms`

Totale stimato:

`T_epoch ~= 1825.11 ms ~= 1.83 s`

Confronto con il tempo reale osservato:

- tempo reale medio per epoch: `10.01 s`
- rapporto reale / stima microbenchmark: circa `5.5x`

Quindi i tempi non combacciano direttamente: il benchmark per operazione singola resta un lower bound abbastanza ottimistico.

Motivi principali del divario:

- `EvalRotate` nel benchmark CKKS attuale misura una rotazione isolata con step fisso `1`, mentre nel training compaiono rotazioni `1..7` e poi `8, 16, ..., 2048`
- `EvalSumCols` e `EvalSumRows` sono operazioni composte che internamente fanno altre rotazioni, addizioni e key switching
- nel training le operazioni sono concatenate sullo stesso ciphertext, quindi si accumulano rescaling, gestione dei livelli e dipendenze seriali
- il microbenchmark misura il kernel singolo, ma non il costo del loop completo con ciphertext piu' grandi e gia' trasformati da operazioni precedenti

## Tempo reale osservato

Run C++ riuscito con:

- `epochs = 10`
- `eta = 0.00001`

Tempi osservati:

- init FHE: `6.23 s`
- encrypt dataset/modello: `1.62 s`
- training loop: `100.06 s`
- totale end-to-end: `101.73 s`

Media:

- circa `10.01 s` per epoch

## Risultato pratico

- tempo totale: circa `1 minuto e 42 secondi`
- nessun bootstrap necessario nelle 10 epoche
- livelli residui finali: `23`

Modello finale osservato:

`y = 0.0063*x0 + 0.0004*x1 + 0.0018*x2 + 0.0107*x3 + 0.0069*x4 + 0.0022*x5 + 0.0001`
