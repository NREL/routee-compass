from datetime import datetime
from pathlib import Path
from typing import Dict
import pandas as pd
import geopandas as gpd
import pickle
import time
import logging
import sqlalchemy as sql

from compass_rust import Graph, Link, Node, RustMap, largest_scc

from shapely.geometry import LineString

from pyproj import CRS

# set up logging to file
date_and_time = datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
logging.basicConfig(filename=f"build_rust_map_{date_and_time}.log", level=logging.DEBUG)

log = logging.getLogger(__name__)

LATLON = CRS("epsg:4326")
WEB_MERCATOR = "epsg:3857"

# also referred to as the 'positive' direction in TomTom
FROM_TO_DIRECTION = 2

# also referred to as the 'negative' direction in TomTom
TO_FROM_DIRECTION = 3

DEFAULT_SPEED_KPH = 40

CHUNK_SIZE = 5_000_000


def build_speed(t, direction):
    # optimisitically return free flow
    if not pd.isna(t.free_flow_speed):
        return int(t.free_flow_speed)

    if direction == TO_FROM_DIRECTION:
        if not pd.isna(t.speed_average_neg):
            return int(t.speed_average_neg)
        return DEFAULT_SPEED_KPH
    elif direction == FROM_TO_DIRECTION:
        if not pd.isna(t.speed_average_pos):
            return int(t.speed_average_pos)
        return DEFAULT_SPEED_KPH
    else:
        raise ValueError("Bad direction value")


def build_link(t, direction, node_id_mapping, node_id_counter):
    if t.junction_id_to in node_id_mapping:
        junction_id_to_id = node_id_mapping[t.junction_id_to]
    else:
        junction_id_to_id = node_id_counter
        node_id_mapping[t.junction_id_to] = junction_id_to_id
        node_id_counter += 1

    if t.junction_id_from in node_id_mapping:
        junction_id_from_id = node_id_mapping[t.junction_id_from]
    else:
        junction_id_from_id = node_id_counter
        node_id_mapping[t.junction_id_from] = junction_id_from_id
        node_id_counter += 1

    speed_kph = build_speed(t, direction)
    if direction == TO_FROM_DIRECTION:
        geom = LineString(reversed(t.geom.coords))
        start_point = geom.coords[0]
        end_point = geom.coords[-1]
        grade = -t.mean_gradient_dec
        start_node = Node(junction_id_to_id, int(start_point[0]), int(start_point[1]))
        end_node = Node(junction_id_from_id, int(end_point[0]), int(end_point[1]))
    elif direction == FROM_TO_DIRECTION:
        geom = t.geom
        start_point = geom.coords[0]
        end_point = geom.coords[-1]
        grade = t.mean_gradient_dec
        start_node = Node(junction_id_from_id, int(start_point[0]), int(start_point[1]))
        end_node = Node(junction_id_to_id, int(end_point[0]), int(end_point[1]))
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

    if pd.isna(grade):
        grade_milli = 0
    else:
        grade_milli = int(grade * 1000)

    distance_cm = t.centimeters

    week_profile_ids = [
        profile_id_integer_mapping.get(t.monday_profile_id),
        profile_id_integer_mapping.get(t.tuesday_profile_id),
        profile_id_integer_mapping.get(t.wednesday_profile_id),
        profile_id_integer_mapping.get(t.thursday_profile_id),
        profile_id_integer_mapping.get(t.friday_profile_id),
        profile_id_integer_mapping.get(t.saturday_profile_id),
        profile_id_integer_mapping.get(t.sunday_profile_id),
    ]

    link = Link(
        start_node,
        end_node,
        speed_kph,
        distance_cm,
        grade_milli,
        week_profile_ids,
        weight_restriction,
        height_restriction,
        width_restriction,
        length_restriction,
    )

    return link, node_id_mapping, node_id_counter


def links_from_df(network_df_chunk, node_id_mapping, node_id_counter):
    gdf = gpd.GeoDataFrame(network_df_chunk, geometry="geom")
    gdf.crs = LATLON
    gdf = gdf.to_crs(WEB_MERCATOR)

    gdf = gdf.astype(
        {
            "netw_id": str,
            "junction_id_to": str,
            "junction_id_from": str,
            "centimeters": int,
            "link_direction": int,
            "monday_profile_id": str,
            "tuesday_profile_id": str,
            "wednesday_profile_id": str,
            "thursday_profile_id": str,
            "friday_profile_id": str,
            "saturday_profile_id": str,
            "sunday_profile_id": str,
        }
    )

    oneway_ft = gdf[gdf.link_direction == FROM_TO_DIRECTION]
    oneway_tf = gdf[gdf.link_direction == TO_FROM_DIRECTION]
    twoway = gdf[gdf.link_direction.isin([1, 9])]

    links = []
    two_way_tf_links = []
    for t in twoway.itertuples():
        link, node_id_mapping, node_id_counter = build_link(
            t, TO_FROM_DIRECTION, node_id_mapping, node_id_counter
        )
        two_way_tf_links.append(link)
    links.extend(two_way_tf_links)

    two_way_ft_links = []
    for t in twoway.itertuples():
        link, node_id_mapping, node_id_counter = build_link(
            t, FROM_TO_DIRECTION, node_id_mapping, node_id_counter
        )
        two_way_ft_links.append(link)
    links.extend(two_way_ft_links)

    oneway_ft_links = []
    for t in oneway_ft.itertuples():
        link, node_id_mapping, node_id_counter = build_link(
            t, FROM_TO_DIRECTION, node_id_mapping, node_id_counter
        )
        oneway_ft_links.append(link)
    links.extend(oneway_ft_links)

    oneway_tf_links = []
    for t in oneway_tf.itertuples():
        link, node_id_mapping, node_id_counter = build_link(
            t, TO_FROM_DIRECTION, node_id_mapping, node_id_counter
        )
        oneway_tf_links.append(link)
    links.extend(oneway_tf_links)

    return links, node_id_mapping, node_id_counter


