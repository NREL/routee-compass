from abc import ABC, abstractmethod
from enum import Enum
from typing import Tuple, List

from compass.utils.geo_utils import Coordinate
from compass.road_network.constructs.link import Link
from compass.datastreams.base import DataStream


class PathWeight(Enum):
    """
    valid weights for computing shortest path
    """
    TIME = 0
    DISTANCE = 1
    ENERGY = 2


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
