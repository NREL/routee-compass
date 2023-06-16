from typing import List, Tuple
import pandas as pd
from pathlib import Path
import pandas as pd
import math
from shapely import wkb
from shapely.geometry import LineString
import os
from tqdm import tqdm
import datetime

NETWORK_PARQUET_DIR = Path("/Users/rfitzger/data/routee/tomtom/tomtom-edges-parquet") # directory with data created via download_us_network.py
EDGES_DIR = Path("/Users/rfitzger/data/routee/tomtom/tomtom-edges")
CONDENSED_DIR = Path("/Users/rfitzger/data/routee/tomtom/tomtom-condensed")              # output directory to write condensed vertexlist / edgelist files
# row link directionality
FWD_REV_LINK, FORWARD_LINK, REVERSE_LINK, OTHER_LINK = 1, 2, 3, 9
DEFAULT_SPEED_KPH = 40

parq_cols = ['netw_id', 'junction_id_from', 'junction_id_to', 'centimeters',
       'routing_class', 'mean_gradient_dec', 'speed_average_pos',
       'speed_average_neg', 'free_flow_speed', 'monday_profile_id',
       'tuesday_profile_id', 'wednesday_profile_id', 'thursday_profile_id',
       'friday_profile_id', 'saturday_profile_id', 'sunday_profile_id',
       'link_direction', 'geom']

if __name__ == '__main__':

    # TODO: 
    # list of things to fix before next run of this script:
    # 1. speed values should be integers, but came out as floats

    # load the complete set of vertex data into memory
    print("loading vertex lookup table")
    vertices = pd.read_csv(CONDENSED_DIR / "vertex_edge_lookup.csv.gz").set_index("junction_id")
    
    def replace_id(junction_id):
        return vertices.loc[junction_id].vertex_id
    
    def grade_to_millis(grade):
        try:
            return int(grade * 1000.0)
        except ValueError:
            return 0
    
    def kph_to_cps(kph):
        try:
            return int(kph * 27.7778) # approximate formula from google
        except ValueError:
            return DEFAULT_SPEED_KPH * 27.7778  # when not defined
    
    def create_fwd_edge(row):
        grade = 0 if math.isnan(row['mean_gradient_dec']) else row['mean_gradient_dec']
        return {
            "edge_uuid": row['netw_id'],
            "src_vertex_uuid": row['junction_id_from'],
            "dst_vertex_uuid": row['junction_id_to'],
            "road_class": row['routing_class'],
            "free_flow_speed_cps": kph_to_cps(row['free_flow_speed']),
            "distance_centimeters": int(row['centimeters']),
            "grade_millis": grade_to_millis(grade),
            "geometry": row['geom']
        }

    def create_rev_edge(row):
        rev_geom = LineString(reversed(row['geom'].coords))
        rev_grade = 0 if math.isnan(row['mean_gradient_dec']) else -row['mean_gradient_dec'] 
        return {
            "edge_uuid": row['netw_id'],
            "src_vertex_uuid": row['junction_id_to'],
            "dst_vertex_uuid": row['junction_id_from'],
            "road_class": row['routing_class'],
            "free_flow_speed_cps": kph_to_cps(row['free_flow_speed']),
            "distance_centimeters": int(row['centimeters']),
            "grade_millis": grade_to_millis(rev_grade),
            "geometry": rev_geom
        }

    def create_edges_from_row(row) -> Tuple[List, int]:
        """creates edges from this row based on the link_direction argument

        :param row: _description_
        :type row: _type_
        :raises TypeError: _description_
        :return: _description_
        :rtype: Tuple[List, int]
        """
        try:
            if row['link_direction'] == FWD_REV_LINK:
                return [create_fwd_edge(row), create_rev_edge(row)]
            if row['link_direction'] == FORWARD_LINK or row['link_direction'] == OTHER_LINK:
                return [create_fwd_edge(row)]
            if row['link_direction'] == REVERSE_LINK:
                return [create_rev_edge(row)]
            raise TypeError()
        except TypeError:  # fail when link_direction is NA, or, unrecognized direction
            return [create_fwd_edge(row)]

    # iterate through edges and assign to/from vertex ids, edge ids
    i = 0
    for path in tqdm(NETWORK_PARQUET_DIR.iterdir()):
        is_parquet = path.name.endswith("parquet")
        dst_file = EDGES_DIR / path.with_suffix(".csv")
        if dst_file.exists():
            print(f"{dst_file} exists; skipping.")
            continue
        if is_parquet:
            now = datetime.datetime.now().isoformat()
            print(f"{now} reading edge source file {path.name}")
            dtypes = {
                'junction_id_from': str,
                'junction_id_to': str,
                'centimeters': float,
                'routing_class': "Int64",
                'mean_gradient_dec': float,
                'free_flow_speed': float,
                'link_direction': "Int64"
            }
            df = pd.read_parquet(path).astype(dtypes)
            print("  decoding geometry binaries")
            df['geom'] = df.geom.apply(wkb.loads)

            print("  splitting rows into directed network links")
            directed = []
            for idx, row in df.iterrows():
                directed.extend(create_edges_from_row(row))

            # assign vertex ids and edge ids to each row
            print(f"  assigning integer ids in range [{i}, {i+len(directed)})")
            directed = pd.DataFrame(directed)
            directed['src_vertex_id'] = directed.src_vertex_uuid.apply(replace_id)
            directed['dst_vertex_id'] = directed.dst_vertex_uuid.apply(replace_id)
            directed['edge_id'] = range(i, i+len(directed))
            i += len(directed)

            print(f"  writing directed edgelist chunk to {EDGES_DIR / path.name}")
            directed.to_csv(dst_file, index=False)
    
    # accumulate csv.gz file outputs
    edges = []
    print(f"reading edge files from {EDGES_DIR}")
    n_edge_files = 0
    for path in tqdm(EDGES_DIR.iterdir()):
        if path.name.startswith("chunk"):
            edges.append(pd.read_csv(path))
            n_edge_files += 1
    print(f"finished reading {n_edge_files} files")
    edges = pd.concat(edges)
    print("concatenated files into indexed edge list")

    edges_raw = CONDENSED_DIR / "edges_raw.csv.gz"
    edges_compass = CONDENSED_DIR / "edges_compass.csv.gz"
    edge_geometries = CONDENSED_DIR / "edge_geometries.csv.gz"
    edges_mapping_file = CONDENSED_DIR / "edge_mapping.csv.gz"
    compass_cols = ['edge_id', 'src_vertex_id', 'dst_vertex_id', 
                    'road_class', 'free_flow_speed_cps', 'distance_centimeters', 'grade_millis']

    print(f"writing {edges_raw}")
    edges.to_csv(edges_raw, index=False)

    print(f"writing {edges_compass}")
    edges[['edge_id', 'geometry']].to_csv(edge_geometries, index=False)
    
    print(f"writing {edge_geometries}")
    edges[compass_cols].to_csv(edges_compass, index=False)
    
    print(f"writing {edges_mapping_file}")
    edges[['edge_id', 'edge_uuid']].to_csv(edges_mapping_file, index=False)

    print("done.")