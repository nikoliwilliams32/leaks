import networkx as nx
import time
import random
from leaks import rust_max_flow


def create_random_flow_network(
    n_nodes, edge_density=0.3, min_capacity=1, max_capacity=100
):
    """Create a random flow network with n nodes and random edge weights."""
    G = nx.DiGraph()

    # Add nodes
    nodes = list(range(n_nodes))
    G.add_nodes_from(nodes)

    # Add random edges
    for i in nodes:
        for j in nodes:
            if i != j and random.random() < edge_density:
                capacity = random.uniform(min_capacity, max_capacity)
                G.add_edge(i, j, capacity=capacity)

    # Ensure source (0) and sink (n_nodes-1) are connected
    if not nx.has_path(G, 0, n_nodes - 1):
        path = list(range(n_nodes))
        for i in range(len(path) - 1):
            G.add_edge(path[i], path[i + 1], capacity=max_capacity)

    return G


def benchmark_max_flow(G, source, sink, num_runs=5):
    """Run both implementations multiple times and return average times."""

    # Warm up run for both implementations
    _ = nx.maximum_flow(G, source, sink)
    _ = rust_max_flow(G, source, sink)

    # NetworkX timing
    nx_times = []
    for _ in range(num_runs):
        start = time.perf_counter()
        flow_value_nx, flow_dict_nx = nx.maximum_flow(
            G, source, sink, flow_func=nx.algorithms.flow.edmonds_karp
        )
        nx_times.append(time.perf_counter() - start)
        print(f"timenx run: {nx_times[-1]:.6f}s")

    # Rust implementation timing
    rust_times = []
    for _ in range(num_runs):
        start = time.perf_counter()
        flow_value_rust, flow_dict_rust = rust_max_flow(G, source, sink)
        rust_times.append(time.perf_counter() - start)
        print(f"timerust run: {rust_times[-1]:.6f}s")

    return {
        "networkx": {
            "avg_time": sum(nx_times) / len(nx_times),
            "flow_value": flow_value_nx,
        },
        "rust": {
            "avg_time": sum(rust_times) / len(rust_times),
            "flow_value": flow_value_rust,
        },
    }


def run_benchmarks():
    """Run benchmarks on different graph sizes."""
    graph_sizes = [10, 50, 100, 500, 1000]
    num_runs = 5

    print(f"Running benchmarks ({num_runs} runs per size)...")
    print("\nGraph Size | NetworkX Time | Rust Time    | Speedup  | Flow Value")
    print("-" * 65)

    for size in graph_sizes:
        # Create a random graph
        G = create_random_flow_network(size)
        source, sink = 0, size - 1

        # Run benchmark
        results = benchmark_max_flow(G, source, sink, num_runs)

        # Calculate speedup
        speedup = results["networkx"]["avg_time"] / results["rust"]["avg_time"]

        # Print results
        print(
            f"{size:>9} | {results['networkx']['avg_time']:>12.6f}s | "
            f"{results['rust']['avg_time']:>11.6f}s | {speedup:>8.2f}x | "
            f"{results['rust']['flow_value']:>10.2f}"
        )

        # Verify results match
        assert (
            abs(results["networkx"]["flow_value"] - results["rust"]["flow_value"])
            < 1e-6
        ), f"Flow values don't match for size {size}"


if __name__ == "__main__":
    print("Maximum Flow Algorithm Benchmark")
    print("Comparing NetworkX vs Rust Implementation")
    print("=" * 50)
    run_benchmarks()
