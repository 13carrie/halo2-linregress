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
    pub x: Vec<Vec<f64>>, // Matrix of independent variables
    pub y: Vec<f64>,      // Dependent variable
    pub coefficients: Vec<f64>, // Coefficients including intercept
}

fn multiple_linear_regression_circuit<F: BigPrimeField>(
    builder: &mut BaseCircuitBuilder<F>,
    input: CircuitInput,
    _make_public: &mut Vec<AssignedValue<F>>,
) where F: BigPrimeField {
    // steps based on https://www.statology.org/multiple-linear-regression-by-hand/
    const PRECISION: u32 = 63;
    let fixed_point_chip = FixedPointChip::<F, PRECISION>::default(builder);
    let ctx = builder.main(0);

    // 1. load inputs
    let x_values_decimal: Vec<Vec<f64>> = input.x;
    let x_values: Vec<Vec<_>> = x_values_decimal.iter()
    .map(|inner_vec| inner_vec.iter().map(|&val| fixed_point_chip.quantization(val)).collect()).collect();

    let y_values_decimal: Vec<f64> = input.y;
    let y_values: Vec<_> = y_values_decimal.iter().map(|&val| fixed_point_chip.quantization(val)).collect();

    let coefficients: Vec<f64> = input.coefficients;


    // 2. calculate sum((x1)^2), sum((x2)^2), sum(x1*y), sum(x2*y) and sum(x1*x2)
    let x1 = &x_values[0];
    let x2 = &x_values[1];

    let sum_x1: AssignedValue<F> = fixed_point_chip.qsum(ctx, x1.iter().map(|&val| QuantumCell::Witness(val)));
    let sum_x2: AssignedValue<F> = fixed_point_chip.qsum(ctx, x2.iter().map(|&val| QuantumCell::Witness(val)));
    let sum_y: AssignedValue<F> = fixed_point_chip.qsum(ctx, y_values.iter().map(|&val| QuantumCell::Witness(val)));

    let sum_x1squared = fixed_point_chip.inner_product(
        ctx,
        x1.iter().map(|&val| QuantumCell::Witness(val)),
        x1.iter().map(|&val| QuantumCell::Witness(val)),
    );

    let sum_x2squared = fixed_point_chip.inner_product(
        ctx,
        x2.iter().map(|&val| QuantumCell::Witness(val)),
        x2.iter().map(|&val| QuantumCell::Witness(val)),
    );

    let sum_x1y = fixed_point_chip.inner_product(
        ctx,
        x1.iter().map(|&val| QuantumCell::Witness(val)),
        y_values.iter().map(|&val| QuantumCell::Witness(val)),
    );

    let sum_x2y = fixed_point_chip.inner_product(
        ctx,
        x2.iter().map(|&val| QuantumCell::Witness(val)),
        y_values.iter().map(|&val| QuantumCell::Witness(val)),
    );

    let sum_x1x2 = fixed_point_chip.inner_product(
        ctx,
        x1.iter().map(|&val| QuantumCell::Witness(val)),
        x2.iter().map(|&val| QuantumCell::Witness(val)),
    );

    // 3. calculate regression sums
    // regression sum x1: (sum_x1squared - (sum_x1)^2/n)
    let sqrd_sum_x1 = fixed_point_chip.qmul(ctx, sum_x1, sum_x1);
    let length = x_values_decimal[0].len();
    let n = QuantumCell::Witness(fixed_point_chip.quantization(length as f64));
    let sqrd_sum_x1_div_n = fixed_point_chip.qdiv(ctx, sqrd_sum_x1, n);
    let regression_sum_x1 = fixed_point_chip.qsub(ctx, sum_x1squared, sqrd_sum_x1_div_n);

    // regression sum x2: (sum_x2squared - (sum_x2)^2/n)
    let sqrd_sum_x2 = fixed_point_chip.qmul(ctx, sum_x2, sum_x2);
    let sqrd_sum_x2_div_n = fixed_point_chip.qdiv(ctx, sqrd_sum_x2, n);
    let regression_sum_x2 = fixed_point_chip.qsub(ctx, sum_x2squared, sqrd_sum_x2_div_n);

    // regression sum x1y: (sum_x1y - (sum_x1*sum_y)/n)
    let sum_x1_mul_sum_y = fixed_point_chip.qmul(ctx, sum_x1, sum_y);
    let sum_x1_mul_sum_y_div_n = fixed_point_chip.qdiv(ctx, sum_x1_mul_sum_y, n);
    let regression_sum_x1y = fixed_point_chip.qsub(ctx, sum_x1y, sum_x1_mul_sum_y_div_n);

    // regression sum x2y: (sum_x2y - (sum_x2*sum_y)/n)
    let sum_x2_mul_sum_y = fixed_point_chip.qmul(ctx, sum_x2, sum_y);
    let sum_x2_mul_sum_y_div_n = fixed_point_chip.qdiv(ctx, sum_x2_mul_sum_y, n);
    let regression_sum_x2y = fixed_point_chip.qsub(ctx, sum_x2y, sum_x2_mul_sum_y_div_n);

    // regression sum x1x2: (sum_x1x2 - (sum_x1*sum_x2)/n)
    let sum_x1_mul_sum_x2 = fixed_point_chip.qmul(ctx, sum_x1, sum_x2);
    let sum_x1_mul_sum_x2_div_n = fixed_point_chip.qdiv(ctx, sum_x1_mul_sum_x2, n);
    let regression_sum_x1x2 = fixed_point_chip.qsub(ctx, sum_x1x2, sum_x1_mul_sum_x2_div_n);

    // 4. calculate b0 (intercept), b1 (coefficient of x1), b2 (coefficient of x2)
    let regression_sum_x2_sum_x1y = fixed_point_chip.qmul(ctx, regression_sum_x2, regression_sum_x1y);
    let regression_sum_x1x2_sum_x2y = fixed_point_chip.qmul(ctx, regression_sum_x1x2, regression_sum_x2y);
    let regression_sum_x1_sum_x2 = fixed_point_chip.qmul(ctx, regression_sum_x1, regression_sum_x2);
    let sqrd_regression_sum_x1x2 = fixed_point_chip.qmul(ctx, regression_sum_x1x2, regression_sum_x1x2);

    let b1_numerator = fixed_point_chip.qsub(ctx, regression_sum_x2_sum_x1y, regression_sum_x1x2_sum_x2y);
    let denominator = fixed_point_chip.qsub(ctx, regression_sum_x1_sum_x2, sqrd_regression_sum_x1x2);
    let b1 = fixed_point_chip.qdiv(ctx, b1_numerator, denominator);

    let regression_sum_x1_sum_x2y = fixed_point_chip.qmul(ctx, regression_sum_x1, regression_sum_x2y);
    let regression_sum_x1x2_sum_x1y = fixed_point_chip.qmul(ctx, regression_sum_x1x2, regression_sum_x1y);
    let b2_numerator = fixed_point_chip.qsub(ctx, regression_sum_x1_sum_x2y, regression_sum_x1x2_sum_x1y);
    let b2 = fixed_point_chip.qdiv(ctx, b2_numerator, denominator);

    let x1_mean = fixed_point_chip.qdiv(ctx, sum_x1, n);
    let x2_mean = fixed_point_chip.qdiv(ctx, sum_x2, n);
    let y_mean = fixed_point_chip.qdiv(ctx, sum_y, n);

    let b1_x1_mean = fixed_point_chip.qmul(ctx, b1, x1_mean);
    let b2_x2_mean = fixed_point_chip.qmul(ctx, b2, x2_mean);
    let y_mean_minus_b1_x1_mean = fixed_point_chip.qsub(ctx, y_mean, b1_x1_mean);
    let b0 = fixed_point_chip.qsub(ctx, y_mean_minus_b1_x1_mean, b2_x2_mean);

    // 5. compare provided coefficients (in vector) with b0, b1, b2
    let error_rate = 0.1;
    let b0_decimal = fixed_point_chip.dequantization(*b0.value());
    println!("b0 (intercept): {:?}", b0_decimal);
    let b1_decimal = fixed_point_chip.dequantization(*b1.value());
    println!("b1: {:?}", b1_decimal);
    let b2_decimal = fixed_point_chip.dequantization(*b2.value());
    println!("b2: {:?}", b2_decimal);
    
    assert!((&coefficients[0] - b0_decimal).abs() <= error_rate);
    assert!((&coefficients[1] - b1_decimal).abs() <= error_rate);
    assert!((&coefficients[2] - b2_decimal).abs() <= error_rate);
}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    let now = Instant::now();
    run(multiple_linear_regression_circuit, args);
    
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}