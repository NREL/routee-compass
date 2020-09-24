from abc import ABC, abstractmethod

from compass.road_network.base import RoadNetwork


class DataStream(ABC):
    """
    abstract base class for a data stream object.

    the idea is take in a road network object and update edge weights with streaming data.

    #TODO: set this up to run asynchronously.
    """

    @abstractmethod
    def update(self, road_network: RoadNetwork) -> RoadNetwork:
        """
        takes a road network and returns an updated version.
        #TODO: eventually this should run over road network partitions
        :param road_network:
        :return:
        """
        pass
