from typing import List, Tuple
from mappymatch.constructs.coordinate import Coordinate

from compass_rust import Graph, Link, Node, py_time_shortest_path

from shapely.geometry import Point, LineString

import rtree as rt
import pandas as pd
import geopandas as gpd


class RustMap:
    def __init__(self, links: List[Tuple[Link, LineString]]):
        """
        Build a map from a list of links and their geometry;
        Currently this just uses the link start point for rtree lookup 
        """
        graph = Graph()
        rtree = rt.index.Index()
        for link, link_geom in links:
            graph.add_edge(link)
            start_node_point = Point(link_geom.coords[0])
            rtree.insert(link.start_node.id, start_node_point.bounds)

        self.graph = graph
        self.rtree = rtree

    def nearest_node(self, coord: Coordinate, buffer: float = 10.0) -> Node:
        """
        get the nearest node to a coordinate
        """
        nearest_nodes = list(self.rtree.nearest(coord.geom.bounds, 1))

        if len(nearest_nodes) == 0:
            raise ValueError(f"No roads found for {coord}")
        else:
            # if there's a tie, pick the first
            nearest_node = nearest_nodes[0]

        return Node(nearest_node)

    def shortest_path(self, start: Coordinate, end: Coordinate) -> List[Node]:
        """
        get the shortest path between two coordinates
        """
        start_node = self.nearest_node(start)
        end_node = self.nearest_node(end)
        result = py_time_shortest_path(self.graph, start_node, end_node)
        if result is None:
            raise ValueError(f"No path found between {start} and {end}")

        weight, path = result
        return path


def build_rust_map_from_gdf(gdf: gpd.geodataframe.GeoDataFrame) -> RustMap:
    """
    build a rust map from a networkx graph; 
    this is useful since networkx can extract the largest strongly connected component;
    eventually, we could find the largest strongly connected component in rust
    """
    # map node ids to integers
    node_ids = set(gdf.junction_id_from.unique()).union(set(gdf.junction_id_to.unique()))
    nodes = {}
    # map the nodes to integers
    for i, n in enumerate(node_ids):
        nodes[n] = i

    # also referred to as the 'positive' direction in TomTom
    FROM_TO_DIRECTION = 2

    # also referred to as the 'negative' direction in TomTom
    TO_FROM_DIRECTION = 3

    oneway_ft = gdf[gdf.link_direction == FROM_TO_DIRECTION]
    oneway_tf = gdf[gdf.link_direction == TO_FROM_DIRECTION]
    twoway = gdf[gdf.link_direction.isin([1, 9])]

    def build_edge_tuple(t, direction):
        if direction == TO_FROM_DIRECTION:
            minutes = t.neg_minutes
            grade = -t.mean_gradient_dec
            start_node = Node(nodes[t.junction_id_to])
            end_node = Node(nodes[t.junction_id_from])
        elif direction == FROM_TO_DIRECTION:
            minutes = t.pos_minutes
            grade = t.mean_gradient_dec
            start_node = Node(nodes[t.junction_id_from])
            end_node = Node(nodes[t.junction_id_to])
        else:
            raise ValueError("Bad direction value")

        if direction == TO_FROM_DIRECTION:
            geom = LineString(reversed(t.geom.coords))
        elif direction == FROM_TO_DIRECTION:
            geom = t.geom
        else:
            raise ValueError("Bad direction value")

        if pd.isna(t.display_class):
            road_class = 100
        else:
            road_class = int(t.display_class)

        if pd.isna(grade):
            grade_milli = 0
        else:
            grade_milli = int(grade * 1000)

        distance_m = int(t.kilometers * 1000)
        restrictions = None
        time_seconds = int(minutes * 60)

        link = Link(
            start_node, end_node, road_class, time_seconds, distance_m, grade_milli, restrictions
        )

        return link, geom

    all_links = []
    for t in twoway.itertuples():
        edge_tuple = build_edge_tuple(t, TO_FROM_DIRECTION)
        all_links.append(edge_tuple)

    for t in twoway.itertuples():
        edge_tuple = build_edge_tuple(t, FROM_TO_DIRECTION)
        all_links.append(edge_tuple)

    for t in oneway_ft.itertuples():
        edge_tuple = build_edge_tuple(t, FROM_TO_DIRECTION)
        all_links.append(edge_tuple)

    for t in oneway_tf.itertuples():
        edge_tuple = build_edge_tuple(t, TO_FROM_DIRECTION)
        all_links.append(edge_tuple)

    return RustMap(all_links)
