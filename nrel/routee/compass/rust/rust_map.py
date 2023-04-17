import time
from typing import Dict, Optional
from compass_rust import Graph, Link, Node, RustMap, largest_scc

from shapely.geometry import LineString

import pandas as pd
import geopandas as gpd


def build_rust_map_from_gdf(
    gdf: gpd.geodataframe.GeoDataFrame,
    weight_restrictions: Optional[Dict[str, Dict[int, float]]] = None,
    height_restrictions: Optional[Dict[str, Dict[int, float]]] = None,
    width_restrictions: Optional[Dict[str, Dict[int, float]]] = None,
    length_restrictions: Optional[Dict[str, Dict[int, float]]] = None,
) -> RustMap:
    """
    build a rust map from a geopandas dataframe; 
    """
    # map node ids to integers
    start_time = time.time()
    print("mapping node ids to integers..")
    node_ids = set(gdf.junction_id_from.unique()).union(set(gdf.junction_id_to.unique()))
    nodes = {}
    # map the nodes to integers
    for i, n in enumerate(node_ids):
        nodes[n] = i

    print(f"mapping took {time.time() - start_time} seconds")

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

        if weight_restrictions is not None:
            wr = weight_restrictions.get(t.netw_id)
            if wr is None:
                weight_restriction = None
            else:
                if 1 in wr:
                    # restriction is bidirectional
                    weight_restriction = int(wr[1] * 2000)  # tons to lbs
                elif direction in wr:
                    # restriction is unidirectional
                    weight_restriction = int(wr[direction] * 2000)  # tons to lbs
                else:
                    weight_restriction = None
        else:
            weight_restriction = None

        if height_restrictions is not None:
            hr = height_restrictions.get(t.netw_id)
            if hr is None:
                height_restriction = None
            else:
                if 1 in hr:
                    # restriction is bidirectional
                    height_restriction = int(hr[1])
                elif direction in hr:
                    # restriction is unidirectional
                    height_restriction = int(hr[direction])
                else:
                    height_restriction = None
        else:
            height_restriction = None

        if width_restrictions is not None:
            wdr = width_restrictions.get(t.netw_id)
            if wdr is None:
                width_restriction = None
            else:
                if 1 in wdr:
                    # restriction is bidirectional
                    width_restriction = int(wdr[1])
                elif direction in wdr:
                    # restriction is unidirectional
                    width_restriction = int(wdr[direction])
                else:
                    width_restriction = None
        else:
            width_restriction = None

        if length_restrictions is not None:
            lr = length_restrictions.get(t.netw_id)
            if lr is None:
                length_restriction = None
            else:
                if 1 in lr:
                    # restriction is bidirectional
                    length_restriction = int(lr[1])
                elif direction in lr:
                    # restriction is unidirectional
                    length_restriction = int(lr[direction])
                else:
                    length_restriction = None
        else:
            length_restriction = None

        if pd.isna(t.display_class):
            road_class = 100
        else:
            road_class = int(t.display_class)

        if pd.isna(grade):
            grade_milli = 0
        else:
            grade_milli = int(grade * 1000)

        distance_m = int(t.kilometers * 1000)
        time_seconds = int(minutes * 60)

        link = Link(
            start_node,
            end_node,
            road_class,
            time_seconds,
            distance_m,
            grade_milli,
            weight_restriction,
            height_restriction,
            width_restriction,
            length_restriction,
        )

        return link

    links = []
    print("building two way links to-from..")
    start_time = time.time()
    two_way_tf_links = [build_link(t, TO_FROM_DIRECTION) for t in twoway.itertuples()]
    links.extend(two_way_tf_links)
    print("building two links took", time.time() - start_time, "seconds")

    print("building two way links from-to..")
    start_time = time.time()
    two_way_ft_links = [build_link(t, FROM_TO_DIRECTION) for t in twoway.itertuples()]
    links.extend(two_way_ft_links)
    print("building two links took", time.time() - start_time, "seconds")

    del twoway

    print("building one way links to-from..")
    start_time = time.time()
    oneway_ft_links = [build_link(t, FROM_TO_DIRECTION) for t in oneway_ft.itertuples()]
    links.extend(oneway_ft_links)
    print("building one way links took", time.time() - start_time, "seconds")

    del oneway_ft

    print("building one way links from-to..")
    start_time = time.time()
    oneway_tf_links = [build_link(t, TO_FROM_DIRECTION) for t in oneway_tf.itertuples()]
    links.extend(oneway_tf_links)
    print("building one way links took", time.time() - start_time, "seconds")

    del oneway_tf
    del gdf

    print("building graph..")
    start_time = time.time()
    graph = Graph()
    graph.add_links_bulk(links)
    print("building graph took", time.time() - start_time, "seconds")

    del links
    del two_way_tf_links
    del two_way_ft_links
    del oneway_ft_links
    del oneway_tf_links

    print("getting largest strongly connected component..")
    start_time = time.time()
    # get the largest strongly connected component
    graph = largest_scc(graph)

    print("getting largest strongly connected component took", time.time() - start_time, "seconds")

    return RustMap(graph)
