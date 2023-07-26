# building datasets for compass v2 from TomTom

The MBAP team has access to a licensed road network dataset stored on the Trolley server in a PostGreSQL database.
For our typical use cases with RouteE Compass, we download and transform the TomTom network into compressed CSV 
files. 
These scripts perform the import process as follows:
  1. run download_us_network_v2.sh on Eagle
    - command: `sbatch download_us_network_v2.sh`
    - batch downloads rows of the network to Eagle scratch
  2. run `build_compass_vertexlist_tomtom.py`
    - reads junctions table from TomTom
    - generates the following files
      - `vertices-complete.csv.gz`: the original junction dataset plus ids
      - `vertices-mapping.csv.gz`: UUIDs and integer ids only
      - `vertices-compass.csv.gz`: integer ids and point as x + y columns 
  3. run `build_compass_edgelist_tomtom.py`
    - loads `vertices-complete.csv.gz` into memory
    - reads in the parquet files one-at-a-time from step 1
    - for each file, constructs directed edge rows and gives them ids
    - loads all edges into a single dataset
    - generates the following files
      - `edges-complete.csv.gz`: the original edges dataset plus ids
      - `edges-mapping.csv.gz`: UUIDs and integer ids only
      - `edges-compass.csv.gz`: integer ids and basic edge attributes 
      - `edges-geometries.txt.gz`: linestring geometries for each edge
      - `edges-free-flow-speed.txt.gz`: free flow speeds for each edge

## dependencies

- geopandas
- tqdm
- sqlalchemy (< 2.0.0)
- pyarrow