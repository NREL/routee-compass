from abc import ABC, abstractmethod
from typing import Tuple

from eor.utils.geo_utils import Coordinate


class Router(ABC):
    """
    abstract base class for a router
    """

    @abstractmethod
    def route(self, origin: Coordinate, destination: Coordinate) -> Tuple[Coordinate, ...]:
        """
        generates a route based on an origin and destiantion coordinate
        :return: a tuple of coordinates
        """
