import argparse
import getpass
import logging as log
import os

import geopandas as gpd
import networkx as nx
import pandas as pd
from sqlalchemy import create_engine
from sqlalchemy.exc import OperationalError

from compass.utils.routee_utils import RouteeModelCollection

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


def add_energy(g: nx.MultiDiGraph) -> nx.MultiDiGraph:
    """
    precompute energy on the graph

    :param g:
    :return:
    """
    routee_model_collection = RouteeModelCollection()

    speed = pd.DataFrame.from_dict(
        nx.get_edge_attributes(g, 'kph'),
        orient="index",
        columns=['gpsspeed'],
    ).multiply(KPH_TO_MPH)
    distance = pd.DataFrame.from_dict(
        nx.get_edge_attributes(g, 'meters'),
        orient="index",
        columns=['miles'],
    ).multiply(METERS_TO_MILES)
    grade = pd.DataFrame.from_dict(
        nx.get_edge_attributes(g, 'grade'),
        orient="index",
        columns=['grade'],
    )
    df = speed.join(distance).join(grade)

    for k, model in routee_model_collection.routee_models.items():
        energy = model.predict(df).to_dict()
        nx.set_edge_attributes(g, name=f"energy_{k}", values=energy)

    return g


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
        }) for t, f, k, mt, mn, kph, g in zip(
            twoway.t_jnctid.values,
            twoway.f_jnctid.values,
            twoway.id,
            twoway.meters.values,
            twoway.minutes.values,
            twoway.kph.values,
            twoway.mean_grad.values,
        )
    ]
    twoway_edges_ft = [
        (f, t, k, {
            'meters': mt,
            'minutes': mn,
            'kph': kph,
            'grade': g,
        }) for t, f, k, mt, mn, kph, g in zip(
            twoway.t_jnctid.values,
            twoway.f_jnctid.values,
            twoway.id,
            twoway.meters.values,
            twoway.minutes.values,
            twoway.kph.values,
            twoway.mean_grad.values,
        )
    ]
    oneway_edges_ft = [
        (f, t, k, {
            'meters': mt,
            'minutes': mn,
            'kph': kph,
            'grade': g,
        }) for t, f, k, mt, mn, kph, g in zip(
            oneway_ft.t_jnctid.values,
            oneway_ft.f_jnctid.values,
            oneway_ft.id,
            oneway_ft.meters.values,
            oneway_ft.minutes.values,
            oneway_ft.kph.values,
            oneway_ft.mean_grad.values,
        )
    ]
    oneway_edges_tf = [
        (t, f, -k, {
            'meters': mt,
            'minutes': mn,
            'kph': kph,
            'grade': -g
        }) for t, f, k, mt, mn, kph, g in zip(
            oneway_tf.t_jnctid.values,
            oneway_tf.f_jnctid.values,
            oneway_tf.id,
            oneway_tf.meters.values,
            oneway_tf.minutes.values,
            oneway_tf.kph.values,
            oneway_tf.mean_grad.values,
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

    return G


if __name__ == "__main__":
    args = parser.parse_args()

    username = input("Please enter your Trolley username: ")
    password = getpass.getpass("Please enter your Trolley password: ")
    try:
        engine = create_engine('postgresql://' + username + ':' + password + '@trolley.nrel.gov:5432/master')
        engine.connect()
        log.info("established connection with Trolley")
    except OperationalError as oe:
        raise IOError("can't connect to Trolley..") from oe

    denver_gdf = gpd.read_file(args.polygon_shp_file)
    denver_polygon = denver_gdf.iloc[0].geometry

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
    where ST_Contains(ST_GeomFromEWKT('SRID={denver_gdf.crs.to_epsg()};{denver_polygon.wkt}'), 
    ST_GeomFromEWKB(mn.wkb_geometry))
    """
    raw_gdf = gpd.GeoDataFrame.from_postgis(
        q,
        con=engine,
        geom_col="wkb_geometry",
    )
    log.info(f"pulled {raw_gdf.shape[0]} links")
    log.info("cleaning raw data..")

    raw_gdf['mean_grad'] = raw_gdf.mean_grad.apply(lambda g: 50 if g > 50 else g)
    raw_gdf['mean_grad'] = raw_gdf.mean_grad.apply(lambda g: -50 if g < -50 else g)

    raw_gdf = raw_gdf[
        (raw_gdf.rdcond == 1) &
        (raw_gdf.frc < 7) &
        (raw_gdf.backrd == 0) &
        (raw_gdf.privaterd == 0) &
        (raw_gdf.roughrd == 0)
        ].fillna(0)

    log.info(f"{raw_gdf.shape[0]} links remain after filtering")

    log.info("building graph from raw network..")
    G = build_graph(raw_gdf)

    log.info("precomputing energy on the network..")
    G = add_energy(G)

    G.graph['compass_network_type'] = 'tomtom'

    log.info("writing to file..")
    nx.write_gpickle(G, args.outfile)
