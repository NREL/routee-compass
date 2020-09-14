from abc import ABC, abstractmethod
from enum import Enum
from typing import Tuple

from eor.utils.geo_utils import Coordinate


class PathWeight(Enum):
    """
    valid weights for computing shortest path
    """
    TIME = 0
    DISTANCE = 1
    ENERGY = 2


class RoadNetwork(ABC):
    """
    abstract base class for road network
    """

    @abstractmethod
    def shortest_path(
            self,
            origin: Coordinate,
            destination: Coordinate,
            weight: PathWeight = PathWeight.ENERGY,
            routee_key: str = "Gasoline",
    ) -> Tuple[Coordinate, ...]:
        """
        computes weighted shortest path
        :return: shortest path as series of coordinates
        """
