import pandas as pd

from polestar.road_map import RoadMap

from compass.rotuee_model_collection import RouteeModelCollection
from compass.utils.units import KILOMETERS_TO_MILES


class CompassMap(RoadMap):
    def compute_energy(self, routee_model_collection: RouteeModelCollection):
        c = []
        for link in self.graph.links:
            dist_mi = link.attributes["kilometers"] * KILOMETERS_TO_MILES 
            time_h = link.attributes["minutes"] / 60
            speed_mph = dist_mi / time_h
            c.append({
                "link_id": link.link_id,
                "speed": speed_mph,
                "grade": 0,
                "distance": dist_mi,
            })

        df = pd.DataFrame(c).set_index("link_id")

        for model_key, model in routee_model_collection.routee_models.items():
            energy = model.predict(df)
            new_attr = {} 

            for link_id, gge in energy.items():
                new_attr[link_id] = {model_key: gge}

            self.set_link_attributes(new_attr)
        
        self.routee_model_collection = routee_model_collection
        self.routee_model_keys = set(routee_model_collection.routee_models.keys())