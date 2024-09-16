use clap::Parser;
use halo2_base::utils::{ScalarField, BigPrimeField};
use halo2_base::AssignedValue;
use halo2_base::gates::circuit::builder::BaseCircuitBuilder;
use halo2_base::gates::{GateChip, GateInstructions};
use halo2_base::poseidon::hasher::PoseidonHasher;
use serde::{Serialize, Deserialize, Serializer, Deserializer, ser::SerializeStruct};
#[allow(unused_imports)]
use halo2_graph::scaffold::cmd::Cli;
use halo2_graph::scaffold::run;
use snark_verifier_sdk::halo2::OptimizedPoseidonSpec;

// graph node size
const NODE_SIZE: usize = 10;
// parameters for the Poseidon hash function
const T: usize = 3;
const RATE: usize = 2;
const R_F: usize = 8;
const R_P: usize = 57;

/// Circuit Input Structure
#[derive(Clone, Debug)]
pub struct CircuitInput {
    /// Public Inputs
    pub start_node: u64,
    pub end_node: u64,

    /// Private Witnesses
    pub adj_matrix: [[u64; NODE_SIZE]; NODE_SIZE], // Adjacency matrix
    pub path_nodes: [u64; NODE_SIZE], // Path nodes
    pub s_values: [u64; NODE_SIZE-1], // Continuation indicators
}

// Implement Serialize manually
impl Serialize for CircuitInput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("CircuitInput", 5)?;
        state.serialize_field("start_node", &self.start_node)?;
        state.serialize_field("end_node", &self.end_node)?;

        // Serialize arrays by converting them to Vecs
        state.serialize_field(
            "adj_matrix", 
            &self.adj_matrix.iter().map(|row| row.to_vec()).collect::<Vec<_>>()
        )?;
        state.serialize_field("path_nodes", &self.path_nodes.to_vec())?;
        state.serialize_field("s_values", &self.s_values.to_vec())?;
        state.end()
    }
}

// Implement Deserialize manually
impl<'de> Deserialize<'de> for CircuitInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct CircuitInputHelper {
            start_node: u64,
            end_node: u64,
            adj_matrix: Vec<Vec<u64>>,
            path_nodes: Vec<u64>,
            s_values: Vec<u64>,
        }

        let helper = CircuitInputHelper::deserialize(deserializer)?;

        // Convert Vecs back into fixed-size arrays
        let mut adj_matrix = [[0u64; NODE_SIZE]; NODE_SIZE];
        for (i, row) in helper.adj_matrix.into_iter().enumerate() {
            adj_matrix[i].copy_from_slice(&row);
        }

        let mut path_nodes = [0u64; NODE_SIZE];
        path_nodes.copy_from_slice(&helper.path_nodes);

        let mut s_values = [0u64; NODE_SIZE-1];
        s_values.copy_from_slice(&helper.s_values);

        Ok(CircuitInput {
            start_node: helper.start_node,
            end_node: helper.end_node,
            adj_matrix,
            path_nodes,
            s_values,
        })
    }
}

/// Node Connectivity Circuit
/// This circuit checks if there is a path between two nodes in a graph
/// The graph is represented by a adjacency matrix
/// The path is represented by a sequence of nodes
/// The continuation indicators are used to check if the path is valid

