use clap::Parser;
use halo2_base::gates::circuit::builder::BaseCircuitBuilder;
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
    _make_public: &mut Vec<AssignedValue<F>>,
) where F: BigPrimeField {
    const PRECISION: u32 = 63;
    let fixed_point_chip = FixedPointChip::<F, PRECISION>::default(builder);
    let ctx = builder.main(0);


    // 1. load inputs
    let x_values_decimal: Vec<f64> = input.x;
    let x_values: Vec<_> = x_values_decimal.iter().map(|&val| fixed_point_chip.quantization(val)).collect();
    println!("x: {:?}", x_values_decimal);

    let y_values_decimal: Vec<f64> = input.y;
    let y_values: Vec<_> = y_values_decimal.iter().map(|&val| fixed_point_chip.quantization(val)).collect();
    println!("y: {:?}", x_values_decimal);


    let a_decimal = input.a;
    let b_decimal = input.b;

    println!("a: {:?}, b: {:?}", a_decimal, b_decimal);


    // 2. compute sums (x, y, xy, x^2)

    let sum_x: AssignedValue<F> = fixed_point_chip.qsum(ctx, x_values.iter().map(|&val| QuantumCell::Witness(val)));
    println!("sum x: {:?}", fixed_point_chip.dequantization(*sum_x.value()));

    let sum_y: AssignedValue<F> = fixed_point_chip.qsum(ctx, y_values.iter().map(|&val| QuantumCell::Witness(val)));
    println!("sum y: {:?}", fixed_point_chip.dequantization(*sum_y.value()));
    let sum_xy = fixed_point_chip.inner_product(
        ctx,
        x_values.iter().map(|&val| QuantumCell::Witness(val)),
        y_values.iter().map(|&val| QuantumCell::Witness(val)),
    );
    println!("sum xy: {:?}", fixed_point_chip.dequantization(*sum_xy.value()));
    let sum_xsquared = fixed_point_chip.inner_product(
        ctx,
        x_values.iter().map(|&val| QuantumCell::Witness(val)),
        x_values.iter().map(|&val| QuantumCell::Witness(val)),
    );
    println!("sum x2: {:?}", fixed_point_chip.dequantization(*sum_xsquared.value()));


    // 3. calculate slope and intercept using zkfixedpointchip

    let sum_y_sum_x2 = fixed_point_chip.qmul(ctx, sum_y, sum_xsquared);
    println!("sum y * sum xsquared: {:?}", fixed_point_chip.dequantization(*sum_y_sum_x2.value()));
    let sum_x_sum_xy = fixed_point_chip.qmul(ctx, sum_x, sum_xy);
    println!("sum x * sum xy: {:?}", fixed_point_chip.dequantization(*sum_x_sum_xy.value()));
    let sqrd_sum_x = fixed_point_chip.qmul(ctx, sum_x, sum_x);
    println!("sqrd_sum_x: {:?}", fixed_point_chip.dequantization(*sqrd_sum_x.value()));


    let intercept_numerator = fixed_point_chip.qsub(ctx, sum_y_sum_x2, sum_x_sum_xy);
    println!("intercept_numerator: {:?}", fixed_point_chip.dequantization(*intercept_numerator.value()));

    let intercept_denominator = fixed_point_chip.qsub(ctx, sum_xsquared, sqrd_sum_x);
    println!("intercept_denominator: {:?}", fixed_point_chip.dequantization(*intercept_denominator.value()));

    let intercept = fixed_point_chip.qdiv(ctx, intercept_numerator, intercept_denominator);
    println!("intercept: {:?}", fixed_point_chip.dequantization(*intercept.value()));



    let n = QuantumCell::Witness(F::from(x_values_decimal.len() as u64));
    let n_sum_xy = fixed_point_chip.qmul(ctx, n, sum_xy);
    let sum_x_sum_y = fixed_point_chip.qmul(ctx, sum_x, sum_y);
    let n_sum_x2 = fixed_point_chip.qmul(ctx, n, sum_xsquared);
    let slope_numerator = fixed_point_chip.qmul(ctx, n_sum_xy, sum_x_sum_y);
    let slope_denominator = fixed_point_chip.qmul(ctx, n_sum_x2, sqrd_sum_x);
    let slope = fixed_point_chip.qdiv(ctx, slope_numerator, slope_denominator);

    // 4. compare a and b with slope and intercept
    let error_rate = 0.3;
    let slope_decimal = fixed_point_chip.dequantization(*slope.value());
    let intercept_decimal = fixed_point_chip.dequantization(*intercept.value());
    assert!((slope_decimal - b_decimal).abs() <= error_rate);
    assert!((intercept_decimal - a_decimal).abs() <= error_rate);
}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    run(linear_regression_circuit, args);
}
