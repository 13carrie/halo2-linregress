use clap::Parser;
#[allow(unused_imports)]
use halo2_base::{
    Context,
    QuantumCell::{Constant, Existing, Witness},
};
use halo2_graph::scaffold::cmd::Cli;
use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{Cell, Chip, Layouter, SimpleFloorPlanner},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance},
};

// Our circuit is going to look like:
// x   | y   | n   | sum_x| sum_y| sum_xy | sum_x2 | slope | intercept | a | b |
// x_0 | y_0 | n_0 | sx_0 | sy_0 | sxy_0  | sx2_0  | sl_0  | int_0     |a_0|b_0|   
// x_1 | y_1 | n_1 | sx_1 | sy_1 | sxy_1  | sx2_1  | sl_1  | int_1     |a_1|b_1|
// x_2 | y_2 | n_2 | sx_2 | sy_2 | sxy_2  | sx2_2  | sl_2  | int_2     |a_2|b_2|
// ...
// with constraints:
// slope == b (within a certain degree of error)
// intercept == a (within a certain degree of error)
// ...

#[derive(Clone, Default)]
struct LinRegressConfig {
    x: Column<Advice>, // private input (change to meta.advice_column();?)
    y: Column<Advice>,
    n: Column<Advice>,
    sum_x: Column<Advice>, // private intermediate
    sum_y: Column<Advice>,
    sum_xy: Column<Advice>,
    sum_x2: Column<Advice>,
    slope: Column<Advice>,
    intercept: Column<Advice>,
    a: Column<Instance>, // public input
    b: Column<Instance>,
}

impl LinRegressConfig {
    pub fn configure<F: FieldExt>(meta: &mut ConstraintSystem<F>) -> Self {
        // define all the columns here + enable equality constraints
        let x = meta.advice_column();
        let y = meta.advice_column();
        let n = meta.advice_column();
        let sum_x = meta.advice_column();
        let sum_y = meta.advice_column();
        let sum_xy = meta.advice_column();
        let sum_x2 = meta.advice_column();
        let slope = meta.advice_column();
        let intercept = meta.advice_column();
        let a = meta.instance_column();
        let b = meta.instance_column();

        // enable equality constraint for comparison columns
        meta.enable_equality(a);
        meta.enable_equality(b);
        meta.enable_equality(slope);
        meta.enable_equality(intercept);

        // ok now do the gates
        // we just need all of our constraints to hold
        // for one row in the table
        // ... i think

        // summing vectors gate:
            // set sum == 0
            // for value in the vector, sum += value
            // return sum

        // multiplying vectors gate:


        // division gate? take from zkfixedpointchip?

        Self { x, y, n, sum_x, sum_y, sum_xy, sum_x2, slope, intercept, a, b }
    }
}

#[derive(Clone, Default)]
struct LinRegressCircuit<F> {
    config: LinRegressConfig,
    // assign value<t>/witness to each column here
}

impl Circuit<F> for LinRegressCircuit<F> {
    type Config = LinRegressConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        unimplemented!()
    }

    // configure method:
    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        //LinRegressConfig::configure::<F>(meta)
    }

    // synthesise method:
    fn synthesize(&self, config: Self::Config, layouter: impl Layouter<F>) -> Result<(), Error> {
        // this is where you compute all the intermediate values like sums
        // and also a and b
        // and also put all the constraints here
    }
}



fn main() {
    env_logger::init();

    let args = Cli::parse();

    // run different zk commands based on the command line arguments
    // run(LinregressConfig, args);
}