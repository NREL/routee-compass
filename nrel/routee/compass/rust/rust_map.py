from compass_rust import Graph, Link, Node, RustMap

from shapely.geometry import LineString

import pandas as pd
import geopandas as gpd


def build_rust_map_from_gdf(gdf: gpd.geodataframe.GeoDataFrame) -> RustMap:
    """
    build a rust map from a geopandas dataframe; 
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

    def build_link(t, direction):
        if direction == TO_FROM_DIRECTION:
            geom = LineString(reversed(t.geom.coords))
            start_point = geom.coords[0]
            end_point = geom.coords[-1]
            minutes = t.neg_minutes
            grade = -t.mean_gradient_dec
            start_node = Node(nodes[t.junction_id_to], int(start_point[0]), int(start_point[1]))
            end_node = Node(nodes[t.junction_id_from], int(end_point[0]), int(end_point[1]))
        elif direction == FROM_TO_DIRECTION:
            geom = t.geom
            start_point = geom.coords[0]
            end_point = geom.coords[-1]
            minutes = t.pos_minutes
            grade = t.mean_gradient_dec
            start_node = Node(nodes[t.junction_id_from], int(start_point[0]), int(start_point[1]))
            end_node = Node(nodes[t.junction_id_to], int(end_point[0]), int(end_point[1]))
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

        return link

    graph = Graph()
    for t in twoway.itertuples():
        link = build_link(t, TO_FROM_DIRECTION)
        graph.add_edge(link)

    for t in twoway.itertuples():
        link = build_link(t, FROM_TO_DIRECTION)
        graph.add_edge(link)

    for t in oneway_ft.itertuples():
        link = build_link(t, FROM_TO_DIRECTION)
        graph.add_edge(link)

    for t in oneway_tf.itertuples():
        link = build_link(t, TO_FROM_DIRECTION)
        graph.add_edge(link)

    return RustMap(graph)
