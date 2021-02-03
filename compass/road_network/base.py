from abc import ABC, abstractmethod
from enum import Enum
from typing import Tuple, List, Set, NamedTuple

from compass.datastreams.base import DataStream
from compass.road_network.constructs.link import Link
from compass.utils.geo_utils import Coordinate

Route = Tuple[Coordinate]


class RouteMetadata(NamedTuple):
    """
    route metadata
    """
    route_distance: float
    route_distance_units: str
    route_time: float
    route_time_units: str
    route_energy: float
    route_energy_units: str


class PathWeight(Enum):
    """
    valid weights for computing shortest path
    """
    TIME = 0
    DISTANCE = 1
    ENERGY = 2


class PathResult(NamedTuple):
    """
    return value wrapper for the road network routes
    """
    route: Route
    metadata: RouteMetadata


class RoadNetwork(ABC):
    """
    abstract base class for road network data base
    """

    @abstractmethod
    def update_links(self, links: Tuple[Link, ...]):
        """
        takes in a tuple of links to update the road network
        :return:
        """

    @property
    @abstractmethod
    def data_streams(self) -> List[DataStream]:
        """
        collection of data streams
        :return:
        """

    @abstractmethod
    def add_data_stream(self, data_stream: DataStream):
        """
        adds a new data stream to the road network database

        :param data_stream: DataStream to be added
        :return:
        """

    @property
    @abstractmethod
    def routee_model_keys(self) -> Set[str]:
        """
        returns the routee models keys associated with the road network
        :return:
        """

    @abstractmethod
    def shortest_path(
            self,
            origin: Coordinate,
            destination: Coordinate,
            weight: PathWeight = PathWeight.ENERGY,
            routee_key: str = "Gasoline",
    ) -> PathResult:
        """
        computes weighted shortest path

        :return: a PathResult that includes the route (tuple of coordinates) and the route metadata
        """
