use petgraph::algo::ford_fulkerson;
use petgraph::graph::{DiGraph, NodeIndex};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;

/// Implements the max-flow algorithm in Rust, taking a NetworkX graph as input.
#[pyfunction]
fn rust_max_flow(
    py: Python<'_>,
    graph: PyObject,
    source: PyObject,
    sink: PyObject,
) -> PyResult<(f64, PyObject)> {
    // Create the Rust graph and node mapping
    let mut rust_graph = DiGraph::<(), f64>::new();
    let mut node_map = HashMap::new();
    let mut node_indices = HashMap::new();

    // Convert graph to PyAny and get edges
    let graph = graph.as_ref(py);
    let edges_method = graph.getattr("edges")?;
    let edges = edges_method.call_method0("__call__")?;
    let data_method = edges.getattr("data")?;
    let edges_with_data = data_method.call1(("capacity",))?;

    // First pass: collect all nodes and create indices
    for edge_result in edges_with_data.iter()? {
        let edge = edge_result?;

        let u_py = edge.get_item(0)?.extract::<&PyAny>()?;
        let v_py = edge.get_item(1)?.extract::<&PyAny>()?;

        let u_hash = u_py.hash()?;
        let v_hash = v_py.hash()?;

        if !node_indices.contains_key(&u_hash) {
            let index = rust_graph.add_node(());
            node_indices.insert(u_hash, index);
            node_map.insert(u_hash, u_py.to_object(py));
        }

        if !node_indices.contains_key(&v_hash) {
            let index = rust_graph.add_node(());
            node_indices.insert(v_hash, index);
            node_map.insert(v_hash, v_py.to_object(py));
        }
    }

    // Second pass: add all edges
    for edge_result in edges_with_data.iter()? {
        let edge = edge_result?;

        let u_py = edge.get_item(0)?.extract::<&PyAny>()?;
        let v_py = edge.get_item(1)?.extract::<&PyAny>()?;
        let capacity_py = edge.get_item(2)?;

        let capacity = if capacity_py.is_none() {
            1.0
        } else {
            capacity_py.extract::<f64>()?
        };

        let u_idx = node_indices[&u_py.hash()?];
        let v_idx = node_indices[&v_py.hash()?];
        rust_graph.add_edge(u_idx, v_idx, capacity);
    }

    // Find source and sink indices
    let source_any = source.as_ref(py);
    let sink_any = sink.as_ref(py);
    let source_hash = source_any.hash()?;
    let sink_hash = sink_any.hash()?;

    let source_idx = node_indices.get(&source_hash).ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>("Source node not in graph")
    })?;

    let sink_idx = node_indices
        .get(&sink_hash)
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("Sink node not in graph"))?;

    // Run max-flow algorithm
    let (total_flow, _) = ford_fulkerson(&rust_graph, *source_idx, *sink_idx);

    // Create and return the flow dictionary
    let result_dict = PyDict::new(py);

    // For now, we're just returning the total flow value in a simple dictionary
    result_dict.set_item("flow_value", total_flow)?;

    Ok((total_flow, result_dict.into()))
}

/// Python module configuration
#[pymodule]
fn leaks(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(rust_max_flow, m)?)?;
    Ok(())
}
