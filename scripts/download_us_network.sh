#!/bin/bash  --login

#SBATCH --job-name=download_us_network
#SBATCH --time=4:00:00
#SBATCH --nodes=1
#SBATCH --tasks-per-node=1
#SBATCH --cpus-per-task=36
#SBATCH --account=aumc
#SBATCH --mem=120G
#SBATCH --mail-user=Nicholas.Reinicke@nrel.gov
#SBATCH --mail-type=ALL

module purge
. activate /home/$USER/.conda/envs/routee-compass

python download_us_network.py