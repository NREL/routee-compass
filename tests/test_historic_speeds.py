import os
from unittest import TestCase, skip

import geopandas as gpd

from compass.datastreams.historic_speeds_tomtom import HistoricSpeedsTomTomStream
from compass.road_network.tomtom_road_network import TomTomRoadNetwork
from compass.utils.geo_utils import BoundingBox


class TestHistoricSpeedsTomTomStream(TestCase):
    def setUp(self) -> None:
        self.road_network_file = os.path.join("test_assets", "denver_downtown_tomtom_network.pickle")
        self.bbox_file = os.path.join("test_assets", "denver_downtown_bounding_box", "denver_downtown_roadnetwork.shp")
        self.road_network = TomTomRoadNetwork(self.road_network_file)
        self.bbox = BoundingBox.from_polygon(gpd.read_file(self.bbox_file).iloc[0].geometry)

    # this test can take several minutes since it's interacting with the TomTom traffic stats API
    @skip
    def test_shortest_path_distance(self):
        stream = HistoricSpeedsTomTomStream(self.bbox)
        result = stream.update(self.road_network)

        self.assertEqual(result, 1, "should have returned success code")
