"""
This script is used to download the whole US road network from trolley and save it
to a set of parquet files. This was done to support quicker iteration for actually
building a routee-compass road network from this raw data.

This script has a companion file `download_us_network.sh` which is a batch script for 
running this on an eagle node
"""
import logging
import pandas as pd
import geopandas as gpd
from pathlib import Path
import sqlalchemy as sql
import math
import os
import time
from datetime import datetime
from multiprocessing import Pool

CHUNK_SIZE = 5_000_000
NUM_PROCS = 4
TABLE_NAME = "network_compass_v2"
HPC_ACCOUNT = "rfitzger"

USER = os.environ.get("TROLLEY_USERNAME")
if USER is None:
    raise ValueError("TROLLEY_USERNAME environment variable must be set to run this script")
PASSWORD = os.environ.get("TROLLEY_PASSWORD")
if PASSWORD is None:
    raise ValueError("TROLLEY_PASSWORD environment variable must be set to run this script")

# set up logging to file
date_and_time = datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
logging.basicConfig(filename=f"log_download_us_network_{date_and_time}.log", level=logging.DEBUG)

log = logging.getLogger(__name__)

OUTPUT_FOLDER = Path(f"/scratch/{HPC_ACCOUNT}/us_network/")
if not OUTPUT_FOLDER.exists():
    OUTPUT_FOLDER.mkdir()


def download_and_save_chunk(chunk_id):
    con = sql.create_engine(f"postgresql://{USER}:{PASSWORD}@trolley.nrel.gov:5432/master")

    offset = chunk_id * CHUNK_SIZE
    query = f"SELECT * FROM {TABLE_NAME} ORDER BY feat_id OFFSET {offset} LIMIT {CHUNK_SIZE}"

    file_path = OUTPUT_FOLDER / f"chunk_{chunk_id}.parquet"
    if file_path.exists():
        log.info(f"Chunk {chunk_id} already exists, skipping")
        return

    log.info(f"Downloading chunk {chunk_id}")
    start_time = time.time()
    df = gpd.read_postgis(query, con)
    elapsed_time = time.time() - start_time
    log.info(f"Chunk {chunk_id} downloaded in {elapsed_time} seconds")
    log.info(f"Casting {chunk_id} to correct types..")
    start_time = time.time()
    df = df.astype(
        {
            "feat_id": str,
            "junction_id_to": str,
            "junction_id_from": str,
            "centimeters": int,
            "routing_class": int
        }
    )
    elapsed_time = time.time() - start_time
    log.info(f"Chunk {chunk_id} casted in {elapsed_time} seconds")
    log.info(f"Saving chunk {chunk_id} to {file_path}")
    start_time = time.time()
    df.to_parquet(file_path, index=False)
    elapsed_time = time.time() - start_time
    log.info(f"Chunk {chunk_id} saved in {elapsed_time} seconds")


engine = sql.create_engine(f"postgresql://{USER}:{PASSWORD}@trolley.nrel.gov:5432/master")

count = pd.read_sql_query(f"select count(*) from {TABLE_NAME}", engine)
row_count = count["count"].values[0]

num_chunks = math.ceil(row_count / CHUNK_SIZE)

log.info(f"Downloading {row_count} rows in {num_chunks} chunks of size {CHUNK_SIZE}")

with Pool(NUM_PROCS) as p:
    p.map(download_and_save_chunk, range(num_chunks))

log.info("Done!")
