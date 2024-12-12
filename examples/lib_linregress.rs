use clap::Parser;
use halo2_base::gates::circuit::builder::BaseCircuitBuilder;
use halo2_base::gates::RangeChip;
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

    let sum_x: AssignedValue<F> = fixed_point_chip.qsum(ctx, x_values.iter().map(|&val| QuantumCell::Witness(val)));
    let sum_y: AssignedValue<F> = fixed_point_chip.qsum(ctx, y_values.iter().map(|&val| QuantumCell::Witness(val)));
    let sum_xy = fixed_point_chip.inner_product(
        ctx,
        x_values.iter().map(|&val| QuantumCell::Witness(val)),
        y_values.iter().map(|&val| QuantumCell::Witness(val)),
    );
    let sum_x2 = fixed_point_chip.inner_product(
        ctx,
        x_values.iter().map(|&val| QuantumCell::Witness(val)),
        x_values.iter().map(|&val| QuantumCell::Witness(val)),
    );

    /*
    let sum_x: AssignedValue<F> = fixed_point_chip.qsum(ctx, x_quantum_cells);
    let sum_y: AssignedValue<F> = fixed_point_chip.qsum(ctx, y_quantum_cells);
    let sum_xy = fixed_point_chip.inner_product(ctx, x_quantum_cells_2, y_quantum_cells_2);
    */

    let a = ctx.load_witness(a);
    let b = ctx.load_witness(b);


    // 3. calculate slope and intercept using zkfixedpointchip
    let two = Constant(F::from(2));

    let sum_y_sum_x2 = fixed_point_chip.qmul(ctx, sum_y, sum_x2);
    let sum_x_sum_xy = fixed_point_chip.qmul(ctx, sum_x, sum_xy);
    let sqrd_sum_x = fixed_point_chip.qpow(ctx, sum_x, two);

    let intercept_numerator = fixed_point_chip.qsub(ctx, sum_y_sum_x2, sum_x_sum_xy);
    let intercept_denominator = fixed_point_chip.qsub(ctx, sum_x2, sqrd_sum_x);
    let intercept = fixed_point_chip.qdiv(ctx, intercept_numerator, intercept_denominator);


    let n = Constant(F::from(x_values_decimal.len() as u64));
    let n_sum_xy = fixed_point_chip.qmul(ctx, n, sum_xy);
    let sum_x_sum_y = fixed_point_chip.qmul(ctx, sum_x, sum_y);
    let n_sum_x2 = fixed_point_chip.qmul(ctx, n, sum_x2);
    let slope_numerator = fixed_point_chip.qmul(ctx, n_sum_xy, sum_x_sum_y);
    let slope_denominator = fixed_point_chip.qmul(ctx, n_sum_x2, sqrd_sum_x);
    let slope = fixed_point_chip.qdiv(ctx, slope_numerator, slope_denominator);


    // 4. compare a and b with slope and intercept
    // assert_eq!(slope, b);
    // assert_eq!(intercept, a);
}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    // run different zk commands based on the command line arguments
    run(linear_regression_circuit, args);
}
