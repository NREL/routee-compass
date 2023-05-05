import logging
import pandas as pd
import geopandas as gpd
from pathlib import Path
import sqlalchemy as sql
import math
from datetime import datetime
from concurrent.futures import ThreadPoolExecutor

chunk_size = 1_000_000
num_threads = 8
table_name = "tomtom_multinet_current.network_w_speed_profiles"
WEB_MERCATOR = "epsg:3857"

user = "nreinick"
password = "NRELisgr8!"
engine = sql.create_engine(f"postgresql://{user}:{password}@trolley.nrel.gov:5432/master")

# set up logging to file
date_and_time = datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
logging.basicConfig(filename=f"log_download_us_network_{date_and_time}.log", level=logging.DEBUG)

log = logging.getLogger(__name__)


def download_and_save_chunk(chunk_id, query, con, output_folder):
    file_path = output_folder / f"chunk_{chunk_id}.parquet"
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


output_folder = Path("/scratch/nreinick/us_network/")
if not output_folder.exists():
    output_folder.mkdir()

count = pd.read_sql_query(f"select count(*) from {table_name}", engine)
row_count = count["count"].values[0]

num_chunks = math.ceil(row_count / chunk_size)

log.info(f"Downloading {row_count} rows in {num_chunks} chunks of size {chunk_size}")

with ThreadPoolExecutor(max_workers=num_threads) as executor:
    for chunk_id in range(num_chunks):
        offset = chunk_id * chunk_size
        query = f"SELECT * FROM {table_name} OFFSET {offset} LIMIT {chunk_size} ORDER BY netw_id"
        executor.submit(download_and_save_chunk, chunk_id, query, engine, output_folder)
