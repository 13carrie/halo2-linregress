use clap::Parser;
use halo2_base::gates::circuit::builder::BaseCircuitBuilder;
use halo2_base::gates::{GateChip, GateInstructions};
use halo2_base::utils::ScalarField;
use halo2_base::AssignedValue;

#[derive(Clone, Debug, Serialize, Deserialize)]

#[allow(unused_imports)]
use halo2_base::{
    Context,
    QuantumCell::{Constant, Existing, Witness},
};
use halo2_graph::scaffold::cmd::Cli;
use halo2_graph::scaffold::run;
use serde::{Deserialize, Serialize};

use std::marker::PhantomData;
use halo2_proofs::circuit::Value;
use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{Cell, Chip, Layouter, SimpleFloorPlanner},
    plonk::{Advice, Assigned, Circuit, Column, ConstraintSystem, Error, Fixed, Instance},
    poly::Rotation,
};

struct LinRegressConfig {
    x: Column<Advice>, // private input
    y: Column<Advice>,
    sum_x: Column<Advice>, // private intermediate
    sum_y: Column<Advice>,
    sum_xy: Column<Advice>,
    sum_x2: Column<Advice>,
    n: Column<Advice>,
    a: Column<Instance>, // public input
    b: Column<Instance>,
}

impl LinRegressConfig {}

struct LinRegressCircuit {
    config: LinRegressConfig,
}

impl Circuit<Fp> for LinRegressCircuit {
    // configure method:

    // synthesise method:
        // this is where you compute all the intermediate values like a and b
}


struct LinRegressChip {
    config: LinRegressConfig,
}

impl LinRegressChip {
    // summing vectors gate:

    // multiplying vectors gate:

    // division gate?

    //
}


fn main() {
    env_logger::init();

    let args = Cli::parse();

    // run different zk commands based on the command line arguments
    // run(LinregressConfig, args);
}