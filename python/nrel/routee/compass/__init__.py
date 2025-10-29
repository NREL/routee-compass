from nrel_routee_compass.compass_app import CompassApp
from nrel_routee_compass.io.generate_dataset import generate_compass_dataset

from pathlib import Path

import logging


def package_root() -> Path:
    return Path(__file__).parent


logging.basicConfig(level=logging.INFO)


__all__ = ("CompassApp", "generate_compass_dataset", "package_root")
