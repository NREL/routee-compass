from typing import NamedTuple, List, Dict

from shapely.geometry import Polygon

GeoJsonFeatures = List


class Coordinate(NamedTuple):
    lat: float
    lon: float


class BoundingBox(NamedTuple):
    left_down_corner: Coordinate
    right_top_corner: Coordinate

    bbox_id: str = "bbox"

    @classmethod
    def from_polygon(cls, polygon: Polygon):
        llon, llat, rlon, rlat = polygon.bounds
        left_down_corner = Coordinate(lat=llat, lon=llon)
        right_top_corner = Coordinate(lat=rlat, lon=rlon)
        return BoundingBox(
            left_down_corner=left_down_corner,
            right_top_corner=right_top_corner
        )

    def as_tomtom_json(self) -> Dict:
        """
        this method is used to generate json object compatiple with tomtom api

        :return:
        """
        out = {
            "leftDownCorner": {
                "longitude": self.left_down_corner.lon,
                "latitude": self.left_down_corner.lat,
            },
            "rightTopCorner": {
                "longitude": self.right_top_corner.lon,
                "latitude": self.right_top_corner.lat,
            }
        }
        return out
