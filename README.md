# <img src="docs/images/routeelogo.png" alt="Routee Compass" width="100"/>

<div align="left">
    <img src="https://img.shields.io/badge/python-3.9%20%7C%203.10%20%7C%203.11%20%7C%203.12-blue"/>
  <a href="https://pypi.org/project/nrel.routee.compass/">
    <img src="https://img.shields.io/pypi/v/nrel.routee.compass" alt="PyPi Latest Release"/>
  </a>
  <a href="https://crates.io/crates/routee-compass">
    <img src="https://img.shields.io/crates/v/routee-compass" alt="Crates.io Latest Release"/>
  </a>
</div>

RouteE Compass is an energy-aware routing engine for the RouteE ecosystem of software tools with the following key features:

- Dynamic and extensible search objectives that allow customized blends of distance, time, cost, and energy (via RouteE Powertrain) at query-time
- Core engine written in Rust for improved runtimes, parallel query execution, and the ability to load nation-sized road networks into memory
- Rust, HTTP, and Python APIs for integration into different research pipelines and other software

RouteE Compass is a part of the [RouteE](https://www.nrel.gov/transportation/route-energy-prediction-model.html) family of mobility tools created at the National Renewable Energy Laboratory.

## Installation

See the [installation](https://nrel.github.io/routee-compass/installation.html) guide for installing RouteE Compass

## Usage

See the [documentation](https://nrel.github.io/routee-compass/) for more information.

## Contributors

RouteE Compass is currently maintained by Nick Reinicke ([@nreinicke](https://github.com/nreinicke)) and Rob Fitzgerald ([@robfitzgerald](https://github.com/robfitzgerald)).

## License

Copyright 2023 Alliance for Sustainable Energy, LLC

Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.

3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote products derived from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
