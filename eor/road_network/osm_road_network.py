from typing import Tuple

from rtree import index
from routee.io.api import read_model

from eor.road_network.base import RoadNetwork, PathWeight
from eor.utils.geo_utils import Coordinate

import networkx as nx
import pandas as pd


class OSMRoadNetwork(RoadNetwork):
    """
    osm road network
    """
    network_weights = {
        PathWeight.DISTANCE: "miles",
        PathWeight.TIME: "travel_time",
        PathWeight.ENERGY: "energy_gge"
    }

    def __init__(
            self,
            osm_network_file: str,
    ):
        self.G = nx.read_gpickle(osm_network_file)
        self.rtree = self._build_rtree()

        # TODO: add feature to load additional models -ndr
        self.routee_model = read_model("../resources/2016_FORD_Explorer_4cyl_2WD_Random_Forest.pickle")

        self._compute_energy()

    def _compute_energy(self):

        # TODO: modify this for mulitple vehicles. We should also think about the best way to update energy if
        #  some feature changes, like weather or traffic. -ndr

        speed = pd.DataFrame.from_dict(
            nx.get_edge_attributes(self.G, 'gpsspeed'),
            orient="index",
            columns=['gpsspeed'],
        )
        distance = pd.DataFrame.from_dict(
            nx.get_edge_attributes(self.G, 'miles'),
            orient="index",
            columns=['miles'],
        )
        grade = pd.DataFrame.from_dict(
            nx.get_edge_attributes(self.G, 'grade'),
            orient="index",
            columns=['grade'],
        )
        df = speed.join(distance).join(grade)

        energy = self.routee_model.predict(df).to_dict()

        nx.set_edge_attributes(self.G, name="energy_gge", values=energy)

    def _build_rtree(self) -> index.Index:
        tree = index.Index()
        for nid in self.G.nodes():
            lat = self.G.nodes[nid]['y']
            lon = self.G.nodes[nid]['x']
            tree.insert(nid, (lat, lon, lat, lon))

        return tree

    def _get_nearest_node(self, coord: Coordinate) -> str:
        node_id = list(self.rtree.nearest((coord.lat, coord.lon, coord.lat, coord.lon), 1))[0]

        return node_id

    def shortest_path(
            self,
            origin: Coordinate,
            destination: Coordinate,
            weight: PathWeight = PathWeight.DISTANCE,
    ) -> Tuple[Coordinate, ...]:
        """
        computes weighted shortest path
        :return: shortest path as series of coordinates
        """
        origin_id = self._get_nearest_node(origin)
        dest_id = self._get_nearest_node(destination)

        nx_route = nx.shortest_path(
            self.G,
            origin_id,
            dest_id,
            weight=self.network_weights[weight],
        )

        route = tuple(Coordinate(lat=self.G.nodes[n]['y'], lon=self.G.nodes[n]['x']) for n in nx_route)

        return route

