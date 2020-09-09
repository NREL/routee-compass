from typing import Tuple

from miniosmnx.core import graph_from_file
from rtree import index

from eor.road_network.base import RoadNetwork, PathWeight
from eor.utils.geo_utils import Coordinate

import networkx as nx


class OSMRoadNetwork(RoadNetwork):
    """
    osm road network
    """
    _unit_conversion = {
        'mph': 1,
        'kmph': 0.6213712,
    }

    def __init__(
            self,
            osm_network_file: str,
    ):
        self.G = graph_from_file(osm_network_file)
        self.rtree = self._build_rtree()

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

        nx_route = nx.shortest_path(self.G, origin_id, dest_id)

        route = tuple(Coordinate(lat=self.G.nodes[n]['y'], lon=self.G.nodes[n]['x']) for n in nx_route)

        return route

