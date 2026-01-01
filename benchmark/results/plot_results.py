import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
import seaborn as sns
import os
import argparse

def parse_benchmark_name(name):
    """
    Parses strings like "BM_KeyGen/1/256" or "BM_Bootstrap/1/256/iterations:1"
    Returns (operation, depth, slots)
    """
    name = name.split('/iterations:')[0]
    parts = name.split('/')
    if len(parts) >= 3:
        operation = parts[0]
        try:
            depth = int(parts[1])
            slots = int(parts[2])
            return operation, depth, slots
        except ValueError:
            return None, None, None
    return None, None, None

def convert_units(df, current_unit):
    """
    Converts time values to the most appropriate human-readable unit.
    Returns (converted_series, new_unit_label)
    """
    # Google Benchmark usually uses 'ns', 'us', 'ms', 's'
    to_ns = {'ns': 1, 'us': 1000, 'ms': 1000000, 's': 1000000000, 'mb': 1000000}
    
    # If the unit is unknown, we just return as is
    if current_unit.lower() not in to_ns:
        return df, current_unit

    values_ns = df * to_ns[current_unit.lower()]
    max_val = values_ns.max()
    
    if max_val >= 1000000000:
        return values_ns / 1000000000, 's'
    elif max_val >= 1000000:
        return values_ns / 1000000, 'ms'
    elif max_val >= 1000:
        return values_ns / 1000, 'us'
    else:
        return values_ns, 'ns'

DEPTH_COLORS = {
    "1": "#f77189",
    "2": "#dc8932",
    "4": "#ae9d31",
    "8": "#77ab31",
    "16": "#33b07a",
    "32": "#36ada4",
    "64": "#38a9c5",
    "128": "#6e9bf4",
    "256": "#cc7af4",
    "512": "#f565cc"
}

