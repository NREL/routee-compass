from unittest import TestCase
import os

from compass.road_network.tomtom_road_network import TomTomRoadNetwork
from compass.router.basic_router import BasicRouter
from compass.utils.geo_utils import Coordinate


class TestBasicRouter(TestCase):
    def setUp(self) -> None:
        self.road_network_file = os.path.join("test_assets", "denver_downtown_tomtom_network.pickle")
        self.road_network = TomTomRoadNetwork(self.road_network_file)

        self.home_plate = Coordinate(lat=39.754372, lon=-104.994300)
        self.bk_lounge = Coordinate(lat=39.779098, lon=-104.951241)
        
    def test_basic_route(self):
        router = BasicRouter(self.road_network)

        path = router.route(self.home_plate, self.bk_lounge)
        start = path[0]
        end = path[-1]

        self.assertAlmostEqual(start.lat, self.home_plate.lat, places=2)
        self.assertAlmostEqual(start.lon, self.home_plate.lon, places=2)
        self.assertAlmostEqual(end.lat, self.bk_lounge.lat, places=2)
        self.assertAlmostEqual(end.lon, self.bk_lounge.lon, places=2)