if __name__ == "__main__":
    user = "nreinick"
    password = "NRELisgr8!"
    engine = sql.create_engine(f"postgresql://{user}:{password}@trolley.nrel.gov:5432/master")

    log.info("getting speed by time of day info from trolley..")

    # write a dummy file with the current date to make sure thes script is running
    dummy_file = Path(__file__).parent / "dummy.txt"
    with open(dummy_file, "w") as f:
        f.write(str(datetime.now()))

    q = """
    select profile_id, speed_per_time_slot_id
    from tomtom_multinet_current.mnr_profile2speed_per_time_slot
    """

    sdf = pd.read_sql(q, engine)
    sdf = sdf.astype(str)

    tq = """
    select *
    from tomtom_multinet_current.mnr_speed_per_time_slot
    """

    tdf = pd.read_sql(tq, engine)
    tdf["speed_per_time_slot_id"] = tdf.speed_per_time_slot_id.astype(str)

    df = (
        sdf.set_index("speed_per_time_slot_id")
        .join(tdf.set_index("speed_per_time_slot_id"))
        .reset_index()
        .drop(columns="speed_per_time_slot_id")
    )

    profile_id_integer_mapping = {}
    for i, pid in enumerate(df.profile_id.unique()):
        profile_id_integer_mapping[pid] = i

    df["profile_id_integer"] = df.profile_id.apply(lambda pid: profile_id_integer_mapping[pid])

    df.to_csv("/projects/mbap/amazon-eco/profile_id_mapping.csv", index=False)

    weight_restrictions_file = "/projects/mbap/amazon-eco/weight_restrictions.pickle"
    log.info("loading weight restrictions..")
    with open(weight_restrictions_file, "rb") as f:
        weight_restrictions = pickle.load(f)
    height_restrictions_file = "/projects/mbap/amazon-eco/height_restrictions.pickle"
    log.info("loading height restrictions..")
    with open(height_restrictions_file, "rb") as f:
        height_restrictions = pickle.load(f)
    width_restrictions_file = "/projects/mbap/amazon-eco/width_restrictions.pickle"
    log.info("loading width restrictions..")
    with open(width_restrictions_file, "rb") as f:
        width_restrictions = pickle.load(f)
    length_restrictions_file = "/projects/mbap/amazon-eco/length_restrictions.pickle"
    log.info("loading length restrictions..")
    with open(length_restrictions_file, "rb") as f:
        length_restrictions = pickle.load(f)

    q = """
    select
        netw_id,
        junction_id_from,
        junction_id_to,
        centimeters,
        mean_gradient_dec,
        speed_average_pos,
        speed_average_neg,
        free_flow_speed,
        monday_profile_id,
        tuesday_profile_id,
        wednesday_profile_id,
        thursday_profile_id,
        friday_profile_id,
        saturday_profile_id,
        sunday_profile_id,
        link_direction,
        geom
    from
        (
            select
                netw.netw_id,
                junction_id_from,
                junction_id_to,
                centimeters,
                mean_gradient_dec,
                speed_average_pos,
                speed_average_neg,
                speed_profile_id,
                validity_direction as link_direction,
                geom
            from
                (
                    select
                        feat_id as netw_id,
                        junction_id_from,
                        junction_id_to,
                        centimeters,
                        mean_gradient_dec,
                        speed_average_pos,
                        speed_average_neg,
                        geom
                    from
                        tomtom_multinet_current.network
                    where
                        routing_class >= 5
                ) as netw
                join tomtom_multinet_current.mnr_netw2speed_profile as nt2sp on netw.netw_id = nt2sp.netw_id
        ) as ntw_w_sp
        join tomtom_multinet_current.mnr_speed_profile as sp on ntw_w_sp.speed_profile_id = sp.speed_profile_id
    """

    log.info("getting links from trolley..")
    dfs = gpd.read_postgis(q, con=engine, chunksize=5_000_000)
    node_id_mapping: Dict[str, int] = {}
    node_id_counter = 0
    all_links = []
    for i, df in enumerate(dfs):
        start_time = time.time()
        log.info(f"working on iteration {i}")
        more_links, node_id_mapping, node_id_counter = links_from_df(
            df, node_id_mapping, node_id_counter
        )
        all_links.extend(more_links)
        log.info(f"iteration {i} took ", time.time() - start_time, " seconds")

    node_map_outfile = Path("/projects/mbap/amazon-eco/node-id-mapping.pickle")
    with node_map_outfile.open("wb") as f:
        pickle.dump(node_id_mapping, f)

    del node_id_mapping

    log.info("building graph..")
    start_time = time.time()
    graph = Graph()
    graph.add_links_bulk(all_links)
    log.info("graph took ", time.time() - start_time, " seconds")

    del all_links

    log.info("extracting largest scc..")
    start_time = time.time()
    graph = largest_scc(graph)
    log.info("largest scc took ", time.time() - start_time, " seconds")

    log.info("building rust map from graph..")
    start_time = time.time()
    rust_map = RustMap(graph)
    log.info("rust map took ", time.time() - start_time, " seconds")

    log.info("saving rust map..")
    rust_map.to_file("/scratch/nreinick/us_network_rust_map.bin")
