from pathlib import Path

import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
import pandas as pd


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = Path(__file__).resolve().parent / "figures"
SCHEMES = ("bfv", "bgv", "ckks")
COLORS = {"bfv": "#1f6f5f", "bgv": "#c26b32", "ckks": "#265d97"}
RING_DIMS = (1024, 2048, 4096, 8192, 16384)
DEPTH = 16
AGGREGATE_SUFFIXES = ("_mean", "_stddev")


def split_aggregate_name(name):
    for suffix in AGGREGATE_SUFFIXES:
        if name.endswith(suffix):
            return name[:-len(suffix)], suffix[1:]
    return name, None


def load_results(scheme):
    path = ROOT / "benchmark" / scheme / "results.csv"
    lines = path.read_text(encoding="utf-8").splitlines()
    header = next(i for i, line in enumerate(lines) if line.startswith("name,"))
    frame = pd.read_csv(path, header=header)
    split_names = frame["name"].map(split_aggregate_name)
    frame["base_name"] = split_names.str[0]
    frame["aggregate_name"] = split_names.str[1]
    parts = frame["base_name"].str.split("/")
    frame["operation"] = parts.str[0]
    frame["depth"] = pd.to_numeric(parts.str[1], errors="coerce")
    frame["ring_dim"] = pd.to_numeric(parts.str[2], errors="coerce")
    frame = frame.dropna(subset=["operation"]).copy()

    value_columns = [col for col in ("cpu_time", "real_time", "MB") if col in frame.columns]
    for col in value_columns:
        frame[col] = pd.to_numeric(frame[col], errors="coerce")

    key_cols = ["base_name", "operation", "depth", "ring_dim"]
    if frame["aggregate_name"].notna().any():
        mean_rows = frame[frame["aggregate_name"] == "mean"].copy()
        if mean_rows.empty:
            mean_rows = frame[frame["aggregate_name"].isna()].copy()
        stddev_rows = frame[frame["aggregate_name"] == "stddev"][key_cols + value_columns].copy()
        stddev_rows = stddev_rows.rename(columns={col: f"{col}_stddev" for col in value_columns})
        return mean_rows.merge(stddev_rows, on=key_cols, how="left")

    raw_rows = frame[frame["aggregate_name"].isna()].copy()
    grouped = raw_rows.groupby(key_cols, as_index=False)
    aggregated = grouped[value_columns].mean()
    stddev = grouped[value_columns].std(ddof=1).rename(columns={col: f"{col}_stddev" for col in value_columns})
    return aggregated.merge(stddev, on=key_cols, how="left")


def draw(operation, filename, ylabel, title, metric="cpu_time", schemes=SCHEMES):
    fig, ax = plt.subplots(figsize=(8.4, 4.8))
    plotted = False
    for scheme in schemes:
        frame = load_results(scheme)
        selected = frame[
            (frame["operation"] == operation)
            & (frame["depth"] == DEPTH)
            & (frame["ring_dim"].isin(RING_DIMS))
        ].sort_values("ring_dim")
        if selected.empty:
            continue
        plotted = True
        ax.plot(
            selected["ring_dim"],
            selected[metric],
            marker="o",
            linewidth=2.2,
            label=scheme.upper(),
            color=COLORS[scheme],
        )
        stddev_col = f"{metric}_stddev"
        if stddev_col in selected.columns and selected[stddev_col].fillna(0).gt(0).any():
            ax.errorbar(
                selected["ring_dim"],
                selected[metric],
                yerr=selected[stddev_col].fillna(0),
                fmt="none",
                ecolor=COLORS[scheme],
                elinewidth=1.1,
                capsize=3,
                alpha=0.75,
            )

    if not plotted:
        plt.close(fig)
        return

    ax.set_xscale("linear")
    ax.xaxis.set_major_locator(ticker.FixedLocator(RING_DIMS))
    ax.xaxis.set_major_formatter(
        ticker.FixedFormatter([str(value) for value in RING_DIMS])
    )
    ax.xaxis.set_minor_locator(ticker.NullLocator())
    ax.set_yscale("linear")
    ax.yaxis.set_major_formatter(ticker.StrMethodFormatter("{x:g}"))
    ax.set_title(title, pad=12)
    ax.set_xlabel("Dimensione dell'anello N")
    ax.set_ylabel(ylabel)
    ax.grid(True, which="both", linestyle=":", alpha=0.45)
    ax.legend(frameon=False, ncols=3)
    fig.tight_layout()
    fig.savefig(OUTPUT / filename, dpi=180)
    plt.close(fig)


if __name__ == "__main__":
    OUTPUT.mkdir(parents=True, exist_ok=True)
    plt.rcParams.update(
        {
            "font.family": "DejaVu Serif",
            "axes.spines.top": False,
            "axes.spines.right": False,
        }
    )
    draw("BM_EvalAdd", "benchmark_add.png", "Tempo (ms)", "Somma omomorfica — BFV, BGV, CKKS")
    draw("BM_EvalMult", "benchmark_mult.png", "Tempo (ms)", "Moltiplicazione omomorfica — BFV, BGV, CKKS")
    draw("BM_Bootstrap", "benchmark_bootstrap.png", "Tempo (ms)", "Bootstrapping — CKKS", schemes=("ckks",))
    draw("BM_ContextCreation", "benchmark_memory.png", "Heap osservato (MB)", "Creazione del contesto — BFV, BGV, CKKS", metric="MB")
