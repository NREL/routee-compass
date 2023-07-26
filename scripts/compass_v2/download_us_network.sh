#!/bin/bash  --login

#SBATCH --job-name=compass-net
#SBATCH --time=4:00:00
#SBATCH --nodes=1
#SBATCH --tasks-per-node=1
#SBATCH --cpus-per-task=36
#SBATCH --account=aumc
#SBATCH --mem=120G
#SBATCH --mail-user=Robert.Fitzgerald@nrel.gov
#SBATCH --mail-type=ALL

# this should be a conda environment that includes the libraries
# described in the README.md file in this directory
CONDA_ENVIRONMENT="${CONDA_ENV:-/home/rfitzger/envs/hive-distributed}" 

module purge
. activate "$CONDA_ENVIRONMENT"

python download_us_network_v2.py