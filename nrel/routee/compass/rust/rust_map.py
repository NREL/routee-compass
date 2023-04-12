from typing import List, Tuple
from mappymatch.constructs.coordinate import Coordinate

from compass_rust import Graph, Link, Node, py_time_shortest_path

from shapely.geometry import Point, LineString

import rtree as rt
import networkx as nx


class RustMap:
    def __init__(self, links: List[Tuple[Link, Point]]):
        graph = Graph()
        rtree = rt.index.Index()
        for link, point in links:
            graph.add_edge(link)
            rtree.insert(link.start_node.id, point.bounds)

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


def build_rust_map_from_nx_graph(graph: nx.MultiDiGraph) -> RustMap:
    """
    build a rust map from a networkx graph; 
    this is useful since networkx can extract the largest strongly connected component;
    eventually, we could find the largest strongly connected component in rust
    """
    nodes = {}
    # map the nodes to integers
    for i, n in enumerate(graph.nodes()):
        nodes[n] = i 

    links = []
    for u, v, data in graph.edges(data=True):
        start_node = Node(nodes[u])
        end_node = Node(nodes[v])
        time = data.get("minutes")
        if time is None:
            raise ValueError(f"Edge {u} -> {v} has no time")
        distance_km = data.get("kilometers")
        if distance_km is None:
            raise ValueError(f"Edge {u} -> {v} has no distance")

        distance_m = int(distance_km * 1000)

        road_class = data.get("display_class")
        if road_class is None:
            raise ValueError(f"Edge {u} -> {v} has no frc")

        grade_dec = data["metadata"].get("grade")
        if grade_dec is None:
            raise ValueError(f"Edge {u} -> {v} has no grade")

        grade_milli = int(grade_dec * 1000)

        restrictions = None

        link = Link(start_node, end_node, road_class, time, distance_m, grade_milli, restrictions)

        geom: LineString = data["geom"]

        link_start_point = Point(geom.coords[0])

        links.append((link, link_start_point))
    
    return RustMap(links)
