import json
import sys
from pathlib import Path
from matplotlib import pyplot as plt


def main():
    default = Path(__file__).parent / "validation.track.json"
    path = Path(sys.argv[1]) if len(sys.argv) > 1 else default
    history: list[float] = [float(x) for x in json.loads(path.read_text())]

    plt.plot(history)
    plt.xlabel("Validation point")
    plt.ylabel("RMSE")
    plt.title("Training Error Over Time")
    plt.show()


if __name__ == "__main__":
    main()
