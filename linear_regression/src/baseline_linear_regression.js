const fs = require('fs');

function loadAndSplitDataset(path, trainRatio = 0.8, maxExponent = 1, standardize = false) {
  // Load CSV as text
  const fileContent = fs.readFileSync(path, 'utf8');

  // Split into lines, skip empty lines and header
  const lines = fileContent.split('\n').filter(line => line.trim() !== '').slice(1);

  let X = [];
  const Y = [];

  for (const line of lines) {
    const values = line.split(',').map(Number);
    X.push(values.slice(0, -1));
    Y.push(values[values.length - 1]);
  }

  const nSamples = X.length;
  const nFeatures = X[0].length;

  let meansX = new Array(nFeatures).fill(0);
  let stdsX = new Array(nFeatures).fill(0);
  let meanY = 0;
  let stdY = 1;

  if (standardize) {
    // Standardize X
    for (let j = 0; j < nFeatures; j++) {
      for (let i = 0; i < nSamples; i++) {
        meansX[j] += X[i][j];
      }
      meansX[j] /= nSamples;
    }
    for (let j = 0; j < nFeatures; j++) {
      for (let i = 0; i < nSamples; i++) {
        stdsX[j] += Math.pow(X[i][j] - meansX[j], 2);
      }
      stdsX[j] = Math.sqrt(stdsX[j] / nSamples) || 1;
    }
    for (let i = 0; i < nSamples; i++) {
      for (let j = 0; j < nFeatures; j++) {
        X[i][j] = (X[i][j] - meansX[j]) / stdsX[j];
      }
    }

    // Standardize Y
    for (let i = 0; i < nSamples; i++) {
      meanY += Y[i];
    }
    meanY /= nSamples;

    for (let i = 0; i < nSamples; i++) {
      stdY += Math.pow(Y[i] - meanY, 2);
    }
    stdY = Math.sqrt(stdY / nSamples) || 1;

    for (let i = 0; i < nSamples; i++) {
      Y[i] = (Y[i] - meanY) / stdY;
    }
  }

  // Expand features to powers up to maxExponent
  const X_expanded = X.map(row => {
    const newRow = [];
    for (let j = 0; j < row.length; j++) {
      for (let p = 1; p <= maxExponent; p++) {
        newRow.push(Math.pow(row[j], p));
      }
    }
    return newRow;
  });

  const nTrain = Math.floor(trainRatio * nSamples);

  const trainSet = {
    X: X_expanded.slice(0, nTrain),
    Y: Y.slice(0, nTrain),
    meansX,
    stdsX,
    meanY,
    stdY
  };

  const testSet = {
    X: X_expanded.slice(nTrain),
    Y: Y.slice(nTrain)
  };

  return [trainSet, testSet];
}

function train(trainingDataset, epochs = 1, eta = 0.1) {
  console.log("TRAINING: Training has been started");

  const X = trainingDataset.X;
  const Y = trainingDataset.Y;

  const nSamples = X.length;
  const nFeatures = X[0].length;

  console.log("Initializing Dataset X and Y and parameters W and B...");

  // Model parameters
  let W = new Array(nFeatures).fill(0.0); // weights
  let B = new Array(nFeatures).fill(0.0); // bias

  console.log("Initial W:", W);
  console.log("Initial B:", B);

  console.log("Dataset and parameters initialized successfully");

  // ---------------------------
  // TRAINING LOOP
  // ---------------------------
  console.log("Starting training loop...");

  for (let epoch = 0; epoch < epochs; epoch++) {
      console.log("\n\n")
    // ---------------------------
    // FORWARD PASS: Y_hat = X * W + b
    // ---------------------------
    const yHat = [];
    
    console.log(`[epoch ${epoch}] bef W: ${W}`)

    for (let i = 0; i < nSamples; i++) {
      // Compute prediction: Y_hat[i] = sum(X[i] * W) + B[0]
      let dotProduct = 0.0;
      for (let j = 0; j < nFeatures; j++) {
        dotProduct += X[i][j] * W[j];
      }
      yHat.push(dotProduct + B[0]);
    }
    
    console.log(`[epoch ${epoch}] yHat: ${yHat}`)

    // ---------------------------
    // ERROR COMPUTATION: E = Y_hat - Y
    // ---------------------------
    const E = [];

    for (let i = 0; i < nSamples; i++) {
      E.push(yHat[i] - Y[i]);
    }
    
    console.log(`[epoch ${epoch}] E: ${E}`)

    // ---------------------------
    // GRADIENT COMPUTATION: gradient = X_T * E
    // ---------------------------
    const gradient = new Array(nFeatures).fill(0.0);

    for (let j = 0; j < nFeatures; j++) {
      for (let i = 0; i < nSamples; i++) {
        gradient[j] += X[i][j] * E[i];
      }
      gradient[j] *= (eta / nSamples);
    }
    
    console.log(`[epoch ${epoch}] gradient: ${gradient}`)

    // ---------------------------
    // WEIGHT UPDATE: W = W - gradient
    // ---------------------------
    for (let j = 0; j < nFeatures; j++) {
      W[j] -= gradient[j];
    }
    
    console.log(`[epoch ${epoch}] W: ${W}`)

    // ---------------------------
    // BIAS UPDATE: B = B - eta * mean(E)
    // ---------------------------
    let eSum = 0.0;
    for (let i = 0; i < E.length; i++) {
      eSum += E[i];
    }
    const scale = eta / nSamples;
    const eScaled = eSum * scale;
    
    for (let j = 0; j < nFeatures; j++) {
      B[j] -= eScaled;
    }
    
    console.log(`[epoch ${epoch}] B: ${B}`)

    // ---------------------------
    // MONITORING
    // ---------------------------
    // Compute MSE
    let loss = 0.0;
    for (let i = 0; i < E.length; i++) {
      loss += E[i] * E[i];
    }
    loss /= E.length;

    console.log(`Epoch ${epoch + 1} : b = ${B[0].toFixed(4)}, MSE = ${loss.toFixed(4)}`);
  }

  // ---------------------------
  // FINAL RESULTS
  // ---------------------------
  console.log("\nFinal model:");
  console.log("Weights:", W.map(w => w.toFixed(4)));
  console.log("Bias:", B[0].toFixed(4));

  // Print final regression function
  console.log("\nRecovered regression function:");
  let equation = "y = ";
  for (let i = 0; i < nFeatures; i++) {
    equation += `${W[i].toFixed(4)} * x${i}`;
    if (i < nFeatures - 1) equation += " + ";
  }
  equation += ` + ${B[0].toFixed(4)}`;
  console.log(equation);

  console.log("\nTRAINING: Training has finished");

  return { W, b: B[0] };
}

function main() {
  const [trainSet, testSet] = loadAndSplitDataset('data/data.csv', 1.0, 1, false);

  console.log(trainSet);

  // Example train call
  train(trainSet, 100, 0.1);
};

main();