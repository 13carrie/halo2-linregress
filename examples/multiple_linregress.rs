use clap::Parser;
use halo2_base::gates::circuit::builder::BaseCircuitBuilder;
// use halo2_base::gates::RangeChip;
use halo2_graph::gadget::fixed_point::{FixedPointChip, FixedPointInstructions};
use halo2_base::utils::BigPrimeField;
use halo2_base::AssignedValue;
use halo2_base::{QuantumCell, QuantumCell::Constant};


#[allow(unused_imports)]
use halo2_graph::scaffold::cmd::Cli;
use halo2_graph::scaffold::run;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    pub x: Vec<Vec<f64>>, // Matrix of independent variables
    pub y: Vec<f64>,      // Dependent variable
    pub coefficients: Vec<f64>, // Coefficients including intercept
}

fn multiple_linear_regression_circuit<F: BigPrimeField>(
    builder: &mut BaseCircuitBuilder<F>,
    input: CircuitInput,
    _make_public: &mut Vec<AssignedValue<F>>,
) where F: BigPrimeField {
    const PRECISION: u32 = 63;
    let fixed_point_chip = FixedPointChip::<F, PRECISION>::default(builder);
    let ctx = builder.main(0);

    // Load x values (independent variables)
    let x_values: Vec<Vec<_>> = input.x.iter()
        .map(|row| row.iter().map(|&val| fixed_point_chip.quantization(val)).collect())
        .collect();

    // Load y values (dependent variable)
    let y_values: Vec<_> = input.y.iter().map(|&val| fixed_point_chip.quantization(val)).collect();
}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    run(multiple_linear_regression_circuit, args);
}