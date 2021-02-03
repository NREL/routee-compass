from pathlib import Path
from typing import Optional, Dict

from powertrain import read_model

from compass import root


class RouteeModelCollection:
    routee_model_default_path = root() / "compass" / "resources" / "routee_models"

    def __init__(self, model_paths: Optional[Dict] = None):
        if not model_paths:
            model_paths = {
                'Gasoline': self.routee_model_default_path / Path('2016_TOYOTA_Corolla_4cyl_2WD_LinearRegression.json'),
                'Electric': self.routee_model_default_path / Path('2016_Nissan_Leaf_30_kWh_LinearRegression.json'),
            }

        if not isinstance(model_paths, dict):
            raise TypeError(f"the model_paths must be a dictionary with at least one model_key and model path pair")

        self.routee_models = {k: read_model(str(v)) for k, v in model_paths.items()}
