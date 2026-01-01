import pandas as pd
import numpy as np
import argparse

def generate_hospital_data_prob(num_rows, output_file, seed=None):
  """
  Generate mock hospital data with numeric features and probabilistic days in hospital.
  """
  if seed is not None:
    np.random.seed(seed)
 
  # Generate skewed distribution
  raw_cci = np.random.exponential(scale=2, size=num_rows) # scale controls spread
  raw_num_procedures = np.random.exponential(scale=12, size=num_rows) # scale controls spread
  
  data = {
    'age': np.random.randint(0, 101, size=num_rows), # Age [0-100]
    'cci': np.clip(np.round(raw_cci), 0, 38).astype(int), # Charlson Comorbidity Index [0-37] (Spiked at 0, mean 2)
    'num_procedures': np.clip(np.round(raw_num_procedures), 0, 20).astype(int), # Number of previous procedures / surgeries [0-20] (Spiked at 0, mean 2)
    'systolic': np.random.randint(80, 201, size=num_rows), # Systolic [120-180]
    'diastolic': np.random.randint(50, 121, size=num_rows), # Diastolic [80-120]
    'bmi': np.round(np.random.uniform(15, 40, size=num_rows), 1) # Body Mass Index [15-39]
  }
  
  df = pd.DataFrame(data)
  
  # Compute expected days based on features
  expected_days = 0.05 * df['age'] + 0.8 * (df['cci'] - 6) + 0.25 * df['num_procedures']
  
  # Only add days for vitals and BMI outside healthy ranges
  expected_days += 0.01 * np.maximum(0, df['systolic'] - 140)    # high systolic
  expected_days += 0.01 * np.maximum(0, 90 - df['systolic'])     # low systolic
  expected_days += 0.01 * np.maximum(0, df['diastolic'] - 90)    # high diastolic
  expected_days += 0.01 * np.maximum(0, 60 - df['diastolic'])    # low diastolic
  expected_days += 0.05 * np.maximum(0, df['bmi'] - 24.9)        # high BMI
  expected_days += 0.05 * np.maximum(0, 18.5 - df['bmi'])        # low BMI
  
  # Sample actual days using Poisson distribution
  df['days_in_hospital'] = np.random.poisson(lam=np.maximum(expected_days, 0.0))
  
  df.to_csv(output_file, index=False)
  print(f"Generated {num_rows} rows of probabilistic hospital data in '{output_file}'")
  
  # Print statistics
  print("=== Data Statistics ===")
  print(df.describe())  # summary for all numeric columns
  percent_zeros = 100 * np.sum(df['days_in_hospital'] == 0) / len(df)
  print(f"Percentage of zeros in 'days_in_hospital': {percent_zeros:.2f}%")

if __name__ == "__main__":
  parser = argparse.ArgumentParser(description='Generate probabilistic hospital data.')
  parser.add_argument('num_rows', type=int, nargs='?', default=100, help='Number of rows')
  parser.add_argument('output_file', type=str, nargs='?', default='hospital_data_prob.csv', help='CSV output file')
  parser.add_argument('--seed', type=int, default=None, help='Random seed for reproducibility')
  args = parser.parse_args()
  
  generate_hospital_data_prob(args.num_rows, args.output_file, args.seed)
