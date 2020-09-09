from unittest import TestCase

from eor.road_network.osm_road_network import OSMRoadNetwork
from eor.utils.geo_utils import Coordinate


class TestOSMRoadNetwork(TestCase):
    def test_shortest_path(self):

        # TODO: replace with smaller mock road network

        osm_file = "../resources/chicago.xml"
        network = OSMRoadNetwork(osm_file)

        ohare = Coordinate(lat=41.981022, lon=-87.878541)
        midway = Coordinate(lat=41.785619, lon=-87.741305)

        path = network.shortest_path(ohare, midway)
        start = path[0]
        end = path[-1]

        self.assertAlmostEqual(start.lat, ohare.lat, places=2)
        self.assertAlmostEqual(start.lon, ohare.lon, places=2)
        self.assertAlmostEqual(end.lat, midway.lat, places=2)
        self.assertAlmostEqual(end.lon, midway.lon, places=2)
