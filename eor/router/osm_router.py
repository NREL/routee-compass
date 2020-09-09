from typing import Tuple

from eor.utils.geo_utils import Coordinate
from eor.router.base import Router


class OSMRouter(Router):
    """
    osm router
    """

    def route(self, origin: Coordinate, destination: Coordinate) -> Tuple[Coordinate, ...]:
        """
        generates a route based on an origin and destiantion coordinate
        :return: a tuple of coordinates
        """
        pass
