from nrel.routee.compass.compass_app import CompassApp
from nrel.routee.compass.io.generate_dataset import generate_compass_dataset

from pathlib import Path


def package_root() -> Path:
    return Path(__file__).parent


__all__ = ("CompassApp", "generate_compass_dataset", "package_root")
