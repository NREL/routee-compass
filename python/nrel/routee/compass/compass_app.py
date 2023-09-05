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

    def run_query(self, query: Dict[str, Any]) -> Dict[str, Any]:
        """
        A wrapper function to run a query on the CompassAppWrapper object
        which expects the inputs to be a JSON string and returns a JSON string
        """
        query_json = json.dumps(query)

        result_json = self._app._run_query(query_json)

        return json.loads(result_json)

    def run_queries(self, queries: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        """
        A wrapper function to run multiple queries on the CompassAppWrapper object
        """
        queries_json = json.dumps(queries)

        results_json: List[str] = self._app._run_queries(queries_json)

        results = list(map(json.loads, results_json))

        return results