fn node_connectivity<F: ScalarField>(
    builder: &mut BaseCircuitBuilder<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,
) where  F: BigPrimeField {
    let ctx = builder.main(0);

    // Load public inputs

    let start_node = ctx.load_witness(F::from(input.start_node));
    let end_node = ctx.load_witness(F::from(input.end_node));
    make_public.push(end_node);
    make_public.push(start_node);

    // Load private witnesses
    let adj_matrix_witnesses: [[AssignedValue<F>; NODE_SIZE]; NODE_SIZE] = 
        input.adj_matrix
        .map(|row| row.map(|val| ctx.load_witness(F::from(val))));

    let path_nodes_witnesses: [AssignedValue<F>; NODE_SIZE] = 
        input
            .path_nodes
            .map(|val| ctx.load_witness(F::from(val)));

    let s_values_witnesses: [AssignedValue<F>; NODE_SIZE-1] = 
        input
            .s_values
            .map(|val| ctx.load_witness(F::from(val)));

    // Flatten the adjacency matrix witnesses
    let mut flat_adj_mat_witnesses: Vec<AssignedValue<F>> = Vec::with_capacity(NODE_SIZE * NODE_SIZE);

    for row in adj_matrix_witnesses.iter() {
        for &val in row.iter() {
            flat_adj_mat_witnesses.push(val);
        }
    }

    // Convert the Vec to an array
    let flat_adj_mat_witnesses: [AssignedValue<F>; NODE_SIZE * NODE_SIZE] = flat_adj_mat_witnesses.try_into().unwrap();

    // Commit the adjacency matrix
    let gate = GateChip::<F>::default();
    let mut poseidon =
        PoseidonHasher::<F, T, RATE>::new(OptimizedPoseidonSpec::new::<R_F, R_P, 0>());
    poseidon.initialize_consts(ctx, &gate);
    let adj_mat_hash = poseidon.hash_fix_len_array(ctx, &gate, &flat_adj_mat_witnesses);
    make_public.push(adj_mat_hash);

    let correct_start_node = gate.is_equal(ctx, start_node, path_nodes_witnesses[0]);
    
    // Check path validity
    let mut correct_end_node = ctx.load_zero();
    let ctx_constants: Vec<AssignedValue<F>> = (0..NODE_SIZE).map(|i| ctx.load_constant(F::from(i as u64))).collect();

    for i in 0..NODE_SIZE-1 {
        let is_active = gate.is_equal(ctx, s_values_witnesses[i], ctx_constants[1]);

        // Check if the edge exists in the adjacency matrix
        let start_node_index = path_nodes_witnesses[i];
        let end_node_index = path_nodes_witnesses[i+1];
        
        // enumerate the adjacency matrix
        let mut total_valid_edge_count = ctx.load_zero();
        for j in 0..NODE_SIZE {
            for k in 0..NODE_SIZE {
                // let is_this_edge = gate.and(ctx, gate.is_equal(ctx, start_node_index, ctx.load_constant(F::from(j as u64))), gate.is_equal(ctx, end_node_index, ctx.load_constant(F::from(k as u64))));
                // let is_edge_active = gate.is_equal(ctx, adj_matrix_witnesses[j][k], ctx.load_constant(F::from(1u64)));
                // let is_valid_edge = gate.and(ctx, is_this_edge, is_edge_active);
                // total_valid_edge_count = gate.add(ctx, total_valid_edge_count, is_valid_edge);

                let is_this_edge = {
                    let j_constant = ctx.load_constant(F::from(j as u64));
                    let k_constant = ctx.load_constant(F::from(k as u64));
                    let start_eq = gate.is_equal(ctx, start_node_index, j_constant);
                    let end_eq = gate.is_equal(ctx, end_node_index, k_constant);
                    gate.and(ctx, start_eq, end_eq)
                };

                let is_edge_active = {
                    let adj_constant = ctx.load_constant(F::from(1u64));
                    gate.is_equal(ctx, adj_matrix_witnesses[j][k], adj_constant)
                };

                let is_valid_edge = gate.and(ctx, is_this_edge, is_edge_active);
                total_valid_edge_count = gate.add(ctx, total_valid_edge_count, is_valid_edge);
            }
        }

        
        // Check if the edge is valid
        // If the edge is inactive, then it is valid
        // If the edge is active, then it is valid only if it appears in the adjacency matrix
        // total_valid_edge_count means the amount of cells in the adjacency matrix that are equal to 1 and have the same indices as the edge
        let valid_edge = {
            let one_constant = ctx.load_constant(F::from(1u64));
            let is_active_not = gate.not(ctx, is_active);
            let is_valid_edge = gate.is_equal(ctx, total_valid_edge_count, one_constant);
            gate.or(ctx, is_active_not, is_valid_edge)
        };
        
        // let is_last_node = gate.and(ctx, is_active, gate.is_zero(ctx, s_values_witnesses[i+1]));
        let no_continuation;
        if (i+1) > NODE_SIZE-2 {
            no_continuation = ctx.load_zero();
        }
        else {
            no_continuation = gate.is_zero(ctx, s_values_witnesses[i+1]);
        }
        let is_last_node = gate.and(ctx, is_active, no_continuation);

        // Check if the end node is correct
        // The end node is correct if it is the last node in the path and it is the same as the end node
        // and the edge is valid
        correct_end_node = {
            let is_end_node = gate.is_equal(ctx, end_node_index, end_node);
            let is_last_node_and_end_node = gate.and(ctx, is_last_node, is_end_node);
            let is_valid_edge_and_last_node = gate.and(ctx, valid_edge, is_last_node_and_end_node);
            gate.or(ctx, correct_end_node, is_valid_edge_and_last_node)
        }
    }

    let correct_path = gate.and(ctx, correct_start_node, correct_end_node);
    make_public.push(correct_path);
    println!("correct_path: {:?}", correct_path.value());
}

fn main() {
    env_logger::init();
    let args = Cli::parse();
    run(node_connectivity, args);
}