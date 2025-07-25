pub mod alternatives;
pub mod generalized_criterion;

#[cfg(feature = "parse")]
pub mod parse;

use itertools::Itertools;
use std::collections::VecDeque;
use tabled;
use tabled::settings::Style;

use alternatives::{AlternativeTable, OptimizationDirection};
use generalized_criterion::GeneralizedCriterion;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Promethee2Result {
    pub positive_flows: Vec<f64>,
    pub unicrit_positive_flows: Vec<Vec<f64>>,
    pub negative_flows: Vec<f64>,
    pub unicrit_negative_flows: Vec<Vec<f64>>,
}

impl Promethee2Result {
    pub fn net_flows(&self) -> Vec<f64> {
        self.positive_flows
            .iter()
            .zip(&self.negative_flows)
            .map(|(p, n)| *p - n)
            .collect::<Vec<f64>>()
    }

    pub fn net_flow(&self, ai: usize) -> Option<f64> {
        if ai >= self.positive_flows.len() {
            None
        } else {
            Some(self.positive_flows[ai] - self.negative_flows[ai])
        }
    }

    pub fn unicriterion_net_flows(&self, k: usize) -> Vec<f64> {
        self.unicrit_positive_flows[k]
            .iter()
            .zip(&self.unicrit_negative_flows[k])
            .map(|(p, n)| *p - n)
            .collect::<Vec<f64>>()
    }

    pub fn unicriterion_net_flow(&self, k: usize, ai: usize) -> Option<f64> {
        if ai >= self.unicrit_positive_flows[k].len() {
            None
        } else {
            Some(self.unicrit_positive_flows[k][ai] - self.unicrit_negative_flows[k][ai])
        }
    }

    /// Return arguments corresponding to the alternatives, ranked in descending order of preference
    pub fn ranked_alts(&self) -> Vec<usize> {
        self.net_flows()
            .iter()
            .enumerate()
            .sorted_by(|flow_i, flow_j| PartialOrd::partial_cmp(flow_i.1, flow_j.1).unwrap())
            .map(|(i, _)| i)
            .rev()
            .collect()
    }

