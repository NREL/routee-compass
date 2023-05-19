#!/bin/bash  --login

#SBATCH --job-name=build_rust_map
#SBATCH --time=6:00:00
#SBATCH --nodes=1
#SBATCH --tasks-per-node=1
#SBATCH --cpus-per-task=36
#SBATCH --account=aumc
#SBATCH --mem=200G
#SBATCH --mail-user=Nicholas.Reinicke@nrel.gov
#SBATCH --mail-type=ALL

module purge
. activate /home/$USER/.conda/envs/routee-compass

export RESTRICTION_FILE = /projects/mbap/amazon-eco/restrictions.pickle
export NETWORK_PATH = /projects/mbap/amazon-eco/us_network/
export OUTPUT_FOLDER = /projects/mbap/amazon-eco/

python build_rust_map.py