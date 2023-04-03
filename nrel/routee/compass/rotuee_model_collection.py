from typing import Optional, Dict

from powertrain import read_model, load_pretrained_model


class RouteeModelCollection:
    def __init__(self, model_paths: Optional[Dict] = None):
        if not model_paths:
            self.routee_models = {
                'Gasoline': load_pretrained_model('ICE'),
                'Electric': load_pretrained_model('EV'),
            }
        elif not isinstance(model_paths, dict):
            raise TypeError(f"the model_paths must be a dictionary with at least one model_key and model path pair")
        else:
            self.routee_models = {k: read_model(str(v)) for k, v in model_paths.items()}
