import argparse
import getpass
import logging as log
import sys
from pathlib import Path

import geopandas as gpd
import networkx as nx
import numpy as np
from shapely.geometry import Point
from sqlalchemy import create_engine
from sqlalchemy.exc import OperationalError

log.basicConfig(level=log.INFO)

METERS_TO_MILES = 0.0006213712
KPH_TO_MPH = 0.621371

parser = argparse.ArgumentParser(description="get osm road network")
parser.add_argument(
    "polygon_shp_file",
    help="path to a polygon shape file that defines road network boundaries."
)
parser.add_argument(
    "outfile",
    help="where should the network pickle file be written?"
)

parser.add_argument("--dual_graph", help="create dual graph rather than base graph", action="store_true")


def build_graph(gdf: gpd.geodataframe.GeoDataFrame) -> nx.MultiDiGraph:
    gdf['id'] = gdf.id.astype(int)
    gdf['f_jnctid'] = gdf.f_jnctid.astype(int)
    gdf['t_jnctid'] = gdf.t_jnctid.astype(int)
    gdf['f_lon'] = gdf.wkb_geometry.apply(lambda g: list(g.coords)[0][0])
    gdf['f_lat'] = gdf.wkb_geometry.apply(lambda g: list(g.coords)[0][1])
    gdf['t_lon'] = gdf.wkb_geometry.apply(lambda g: list(g.coords)[-1][0])
    gdf['t_lat'] = gdf.wkb_geometry.apply(lambda g: list(g.coords)[-1][1])
    oneway_ft = gdf[gdf.oneway == 'FT']
    oneway_tf = gdf[gdf.oneway == 'TF']
    twoway = gdf[~(gdf.oneway == 'FT') & ~(gdf.oneway == 'TF')]

    twoway_edges_tf = [
        (t, f, -k, {
            'meters': mt,
            'minutes': mn,
            'kph': kph,
            'grade': -g,
            'geom': geom,
        }) for t, f, k, mt, mn, kph, g, geom in zip(
            twoway.t_jnctid.values,
            twoway.f_jnctid.values,
            twoway.id,
            twoway.meters.values,
            twoway.minutes.values,
            twoway.kph.values,
            twoway.mean_grad.values,
            twoway.wkb_geometry.values,
        )
    ]
    twoway_edges_ft = [
        (f, t, k, {
            'meters': mt,
            'minutes': mn,
            'kph': kph,
            'grade': g,
            'geom': geom,
        }) for t, f, k, mt, mn, kph, g, geom in zip(
            twoway.t_jnctid.values,
            twoway.f_jnctid.values,
            twoway.id,
            twoway.meters.values,
            twoway.minutes.values,
            twoway.kph.values,
            twoway.mean_grad.values,
            twoway.wkb_geometry.values,
        )
    ]
    oneway_edges_ft = [
        (f, t, k, {
            'meters': mt,
            'minutes': mn,
            'kph': kph,
            'grade': g,
            'geom': geom,
        }) for t, f, k, mt, mn, kph, g, geom in zip(
            oneway_ft.t_jnctid.values,
            oneway_ft.f_jnctid.values,
            oneway_ft.id,
            oneway_ft.meters.values,
            oneway_ft.minutes.values,
            oneway_ft.kph.values,
            oneway_ft.mean_grad.values,
            oneway_ft.wkb_geometry.values,
        )
    ]
    oneway_edges_tf = [
        (t, f, -k, {
            'meters': mt,
            'minutes': mn,
            'kph': kph,
            'grade': -g,
            'geom': geom,
        }) for t, f, k, mt, mn, kph, g, geom in zip(
            oneway_tf.t_jnctid.values,
            oneway_tf.f_jnctid.values,
            oneway_tf.id,
            oneway_tf.meters.values,
            oneway_tf.minutes.values,
            oneway_tf.kph.values,
            oneway_tf.mean_grad.values,
            oneway_tf.wkb_geometry.values,
        )
    ]

    flats = {nid: lat for nid, lat in zip(gdf.f_jnctid.values, gdf.f_lat)}
    flons = {nid: lon for nid, lon in zip(gdf.f_jnctid.values, gdf.f_lon)}
    tlats = {nid: lat for nid, lat in zip(gdf.t_jnctid.values, gdf.t_lat)}
    tlons = {nid: lon for nid, lon in zip(gdf.t_jnctid.values, gdf.t_lon)}

    G = nx.MultiDiGraph()
    G.add_edges_from(twoway_edges_tf)
    G.add_edges_from(twoway_edges_ft)
    G.add_edges_from(oneway_edges_ft)
    G.add_edges_from(oneway_edges_tf)

    nx.set_node_attributes(G, flats, "lat")
    nx.set_node_attributes(G, flons, "lon")
    nx.set_node_attributes(G, tlats, "lat")
    nx.set_node_attributes(G, tlons, "lon")

    log.info("extracting largest connected component..")
    n_edges_before = G.number_of_edges()
    G = nx.MultiDiGraph(G.subgraph(max(nx.strongly_connected_components(G), key=len)))
    n_edges_after = G.number_of_edges()
    log.info(f"final graph has {n_edges_after} edges, lost {n_edges_before - n_edges_after}")

    G.graph['compass_network_type'] = 'tomtom'

    return G


