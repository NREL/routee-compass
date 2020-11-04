from pathlib import Path

from powertrain import read_model

from compass import root


class RouteeModelCollection:
    routee_model_base_path = root() / "compass" / "resources" / "routee_models"

    routee_model_paths = {
        'Gasoline': Path('2016_TOYOTA_Corolla_4cyl_2WD_LinearRegression.json'),
        'Electric': Path('2016_Nissan_Leaf_30_kWh_LinearRegression.json'),
    }

    def _load_model(self, routee_model_file: Path):
        return read_model(str(self.routee_model_base_path.joinpath(routee_model_file)))

    def __init__(self):
        self.routee_models = {k: self._load_model(v) for k, v in self.routee_model_paths.items()}