def plot_benchmark_results(scheme_name, csv_file, output_dir, max_depth=None, max_ring_dim=None):
    if not os.path.exists(csv_file):
        print(f"Skipping {scheme_name}: {csv_file} not found.")
        return

    os.makedirs(output_dir, exist_ok=True)
    print(f"Processing {scheme_name} results from {csv_file}...")

    try:
        # Find the line where the actual CSV header starts
        header_row = 0
        with open(csv_file, 'r') as f:
            lines = f.readlines()
            for i, line in enumerate(lines):
                if line.startswith('name,'):
                    header_row = i
                    break
        
        df = pd.read_csv(csv_file, header=header_row)

        # Parse metadata from 'name'
        ops, depths, ring_dims = [], [], []
        for name in df['name']:
            op, d, r = parse_benchmark_name(name)
            ops.append(op)
            depths.append(d)
            ring_dims.append(r)
            
        df['Operation'] = ops
        df['Depth_int'] = depths
        df['Depth'] = [str(d) for d in depths]
        df['RingDim'] = ring_dims
        df = df.dropna(subset=['Operation'])

        # Apply Filters
        if max_depth is not None:
            df = df[df['Depth_int'] <= max_depth]
        if max_ring_dim is not None:
            df = df[df['RingDim'] <= max_ring_dim]

        if df.empty:
            print(f"No results remain for {scheme_name} after filtering.")
            return

        # Unique depths for color mapping
        unique_depths_str = [str(d) for d in sorted(df['Depth_int'].unique())]
        
        # Create a palette using the fixed colors if available, otherwise fallback to husl
        depth_palette = {}
        for d_str in unique_depths_str:
            if d_str in DEPTH_COLORS:
                depth_palette[d_str] = DEPTH_COLORS[d_str]
            else:
                # Fallback for unexpected depths
                depth_palette[d_str] = sns.color_palette("husl", 10)[hash(d_str) % 10]
        
        hue_order = unique_depths_str

        operations = df['Operation'].unique()
        sns.set_theme(style="whitegrid")

        for op in operations:
            subset = df[df['Operation'] == op].copy()
            if subset.empty:
                continue

            metrics = []
            # Prefer cpu_time if available
            time_col = 'cpu_time' if 'cpu_time' in df.columns else 'real_time'
            if time_col in subset.columns:
                metrics.append((time_col, 'Time'))
            if 'MB' in subset.columns:
                metrics.append(('MB', 'Memory'))

            for metric_col, metric_type in metrics:
                y_col = metric_col
                if metric_type == 'Time':
                    raw_unit = subset['time_unit'].iloc[0] if 'time_unit' in subset.columns else 'ms'
                    subset_p = subset.copy()
                    # Convert to best unit for visualization
                    subset_p[y_col], unit_label = convert_units(subset_p[y_col], raw_unit)
                    y_label = f'Time ({unit_label})'
                    plot_title = f'{scheme_name} Benchmark: {op} (Execution Time)'
                    suffix = ""
                else:
                    subset_p = subset.copy()
                    y_label = 'Memory (MB)'
                    plot_title = f'{scheme_name} Benchmark: {op} (Memory Usage)'
                    suffix = "_memory"

                plt.figure(figsize=(10, 6))
                op_hue_order = [d for d in hue_order if d in subset_p['Depth'].unique()]
                
                ax = sns.lineplot(
                    data=subset_p, 
                    x='RingDim', 
                    y=y_col, 
                    hue='Depth', 
                    hue_order=op_hue_order,
                    palette=depth_palette,
                    marker='o', 
                    linewidth=2.5
                )
                
                ax.set_xscale('log')
                unique_rd = sorted(subset_p['RingDim'].unique())
                ax.set_xticks(unique_rd)
                formatter = ticker.ScalarFormatter()
                formatter.set_scientific(False)
                ax.xaxis.set_major_formatter(formatter)
                ax.xaxis.set_minor_formatter(ticker.NullFormatter()) 
                
                ax.set_yscale('linear')
                max_y = subset_p[y_col].max()
                if max_y > 0:
                    plt.ylim(0, max_y * 1.5)
                else:
                    plt.ylim(0, 1)
                ax.yaxis.set_major_locator(ticker.MaxNLocator(nbins=12, integer=True))
                y_formatter = ticker.ScalarFormatter()
                y_formatter.set_scientific(False)
                ax.yaxis.set_major_formatter(y_formatter)
                
                plt.title(plot_title, fontsize=14, pad=15)
                plt.xlabel('Ring Dimension (N)', fontsize=12)
                plt.ylabel(y_label, fontsize=12)
                
                plt.legend(title='Depth', title_fontsize=12, fontsize=10, bbox_to_anchor=(1.05, 1), loc='upper left')
                plt.grid(True, which="major", ls="-", alpha=0.6)
                plt.grid(True, which="minor", ls=":", alpha=0.3)
                
                plt.tight_layout()
                filename = f"{output_dir}/{op}{suffix}.png"
                plt.savefig(filename, dpi=150)
                plt.close()
                print(f"Saved plot: {filename}")
            
    except Exception as e:
        print(f"Failed to process {scheme_name} CSV: {e}")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='Generate plots for FHE benchmarks.')
    parser.add_argument('--scheme', choices=['ckks', 'bgv', 'bfv'],
                        help='Specific scheme to generate plots for. If omitted, all detected results are processed.')
    parser.add_argument('--max-depth', type=int, help='Maximum multiplicative depth to include in plots.')
    parser.add_argument('--max-ring-dim', type=int, help='Maximum ring dimension to include in plots.')
    
    args = parser.parse_args()

    # Script location
    script_dir = os.path.dirname(os.path.abspath(__file__))
    # Base directory is one level up from results/ (if script is in results/)
    base_dir = os.path.dirname(script_dir)
    
    schemes_to_process = [args.scheme] if args.scheme else ['ckks', 'bgv', 'bfv']
    for scheme in schemes_to_process:
        csv_path = os.path.join(base_dir, scheme, 'results.csv')
        output_path = os.path.join(script_dir, scheme)
        if os.path.exists(csv_path):
            plot_benchmark_results(scheme.upper(), csv_path, output_path, 
                                   max_depth=args.max_depth, 
                                   max_ring_dim=args.max_ring_dim)
        else:
            print(f"Skipping {scheme.upper()} (results.csv not found at {csv_path})")