def build_dual_graph(g: nx.MultiDiGraph) -> nx.MultiDiGraph:
    """
    builds a dual graph and computes the angle between each edge for turn cost energy estimation

    :param g: the original graph
    :return: a graph dual of g
    """

    def _compute_angle(e1_id: int, e2_id: int) -> float:
        """
        helper function to compute the angle between two links.
        """

        def _azimuth(point1, point2):
            angle = np.arctan2(point2.x - point1.x, point2.y - point1.y)
            return np.degrees(angle)

        e1_coords = list(g.edges[e1_id]['geom'].coords)
        if e1_id[2] >= 0:
            e1_points = e1_coords[-2:]
        else:
            e1_points = list(reversed(e1_coords))[-2:]

        e2_coords = list(g.edges[e2_id]['geom'].coords)
        if e2_id[2] >= 0:
            e2_points = e2_coords[:2]
        else:
            e2_points = list(reversed(e2_coords))[:2]

        a1 = _azimuth(Point(e1_points[0]), Point(e1_points[1]))
        a2 = _azimuth(Point(e2_points[1]), Point(e2_points[0]))

        return abs(180 - abs((a2 - a1)))

    g_dual = nx.line_graph(g)

    graph_data = {}
    for u, v, k in g_dual.edges(keys=True):
        e1_data = g.edges[u]
        e2_data = g.edges[v]

        angle = _compute_angle(u, v)
        meters = e1_data['meters'] / 2 + e2_data['meters'] / 2  # half from each link
        minutes = e1_data['minutes'] / 2 + e2_data['minutes'] / 2  # half from each link

        # distance weighted mean
        kph = np.average([e1_data['kph'], e2_data['kph']], weights=[e1_data['meters'], e1_data['meters']])

        grade = (e1_data['grade'] + e2_data['grade']) / 2  # mean

        graph_data[(u, v, k)] = {
            'angle': angle,
            'meters': meters,
            'minutes': minutes,
            'kph': kph,
            'grade': grade,
        }

    nx.set_edge_attributes(g_dual, graph_data)

    # dual node coordinates get set to the coordinate of the first node in the dual node.
    node_data = {}
    for n1, n2, k in g_dual.nodes():
        coords = g.nodes[n1]
        node_data[(n1, n2, k)] = {
            'lat': coords['lat'],
            'lon': coords['lon']
        }
    nx.set_node_attributes(g_dual, node_data)

    g_dual.graph['compass_network_type'] = 'tomtom_dual'

    return g_dual


def get_tomtom_gdf(shp_filepath: str) -> gpd.GeoDataFrame:
    """
    pull raw tomtom data from trolley

    :return: gdf of tomtom links
    """
    username = input("Please enter your Trolley username: ")
    password = getpass.getpass("Please enter your Trolley password: ")
    try:
        engine = create_engine('postgresql://' + username + ':' + password + '@trolley.nrel.gov:5432/master')
        engine.connect()
        log.info("established connection with Trolley")
    except OperationalError as oe:
        raise IOError("can't connect to Trolley..") from oe

    shp_gdf = gpd.read_file(shp_filepath)
    polygon = shp_gdf.iloc[0].geometry

    log.info("pulling raw tomtom network from Trolley..")
    q = f"""
    select mn.id, f_jnctid, t_jnctid, frc, backrd, rdcond, privaterd, roughrd, 
    meters, minutes, kph, oneway, gd.mean_grad, wkb_geometry 
    from tomtom_multinet_2017.multinet_2017 as mn
    left join 
    (
        select id, avg(grad) as mean_grad from tomtom_ada_2017.gradient
        where gradsrc > 0 
        group by id
    ) as gd
    on mn.id = gd.id
    where ST_Contains(ST_GeomFromEWKT('SRID={shp_gdf.crs.to_epsg()};{polygon.wkt}'), 
    ST_GeomFromEWKB(mn.wkb_geometry))
    """
    raw_gdf = gpd.GeoDataFrame.from_postgis(
        q,
        con=engine,
        geom_col="wkb_geometry",
    )
    log.info(f"pulled {raw_gdf.shape[0]} links")
    log.info("cleaning raw data..")

    raw_gdf['mean_grad'] = raw_gdf.mean_grad / 10  # convert from 1 / 1000 to 1 / 100

    raw_gdf = raw_gdf[
        (raw_gdf.rdcond == 1) &
        (raw_gdf.frc < 7) &
        (raw_gdf.backrd == 0) &
        (raw_gdf.privaterd == 0) &
        (raw_gdf.roughrd == 0)
        ].fillna(0)

    log.info(f"{raw_gdf.shape[0]} links remain after filtering")

    return raw_gdf


def graph_to_file(g: nx.MultiDiGraph, outfile: Path):
    # remove the link geometry to save space
    for _, _, data in g.edges(data=True):
        if 'geom' in data:
            del (data['geom'])

    log.info(f"writing to file {outfile}..")
    nx.write_gpickle(g, outfile)


def get_tomtom_network():
    args = parser.parse_args()

    tomtom_gdf = get_tomtom_gdf(args.polygon_shp_file)

    log.info("building graph from raw network..")
    g = build_graph(tomtom_gdf)
    g_outfile = Path(args.outfile)

    if args.dual_graph:
        log.info("building graph dual from base graph..")
        g = build_dual_graph(g)

    graph_to_file(g, g_outfile)

    return 1


if __name__ == "__main__":
    sys.exit(get_tomtom_network() or 0)
