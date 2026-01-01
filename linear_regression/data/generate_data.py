import pandas as pd
import numpy as np
import argparse

def generate_mock_data(num_rows, output_file, max_number=0, noise=0):
  """
  Generate mock data following the formula: y = x1 + 2*x2 + 2.5*x3
  
  Parameters:
  - num_rows: number of data rows to generate
  - output_file: name of the output CSV file
  - noise: standard deviation of Gaussian noise to add to y (0 for no noise)
  """
  
  # Generate random data
  data = {
    'x1': np.random.uniform(0, max_number, size=num_rows),
    'x2': np.random.uniform(0, max_number/2, size=num_rows),
    'x3': np.random.uniform(0, max_number/2.5, size=num_rows)
  }
  
  df = pd.DataFrame(data)
  
  # Calculate y using the formula
  df['y'] = df['x1'] + 2 * df['x2'] + 2.5 * df['x3']
  
  # Add noise if specified
  if noise > 0:
    df['y'] += np.random.normal(0, noise, size=num_rows)
  
  # Save to CSV
  df.to_csv(output_file, index=False)
  
  print(f"Generated {num_rows} rows of mock data in '{output_file}'")

if __name__ == "__main__":
  parser = argparse.ArgumentParser(description='Generate mock data following y = x1 + 2*x2 + 2.5*x3')
  parser.add_argument('num_rows', type=int, nargs='?', default=100, help='Number of rows to generate (default: 100)')
  parser.add_argument('output_file', type=str, nargs='?', default='mock_data.csv', help='Output CSV file name (default: mock_data.csv)')
  parser.add_argument('--noise', type=float, default=0, help='Standard deviation of Gaussian noise (default: 0)')
  parser.add_argument('--max_number', type=float, default=16, help='Maximum value for x1, x2, and x3 (default: 16)')
  
  args = parser.parse_args()
  
  generate_mock_data(args.num_rows, args.output_file, args.max_number, args.noise)