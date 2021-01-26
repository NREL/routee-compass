import os
from unittest import TestCase

import geopandas as gpd

from compass.road_network.base import PathWeight
from compass.road_network.constructs.link import Link
from compass.road_network.tomtom_networkx import TomTomNetworkX
from compass.utils.geo_utils import Coordinate, BoundingBox
from tests import test_dir


class TestTomTomRoadNetwork(TestCase):
    def setUp(self) -> None:
        self.road_network_file = test_dir() / "test_assets" / "denver_downtown_tomtom_network.pickle"
        self.bbox_file = os.path.join("test_assets", "denver_downtown_bounding_box", "denver_downtown_roadnetwork.shp")
        self.bbox = BoundingBox.from_polygon(gpd.read_file(self.bbox_file).iloc[0].geometry)
        self.road_network = TomTomNetworkX(self.road_network_file)

        self.home_plate = Coordinate(lat=39.754372, lon=-104.994300)
        self.bk_lounge = Coordinate(lat=39.779098, lon=-104.951241)

    def test_shortest_path_distance(self):
        path, _ = self.road_network.shortest_path(self.home_plate, self.bk_lounge, weight=PathWeight.DISTANCE)
        start = path[0]
        end = path[-1]

        self.assertAlmostEqual(start.lat, self.home_plate.lat, places=2)
        self.assertAlmostEqual(start.lon, self.home_plate.lon, places=2)
        self.assertAlmostEqual(end.lat, self.bk_lounge.lat, places=2)
        self.assertAlmostEqual(end.lon, self.bk_lounge.lon, places=2)

    def test_shortest_path_time(self):
        # TODO: how can we actually test this is the shortest time route? -ndr

        path, _ = self.road_network.shortest_path(self.home_plate, self.bk_lounge, weight=PathWeight.TIME)
        start = path[0]
        end = path[-1]

        self.assertAlmostEqual(start.lat, self.home_plate.lat, places=2)
        self.assertAlmostEqual(start.lon, self.home_plate.lon, places=2)
        self.assertAlmostEqual(end.lat, self.bk_lounge.lat, places=2)
        self.assertAlmostEqual(end.lon, self.bk_lounge.lon, places=2)

    def test_shortest_path_energy(self):
        # TODO: how can we actually test this is the shortest energy route? -ndr

        path, _ = self.road_network.shortest_path(self.home_plate, self.bk_lounge, weight=PathWeight.ENERGY)
        start = path[0]
        end = path[-1]

        self.assertAlmostEqual(start.lat, self.home_plate.lat, places=2)
        self.assertAlmostEqual(start.lon, self.home_plate.lon, places=2)
        self.assertAlmostEqual(end.lat, self.bk_lounge.lat, places=2)
        self.assertAlmostEqual(end.lon, self.bk_lounge.lon, places=2)

    def test_update_links(self):
        """
        :return:
        """

        # these link ids were picked at random from the road network
        updated_link = Link(link_id=-88400000018714, attributes={'kph': 100, 'minutes': 100})
        another_updated_link = Link(link_id=88400000018763, attributes={'kph': -100, 'minutes': -100})

        self.road_network.update_links((updated_link, another_updated_link))

        link_1 = list(
            filter(lambda t: t[2] == updated_link.link_id, self.road_network.G.edges(data=True, keys=True))
        )[0]

        link_2 = list(
            filter(lambda t: t[2] == another_updated_link.link_id, self.road_network.G.edges(data=True, keys=True))
        )[0]

        self.assertEqual(updated_link.attributes['kph'], link_1[3]['kph'])
        self.assertEqual(another_updated_link.attributes['minutes'], link_2[3]['minutes'])
