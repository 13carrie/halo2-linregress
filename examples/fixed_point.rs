use clap::Parser;
use halo2_base::utils::{ScalarField, BigPrimeField};
use halo2_base::AssignedValue;
use halo2_graph::gadget::fixed_point::{FixedPointChip, FixedPointInstructions};
use halo2_base::gates::circuit::builder::BaseCircuitBuilder;
use serde::{Serialize, Deserialize};
#[allow(unused_imports)]
use halo2_graph::scaffold::cmd::Cli;
use halo2_graph::scaffold::run;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    pub x: f64,
    pub y: f64,
}

fn fixed_point_mul<F: ScalarField>(
    builder: &mut BaseCircuitBuilder<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,
) where  F: BigPrimeField {
    const PRECISION: u32 = 63;
    println!("build_lookup_bit: {:?}", builder.lookup_bits());
    let fixed_point_chip = FixedPointChip::<F, PRECISION>::default(builder);
    let ctx = builder.main(0);

    let x_decimal: f64 = input.x;
    let x = fixed_point_chip.quantization(input.x);
    let y_decimal: f64 = input.y;
    let y = fixed_point_chip.quantization(input.y);
    println!("x: {:?}, x_decimal: {:?}, y: {:?}, y_decimal: {:?}", x, x_decimal, y, y_decimal);

    let x = ctx.load_witness(x);
    let y = ctx.load_witness(y);
    // make_public.extend([x, y]);

    let prod = fixed_point_chip.qmul(ctx, x, y);
    let add = fixed_point_chip.qadd(ctx, x, y);
    
    make_public.push(prod);
    let prod_decimal = fixed_point_chip.dequantization(*prod.value());
    let prod_native = input.x * input.y;
    println!("prod: {:?}, prod_decimal: {:?}, prod_native: {:?}", prod, prod_decimal, prod_native);
}

fn main() {
    env_logger::init();

    let args = Cli::parse();
    run(fixed_point_mul, args);
}
