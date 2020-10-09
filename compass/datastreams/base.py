from abc import ABCMeta, abstractmethod
from typing import Tuple, List, Callable

from compass.road_network.constructs.link import Link


class DataStream(metaclass=ABCMeta):
    """
    abstract base class for a data stream object.

    the idea is take in a road network object and update edge weights with streaming data.

    #TODO: set this up to run asynchronously.
    """

    @property
    @abstractmethod
    def _observers(self) -> List[Callable[[Tuple[Link, ...]], None]]:
        """
        list of observers of this data stream
        :return:
        """

    @abstractmethod
    def collect(self) -> int:
        """
        gathers data
        :return: 1 for success 0 for failure
        """

    @abstractmethod
    def bind_to(self, callback: Callable):
        """
        allows road network update methods to receive updates
        :param callback:
        :return:
        """
