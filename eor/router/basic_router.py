from typing import Tuple

from eor.road_network.base import RoadNetwork, PathWeight
from eor.router.base import Router
from eor.utils.geo_utils import Coordinate


class BasicRouter(Router):
    """
    basic router that returns lowest energy route
    """

    def __init__(self, road_network: RoadNetwork):
        self.road_network = road_network

    def route(
            self,
            origin: Coordinate,
            destination: Coordinate,
            routee_key: str = "Gasoline",
    ) -> Tuple[Coordinate, ...]:
        """
        generates a route based on an origin and destiantion coordinate
        :return: a tuple of coordinates
        """
        return self.road_network.shortest_path(origin, destination, weight=PathWeight.ENERGY, routee_key=routee_key)
