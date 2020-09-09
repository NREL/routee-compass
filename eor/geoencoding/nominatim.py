import requests

from eor.utils.geo_utils import Coordinate
from eor.geoencoding.base import GeoEncoder


class Nominatim(GeoEncoder):
    """
    nominatim geoencoder
    """
    BASE_URL: str = "https://nominatim.openstreetmap.org/search?"
    FORMAT: str = "json"

    def get_coordinates(self, location: str) -> Coordinate:
        """
        gets coordinates from a string
        :return: the coordinate that corresponds to the location string
        """
        url = self.BASE_URL + f"q={location}&format={self.FORMAT}"
        try:
            result = requests.get(url)
        except requests.exceptions.ConnectionError as e:
            raise Exception("Can't connect to Nominatim geoencoder") from e

        result_json = result.json()
        lat = float(result_json[0]['lat'])
        lon = float(result_json[0]['lon'])

        return Coordinate(lat=lat, lon=lon)


