# Discorso per la discussione della tesi

Durata prevista: circa 15 minuti. Il testo e' pensato come traccia da esporre: non e' necessario leggerlo parola per parola, ma e' utile mantenere i messaggi chiave e i passaggi tra le slide.

---

## Slide 1 - Titolo

Buongiorno. In questa presentazione descrivero' il lavoro svolto sulle tecniche di Fully Homomorphic Encryption, o FHE. Il filo conduttore della tesi e' capire come sia possibile elaborare dati sensibili senza renderli leggibili al soggetto che esegue il calcolo. Partiro' dal problema e dagli schemi crittografici, poi presentero' i benchmark e due casi di studio: una regressione lineare in ambito sanitario e un protocollo di aggregazione privata.

---

## Slide 2 - Problematica: calcolo privacy-aware e machine learning

Oggi grandi quantita' di dati vengono raccolte da ospedali, aziende e servizi digitali. Questi dati sono utili per analisi statistiche e modelli di machine learning, ma possono contenere informazioni personali o sanitarie molto sensibili.

La cifratura tradizionale protegge bene i dati mentre sono memorizzati o trasmessi. Tuttavia, nel momento in cui il server deve elaborarli, normalmente deve prima decifrarli. Di conseguenza, il server torna a vedere i valori originali: proprio il punto che vorremmo proteggere.

Questo e' particolarmente rilevante nel machine learning. Per addestrare un modello o produrre una previsione, spesso e' necessario centralizzare dati appartenenti a soggetti diversi. La domanda di partenza della tesi e' quindi: possiamo eseguire almeno una parte di questi calcoli senza consegnare i dati in chiaro all'infrastruttura che li elabora?

La cifratura omomorfica fornisce una risposta possibile.

---

## Slide 3 - Obiettivo della tesi

L'obiettivo della tesi non e' soltanto descrivere la cifratura omomorfica dal punto di vista teorico, ma valutarne l'uso pratico.

Ho utilizzato la libreria OpenFHE per sperimentare tre schemi: BFV, BGV e CKKS. Il lavoro e' stato organizzato in tre fasi. Prima ho costruito una suite di benchmark per misurare tempo e memoria delle operazioni principali. Poi ho applicato CKKS a una regressione lineare su dati sanitari sintetici. Infine ho realizzato un prototipo di aggregazione ispirato alle VDAF, usando BGV per elaborare contributi interi cifrati.

L'idea principale e' collegare tre livelli: le proprieta' degli schemi, i loro costi misurabili e le decisioni architetturali necessarie per usarli in un'applicazione reale.

---

## Slide 4 - Tecniche omomorfiche: BFV, BGV e CKKS

Il principio comune dei tre schemi e' semplice: un client codifica e cifra un dato con una chiave pubblica; il server riceve solo ciphertext, esegue somme o moltiplicazioni direttamente su di essi e produce un ciphertext risultato. Solo chi possiede la chiave segreta puo' decifrarlo.

BFV e BGV sono adatti a valori interi. I calcoli sono esatti, ma avvengono in aritmetica modulare: bisogna quindi scegliere un modulo abbastanza grande da rappresentare correttamente i risultati intermedi.

CKKS, invece, e' progettato per numeri reali o complessi approssimati. Questa caratteristica lo rende adatto a statistica e machine learning, dove coefficienti e risultati non sono necessariamente interi. Il compromesso e' che il risultato non e' esatto cifra per cifra, ma ha un piccolo errore numerico controllato.

Quindi non esiste uno schema migliore in assoluto: dipende dal tipo di dato e dal calcolo richiesto.

---

## Slide 5 - Tecniche omomorfiche: rumore, profondita' e packing

Per capire il costo di FHE servono tre concetti. Il primo e' il rumore: ogni ciphertext contiene una componente casuale che garantisce sicurezza. Durante i calcoli il rumore cresce; le somme lo aumentano poco, mentre le moltiplicazioni lo aumentano molto di piu'. Se supera un limite, la decifratura non e' piu' corretta.

Per questo introduciamo la profondita' moltiplicativa, cioe' il numero massimo di moltiplicazioni consecutive che il circuito deve sostenere. Aumentarla rende possibile un calcolo piu' lungo, ma aumenta anche tempo, memoria e dimensione delle chiavi.

Il secondo concetto e' il packing: un singolo ciphertext puo' contenere molti valori, chiamati slot. Una stessa operazione viene applicata in parallelo agli slot, in modo simile a SIMD. Questo e' essenziale per rendere efficienti aggregazioni e calcoli vettoriali.

Infine, se il rumore sta per esaurire il budget, CKKS puo' usare il bootstrapping per rinnovare il ciphertext. E' molto costoso, quindi conviene evitarlo quando e' sufficiente una configurazione a profondita' prefissata.

---

## Slide 6 - Scenari e applicazioni: un client e un server

Nel caso piu' semplice abbiamo un client con un dato sensibile e un computation server. Il client cifra il proprio input con la chiave pubblica e invia il ciphertext al server. Il server esegue il calcolo, ma non possiede la chiave segreta e quindi non puo' leggere ne' l'input ne' il risultato intermedio.

