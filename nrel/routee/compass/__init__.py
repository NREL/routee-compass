from pathlib import Path

from polestar.constructs.geometry import Coordinate


def root() -> Path:
    return Path(__file__).parent.parent
