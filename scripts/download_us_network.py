import logging
import pandas as pd
import geopandas as gpd
from pathlib import Path
import sqlalchemy as sql
import math
from datetime import datetime
from multiprocessing import Pool

CHUNK_SIZE = 1_000_000
NUM_PROCS = 8
TABLE_NAME = "tomtom_multinet_current.network_w_speed_profiles"
WEB_MERCATOR = "epsg:3857"

USER = "nreinick"
PASSWORD = "NRELisgr8!"

# set up logging to file
date_and_time = datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
logging.basicConfig(filename=f"log_download_us_network_{date_and_time}.log", level=logging.DEBUG)

log = logging.getLogger(__name__)

OUTPUT_FOLDER = Path("/scratch/nreinick/us_network/")
if not OUTPUT_FOLDER.exists():
    OUTPUT_FOLDER.mkdir()


def download_and_save_chunk(chunk_id):
    con = sql.create_engine(f"postgresql://{USER}:{PASSWORD}@trolley.nrel.gov:5432/master")

    offset = chunk_id * CHUNK_SIZE
    query = f"SELECT * FROM {TABLE_NAME} OFFSET {offset} LIMIT {CHUNK_SIZE} ORDER BY netw_id"

    file_path = OUTPUT_FOLDER / f"chunk_{chunk_id}.parquet"
    if file_path.exists():
        log.info(f"Chunk {chunk_id} already exists, skipping")
        return

    df = gpd.read_postgis(query, con)
    df = df.to_crs(WEB_MERCATOR)
    df = df.astype(
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
    df.to_parquet(file_path, index=False)
    log.info(f"Chunk {chunk_id} saved to {file_path}")


engine = sql.create_engine(f"postgresql://{USER}:{PASSWORD}@trolley.nrel.gov:5432/master")
count = pd.read_sql_query(f"select count(*) from {TABLE_NAME}", engine)
row_count = count["count"].values[0]

num_chunks = math.ceil(row_count / CHUNK_SIZE)

log.info(f"Downloading {row_count} rows in {num_chunks} chunks of size {CHUNK_SIZE}")

with Pool(NUM_PROCS) as p:
    p.map(download_and_save_chunk, range(num_chunks))

log.info("Done!")