Il risultato cifrato viene poi inviato a un soggetto autorizzato alla decifratura, che puo' restituire solo il valore finale necessario. Questo modello e' utile, per esempio, per una query medica privata: il server puo' applicare un modello predittivo ai dati di un paziente senza visualizzarli in chiaro.

Il vantaggio e' la separazione tra chi calcola e chi puo' leggere. Il costo e' maggiore rispetto al calcolo ordinario e richiede una gestione accurata di chiavi pubbliche, chiavi segrete e chiavi di valutazione.

---

## Slide 7 - Scenario di aggregazione: molti client, aggregatore e collector

Il secondo scenario e' l'aggregazione. Qui piu' client inviano contributi numerici, per esempio conteggi o misurazioni. L'obiettivo non e' conoscere ogni valore individuale, ma soltanto la somma complessiva.

Ogni client cifra il proprio report con la chiave pubblica. L'aggregatore riceve tutti i ciphertext, li valida e li somma senza decifrarli. Al termine invia un unico ciphertext aggregato al collector, che possiede la chiave segreta e decifra solo il totale.

Questo modello e' utile quando il dato collettivo ha valore, ma i contributi individuali non devono essere esposti. Nel mio prototipo il sistema e' volutamente semplice, con un aggregatore e un collector. Per questo e' corretto definirlo VDAF-like: riprende l'idea dell'aggregazione privata, ma non offre la stessa distribuzione fiduciaria di Prio3 completo con aggregatori indipendenti.

---

## Slide 8 - Piano dei test

Il piano sperimentale segue una progressione precisa. Il primo livello e' costituito dai microbenchmark: misurano il costo delle primitive crittografiche isolandole dal resto dell'applicazione.

Il secondo livello e' la regressione lineare. Qui non basta piu' misurare una singola somma o moltiplicazione, perche' entrano in gioco packing, rotazioni degli slot, rescaling e dipendenze tra operazioni. Questo permette di capire la differenza tra il costo teorico delle primitive e quello reale di un algoritmo.

Il terzo livello e' l'aggregazione: viene verificato che i client possano inviare report cifrati, che l'aggregatore validi i report senza leggerli e che il collector recuperi il totale corretto.

In questo modo i test passano dalla misura delle primitive a due applicazioni con requisiti diversi: valori reali approssimati per il machine learning e interi esatti per l'aggregazione.

---

## Slide 9 - Benchmark: metodologia

I benchmark sono stati implementati in C++17 con OpenFHE e Google Benchmark. Sono stati eseguiti su un AMD Ryzen 9800X3D con 48 gigabyte di memoria DDR5, in ambiente WSL e con compilazione ottimizzata tramite `-march=native`.

Per BFV, BGV e CKKS ho misurato la creazione del contesto, la generazione delle chiavi, la cifratura, la decifratura, l'addizione e la moltiplicazione omomorfica. Per CKKS ho aggiunto la generazione delle chiavi di bootstrapping e il bootstrapping stesso.

I parametri variati sono la ring dimension e la profondita' moltiplicativa. Nelle figure principali la profondita' e' fissata a 16, mentre la ring dimension varia fino a 16384. BFV e BGV usano il modulo plaintext primo 786433; CKKS usa una configurazione dedicata al calcolo approssimato.

Un limite importante: per osservare direttamente l'effetto della ring dimension, i benchmark non impongono un livello di sicurezza standard per ogni configurazione. I grafici descrivono quindi tendenze prestazionali, non parametri da copiare direttamente in produzione.

---

## Slide 10 - Benchmark: risultati

Il primo risultato e' che l'addizione omomorfica e' molto piu' economica della moltiplicazione. A ring dimension 4096 e profondita' 16, l'addizione richiede circa 0,158 millisecondi in BFV, 0,232 in BGV e 0,085 in CKKS.

Per la moltiplicazione la differenza aumenta: BFV richiede circa 13,652 millisecondi, BGV 4,392 e CKKS 3,879. Questo non significa che CKKS sia sempre preferibile, perche' rappresenta dati diversi; mostra pero' che la scelta dello schema e del circuito influenza direttamente il costo.

All'aumentare della ring dimension aumentano sia gli slot disponibili sia tempo e memoria. Il packing diventa vantaggioso solo se gli slot sono realmente utilizzati.

Infine, il bootstrapping CKKS e' di gran lunga piu' costoso delle operazioni normali. La conclusione pratica e' progettare prima il circuito, limitare le moltiplicazioni consecutive e usare il bootstrapping solo se necessario per proseguire il calcolo.

---

## Slide 11 - Regressione lineare: caso di studio e implementazione

Il primo caso applicativo e' una regressione lineare per stimare i giorni di ricovero ospedaliero. Il dataset sintetico contiene sei feature: eta', indice di comorbilita' di Charlson, numero di procedure, pressione sistolica, pressione diastolica e indice di massa corporea.

