use std::time::Instant;

use clap::Parser;
use halo2_base::gates::circuit::builder::BaseCircuitBuilder;
use halo2_graph::gadget::fixed_point::{FixedPointChip, FixedPointInstructions};
use halo2_base::utils::BigPrimeField;
use halo2_base::AssignedValue;
use halo2_base::QuantumCell;


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
    _make_public: &mut Vec<AssignedValue<F>>,
) where F: BigPrimeField {
    const PRECISION: u32 = 63;
    let fixed_point_chip = FixedPointChip::<F, PRECISION>::default(builder);
    let ctx = builder.main(0);


    // 1. load inputs
    let x_values_decimal: Vec<f64> = input.x;
    let x_values: Vec<_> = x_values_decimal.iter().map(|&val| fixed_point_chip.quantization(val)).collect();
    let y_values_decimal: Vec<f64> = input.y;
    let y_values: Vec<_> = y_values_decimal.iter().map(|&val| fixed_point_chip.quantization(val)).collect();
    let a_decimal = input.a;
    let b_decimal = input.b;

    // 2. compute sums (x, y, xy, x^2)

    let sum_x: AssignedValue<F> = fixed_point_chip.qsum(ctx, x_values.iter().map(|&val| QuantumCell::Witness(val)));
    let sum_y: AssignedValue<F> = fixed_point_chip.qsum(ctx, y_values.iter().map(|&val| QuantumCell::Witness(val)));
    let sum_xy = fixed_point_chip.inner_product(
        ctx,
        x_values.iter().map(|&val| QuantumCell::Witness(val)),
        y_values.iter().map(|&val| QuantumCell::Witness(val)),
    );
    let sum_xsquared = fixed_point_chip.inner_product(
        ctx,
        x_values.iter().map(|&val| QuantumCell::Witness(val)),
        x_values.iter().map(|&val| QuantumCell::Witness(val)),
    );


    // 3. calculate slope and intercept using zkfixedpointchip
    let length = x_values_decimal.len();
    let n = QuantumCell::Witness(fixed_point_chip.quantization(length as f64));

    let sum_y_sum_x2 = fixed_point_chip.qmul(ctx, sum_y, sum_xsquared);
    let sum_x_sum_xy = fixed_point_chip.qmul(ctx, sum_x, sum_xy);
    let intercept_numerator = fixed_point_chip.qsub(ctx, sum_y_sum_x2, sum_x_sum_xy);
   
    let n_sum_x2 = fixed_point_chip.qmul(ctx, n, sum_xsquared);
    let sqrd_sum_x = fixed_point_chip.qmul(ctx, sum_x, sum_x);
    let denominator = fixed_point_chip.qsub(ctx, n_sum_x2, sqrd_sum_x);
    let intercept = fixed_point_chip.qdiv(ctx, intercept_numerator, denominator);

    let n_sum_xy = fixed_point_chip.qmul(ctx, n, sum_xy);
    let sum_x_sum_y = fixed_point_chip.qmul(ctx, sum_x, sum_y);
    let slope_numerator = fixed_point_chip.qsub(ctx, n_sum_xy, sum_x_sum_y);
    let slope = fixed_point_chip.qdiv(ctx, slope_numerator, denominator);

    // 4. compare a and b with slope and intercept
    let error_rate = 0.1;
    let slope_decimal = fixed_point_chip.dequantization(*slope.value());
    println!("slope: {:?}", slope_decimal);
    let intercept_decimal = fixed_point_chip.dequantization(*intercept.value());
    println!("intercept: {:?}", intercept_decimal);

    assert!((slope_decimal - b_decimal).abs() <= error_rate);
    assert!((intercept_decimal - a_decimal).abs() <= error_rate);
}

fn main() {
    env_logger::init();

    let args = Cli::parse();
    
    let now = Instant::now();
    run(linear_regression_circuit, args);

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}
