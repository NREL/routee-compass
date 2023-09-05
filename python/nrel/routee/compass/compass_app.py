from __future__ import annotations

import json

from pathlib import Path
from typing import Any, Dict, List, Union
from nrel.routee.compass.compass_app_py import (
    CompassAppWrapper,
)


class CompassApp:
    _app: CompassAppWrapper

    def __init__(self, app: CompassAppWrapper):
        self._app = app

    @classmethod
    def from_config_file(cls, config_file: Union[str, Path]) -> CompassApp:
        config_path = Path(config_file)
        if not config_path.is_file():
            raise ValueError(f"Config file {str(config_path)} does not exist")
        app = CompassAppWrapper._from_config_file(str(config_path.absolute()))
        return cls(app)

    def run(
        self, query: Union[Dict[str, Any], List[Dict[str, Any]]]
    ) -> List[Dict[str, Any]]:
        """
        A wrapper function to run a query on the CompassAppWrapper object
        which expects the inputs to be a JSON string and returns a JSON string
        """
        if isinstance(query, dict):
            queries = [query]
        elif isinstance(query, list):
            queries = query
        else:
            raise ValueError(
                f"Query must be a dict or list of dicts, not {type(query)}"
            )

        queries_json = list(map(json.dumps, queries))

        results_json: List[str] = self._app._run_queries(queries_json)

        results = list(map(json.loads, results_json))

        return results
