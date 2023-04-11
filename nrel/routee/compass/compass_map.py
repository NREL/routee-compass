from __future__ import annotations

import pandas as pd

from mappymatch.maps.map_interface import MapInterface 


from nrel.routee.compass.rotuee_model_collection import RouteeModelCollection
from nrel.routee.compass.utils.units import KILOMETERS_TO_MILES


def compute_energy(mappy_map: MapInterface, routee_model_collection: RouteeModelCollection):
    c = []
    for road in mappy_map.roads:
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

        mappy_map.set_road_attributes(new_attr)

    mappy_map.routee_model_collection = routee_model_collection
    mappy_map.routee_model_keys = set(routee_model_collection.routee_models.keys())
