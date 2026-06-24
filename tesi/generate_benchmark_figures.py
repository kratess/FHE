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


def load_results(scheme):
    path = ROOT / "benchmark" / scheme / "results.csv"
    lines = path.read_text(encoding="utf-8").splitlines()
    header = next(i for i, line in enumerate(lines) if line.startswith("name,"))
    frame = pd.read_csv(path, header=header)
    parts = frame["name"].str.split("/")
    frame["operation"] = parts.str[0]
    frame["depth"] = pd.to_numeric(parts.str[1], errors="coerce")
    frame["ring_dim"] = pd.to_numeric(parts.str[2], errors="coerce")
    return frame


def draw(operation, filename, ylabel, metric="cpu_time"):
    fig, ax = plt.subplots(figsize=(8.4, 4.8))
    for scheme in SCHEMES:
        frame = load_results(scheme)
        selected = frame[
            (frame["operation"] == operation)
            & (frame["depth"] == DEPTH)
            & (frame["ring_dim"].isin(RING_DIMS))
        ].sort_values("ring_dim")
        ax.plot(
            selected["ring_dim"],
            selected[metric],
            marker="o",
            linewidth=2.2,
            label=scheme.upper(),
            color=COLORS[scheme],
        )

    ax.set_xscale("linear")
    ax.xaxis.set_major_locator(ticker.FixedLocator(RING_DIMS))
    ax.xaxis.set_major_formatter(
        ticker.FixedFormatter([str(value) for value in RING_DIMS])
    )
    ax.xaxis.set_minor_locator(ticker.NullLocator())
    ax.set_yscale("linear")
    ax.yaxis.set_major_formatter(ticker.StrMethodFormatter("{x:g}"))
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
    draw("BM_Encrypt", "benchmark_encrypt.png", "Tempo CPU (ms)")
    draw("BM_EvalAdd", "benchmark_add.png", "Tempo CPU (ms)")
    draw("BM_EvalMult", "benchmark_mult.png", "Tempo CPU (ms)")
    draw("BM_Decrypt", "benchmark_decrypt.png", "Tempo CPU (ms)")
    draw("BM_Encrypt", "benchmark_memory.png", "Heap osservato (MB)", metric="MB")