Il modello calcola una funzione lineare: ogni feature viene moltiplicata per un peso, i risultati vengono sommati e viene aggiunto un bias. I pesi vengono aggiornati con gradient descent, confrontando a ogni epoca la previsione con il numero reale di giorni di ricovero.

Ho scelto CKKS perche' coefficienti, medie ed errori intermedi sono numeri reali. I dati sono organizzati negli slot del ciphertext: con sei feature il circuito effettua padding a otto, e 512 campioni occupano 512 per 8, cioe' 4096 slot. Con ring dimension 8192, il dataset entra esattamente in un ciphertext packed.

La regressione e' quindi un esempio utile: ha una formula semplice, ma rende visibile il costo reale di un training FHE iterativo.

---

## Slide 12 - Regressione lineare: risultati e lettura

Nel caso con 512 campioni e 10 epoche, l'inizializzazione FHE ha richiesto circa 6,23 secondi e la cifratura di dataset e modello circa 1,62 secondi. Il training ha richiesto 100,06 secondi, per un tempo end-to-end di circa 101,73 secondi: in media circa 10 secondi per epoca.

Il risultato e' piu' costoso di quanto suggeriscano i microbenchmark isolati. La stima basata sulle primitive era circa 1,83 secondi per epoca, mentre il tempo osservato e' circa cinque volte e mezzo superiore.

Il motivo e' che il training include rotazioni con chiavi diverse, somme strutturate tra slot, key switching, rescaling e una sequenza di ciphertext che dipendono l'uno dall'altro. Quindi il microbenchmark e' un limite inferiore utile, ma non sostituisce il test dell'algoritmo completo.

In queste 10 epoche non e' stato necessario il bootstrapping e sono rimasti 23 livelli. Il risultato mostra che CKKS e' utilizzabile per un prototipo di machine learning privacy-aware, ma richiede ottimizzazione e una scelta accurata dei parametri per scalare.

---

## Slide 13 - Aggregazione: sistema e chiavi

Il secondo caso di studio usa BGV per aggregare valori interi. Il sistema ha quattro ruoli: una setup authority, i client, l'aggregatore e il collector. La setup authority genera il contesto, la chiave pubblica, la chiave segreta e le evaluation keys.

I client ricevono contesto e chiave pubblica, quindi possono cifrare i report. L'aggregatore possiede le evaluation keys necessarie per somme, rotazioni e moltiplicazioni omomorfiche, ma non la chiave segreta. Il collector possiede invece la chiave segreta e puo' decifrare il risultato finale.

Ogni valore da 0 a 100 viene rappresentato in sette slot binari con pesi 1, 2, 4, 8, 16, 32 e 37. L'ultimo peso non e' una potenza di due, ma permette al vettore di tutti uno di rappresentare esattamente 100.

La scelta di BGV deriva dalla natura del problema: i report e il totale sono interi e la validazione sfrutta l'aritmetica modulare esatta.

---

## Slide 14 - Aggregazione: validazione e risultato

L'aggregatore deve evitare di sommare report malformati senza poterli leggere. Per verificare che ogni slot sia un bit, cioe' zero o uno, calcola il polinomio `x per x meno 1`. Il risultato e' zero per zero e uno, mentre e' diverso da zero per altri valori.

Gli errori dei sette slot vengono sommati. Tramite il piccolo teorema di Fermat, l'aggregatore trasforma questa somma in un flag cifrato: zero se il report e' valido, uno se e' invalido. Ottiene quindi un flag di validita' opposto e moltiplica il report per quel flag. Il meccanismo si chiama validate-or-zero: un report valido resta invariato, uno invalido diventa il vettore nullo.

Nella validazione finale sono stati aggregati i valori 51, 7, 4, 49 e 13. Il collector ha decifrato gli slot sommati e ha ricostruito il totale 124, uguale al valore atteso.

Il prototipo dimostra la correttezza funzionale, ma il limite resta la fiducia centralizzata: non e' una sostituzione completa di Prio3, che distribuisce la fiducia tra piu' aggregatori indipendenti.

---

## Slide 15 - Conclusioni

In conclusione, la tesi mostra che la cifratura omomorfica rende possibile separare il dato dal calcolo: il server puo' elaborare ciphertext senza conoscere necessariamente gli input.

I benchmark confermano pero' che questa privacy ha un costo. Le addizioni sono economiche, le moltiplicazioni pesano molto di piu', la ring dimension incide su tempo e memoria e il bootstrapping deve essere una scelta mirata.

I due casi di studio mostrano due usi complementari. CKKS e' adatto a una regressione lineare su dati sanitari grazie ai numeri reali approssimati e al packing. BGV e' invece adatto a un'aggregazione di conteggi interi con validazione sotto cifratura.

Il risultato principale non e' quindi che FHE sostituisca il calcolo tradizionale in ogni scenario, ma che, scegliendo correttamente schema, parametri e architettura, permette di costruire applicazioni in cui il dato sensibile non deve essere consegnato in chiaro a chi esegue l'elaborazione.

