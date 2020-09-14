from unittest import TestCase

from eor.road_network.osm_road_network import OSMRoadNetwork
from eor.router.basic_router import BasicRouter
from eor.utils.geo_utils import Coordinate


class TestBasicRouter(TestCase):
    def test_basic_route(self):
        osm_file = "../resources/denver_roadnetwork.pickle"
        network = OSMRoadNetwork(osm_file)
        router = BasicRouter(network)

        home_plate = Coordinate(lat=39.754372, lon=-104.994300)
        wash_park = Coordinate(lat=39.693000, lon=-104.9734343)

        path = router.route(home_plate, wash_park)
        start = path[0]
        end = path[-1]

        self.assertAlmostEqual(start.lat, home_plate.lat, places=2)
        self.assertAlmostEqual(start.lon, home_plate.lon, places=2)
        self.assertAlmostEqual(end.lat, wash_park.lat, places=2)
        self.assertAlmostEqual(end.lon, wash_park.lon, places=2)
