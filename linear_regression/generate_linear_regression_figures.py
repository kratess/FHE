from pathlib import Path

import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
import pandas as pd


ROOT = Path(__file__).resolve().parent
INPUT_10_EPOCHS = ROOT / "results" / "64_to_512_samples_10_epochs.csv"
INPUT_20_EPOCHS = ROOT / "results" / "64_to_512_samples_20_epochs.csv"
INPUT_TIMES_512_20_EPOCHS = ROOT / "results" / "times_512_samples_20_epochs.csv"
OUTPUT = ROOT / "figures"
OUTPUT_FILE = OUTPUT / "linear_regression_scaling.png"
OUTPUT_TIMES_FILE = OUTPUT / "linear_regression_512_samples_20_epochs.png"

PADDED_FEATURES = 8
SERIES = (
    ("10 epoche", INPUT_10_EPOCHS, "#265d97"),
    ("20 epoche", INPUT_20_EPOCHS, "#c26b32"),
)
ANNOTATION_COLOR = "#7a4a1c"


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
    return frame


def load_epoch_times(path):
    frame = pd.read_csv(path)
    required_columns = {"epoch", "epoch_time_seconds"}
    missing = required_columns.difference(frame.columns)
    if missing:
        raise ValueError(f"Missing required columns: {', '.join(sorted(missing))}")

    frame = frame.copy()
    frame["epoch"] = pd.to_numeric(frame["epoch"], errors="coerce")
    frame["epoch_time_seconds"] = pd.to_numeric(frame["epoch_time_seconds"], errors="coerce")
    frame = frame.dropna(subset=["epoch", "epoch_time_seconds"]).sort_values("epoch")
    frame["epoch"] = frame["epoch"].astype(int)
    return frame


def sample_to_ring_dim(samples):
    return samples * 2 * PADDED_FEATURES


def ring_dim_to_samples(ring_dim):
    return ring_dim / (2 * PADDED_FEATURES)


def draw_scaling(frames, output_path):
    fig, ax_time = plt.subplots(figsize=(8.8, 5.2))

    base_frame = frames[0][1]
    y_max = max(frame["epoch_time_seconds"].max() for _, frame, _ in frames)

    for label, frame, color in frames:
        ax_time.plot(
            frame["samples"],
            frame["epoch_time_seconds"],
            marker="o",
            markersize=7,
            linewidth=2.5,
            color=color,
            label=label,
        )
        ax_time.fill_between(
            frame["samples"],
            frame["epoch_time_seconds"],
            [0] * len(frame),
            color=color,
            alpha=0.06,
        )

    for row in base_frame.itertuples(index=False):
        ax_time.annotate(
            f"N={row.ring_dim}",
            (row.samples, row.epoch_time_seconds),
            textcoords="offset points",
            xytext=(0, -24),
            ha="center",
            va="top",
            color=ANNOTATION_COLOR,
            fontsize=10,
            fontweight="semibold",
        )

    ax_time.set_ylabel("Tempo medio per epoca (s)")
    ax_time.set_xlabel("Numero di sample")
    ax_time.grid(True, which="major", linestyle=":", alpha=0.45)
    ax_time.yaxis.set_major_formatter(ticker.StrMethodFormatter("{x:g}"))
    ax_time.legend(frameon=False, loc="upper left", ncols=2, bbox_to_anchor=(0.01, 0.99), borderaxespad=0.0)
    ax_time.set_ylim(0, y_max * 1.24)

    sample_ticks = base_frame["samples"].tolist()
    ax_time.xaxis.set_major_locator(ticker.FixedLocator(sample_ticks))
    ax_time.xaxis.set_major_formatter(
        ticker.FixedFormatter([str(value) for value in sample_ticks])
    )

    top_axis = ax_time.secondary_xaxis("top", functions=(sample_to_ring_dim, ring_dim_to_samples))
    top_axis.set_xlabel("Ring dimension N")
    top_axis.set_xticks(base_frame["ring_dim"])
    top_axis.set_xticklabels([str(value) for value in base_frame["ring_dim"]])
    top_axis.tick_params(axis="x", pad=4)

    fig.tight_layout()
    fig.savefig(output_path, dpi=180)
    plt.close(fig)


def draw_epoch_times(frame, output_path):
    fig, ax = plt.subplots(figsize=(8.8, 5.2))

    bootstrap_epoch = 14
    bootstrap_time = frame.loc[frame["epoch"] == bootstrap_epoch, "epoch_time_seconds"].iloc[0]
    pre_mask = frame["epoch"] < bootstrap_epoch
    post_mask = frame["epoch"] >= bootstrap_epoch

    ax.axvspan(0.5, bootstrap_epoch - 0.5, color="#265d97", alpha=0.05)
    ax.axvspan(bootstrap_epoch - 0.5, frame["epoch"].max() + 0.5, color="#c26b32", alpha=0.05)

    ax.plot(
        frame["epoch"],
        frame["epoch_time_seconds"],
        marker="o",
        markersize=7,
        linewidth=2.6,
        color="#265d97",
    )
    ax.plot(
        frame.loc[pre_mask, "epoch"],
        frame.loc[pre_mask, "epoch_time_seconds"],
        linewidth=0,
        marker="o",
        markersize=7,
        color="#265d97",
    )
    ax.plot(
        frame.loc[post_mask, "epoch"],
        frame.loc[post_mask, "epoch_time_seconds"],
        linewidth=0,
        marker="o",
        markersize=7,
        color="#c26b32",
    )

    ax.axvline(bootstrap_epoch, color="#c26b32", linestyle="--", linewidth=1.6, alpha=0.9)
    ax.annotate(
        "bootstrap",
        (bootstrap_epoch, bootstrap_time),
        textcoords="offset points",
        xytext=(12, 12),
        ha="left",
        color="#c26b32",
        fontsize=11,
        fontweight="semibold",
    )

    ax.text(0.02, 0.96, "Prima del bootstrap", transform=ax.transAxes, va="top", ha="left", fontsize=10, color="#265d97")
    ax.text(0.98, 0.96, "Dopo il bootstrap", transform=ax.transAxes, va="top", ha="right", fontsize=10, color="#c26b32")

    ax.set_xlabel("Epoca")
    ax.set_ylabel("Tempo per epoca (s)")
    ax.grid(True, which="major", linestyle=":", alpha=0.45)
    ax.yaxis.set_major_formatter(ticker.StrMethodFormatter("{x:g}"))
    ax.xaxis.set_major_locator(ticker.FixedLocator(frame["epoch"].tolist()))
    ax.xaxis.set_major_formatter(
        ticker.FixedFormatter([str(value) for value in frame["epoch"].tolist()])
    )
    ax.set_xlim(0.5, frame["epoch"].max() + 0.5)

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
    frames = [(label, load_results(path), color) for label, path, color in SERIES]
    draw_scaling(frames, OUTPUT_FILE)
    draw_epoch_times(load_epoch_times(INPUT_TIMES_512_20_EPOCHS), OUTPUT_TIMES_FILE)
