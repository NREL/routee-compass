from pathlib import Path

from nrel.routee.compass.compass_app_py import (
    CompassAppWrapper as CompassApp,
)


def root() -> Path:
    return Path(__file__).parent.parent
