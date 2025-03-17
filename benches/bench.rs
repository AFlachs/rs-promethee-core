use rand::prelude::*;
use rs_promethee_core::generalized_criterion::GeneralizedCriterion;
use rs_promethee_core::*;

fn random_promethee_problem(n: usize, q: usize, max_val: f64) -> PrometheeProblem {
    let mut eval_matrix: Vec<Vec<f64>> = vec![vec![0.0; n]; q];
    let mut generalized_criteria = Vec::<GeneralizedCriterion>::new();
    for k in 0..q {
        for i in 0..n {
            eval_matrix[k][i] = max_val * random::<f64>();
        }

        generalized_criteria.push(GeneralizedCriterion::VShape {
            p: random::<f64>() * max_val * 0.1,
        })
    }
    let mut weights = vec![0f64; q];
    thread_rng().fill(&mut weights[..]);

    PrometheeProblem::new(eval_matrix, generalized_criteria, weights)
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
