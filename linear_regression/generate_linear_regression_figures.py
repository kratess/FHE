from pathlib import Path

import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
import pandas as pd


ROOT = Path(__file__).resolve().parent
INPUT = ROOT / "results" / "64_to_512_samples.csv"
OUTPUT = ROOT / "figures"
OUTPUT_FILE = OUTPUT / "linear_regression_scaling.png"

REAL_FEATURES = 6
PADDED_FEATURES = 8
COLOR = "#265d97"
ACCENT = "#c26b32"


def load_results(path):
    frame = pd.read_csv(path)
    required_columns = {"samples", "epoch_time_seconds"}
    missing = required_columns.difference(frame.columns)
    if missing:
        raise ValueError(f"Missing required columns: {', '.join(sorted(missing))}")

    frame = frame.copy()
    frame["samples"] = pd.to_numeric(frame["samples"], errors="coerce")
    frame["epoch_time_seconds"] = pd.to_numeric(frame["epoch_time_seconds"], errors="coerce")
    frame = frame.dropna(subset=["samples", "epoch_time_seconds"]).sort_values("samples")
    frame["samples"] = frame["samples"].astype(int)
    frame["required_slots"] = frame["samples"] * PADDED_FEATURES
    frame["ring_dim"] = 2 * frame["required_slots"]
    frame["time_per_sample_ms"] = (frame["epoch_time_seconds"] * 1000.0) / frame["samples"]
    return frame


def sample_to_ring_dim(samples):
    return samples * 2 * PADDED_FEATURES


def ring_dim_to_samples(ring_dim):
    return ring_dim / (2 * PADDED_FEATURES)


def draw_scaling(frame, output_path):
    fig, (ax_time, ax_efficiency) = plt.subplots(
        2,
        1,
        figsize=(8.6, 6.6),
        sharex=True,
        gridspec_kw={"height_ratios": [3.2, 1.6]},
    )

    ax_time.plot(
        frame["samples"],
        frame["epoch_time_seconds"],
        marker="o",
        linewidth=2.2,
        color=COLOR,
    )

    for row in frame.itertuples(index=False):
        ax_time.annotate(
            f"N={row.ring_dim}",
            (row.samples, row.epoch_time_seconds),
            textcoords="offset points",
            xytext=(0, 8),
            ha="center",
            color=ACCENT,
            fontsize=9,
        )

    ax_time.set_ylabel("Tempo medio per epoca (s)")
    ax_time.grid(True, which="major", linestyle=":", alpha=0.45)
    ax_time.yaxis.set_major_formatter(ticker.StrMethodFormatter("{x:g}"))

    top_axis = ax_time.secondary_xaxis("top", functions=(sample_to_ring_dim, ring_dim_to_samples))
    top_axis.set_xlabel("Ring dimension N")
    top_axis.set_xticks(frame["ring_dim"])
    top_axis.set_xticklabels([str(value) for value in frame["ring_dim"]])

    ax_time.text(
        0.02,
        0.96,
        (
            "CKKS linear regression scaling\n"
            f"d = {REAL_FEATURES}, F = nextPow2(d) = {PADDED_FEATURES}, "
            "N = 2 * samples * F"
        ),
        transform=ax_time.transAxes,
        va="top",
        ha="left",
        fontsize=10,
        bbox={"facecolor": "white", "edgecolor": "none", "alpha": 0.85, "pad": 4},
    )

    ax_efficiency.bar(
        frame["samples"],
        frame["time_per_sample_ms"],
        width=36,
        color=ACCENT,
        alpha=0.85,
    )
    ax_efficiency.set_xlabel("Numero di sample")
    ax_efficiency.set_ylabel("ms / sample")
    ax_efficiency.grid(True, axis="y", linestyle=":", alpha=0.45)
    ax_efficiency.yaxis.set_major_formatter(ticker.StrMethodFormatter("{x:g}"))

    sample_ticks = frame["samples"].tolist()
    ax_efficiency.xaxis.set_major_locator(ticker.FixedLocator(sample_ticks))
    ax_efficiency.xaxis.set_major_formatter(
        ticker.FixedFormatter([str(value) for value in sample_ticks])
    )

    fig.tight_layout()
    fig.savefig(output_path, dpi=180)
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
    draw_scaling(load_results(INPUT), OUTPUT_FILE)
