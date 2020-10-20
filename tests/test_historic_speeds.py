import os
from unittest import TestCase, skip

import geopandas as gpd

from compass.datastreams.historic_speeds_tomtom import HistoricSpeedsTomTomStream
from compass.road_network.tomtom_networkx import TomTomNetworkX
from compass.road_network.constructs.link import Link
from compass.utils.geo_utils import BoundingBox


class TestHistoricSpeedsTomTomStream(TestCase):
    def setUp(self) -> None:
        self.bbox_file = os.path.join("test_assets", "denver_downtown_bounding_box", "denver_downtown_roadnetwork.shp")
        self.bbox = BoundingBox.from_polygon(gpd.read_file(self.bbox_file).iloc[0].geometry)
        self.road_network_file = os.path.join("test_assets", "denver_downtown_tomtom_network.pickle")
        self.road_network = TomTomNetworkX(self.road_network_file)

    # this test can take several minutes since it's interacting with the TomTom traffic stats API
    @skip
    def test_collect_historical_speeds_tom_tom(self):
        stream = HistoricSpeedsTomTomStream(timezone_str="US/Mountain", bounding_box=self.bbox)
        self.road_network.add_data_stream(stream)

        # setting a link with negative speeds to make sure it gets updated by the data stream
        bad_speed_link = Link(link_id=88400000018763, attributes={'kph': -100, 'minutes': -100})
        self.road_network.update_links((bad_speed_link,))

        result = stream.collect()
        self.assertEqual(result, 1, "should have returned success code")

        link_attributes = list(
            filter(lambda t: t[2] == bad_speed_link.link_id, self.road_network.G.edges(data=True, keys=True))
        )[0][3]

        self.assertGreaterEqual(link_attributes['kph'], 0, "should have updated link to be non-negative")
