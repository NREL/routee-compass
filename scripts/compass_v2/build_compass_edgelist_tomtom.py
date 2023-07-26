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

#
# this script builds the compass edgelist from a TomTom network stored on a database.
# it depends on
#  - running build_compass_vertexlist_tomtom.py first, which creates a dependency


NETWORK_PARQUET_DIR = Path("/Users/rfitzger/data/routee/tomtom/tomtom-edges-parquet") # directory with data created via download_us_network.py
EDGES_DIR = Path("/Users/rfitzger/data/routee/tomtom/edges-tmp")
CONDENSED_DIR = Path("/Users/rfitzger/data/routee/tomtom/tomtom-condensed")              # output directory to write condensed vertexlist / edgelist files

def get_env(key):
    value = os.environ.get(key)
    if value is None:
        raise KeyError("expected environment variable {key} is missing")
    return value

# NETWORK_PARQUET_DIR = get_env('NETWORK_PARQUET_DIR')
# EDGES_DIR = get_env('EDGES_DIR')
# CONDENSED_DIR = get_env('CONDENSED_DIR')



# row link directionality
FWD_REV_LINK, FORWARD_LINK, REVERSE_LINK, OTHER_LINK = 1, 2, 3, 9
DEFAULT_SPEED_KPH = 40

parq_cols = ['feat_id', 'junction_id_from', 'junction_id_to', 'centimeters',
       'routing_class', 'mean_gradient_dec', 'speed_average_pos',
       'speed_average_neg', 'free_flow_speed', 'monday_profile_id',
       'tuesday_profile_id', 'wednesday_profile_id', 'thursday_profile_id',
       'friday_profile_id', 'saturday_profile_id', 'sunday_profile_id',
       'validity_direction', 'geom']

def create_fwd_edge(row):
    grade = 0 if math.isnan(row['mean_gradient_dec']) else row['mean_gradient_dec']
    distance_meters = row['centimeters'] / 100.0
    return {
        "edge_uuid": row['feat_id'],
        "src_vertex_uuid": row['junction_id_from'],
        "dst_vertex_uuid": row['junction_id_to'],
        "road_class": row['routing_class'],
        "speed_free_flow_kph": row['free_flow_speed'],
        # "speed_average_kph": row['speed_average_pos'],
        "distance_meters": distance_meters,
        "grade": grade,
        "geometry": row['geom']
    }

def create_rev_edge(row):
    rev_geom = LineString(reversed(row['geom'].coords))
    rev_grade = 0 if math.isnan(row['mean_gradient_dec']) else -row['mean_gradient_dec'] 
    distance_meters = row['centimeters'] / 100.0
    return {
        "edge_uuid": row['feat_id'],
        "src_vertex_uuid": row['junction_id_to'],
        "dst_vertex_uuid": row['junction_id_from'],
        "road_class": row['routing_class'],
        "speed_free_flow_kph": row['free_flow_speed'],
        # "speed_average_kph": row['speed_average_neg'],
        "distance_meters": distance_meters,
        "grade": rev_grade,
        "geometry": rev_geom
    }

def create_edges_from_row(row) -> Tuple[List, int]:
    """creates edges from this row based on the validity_direction argument

    :param row: _description_
    :type row: _type_
    :raises TypeError: _description_
    :return: _description_
    :rtype: Tuple[List, int]
    """
    try:
        if row['validity_direction'] == FWD_REV_LINK:
            return [create_fwd_edge(row), create_rev_edge(row)]
        if row['validity_direction'] == FORWARD_LINK or row['validity_direction'] == OTHER_LINK:
            return [create_fwd_edge(row)]
        if row['validity_direction'] == REVERSE_LINK:
            return [create_rev_edge(row)]
        raise TypeError()
    except TypeError:  # fail when validity_direction is NA, or, unrecognized direction
        return [create_fwd_edge(row)]

def get_number(suffix: str = "csv"):
    def _get(path: Path):
        number_str = path.name.replace(f'.{suffix}', '').split("_")[-1]
        return int(number_str)
    return _get

if __name__ == '__main__':

    # TODO: 
    # list of things to fix before next run of this script:
    # 1. speed values should be integers, but came out as floats

    # load the complete set of vertex data into memory
    print("loading vertex lookup table")
    vertices = pd.read_csv(CONDENSED_DIR / "vertices-complete.csv.gz").set_index("junction_id")
    
    def replace_id(junction_id):
        return vertices.loc[junction_id].vertex_id

    # iterate through edges and assign to/from vertex ids, edge ids
    i = 0
    parquet_filenames = sorted(
        list(filter(lambda p: p.name.endswith("parquet"), NETWORK_PARQUET_DIR.iterdir())),
        key=get_number("parquet")
    )
    for path in tqdm(parquet_filenames):
        dst_file = EDGES_DIR / path.with_suffix(".csv")
        if dst_file.exists():
            print(f"{dst_file} exists; skipping.")
            continue

        now = datetime.datetime.now().isoformat()
        print(f"{now} reading edge source file {path.name}")
        dtypes = {
            'junction_id_from': str,
            'junction_id_to': str,
            'centimeters': float,
            'routing_class': "Int64",
            'mean_gradient_dec': float,
            'free_flow_speed': float,
            'validity_direction': "Int64"
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
    parquet_filenames = sorted(
        list(filter(lambda p: p.name.startswith("chunk"), EDGES_DIR.iterdir())),
        key=get_number("csv")
    )
    for path in tqdm(parquet_filenames):
        edges.append(pd.read_csv(path))
        n_edge_files += 1
    print(f"finished reading {n_edge_files} files")
    edges = pd.concat(edges)
    print("concatenated files into integer-indexed, sorted edge list")

    edges_complete_file = CONDENSED_DIR / "edges-complete.csv.gz"
    edges_compass_file = CONDENSED_DIR / "edges-compass.csv.gz"
    edges_mapping_file = CONDENSED_DIR / "edges-mapping.csv.gz"
    
    edges_geometries_file = CONDENSED_DIR / "edges-geometries.txt.gz"
    edges_free_flow_speed_file = CONDENSED_DIR / "edges-free-flow-speed.txt.gz"
    # edges_average_speed_file = CONDENSED_DIR / "edges-average-speed.txt.gz"

    compass_cols = ['edge_id', 'src_vertex_id', 'dst_vertex_id', 
                    'road_class', 'distance_meters', 'grade']

    print("writing csv outputs")
    print(f"writing {edges_complete_file}")
    edges.to_csv(edges_complete_file, index=False)

    print(f"writing {edges_compass_file}")
    edges[compass_cols].to_csv(edges_compass_file, index=False)
    
    print(f"writing {edges_mapping_file}")
    edges[['edge_id', 'edge_uuid']].to_csv(edges_mapping_file, index=False)

    print("writing flat outputs")
    print(f"writing {edges_geometries_file}")
    edges.geometry.to_csv(edges_geometries_file, index=False, header=False)
    
    print(f"writing {edges_free_flow_speed_file}")
    edges.speed_free_flow_kph.to_csv(edges_free_flow_speed_file, index=False, header=False)

    # print(f"writing {edges_average_speed_file}")
    # edges.speed_average_kph.to_csv(edges_average_speed_file, index=False, header=False)

    print("done.")