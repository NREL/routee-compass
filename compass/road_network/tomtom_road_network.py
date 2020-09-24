import logging
from typing import Tuple

import networkx as nx
import pandas as pd
from rtree import index

from compass.road_network.base import RoadNetwork, PathWeight
from compass.utils.geo_utils import Coordinate, BoundingBox
from compass.utils.routee_utils import RouteeModelCollection

METERS_TO_MILES = 0.0006213712
KPH_TO_MPH = 0.621371

log = logging.getLogger(__name__)


class TomTomRoadNetwork(RoadNetwork):
    """
    osm road network
    """
    network_weights = {
        PathWeight.DISTANCE: "meters",
        PathWeight.TIME: "minutes",
        PathWeight.ENERGY: "energy"
    }

    def __init__(
            self,
            network_file: str,
            bounding_box: BoundingBox,
            routee_model_collection: RouteeModelCollection = RouteeModelCollection(),
    ):
        self.G = nx.read_gpickle(network_file)

        if not isinstance(self.G, nx.MultiDiGraph):
            raise TypeError("graph must be a MultiDiGraph")

        self.bbox = bounding_box
        self.rtree = self._build_rtree()

        self.routee_model_collection = routee_model_collection

    def _compute_energy(self):
        """
        computes energy over the road network for all routee models in the routee model collection.

        this isn't currently called by anything since we're pre-computing energy for the prototype but
        would presumably be called if we want to do live updates.
        """
        log.info("recomputing energy on network..")

        speed = pd.DataFrame.from_dict(
            nx.get_edge_attributes(self.G, 'kph'),
            orient="index",
            columns=['gpsspeed'],
        ).multiply(KPH_TO_MPH)
        distance = pd.DataFrame.from_dict(
            nx.get_edge_attributes(self.G, 'meters'),
            orient="index",
            columns=['miles'],
        ).multiply(METERS_TO_MILES)
        grade = pd.DataFrame.from_dict(
            nx.get_edge_attributes(self.G, 'grade'),
            orient="index",
            columns=['grade'],
        )
        df = speed.join(distance).join(grade)

        for k, model in self.routee_model_collection.routee_models.items():
            energy = model.predict(df).to_dict()
            nx.set_edge_attributes(self.G, name=f"{self.network_weights[PathWeight.ENERGY]}_{k}", values=energy)

    def _build_rtree(self) -> index.Index:
        tree = index.Index()
        for nid in self.G.nodes():
            lat = self.G.nodes[nid]['lat']
            lon = self.G.nodes[nid]['lon']
            tree.insert(nid, (lat, lon, lat, lon))

        return tree

    def _get_nearest_node(self, coord: Coordinate) -> str:
        node_id = list(self.rtree.nearest((coord.lat, coord.lon, coord.lat, coord.lon), 1))[0]

        return node_id

    def update(self):
        self._compute_energy()

    def shortest_path(
            self,
            origin: Coordinate,
            destination: Coordinate,
            weight: PathWeight = PathWeight.DISTANCE,
            routee_key: str = "Gasoline",
    ) -> Tuple[Coordinate, ...]:
        """
        computes weighted shortest path
        :return: shortest path as series of coordinates
        """
        origin_id = self._get_nearest_node(origin)
        dest_id = self._get_nearest_node(destination)

        network_weight = self.network_weights[weight]

        if weight == PathWeight.ENERGY:
            network_weight += f"_{routee_key}"

        nx_route = nx.shortest_path(
            self.G,
            origin_id,
            dest_id,
            weight=self.network_weights[weight],
        )

        route = tuple(Coordinate(lat=self.G.nodes[n]['lat'], lon=self.G.nodes[n]['lon']) for n in nx_route)

        return route
