from os import path

from setuptools import setup, find_packages

# Get the long description from the README file
here = path.abspath(path.dirname(__file__))
with open(path.join(here, 'README.md'), encoding='utf-8') as f:
    long_description = f.read()

MBAP_PYPI = "https://github.nrel.gov/pages/MBAP/mbap-pypi/"

setup(
    name="routee-compass",
    version="0.1.0-alpha",
    description=
    "routee compass is a package for producing energy optimal routes",
    long_description=long_description,
    long_description_content_type='text/markdown',
    url="https://github.nrel.gov/MBAP/routee-compass",
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Science/Research",
        "License :: Other/Proprietary License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3.8",
        "Topic :: Scientific/Engineering"
    ],
    packages=find_packages(),
    python_requires=">=3.8",
    install_requires=[
        "pandas",
        "numpy",
        "networkx",
        "scipy",
        "shapely",
        "geopandas",
        "sqlalchemy",
        "psycopg2",
        f"routee-powertrain @ {MBAP_PYPI}routee-powertrain/routee_powertrain-0.4.0a0-py3-none-any.whl"
    ],
    extras_require={
       "optional": [
           "osmnx",
           "requests",
       ]
    },
    include_package_data=True,
    package_data={
        "compass.resources.routee_models": ["*"],
    },
    entry_points={
        'console_scripts': [
            'get-tomtom-network=scripts.get_tomtom_road_network:get_tomtom_network',
            'get-osm-network=scripts.get_osm_road_network:get_osm_network',
        ]
    },
    author="National Renewable Energy Laboratory",
    license="Copyright ©2020 Alliance for Sustainable Energy, LLC All Rights Reserved",
)