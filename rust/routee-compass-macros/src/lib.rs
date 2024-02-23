extern crate proc_macro;
extern crate proc_macro_error;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_error]
#[proc_macro_attribute]
pub fn pybindings(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let name = &input.ident;

    let expanded = quote! {
        use pyo3::{exceptions::PyException, prelude::*, types::PyType, PyResult};

        #[pyclass]
        #input

        #[pymethods]
        impl #name {
            fn graph_edge_origin(&self, edge_id: usize) -> PyResult<usize> {
                CompassAppBindings::graph_edge_origin(self, edge_id).map_err(|e| {
                    PyException::new_err(format!(
                        "error retrieving edge origin for edge_id {}: {}",
                        edge_id, e
                    ))
                })
            }
            fn graph_edge_destination(&self, edge_id: usize) -> PyResult<usize> {
                CompassAppBindings::graph_edge_destination(self, edge_id).map_err(|e| {
                    PyException::new_err(format!(
                        "error retrieving edge destination for edge_id {}: {}",
                        edge_id, e
                    ))
                })
            }
            fn graph_edge_distance(&self, edge_id: usize, distance_unit: Option<String>) -> PyResult<f64> {
                CompassAppBindings::graph_edge_distance(self, edge_id, distance_unit).map_err(|e| {
                    PyException::new_err(format!(
                        "error retrieving edge distance for edge_id {}: {}",
                        edge_id, e
                    ))
                })
            }
            fn graph_get_out_edge_ids(&self, vertex_id: usize) -> PyResult<Vec<usize>> {
                CompassAppBindings::graph_get_out_edge_ids(self, vertex_id).map_err(|e| {
                    PyException::new_err(format!(
                        "error retrieving out edge ids for vertex_id {}: {}",
                        vertex_id, e
                    ))
                })
            }
            fn graph_get_in_edge_ids(&self, vertex_id: usize) -> PyResult<Vec<usize>> {
                CompassAppBindings::graph_get_in_edge_ids(self, vertex_id).map_err(|e| {
                    PyException::new_err(format!(
                        "error retrieving in edge ids for vertex_id {}: {}",
                        vertex_id, e
                    ))
                })
            }
            #[classmethod]
            pub fn _from_config_toml_string(
                _cls: &PyType,
                config_string: String,
                original_file_path: String,
            ) -> PyResult<#name> {
                CompassAppBindings::from_config_toml_string(config_string, original_file_path).map_err(
                    |e| {
                        PyException::new_err(format!(
                            "Error while creating CompassApp from config toml string: {}",
                            e
                        ))
                    },
                )
            }

            pub fn _run_queries(
                &self,
                queries: Vec<String>,
                config: Option<String>,
            ) -> PyResult<Vec<String>> {
                CompassAppBindings::run_queries(self, queries, config)
                    .map_err(|e| PyException::new_err(format!("Error while running queries: {}", e)))
            }
        }
    };

    TokenStream::from(expanded)
}
