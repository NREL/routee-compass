import os
from unittest import TestCase

from compass.road_network.base import PathWeight
from compass.road_network.osm_road_network import OSMRoadNetwork
from compass.utils.geo_utils import Coordinate


class TestOSMRoadNetwork(TestCase):
    def setUp(self) -> None:
        self.road_network_file = os.path.join("test_assets", "denver_downtown_osm_network.pickle")
        self.road_network = OSMRoadNetwork(self.road_network_file)

        self.home_plate = Coordinate(lat=39.754372, lon=-104.994300)
        self.bk_lounge = Coordinate(lat=39.779098, lon=-104.951241)

    def test_shortest_path_distance(self):
        path = self.road_network.shortest_path(self.home_plate, self.bk_lounge, weight=PathWeight.DISTANCE)
        start = path[0]
        end = path[-1]

        self.assertAlmostEqual(start.lat, self.home_plate.lat, places=2)
        self.assertAlmostEqual(start.lon, self.home_plate.lon, places=2)
        self.assertAlmostEqual(end.lat, self.bk_lounge.lat, places=2)
        self.assertAlmostEqual(end.lon, self.bk_lounge.lon, places=2)

    def test_shortest_path_time(self):
        # TODO: how can we actually test this is the shortest time route? -ndr

        path = self.road_network.shortest_path(self.home_plate, self.bk_lounge, weight=PathWeight.TIME)
        start = path[0]
        end = path[-1]

        self.assertAlmostEqual(start.lat, self.home_plate.lat, places=2)
        self.assertAlmostEqual(start.lon, self.home_plate.lon, places=2)
        self.assertAlmostEqual(end.lat, self.bk_lounge.lat, places=2)
        self.assertAlmostEqual(end.lon, self.bk_lounge.lon, places=2)

    def test_shortest_path_energy(self):
        # TODO: how can we actually test this is the shortest energy route? -ndr

        path = self.road_network.shortest_path(self.home_plate, self.bk_lounge, weight=PathWeight.ENERGY)
        start = path[0]
        end = path[-1]

        self.assertAlmostEqual(start.lat, self.home_plate.lat, places=2)
        self.assertAlmostEqual(start.lon, self.home_plate.lon, places=2)
        self.assertAlmostEqual(end.lat, self.bk_lounge.lat, places=2)
        self.assertAlmostEqual(end.lon, self.bk_lounge.lon, places=2)
