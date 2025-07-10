use rand::prelude::*;
use rs_promethee_core::generalized_criterion::GeneralizedCriterion;
use rs_promethee_core::*;
use rs_promethee_core::alternatives::{Alternative, AlternativeTable};

fn random_promethee_problem(n: usize, q: usize, max_val: f64) -> PrometheeProblem {
    let mut evaluations = vec![0.0; q];
    let mut alternatives = Vec::<Alternative>::with_capacity(n);
    for i in 0..n {
        for k in 0..q {
            evaluations[k] = max_val * random::<f64>();
        }
        alternatives.push(Alternative::new(format!("a_{}", i), evaluations.clone()));
    }
    let alt_table = AlternativeTable::new(alternatives.into());

    let mut generalized_criteria = Vec::<GeneralizedCriterion>::new();
    for _ in 0..q {
        generalized_criteria.push(GeneralizedCriterion::VShape {
            p: random::<f64>() * max_val * 0.1,
        })
    }
    let mut weights = vec![0f64; q];
    thread_rng().fill(&mut weights[..]);

    PrometheeProblem::new(alt_table, generalized_criteria, weights)
}

fn main() {
    for n in (10..5000).step_by(20) {
        for k in 2..10 {
            let problem = random_promethee_problem(n, k, 10e5);
            let solution = problem.solve();
            solution.net_flows();
        }
    }
}
