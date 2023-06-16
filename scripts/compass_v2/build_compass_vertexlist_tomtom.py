import pandas as pd
from pathlib import Path
from pathlib import Path
import pandas as pd
import math
import sqlalchemy as sql
from multiprocessing import Pool
import geopandas as gpd
from tqdm import tqdm
import os

USER = os.getenv("TROLLEY_USERNAME")              # trolley.nrel.gov account username
PASSWORD = os.getenv("TROLLEY_PASSWORD")          # trolley.nrel.gov account password
CHUNK_SIZE = 1_000_000 # number of junction rows to download at a time in each process (~57 million rows total)
NUM_PROCS = 4          # parallel execution for each chunk
VERTICES_DIR = Path("/Users/rfitzger/data/routee/tomtom/tomtom-vertices")      # output directory to write individual vertex files
CONDENSED_DIR = Path("/Users/rfitzger/data/routee/tomtom/tomtom-condensed")  # output directory to write vertex mapping file

def download_save_vertices(chunk_id: int):
    vertices_condensed_path = VERTICES_DIR / f"vertex_chunk_{chunk_id}.csv"
    mapping_path = VERTICES_DIR / f"vertex_mapping_chunk_{chunk_id}.csv"
    vertices_edge_lookup_path = VERTICES_DIR / f"vertex_edge_lookup_chunk_{chunk_id}.csv"
    if vertices_condensed_path.exists() or mapping_path.exists():
        print(f"Chunk {chunk_id} file already exists, skipping")
        return
    
    con = sql.create_engine(f"postgresql://{USER}:{PASSWORD}@trolley.nrel.gov:5432/master")

    offset = chunk_id * CHUNK_SIZE
    query = (
        f"SELECT feat_id as junction_id, geom "
        f"FROM tomtom_multinet_current.mnr_junction "
        f"ORDER BY junction_id OFFSET {offset} LIMIT {CHUNK_SIZE}"
    )

    df = gpd.read_postgis(query, con)
    df['x'] = df.geometry.apply(lambda g: g.x)
    df['y'] = df.geometry.apply(lambda g: g.y)
    df['vertex_id'] = range(offset, offset+len(df))
    df[['vertex_id', 'junction_id']].to_csv(mapping_path, index=False)
    df[['vertex_id', 'x', 'y']].to_csv(vertices_condensed_path, index=False)
    df.to_csv(vertices_edge_lookup_path, index=False)
    print(f"finished writing chunk {chunk_id} to disk.")

if __name__ == '__main__':
    if USER is None or PASSWORD is None:
        raise IOError("must set TROLLEY_USERNAME and TROLLEY_PASSWORD environment variables")

    engine = sql.create_engine(f"postgresql://{USER}:{PASSWORD}@trolley.nrel.gov:5432/master")

    count = pd.read_sql_query(f"select count(*) from tomtom_multinet_current.mnr_junction", engine)
    n_vertices = count["count"].values[0]
    print(f"found {n_vertices} vertices")

    n_chunks = math.ceil(n_vertices / CHUNK_SIZE)
    print(f"submitting sql queries across {n_chunks} queries")

    with Pool(NUM_PROCS) as p:
        p.map(download_save_vertices, range(n_chunks))

    # build mapping table from all vertex mapping chunks. this will be very large
    vertices = []
    print(f"reading vertex mapping files from {VERTICES_DIR}")
    n_vertex_files = 0
    for path in tqdm(VERTICES_DIR.iterdir()):
        if path.name.startswith("vertex_mapping_chunk"):
            vertices.append(pd.read_csv(path))
            n_vertex_files += 1
    print(f"finished reading {n_vertex_files} files")
    vertices = pd.concat(vertices)
    print("concatenated files into indexed vertex list")
    
    vertex_mapping_file = CONDENSED_DIR / "vertex_mapping.csv.gz"
    vertices.to_csv(vertex_mapping_file)
    del vertices

    # build lookup table
    vertices = []
    print(f"reading vertex lookup files from {VERTICES_DIR}")
    n_vertex_files = 0
    for path in tqdm(VERTICES_DIR.iterdir()):
        if path.name.startswith("vertex_edge_lookup_chunk"):
            vertices.append(pd.read_csv(path))
            n_vertex_files += 1
    print(f"finished reading {n_vertex_files} files")
    vertices = pd.concat(vertices)
    print("concatenated files into indexed vertex list")
    
    vertex_mapping_file = CONDENSED_DIR / "vertex_edge_lookup.csv.gz"
    vertices.to_csv(vertex_mapping_file)
    del vertices

    # build condensed vertex dataset file
    vertices = []
    print(f"reading vertex mapping files from {VERTICES_DIR}")
    n_vertex_files = 0
    for path in tqdm(VERTICES_DIR.iterdir()):
        if path.name.startswith("vertex_chunk"):
            vertices.append(pd.read_csv(path))
            n_vertex_files += 1
    print(f"finished reading {n_vertex_files} files")
    vertices = pd.concat(vertices)
    print("concatenated files into indexed vertex list")
    
    vertex_mapping_file = CONDENSED_DIR / "vertices.csv.gz"
    vertices.to_csv(vertex_mapping_file)