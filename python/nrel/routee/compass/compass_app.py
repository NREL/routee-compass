from __future__ import annotations

import json
import logging

from pathlib import Path
from typing import Any, Dict, List, Optional, Union
from nrel.routee.compass.routee_compass_py import (
    CompassAppPy,
)

import toml


Query = Dict[str, Any]
Result = List[Dict[str, Any]]


log = logging.getLogger(__name__)


class CompassApp:
    """
    The CompassApp holds everything needed to run a route query.
    """

    _app: CompassAppPy

    def __init__(self, app: CompassAppPy):
        self._app = app

    @classmethod
    def from_config_file(
        cls, config_file: Union[str, Path]  # , output_file: Optional[str] = None
    ) -> CompassApp:
        """
        Build a CompassApp from a config file

        Args:
            config_file (Union[str, Path]): Path to the config file

        Returns:
            CompassApp: A CompassApp object

        Example:
            >>> from nrel.routee.compass import CompassApp
            >>> app = CompassApp.from_config_file("config.toml")
        """
        config_path = Path(config_file)
        if not config_path.is_file():
            raise ValueError(f"Config file {str(config_path)} does not exist")
        with open(config_path) as f:
            toml_config = toml.load(f)

        return CompassApp.from_dict(toml_config, config_path)

    @classmethod
    def from_dict(cls, config: Dict, working_dir: Optional[Path] = None) -> CompassApp:
        """
        Build a CompassApp from a configuration object

        Args:
            config (Dict): Configuration dictionary
            working_dir (Path): optional path to working directory

        Returns:
            CompassApp: a CompassApp object

        Example:
            >>> from nrel.routee.compass import CompassApp
            >>> conf = { parallelism: 2 }
            >>> app = CompassApp.from_config(conf)
        """
        path_str = str(working_dir.absolute()) if working_dir is not None else ""
        toml_string = toml.dumps(config)
        app = CompassAppPy._from_config_toml_string(toml_string, path_str)
        return cls(app)

    def run(
        self, query: Union[Query, List[Query]], config: Optional[Dict] = None
    ) -> Result:
        """
        Run a query (or multiple queries) against the CompassApp

        Args:
            query (Union[Dict[str, Any], List[Dict[str, Any]]]): A query or list of queries to run

        Returns:
            List[Dict[str, Any]]: A list of results (or a single result if a single query was passed)

        Example:
            >>> from nrel.routee.compass import CompassApp
            >>> app = CompassApp.from_config_file("config.toml")
            >>> query = {
                    "origin_name": "NREL",
                    "destination_name": "Comrade Brewing Company",
                    "origin_x": -105.1710052,
                    "origin_y": 39.7402804,
                    "destination_x": -104.9009913,
                    "destination_y": 39.6757025
                }

            >>> result = app.run(query)

        """
        if isinstance(query, dict):
            queries = [query]
            single_query = True
        elif isinstance(query, list):
            queries = query
            single_query = False
        else:
            raise ValueError(
                f"Query must be a dict or list of dicts, not {type(query)}"
            )

        queries_str = list(map(json.dumps, queries))
        config_str = json.dumps(config) if config is not None else None

        results_json: List[str] = self._app._run_queries(queries_str, config_str)

        results = list(map(json.loads, results_json))
        if single_query and len(results) == 1:
            return results[0]
        return results

    def graph_edge_origin(self, edge_id: int) -> int:
        """
        get the origin vertex id for some edge

        Args:
            edge_id (int): the id of the edge

        Returns:
            int: the vertex id at the source of the edge
        """
        return self._app.graph_edge_origin(edge_id)

    def graph_edge_destination(self, edge_id: int) -> int:
        """
        get the destination vertex id for some edge

        Args:
            edge_id (int): the id of the edge

        Returns:
            int: the vertex id at the destination of the edge
        """
        return self._app.graph_edge_destination(edge_id)

    def graph_edge_distance(
        self, edge_id: int, distance_unit: Optional[str] = None
    ) -> float:
        """
        get the distance for some edge

        Args:
            edge_id (int): the id of the edge
            distance_unit (Optional[str]): distance unit, by default meters

        Returns:
            int: the distance covered by traversing the edge
        """
        return self._app.graph_edge_distance(edge_id, distance_unit)

    def graph_get_out_edge_ids(self, vertex_id: int) -> List[int]:
        """
        get the list of edge ids that depart from some vertex

        Args:
            vertex_id (int): the id of the vertex

        Returns:
            List[int]: the edge ids of edges departing from this vertex
        """
        return self._app.graph_get_out_edge_ids(vertex_id)

    def graph_get_in_edge_ids(self, vertex_id: int) -> List[int]:
        """
        get the list of edge ids that arrive from some vertex

        Args:
            vertex_id (int): the id of the vertex

        Returns:
            List[int]: the edge ids of edges arriving at this vertex
        """
        return self._app.graph_get_in_edge_ids(vertex_id)


# def inject_to_disk_plugin(output_file: str, toml_config: dict) -> dict:
#     """
#     Inject or override the to_disk plugin in the config dictionary

#     Args:
#         output_file (str): Path to the output file
#         toml_config (dict): The existing config dictionary

#     Returns:
#         dict: A dictionary with the to_disk plugin injected or overriden
#     """
#     plugins = toml_config.get("plugin")
#     if plugins is None:
#         # inject a whole plugin section with the to_disk output plugin
#         toml_config["plugin"] = {
#             "output_plugins": [{"type": "to_disk", "output_file": output_file}]
#         }
#     else:
#         output_plugins = plugins.get("output_plugins")
#         if output_plugins is None:
#             # inject the to_disk output plugin into the existing plugin section
#             plugins["output_plugins"] = [
#                 {"type": "to_disk", "output_file": output_file}
#             ]
#         else:
#             to_disk_exists = False
#             for plugin in output_plugins:
#                 if plugin.get("type") == "to_disk":
#                     to_disk_exists = True
#                     plugin["output_file"] = output_file
#             if not to_disk_exists:
#                 output_plugins.append({"type": "to_disk", "output_file": output_file})
#     return toml_config
