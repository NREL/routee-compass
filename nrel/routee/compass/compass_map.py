from __future__ import annotations

import pandas as pd
import networkx as nx

from sqlalchemy.engine import Engine

from mappymatch.maps.nx.nx_map import NxMap
from mappymatch.constructs.geofence import Geofence
from mappymatch.utils.crs import CRS, LATLON_CRS

from nrel.mappymatch.readers.tomtom import get_tomtom_gdf, tomtom_gdf_to_nx_graph
from nrel.mappymatch.readers.tomtom_config import TomTomConfig

from compass.rotuee_model_collection import RouteeModelCollection
from compass.utils.units import KILOMETERS_TO_MILES


class CompassMap(NxMap):

    def __init__(self, graph: nx.MultiDiGraph):
        super().__init__(graph)
        self.routee_model_collection = None
        self.routee_model_keys = None

    @classmethod
    def from_tomtom(
        cls,
        geofence: Geofence,
        sql_connection: Engine,
        to_crs: CRS = LATLON_CRS,
        tomtom_config: TomTomConfig = TomTomConfig(),
    ) -> CompassMap:
        """
        Build a CompassMap from the tomtom data on trolley.  
        """

        gdf = get_tomtom_gdf(sql_connection, geofence, to_crs, tomtom_config)
        g = tomtom_gdf_to_nx_graph(gdf, tomtom_config)

        return cls(g) 

    def compute_energy(self, routee_model_collection: RouteeModelCollection):
        c = []
        for road in self.roads:
            dist_km = road.metadata.get("kilometers")
            if dist_km is None:
                raise ValueError(f"link {road.road_id} is missing a kilometers attribute")
            dist_mi = dist_km * KILOMETERS_TO_MILES

            time_minutes = road.metadata.get("minutes") / 60
            if time_minutes is None:
                raise ValueError(f"link {road.road_id} is missing a minutes attribute")
            time_h = time_minutes / 60

            grade = road.metadata.get("grade")
            if grade is None:
                raise ValueError(f"link {road.road_id} is missing a grade attribute")
            elif pd.isna(grade):
                grade = 0.0

            speed_mph = dist_mi / time_h
            c.append(
                {"road_id": road.road_id, "speed": speed_mph, "grade": grade, "distance": dist_mi,}
            )

        df = pd.DataFrame(c).set_index("road_id")

        for model_key, model in routee_model_collection.routee_models.items():
            energy = model.predict(df)
            new_attr = {}

            for link_id, gge in energy.items():
                new_attr[link_id] = {model_key: gge}

            self.set_road_attributes(new_attr)

        self.routee_model_collection = routee_model_collection
        self.routee_model_keys = set(routee_model_collection.routee_models.keys())
