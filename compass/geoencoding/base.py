from abc import ABC, abstractmethod

from compass.utils.geo_utils import Coordinate


class GeoEncoder(ABC):
    """
    abstract base class for geoencoding
    """

    @abstractmethod
    def get_coordinates(self, location: str) -> Coordinate:
        """
        gets coordinates from a string
        :return: the coordinate that corresponds to the location string
        """
