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

python build_rust_map.py