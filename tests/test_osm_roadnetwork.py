from unittest import TestCase

from eor.road_network.base import PathWeight
from eor.road_network.osm_road_network import OSMRoadNetwork
from eor.utils.geo_utils import Coordinate


class TestOSMRoadNetwork(TestCase):
    def test_shortest_path_distance(self):
        # TODO: replace with smaller mock road network and share across tests -ndr

        osm_file = "../resources/denver_roadnetwork.pickle"
        network = OSMRoadNetwork(osm_file)

        home_plate = Coordinate(lat=39.754372, lon=-104.994300)
        wash_park = Coordinate(lat=39.693000, lon=-104.9734343)

        path = network.shortest_path(home_plate, wash_park, weight=PathWeight.DISTANCE)
        start = path[0]
        end = path[-1]

        self.assertAlmostEqual(start.lat, home_plate.lat, places=2)
        self.assertAlmostEqual(start.lon, home_plate.lon, places=2)
        self.assertAlmostEqual(end.lat, wash_park.lat, places=2)
        self.assertAlmostEqual(end.lon, wash_park.lon, places=2)

    def test_shortest_path_time(self):
        # TODO: how can we actually test this is the shortest time route? -ndr

        osm_file = "../resources/denver_roadnetwork.pickle"
        network = OSMRoadNetwork(osm_file)

        home_plate = Coordinate(lat=39.754372, lon=-104.994300)
        wash_park = Coordinate(lat=39.693000, lon=-104.9734343)

        path = network.shortest_path(home_plate, wash_park, weight=PathWeight.TIME)
        start = path[0]
        end = path[-1]

        self.assertAlmostEqual(start.lat, home_plate.lat, places=2)
        self.assertAlmostEqual(start.lon, home_plate.lon, places=2)
        self.assertAlmostEqual(end.lat, wash_park.lat, places=2)
        self.assertAlmostEqual(end.lon, wash_park.lon, places=2)

    def test_shortest_path_energy(self):
        # TODO: how can we actually test this is the shortest energy route? -ndr

        osm_file = "../resources/denver_roadnetwork.pickle"
        network = OSMRoadNetwork(osm_file)

        home_plate = Coordinate(lat=39.754372, lon=-104.994300)
        wash_park = Coordinate(lat=39.693000, lon=-104.9734343)

        path = network.shortest_path(home_plate, wash_park, weight=PathWeight.ENERGY)
        start = path[0]
        end = path[-1]

        self.assertAlmostEqual(start.lat, home_plate.lat, places=2)
        self.assertAlmostEqual(start.lon, home_plate.lon, places=2)
        self.assertAlmostEqual(end.lat, wash_park.lat, places=2)
        self.assertAlmostEqual(end.lon, wash_park.lon, places=2)
