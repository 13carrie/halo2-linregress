use clap::Parser;
use halo2_base::gates::circuit::builder::BaseCircuitBuilder;
use halo2_base::gates::RangeChip;
use halo2_graph::gadget::fixed_point::{FixedPointChip, FixedPointInstructions};
use halo2_base::utils::BigPrimeField;
use halo2_base::AssignedValue;
use halo2_base::{QuantumCell};


#[allow(unused_imports)]
use halo2_graph::scaffold::cmd::Cli;
use halo2_graph::scaffold::run;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub a: f64, // intercept
    pub b: f64, // slope
}

fn linear_regression_circuit<F: BigPrimeField>(
    builder: &mut BaseCircuitBuilder<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,
) where F: BigPrimeField {
    const PRECISION: u32 = 63;
    let fixed_point_chip = FixedPointChip::<F, PRECISION>::default(builder);
    let ctx = builder.main(0);
    let range: &RangeChip<F> = fixed_point_chip.range_gate();

    // 1. load inputs
    let x_values_decimal: Vec<_> = input.x;
    let x_values: Vec<_> = x_values_decimal.iter().map(|&val| fixed_point_chip.quantization(val)).collect();
    let y_values_decimal: Vec<_> = input.y;
    let y_values: Vec<_> = y_values_decimal.iter().map(|&val| fixed_point_chip.quantization(val)).collect();

    // Convert field elements to QuantumCell<F>
    let x_quantum_cells: Vec<_> = x_values.iter().map(|&val| QuantumCell::Witness(val)).collect();
    let y_quantum_cells: Vec<_> = y_values.iter().map(|&val| QuantumCell::Witness(val)).collect();

    let a_decimal = input.a;
    let a = fixed_point_chip.quantization(input.a);
    let b_decimal = input.b;
    let b = fixed_point_chip.quantization(input.b);
    println!("a: {:?}, a_decimal: {:?}, b: {:?}, b_decimal: {:?}", a, a_decimal, b, b_decimal);


    // 2. compute sums (x, y, xy, x^2)
    let n = x_values_decimal.len();
    println!("n: {:?}", n);

    let sum_x: AssignedValue<F> = fixed_point_chip.qsum(ctx, x_quantum_cells);
    let sum_y: AssignedValue<F> = fixed_point_chip.qsum(ctx, y_quantum_cells);
    // let sum_xy = fixed_point_chip.inner_product(ctx, x_quantum_cells, y_quantum_cells);

    let a = ctx.load_witness(a);
    let b = ctx.load_witness(b);


    // 3. calculate slope and intercept using zkfixedpointchip


    // 4. compare a and b with slope and intercept
    // gate.assert_equal(ctx, slope_assigned, b_assigned);
    // gate.assert_equal(ctx, intercept_assigned, a_assigned);
}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    // run different zk commands based on the command line arguments
    run(linear_regression_circuit, args);
}
