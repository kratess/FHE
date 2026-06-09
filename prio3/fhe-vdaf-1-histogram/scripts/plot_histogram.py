import argparse
import ast
from pathlib import Path

import matplotlib.pyplot as plt


def parse_histogram(result_path: Path) -> list[int]:
    text = result_path.read_text()
    for line in text.splitlines():
        if line.startswith("collector_histogram="):
            return list(ast.literal_eval(line.split("=", 1)[1].strip()))
        if line.startswith("collector_decoded_slots="):
            fallback = list(ast.literal_eval(line.split("=", 1)[1].strip()))
    try:
        return fallback
    except UnboundLocalError as exc:
        raise ValueError(f"no histogram found in {result_path}") from exc


def main() -> None:
    parser = argparse.ArgumentParser(description="Display a histogram from collector output.")
    parser.add_argument(
        "result",
        nargs="?",
        default=Path("runtime") / "collector" / "result.txt",
        type=Path,
        help="Path to collector result.txt",
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="Optional image path to save instead of displaying interactively",
    )
    args = parser.parse_args()

    histogram = parse_histogram(args.result)
    bucket_ids = list(range(len(histogram)))

    plt.figure(figsize=(8, 4.5))
    plt.bar(bucket_ids, histogram, color="#2f6db2")
    plt.title("FHE VDAF 1 Histogram")
    plt.xlabel("Bucket")
    plt.ylabel("Count")
    plt.xticks(bucket_ids)
    plt.tight_layout()

    if args.output is not None:
        plt.savefig(args.output, dpi=150)
        print(f"saved plot to {args.output}")
    else:
        plt.show()


if __name__ == "__main__":
    main()