    pub fn is_better(&self, a1: usize, a2: usize) -> bool {
        self.positive_flows[a1] - self.negative_flows[a1]
            > self.positive_flows[a2] - self.negative_flows[a2]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrometheeProblem {
    n: usize,
    q: usize,
    /// Matrix of criteria evaluations of size (q, n)
    alt_table: AlternativeTable,
    /// For each criterion, the Option may contain the indices of the alternatives sorted in ascending order of evaluations
    argsorted_eval_matrix: Vec<Option<Vec<usize>>>,
    generalized_criteria: Vec<GeneralizedCriterion>,
    weights: Vec<f64>,
}

impl PrometheeProblem {
    pub fn new(
        alt_table: AlternativeTable,
        generalized_criteria: Vec<GeneralizedCriterion>,
        mut weights: Vec<f64>,
    ) -> Self {
        // normalize weights
        let tot_w: f64 = weights.iter().sum();
        weights = weights.into_iter().map(|w| w / tot_w).collect();

        // TODO todo!("Use optimization direction somehow");

        let n = alt_table.n();
        let q = alt_table.q();

        // Verify validity of inputs
        if generalized_criteria.len() != q {
            panic!(
                "Wrong number of generalized criteria given, {} given, {} expected",
                generalized_criteria.len(),
                q
            );
        }

        if weights.len() != q {
            panic!(
                "Wrong number of weights given, {} given, {} expected",
                weights.len(),
                q
            );
        }

        let argsorted_eval_matrix = generalized_criteria
            .iter()
            .enumerate()
            .map(|(k, criterion)| match criterion {
                GeneralizedCriterion::Linear { q: _, p: _ }
                | GeneralizedCriterion::VShape { p: _ } => {
                    let fks =
                        |alt: usize| -> f64 { alt_table.performance(alt, k).unwrap().to_owned() };
                    let mut argsorted_fks: Vec<usize> = (0..n).collect();
                    argsorted_fks.sort_unstable_by(|&i, &j| fks(i).partial_cmp(&fks(j)).unwrap());
                    Some(argsorted_fks)
                }
                GeneralizedCriterion::Usual => None,
                _ => unimplemented!("Not implemented for this criterion"),
            })
            .collect();

        Self {
            n,
            q,
            alt_table,
            generalized_criteria,
            weights: weights.to_vec(),
            argsorted_eval_matrix,
        }
    }

    fn argsort_evals(&mut self, k: usize) {
        self.argsorted_eval_matrix[k] = {
            let fks =
                |alt: usize| -> f64 { self.alt_table.performance(alt, k).unwrap().to_owned() };

            let mut argsorted_fks: Vec<usize> = (0..self.n()).collect();
            argsorted_fks.sort_unstable_by(|&i, &j| fks(i).partial_cmp(&fks(j)).unwrap());
            Some(argsorted_fks)
        };
    }

    pub fn n(&self) -> usize {
        self.n
    }

    pub fn q(&self) -> usize {
        self.q
    }

    pub fn w(&self, k: usize) -> Option<&f64> {
        self.weights.get(k)
    }

    pub fn pref_fun(&self, k: usize) -> Option<&GeneralizedCriterion> {
        self.generalized_criteria.get(k)
    }

    pub fn alt_name(&self, i: usize) -> Option<&str> {
        self.alt_table.alt_name(i)
    }

    pub fn alt_names(&self) -> Vec<&str> {
        self.alt_table.alt_names()
    }

    pub fn criterion_name(&self, k: usize) -> Option<&str> {
        self.alt_table.criterion_name(k)
    }

    pub fn criteria_names(&self) -> &[Box<str>] {
        self.alt_table.criteria_names()
    }

    pub fn smallest_p_vshape(&self, k: usize) -> Option<f64> {
        if k >= self.q {
            None
        } else {
            match self.generalized_criteria[k] {
                GeneralizedCriterion::VShape { p: _ } => Some(
                    self.argsorted_eval_matrix[k]
                        .as_ref()
                        .expect("to be computed at init")
                        .windows(2)
                        .map(|w| self.perf(k, w[1]).unwrap() - self.perf(k, w[0]).unwrap())
                        .fold(
                            f64::INFINITY,
                            |acc, b| {
                                if b == 0.0 {
                                    acc
                                } else {
                                    acc.min(b)
                                }
                            },
                        ),
                ),
                _ => None,
            }
        }
    }

    pub fn fast_pos_unicriterion_flow(
        &self,
        k: usize,
        q: f64,
        p: f64,
        argsorted_fks: &[usize],
    ) -> Vec<f64> {
        let fks = |alt: usize| -> f64 { self.alt_table.performance(alt, k).unwrap().to_owned() };
        let mut positive_flow = vec![0.0; self.n];
        let (mut w, mut r) = (
            VecDeque::<usize>::new(),
            VecDeque::from(argsorted_fks.to_owned()),
        );
        let mut card_l = 0;
        let mut sum = 0.0;
        let (mut low, mut up): (f64, f64);
        let (mut const_fact, mut last_term): (f64, f64);

        for &idx in argsorted_fks {
            low = fks(idx) - p;
            up = fks(idx) - q;

            // Remove elements leaving window
            while !w.is_empty() && fks(*w.front().unwrap()) <= low {
                sum -= fks(w.pop_front().unwrap());
                card_l += 1;
            }

            // Remove elements leaving the right part
            while !r.is_empty() && fks(*r.front().unwrap()) <= up {
                let x = r.pop_front().unwrap();
                if fks(x) >= low {
                    w.push_back(x);
                    sum += fks(x);
                } else {
                    card_l += 1;
                }
            }

            (const_fact, last_term) = match p != q {
                true => ((fks(idx) - q) / (p - q), -sum / (p - q)),
                false => (0.0, 0.0),
            };
            positive_flow[idx] = 1.0 / (self.n as f64 - 1.0)
                * (card_l as f64 + w.len() as f64 * const_fact + last_term);
        }
        positive_flow
    }

    pub fn perf(&self, k: usize, i: usize) -> Option<&f64> {
        self.alt_table.performance(i, k)
    }

    pub fn fast_neg_unicriterion_flow(
        &self,
        k: usize,
        q: f64,
        p: f64,
        argsorted_fks: &[usize],
    ) -> Vec<f64> {
        let fks = |alt: usize| -> f64 { self.alt_table.performance(alt, k).unwrap().to_owned() };
        let mut negative_flows = vec![0.0; self.n];
        let (mut l, mut w) = (
            VecDeque::from(argsorted_fks.to_owned()),
            VecDeque::<usize>::new(),
        );
        let mut card_r = 0;
        let mut sum = 0.0;
        let (mut low, mut up): (f64, f64);
        let (mut const_fact, mut last_term): (f64, f64);

        for &idx in argsorted_fks.iter().rev() {
            low = fks(idx) + q;
            up = fks(idx) + p;

            // Remove elements leaving window
            while !w.is_empty() && fks(*w.back().expect("not empty")) >= up {
                sum -= fks(w.pop_back().expect("not empty"));
                card_r += 1;
            }

            // Remove elements leaving the left part
            while !l.is_empty() && fks(*l.back().expect("not empty")) >= low {
                let x = l.pop_back().expect("not empty");
                if fks(x) <= up {
                    w.push_front(x);
                    sum += fks(x);
                } else {
                    card_r += 1;
                }
            }

            if p != q {
                const_fact = (fks(idx) + q) / (p - q);
                last_term = sum / (p - q);
            } else {
                (const_fact, last_term) = (0.0, 0.0);
            }
            negative_flows[idx] = 1.0 / (self.n as f64 - 1.0)
                * (card_r as f64 - (w.len() as f64 * const_fact) + last_term);
        }
        negative_flows
    }

    /// Compute the unicriterion positive and negative flows for criterion k
    /// using the O(qnlogn) method from Van Asche, 2018
    fn fast_unicriterion_flows(&self, k: usize) -> Option<(Vec<f64>, Vec<f64>)> {
        if k >= self.q {
            return None;
        }
        let (q, p) = match self.generalized_criteria[k] {
            GeneralizedCriterion::VShape { p } => (0.0, p),
            GeneralizedCriterion::Linear { q, p } => (q, p),
            _ => panic!("Wrong type of criterion for fast method"),
        };
        //
        // We work with argsort instead of sort to work with usize instead of ints
        let argsorted_fks = self.argsorted_eval_matrix[k]
            .as_ref()
            .expect("to be computed at construction");

        let positive_flow = self.fast_pos_unicriterion_flow(k, q, p, argsorted_fks);
        let negative_flows = self.fast_neg_unicriterion_flow(k, q, p, argsorted_fks);
        Some((positive_flow, negative_flows))
    }

    fn slow_unicriterion_flows(
        &self,
        dist_mat: &[Vec<f64>],
        generalized_criterion: &GeneralizedCriterion,
    ) -> (Vec<f64>, Vec<f64>) {
        dist_mat
            .iter()
            .map(|di| {
                let (pos, neg): (f64, f64) = (*di)
                    .iter()
                    .map(|&dij| {
                        (
                            generalized_criterion.normalisation(dij),
                            generalized_criterion.normalisation(-dij),
                        )
                    })
                    .fold((0.0, 0.0), |(acc_p, acc_neg), (pos, neg)| {
                        (acc_p + pos, acc_neg + neg)
                    });
                (pos / (self.n as f64 - 1.0), neg / (self.n as f64 - 1.0))
            })
            .unzip()
    }

    fn unicriterion_flows(&self, k: usize) -> Option<(Vec<f64>, Vec<f64>)> {
        if k >= self.q {
            panic!("Wrong criterion index used, {}>{}", k, self.q)
        }

        let generalized_criterion = &self.generalized_criteria[k];

        match generalized_criterion {
            GeneralizedCriterion::VShape { p: _ } | GeneralizedCriterion::Linear { q: _, p: _ } => {
                self.fast_unicriterion_flows(k)
            }
            _ => {
                let dist_mat: Vec<Vec<f64>> = self
                    .alt_table
                    .criterion(k)
                    .unwrap()
                    .iter()
                    .map(|&a_i| {
                        self.alt_table
                            .criterion(k)
                            .unwrap()
                            .iter()
                            .map(move |&a_j| a_i - a_j.to_owned())
                            .collect()
                    })
                    .collect();
                Some(self.slow_unicriterion_flows(&dist_mat, generalized_criterion))
            }
        }
    }

    pub fn solve(&self) -> Promethee2Result {
        let mut positive_flows: Vec<f64> = vec![0.0; self.n];
        let mut negative_flows: Vec<f64> = vec![0.0; self.n];

        let mut pos_unicriterion_flow: Vec<f64>;
        let mut positive_unicriterions_flows: Vec<Vec<f64>> = Vec::new();
        let mut neg_unicriterion_flow: Vec<f64>;
        let mut negative_unicriterions_flows: Vec<Vec<f64>> = Vec::new();

        for k in 0..self.q {
            // compute positive and negative unicriterion flow and add it to the global
            (pos_unicriterion_flow, neg_unicriterion_flow) = self.unicriterion_flows(k).unwrap();
            positive_unicriterions_flows.push(pos_unicriterion_flow);
            negative_unicriterions_flows.push(neg_unicriterion_flow);

            for i in 0..self.n {
                positive_flows[i] +=
                    self.weights[k] * positive_unicriterions_flows.last().unwrap()[i];
                negative_flows[i] +=
                    self.weights[k] * negative_unicriterions_flows.last().unwrap()[i];
            }
        }

        Promethee2Result {
            positive_flows,
            unicrit_positive_flows: positive_unicriterions_flows,
            negative_flows,
            unicrit_negative_flows: negative_unicriterions_flows,
        }
    }

    pub fn get_parameter(&self, k: usize) -> f64 {
        match self.generalized_criteria[k] {
            crate::generalized_criterion::GeneralizedCriterion::VShape { p } => p,
            _ => panic!("Invalid criterion"),
        }
    }

    /// Return the evaluation of the alternatives for criterion k, sorted in ascending order
    /// If the evaluation matrix is not sorted, compute it
    pub fn sorted_evals(&self, k: usize) -> Vec<f64> {
        match self.argsorted_eval_matrix[k].as_ref() {
            Some(sorted_indices) => sorted_indices
                .iter()
                .map(|&i| self.perf(k, i).unwrap().to_owned())
                .collect(),
            None => {
                let mut sorted_fks = self.alt_table.criterion(k).unwrap();
                sorted_fks.sort_unstable_by(|&i, &j| i.partial_cmp(&j).unwrap());
                sorted_fks
            }
        }
    }

    /// Shift the evaluation of alternative a for criterion k by shift
    /// This sorts the evaluation matrix for criterion k afterwards (naively)
    pub fn shift_eval(&mut self, k: usize, i: usize, shift: f64) {
        self.alt_table.shift_performance(i, k, shift);
        self.argsort_evals(k);
    }

    pub fn print(&self) {
        println!(
            "Promethee problem with {} alternatives and {} criteria",
            self.n, self.q
        );
        let mut builder = tabled::builder::Builder::default();
        let crit_names = self.alt_table.criteria_names();

        use std::iter::once;

        builder.push_record(
            once("Criteria")
                .chain(crit_names.iter().map(|s| s.as_ref()))
                .collect::<Vec<_>>(),
        );

        builder.push_record(
            once("Directions")
                .chain(
                    self.alt_table
                        .criteria_directions()
                        .iter()
                        .map(|d| match d {
                            OptimizationDirection::Min => "Min",
                            OptimizationDirection::Max => "Max",
                        }),
                )
                .collect::<Vec<_>>(),
        );

        builder.push_record(
            once("Weights".to_string())
                .chain(self.weights.iter().map(|&w| w.to_string()))
                .collect::<Vec<_>>(),
        );

        builder.push_record(
            once("Preference functions".to_string())
                .chain(self.generalized_criteria.iter().map(|c| match c {
                    GeneralizedCriterion::UShape { p } => format!("UShape({})", p),
                    GeneralizedCriterion::VShape { p } => format!("VShape({})", p),
                    GeneralizedCriterion::Linear { q, p } => format!("Linear({}, {})", q, p),
                    GeneralizedCriterion::Usual => "Usual".to_string(),
                }))
                .collect::<Vec<_>>(),
        );

        // Print alternatives
        for alternative in self.alt_table.alternatives() {
            builder.push_record(
                once(alternative.name().to_string())
                    .chain(alternative.perfs().iter().map(|&v| v.to_string()))
                    .collect::<Vec<_>>(),
            );
        }
        let mut table = builder.build();
        table.with(Style::modern());

        println!("{}", table);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alternatives::{Alternative, AlternativeTable};

    fn init_simple_problem() -> PrometheeProblem {
        let alt_table = AlternativeTable::new(
            vec![
                Alternative::new("A".to_string(), vec![3.0, 1.0]),
                Alternative::new("B".to_string(), vec![2.0, 4.0]),
                Alternative::new("C".to_string(), vec![2.0, 3.0]),
            ]
            .into(),
        );
        // let prob_matrix: Vec<Vec<f64>> = vec![vec![3.0, 2.0, 2.0], vec![1.0, 4.0, 3.0]];
        let weights: Vec<f64> = vec![3.0, 7.0];
        let criteria: Vec<GeneralizedCriterion> = vec![
            GeneralizedCriterion::VShape { p: 3.0 },
            GeneralizedCriterion::Linear { q: 1.0, p: 3.0 },
        ];

        PrometheeProblem::new(alt_table, criteria, weights)
    }

    fn round_vec(v: &mut Vec<f64>) -> Vec<f64> {
        v.iter().map(|fl| (fl * 1000.0).round() / 1000.0).collect()
    }

    #[test]
    fn test_solve() {
        let problem = init_simple_problem();

        let solution = problem.solve();
        let final_net_flow: Vec<f64> = solution
            .net_flows()
            .iter()
            .map(|fl| (fl * 1000.0).round() / 1000.0)
            .collect();
        let real_solution = vec![-0.425, 0.3, 0.125];
        let mut equality = real_solution
            .iter()
            .enumerate()
            .map(|(i, &val)| val == final_net_flow[i]);

        assert!(equality.all(|x| x))
    }

    #[test]
    fn test_solve_2() {
        let alt_table = AlternativeTable::new(
            vec![
                Alternative::new("A".to_string(), vec![3.0, 1.0]),
                Alternative::new("B".to_string(), vec![2.0, 4.0]),
                Alternative::new("C".to_string(), vec![0.0, 5.0]),
            ]
            .into(),
        );
        // let prob_matrix: Vec<Vec<f64>> = vec![vec![3.0, 2.0, 0.0], vec![1.0, 4.0, 5.0]];
        let weights: Vec<f64> = vec![1.0, 1.0];
        let criteria: Vec<GeneralizedCriterion> = vec![
            GeneralizedCriterion::VShape { p: 2.0 },
            GeneralizedCriterion::VShape { p: 3.0 },
        ];

        let problem = PrometheeProblem::new(alt_table, criteria, weights);

        let solution = problem.solve();
        let final_net_flow: Vec<f64> = solution
            .net_flows()
            .iter()
            .map(|fl| (fl * 1000.0).round() / 1000.0)
            .collect();
        let real_solution = vec![-0.125, 0.292, -0.167];
        // println!("{:#?}", final_net_flow);
        let mut equality = real_solution
            .iter()
            .enumerate()
            .map(|(i, &val)| val == final_net_flow[i]);

        assert!(equality.all(|x| x))
    }

    #[test]
    fn solve_fast_and_slow_equivalent() {
        let problem = init_simple_problem();

        for k in 0..problem.q {
            let generalized_criterion = &problem.generalized_criteria[k];
            let dist_mat: Vec<Vec<f64>> = problem
                .alt_table
                .criterion(k)
                .unwrap()
                .iter()
                .map(|&a_i| {
                    problem
                        .alt_table
                        .criterion(k)
                        .unwrap()
                        .iter()
                        .map(move |&a_j| a_i - a_j.to_owned())
                        .collect()
                })
                .collect();
            let (mut slow_pos_flow, mut slow_neg_flow) =
                problem.slow_unicriterion_flows(&dist_mat, generalized_criterion);
            slow_pos_flow = round_vec(&mut slow_pos_flow);
            slow_neg_flow = round_vec(&mut slow_neg_flow);

            let (mut fast_pos_flow, mut fast_neg_flow) =
                problem.fast_unicriterion_flows(k).unwrap();
            fast_pos_flow = round_vec(&mut fast_pos_flow);
            fast_neg_flow = round_vec(&mut fast_neg_flow);

            println!("{:#?}", slow_pos_flow);
            println!("{:#?}", slow_neg_flow);
            println!("");

            println!("{:#?}", fast_pos_flow);
            println!("{:#?}", fast_neg_flow);

            assert!(slow_pos_flow
                .iter()
                .enumerate()
                .map(|(i, &val)| val == fast_pos_flow[i])
                .all(|x| x));

            assert!(slow_neg_flow
                .iter()
                .enumerate()
                .map(|(i, &val)| val == fast_neg_flow[i])
                .all(|x| x));
        }
    }
}
