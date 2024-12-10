use clap::Parser;
use halo2_base::gates::circuit::builder::BaseCircuitBuilder;
use halo2_base::gates::{GateChip, GateInstructions};
use halo2_graph::gadget::fixed_point::{FixedPointChip, FixedPointInstructions};
use halo2_base::utils::{ScalarField, BigPrimeField};
use halo2_base::AssignedValue;
#[allow(unused_imports)]
use halo2_base::{
    Context,
    QuantumCell::{Constant, Existing, Witness},
};
use halo2_graph::scaffold::cmd::Cli;
use halo2_graph::scaffold::run;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    pub x: Vec<f64>, // Vector of field elements as strings
    pub y: Vec<f64>,
    pub a: f64, // Intercept
    pub b: f64, // Slope
}

// this algorithm takes a public input x, computes x^2 + 72, and outputs the result as public output
fn linear_regression_circuit<F: ScalarField>(
    builder: &mut BaseCircuitBuilder<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,
) where  F: BigPrimeField {
    const PRECISION: u32 = 63;
    let ctx = builder.main(0);
    let fixed_point_chip = FixedPointChip::<F, PRECISION>::default(builder);

    // 1. load inputs
    let x_values_decimal: Vec<_> = input.x;
    // let x_values = for (val in x_values_decimal fixed_point_chip.quantization(val))
    let x_values = fixed_point_chip.quantization(input.x);
    let y_values_decimal: Vec<_> = input.y;
    // let y_values = for (val in x_values_decimal fixed_point_chip.quantization(val))
    let x_values = fixed_point_chip.quantization(input.y);

    let a_decimal = input.a;
    let a = fixed_point_chip.quantization(input.a);
    let b_decimal = input.b;
    let b = fixed_point_chip.quantization(input.b);
    println!("a: {:?}, a_decimal: {:?}, b: {:?}, b_decimal: {:?}", a, a_decimal, b, b_decimal);
    

    // let gate = GateChip::<F>::default();

    // 2. compute sums (x, y, xy, x^2)
    // use zkfixedpointsum?
    

    // 3. calculate slope and intercept using zkfixedpointchip

    // 4. compare a and b with slope and intercept
}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    // run different zk commands based on the command line arguments
    run(linear_regression_circuit, args);
}
