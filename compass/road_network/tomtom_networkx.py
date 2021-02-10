import logging
from pathlib import Path
from typing import Tuple, Set

import networkx as nx
import numpy as np
import pandas as pd
from scipy.spatial import cKDTree

from compass.datastreams.base import DataStream
from compass.road_network.base import RoadNetwork, PathWeight, PathResult, RouteMetadata
from compass.road_network.constructs.link import Link
from compass.utils.geo_utils import Coordinate
from compass.utils.routee_utils import RouteeModelCollection

METERS_TO_MILES = 0.0006213712
KPH_TO_MPH = 0.621371

log = logging.getLogger(__name__)


class TomTomNetworkX(RoadNetwork):
    """
    tom tom road network database that uses networkx

    """
    data_streams = []

    network_weights = {
        PathWeight.DISTANCE: "meters",
        PathWeight.TIME: "minutes",
        PathWeight.ENERGY: "energy"
    }

    registered_network_types = {
        'tomtom',
        'tomtom_dual'
    }

    def __init__(
            self,
            network_file: Path,
            routee_model_collection: RouteeModelCollection = RouteeModelCollection(),
    ):
        self.G = nx.read_gpickle(network_file)

        network_type = self.G.graph.get('compass_network_type')
        if network_type not in self.registered_network_types:
            raise TypeError(f"graph type {network_type} not valid for TomTomNetworkX")

        self._nodes = [nid for nid in self.G.nodes()]

        if not isinstance(self.G, nx.MultiDiGraph):
            raise TypeError("graph must be a MultiDiGraph")

        self.kdtree = self._build_kdtree()

        self.routee_model_collection = routee_model_collection

        self._compute_energy()

    @property
    def routee_model_keys(self) -> Set[str]:
        return set([k for k in self.routee_model_collection.routee_models.keys()])

    def add_data_stream(self, data_stream: DataStream):
        data_stream.bind_to(self.update_links)
        self.data_streams.append(data_stream)

    def _compute_energy(self):
        """
        computes energy over the road network for all routee models in the routee model collection.
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

            # TODO: I clipped predicted energy to 0 since the shortest path computation was failing with
            #  negative values. But, it would be nice to consider negative energy in certain instances.
            #  For example, an electric vehicle can regain energy while traveling downhill.
            energy = model.predict(df).clip(0)

            df['energy'] = energy.values
            edge_values = df['energy'].to_dict()
            nx.set_edge_attributes(self.G, name=f"{self.network_weights[PathWeight.ENERGY]}_{k}", values=edge_values)

    def _build_kdtree(self) -> cKDTree:
        points = [(self.G.nodes[nid]['lat'], self.G.nodes[nid]['lon']) for nid in self._nodes]
        tree = cKDTree(np.array(points))

        return tree

    def _get_nearest_node(self, coord: Coordinate) -> str:
        _, i = self.kdtree.query([coord.lat, coord.lon])
        return self._nodes[i]

    def update_links(self, links: Tuple[Link, ...]):
        link_df = pd.DataFrame({"segment_id": l.link_id, **l.attributes} for l in links)
        for _, _, k, d in self.G.edges(data=True, keys=True):
            segment = link_df[link_df.segment_id == k]
            if len(segment) < 1:
                log.debug(f"skipping segemnt {k}, couldn't find in speed data")
                continue
            elif len(segment) > 1:
                log.warning(f"found multiple instances of segment {k} in data stream, skipping")
                continue

            # TODO: 🚨side effect alert🚨 not sure if we should do this another way? -ndr
            d["kph"] = segment["kph"].values[0]
            d["minutes"] = segment["minutes"].values[0]

        self._compute_energy()

    def shortest_path(
            self,
            origin: Coordinate,
            destination: Coordinate,
            weight: PathWeight = PathWeight.DISTANCE,
            routee_key: str = "Gasoline",
    ) -> PathResult:
        """
        computes weighted shortest path

        :return: the route as a tuple of coordinates and the route metadata
        """
        origin_id = self._get_nearest_node(origin)
        dest_id = self._get_nearest_node(destination)

        network_weight = self.network_weights[weight]

        if routee_key not in self.routee_model_keys:
            raise Exception(f"road network doesn't have routee model key {routee_key}")

        if weight == PathWeight.ENERGY:
            network_weight += f"_{routee_key}"

        nx_route = nx.shortest_path(
            self.G,
            origin_id,
            dest_id,
            weight=network_weight,
        )
        route = tuple(Coordinate(lat=self.G.nodes[n]['lat'], lon=self.G.nodes[n]['lon']) for n in nx_route)

        energy = []
        energy_key = f"{self.network_weights[PathWeight.ENERGY]}_{routee_key}"
        time = []
        time_key = f"{self.network_weights[PathWeight.TIME]}"
        distance = []
        distance_key = f"{self.network_weights[PathWeight.DISTANCE]}"
        for e in zip(nx_route[0:], nx_route[1:]):
            edge_data = list(self.G.get_edge_data(e[0], e[1]).values())[0]
            energy.append(edge_data[energy_key])
            time.append(edge_data[time_key])
            distance.append(edge_data[distance_key])

        energy_units = self.routee_model_collection.routee_models[routee_key].metadata.estimator_features['energy']['units']

        route_metadata = RouteMetadata(
            route_distance=sum(distance),
            route_distance_units=distance_key,
            route_time=sum(time),
            route_time_units=time_key,
            route_energy=sum(energy),
            route_energy_units=energy_units
        )

        return PathResult(route, route_metadata)
