from pkg_resources import resource_filename
from routee import read_model


class RouteeModelCollection:
    routee_model_base_path = "resources.routee_models"

    routee_model_paths = {
        'Gasoline': '2016_TOYOTA_Corolla_4cyl_2WD_Random_Forest.pickle',
        'Electric': '2016_Nissan_Leaf_30_kWh_Random_Forest.pickle'
    }

    def _load_model(self, routee_model_file: str):
        path = resource_filename(self.routee_model_base_path, routee_model_file)
        return read_model(path)

    def __init__(self):
        self.routee_models = {k: self._load_model(v) for k, v in self.routee_model_paths.items()}
