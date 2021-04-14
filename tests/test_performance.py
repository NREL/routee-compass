import os
import random
import timeit
from typing import Tuple
from unittest import TestCase, skip

import geopandas as gpd

from compass.road_network.base import PathWeight
from compass.road_network.tomtom_networkx import TomTomNetworkX
from compass.utils.geo_utils import Coordinate, BoundingBox
from tests import test_dir


# don't run with the entire test suite

@skip
class TestPerformance(TestCase):
    def setUp(self) -> None:
        self.road_network_file = test_dir() / "test_assets" / "denver_downtown_tomtom_network.pickle"
        self.bbox_file = os.path.join("test_assets", "denver_downtown_bounding_box", "denver_downtown_roadnetwork.shp")
        self.bbox = BoundingBox.from_polygon(gpd.read_file(self.bbox_file).iloc[0].geometry)
        self.road_network = TomTomNetworkX(self.road_network_file)

        def _random_od_pair() -> Tuple[Coordinate, Coordinate]:
            nodes = list(self.road_network.G.nodes(data=True))

            while True:
                o_nid, o_data = random.choice(nodes)
                d_nid, d_data = random.choice(nodes)
                if o_nid == d_nid:
                    continue
                else:
                    o_coord = Coordinate(lat=o_data['lat'], lon=o_data['lon'])
                    d_coord = Coordinate(lat=d_data['lat'], lon=d_data['lon'])
                    return o_coord, d_coord

        self.random_od_pair = _random_od_pair

    def test_1000_energy_shortest_path(self):
        def random_shortest_path():
            o, d = self.random_od_pair()
            self.road_network.shortest_path(o, d, weight=PathWeight.ENERGY)

        avg_seconds = timeit.timeit(random_shortest_path, number=1000) / 1000
        print(f"average energy shortest path time was {round(avg_seconds, 4)} seconds")
