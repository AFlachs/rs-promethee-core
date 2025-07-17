use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Alternative {
    name: String,
    performances: Vec<f64>,
}

impl Alternative {
    pub fn new(name: String, performances: Vec<f64>) -> Self {
        Self { name, performances }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn perfs(&self) -> &[f64] {
        &self.performances
    }

    pub fn perf(&self, k: usize) -> Option<&f64> {
        self.performances.get(k)
    }

    pub fn change_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn change_perf(&mut self, k: usize, val: f64) {
        self.performances[k] = val;
    }
}

#[derive(Debug, Clone)]
pub enum OptimizationDirection {
    Min,
    Max,
}

impl FromStr for OptimizationDirection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "min" | "Min" | "MIN" => Ok(Self::Min),
            "max" | "Max" | "MAX" => Ok(Self::Max),
            _ => Err(format!("Invalid optimization direction: {}", s)),
        }
    }
}

#[derive(Debug)]
pub struct AlternativeTable {
    alternatives: Box<[Alternative]>,
    criteria_names: Box<[Box<str>]>,
    /// Direction of optimization for each criterion, only used as an indicator for display.
    /// If a criterion is indicated to minimize, its evaluations are multiplied by -1 at build time
    /// so that it can be maximized.
    criteria_direction: Box<[OptimizationDirection]>,
}

impl AlternativeTable {
    pub fn new(alternatives: Box<[Alternative]>) -> Self {
        if alternatives.is_empty() {
            panic!("Empty table of alternatives");
        }
        if alternatives
            .iter()
            .any(|alt| alt.perfs().len() != alternatives[0].perfs().len())
        {
            panic!("Inconsistent number of evaluations for alternatives");
        }

        let q = alternatives[0].perfs().len();
        let criteria_names = (0..q)
            .map(|k| format!("Criterion {}", k + 1).into())
            .collect();

        Self {
            alternatives,
            criteria_names,
            criteria_direction: vec![OptimizationDirection::Max; q].into(),
        }
    }

    pub fn with_criteria_directions(
        mut self,
        criteria_direction: Vec<OptimizationDirection>,
    ) -> Self {
        self.criteria_direction = criteria_direction.into();
        self
    }

    pub fn set_criterion_direction(&mut self, k: usize, direction: OptimizationDirection) {
        self.criteria_direction[k] = direction;
    }

    pub fn criterion_direction(&self, k: usize) -> &OptimizationDirection {
        &self.criteria_direction[k]
    }

    pub fn criteria_directions(&self) -> &[OptimizationDirection] {
        &self.criteria_direction
    }

    pub fn swap_criteria_direction(&mut self, k: usize) {
        if k >= self.criteria_direction.len() {
            panic!("Invalid criterion index");
        }
        self.criteria_direction[k] = match self.criteria_direction[k] {
            OptimizationDirection::Min => OptimizationDirection::Max,
            OptimizationDirection::Max => OptimizationDirection::Min,
        };
        // Invert evaluations for this criterion
        for alt in self.alternatives.iter_mut() {
            let val = alt.perf(k).unwrap();
            alt.change_perf(k, -val);
        }
    }

    pub fn with_criteria_names(mut self, criteria_names: Vec<String>) -> Self {
        self.criteria_names = criteria_names.into_iter().map(|s| s.into()).collect();
        self
    }

    pub fn alternative(&self, i: usize) -> Option<&Alternative> {
        self.alternatives.get(i)
    }

    pub fn alternatives(&self) -> &[Alternative] {
        &self.alternatives
    }

    pub fn alt_name(&self, i: usize) -> Option<&str> {
        match self.alternatives.get(i) {
            Some(alt) => Some(alt.name()),
            None => None,
        }
    }

    pub fn alt_names(&self) -> Vec<&str> {
        self.alternatives.iter().map(|alt| alt.name()).collect()
    }

    pub fn criterion(&self, k: usize) -> Option<Vec<f64>> {
        if k < self.alternatives[0].perfs().len() {
            Some(
                self.alternatives
                    .iter()
                    .map(|alt| alt.perf(k).copied().unwrap())
                    .collect(),
            )
        } else {
            None
        }
    }

    pub fn criteria(&self) -> Vec<Vec<f64>> {
        (0..self.alternatives[0].perfs().len())
            .map(|k| self.criterion(k).unwrap())
            .collect()
    }

    pub fn criteria_names(&self) -> &[Box<str>] {
        &self.criteria_names
    }

    pub fn criterion_name(&self, k: usize) -> Option<&str> {
        (*self.criteria_names).get(k).map(|s| s.as_ref())
    }

    pub fn performance(&self, i: usize, k: usize) -> Option<&f64> {
        self.alternatives.get(i)?.perf(k)
    }

    pub fn set_performance(&mut self, i: usize, k: usize, val: f64) {
        if let Some(alt) = self.alternatives.get_mut(i) {
            alt.change_perf(k, val);
        }
    }

    pub fn shift_performance(&mut self, i: usize, k: usize, shift: f64) {
        if let Some(alt) = self.alternatives.get_mut(i) {
            let new_val = alt.perf(k).unwrap() + shift;
            alt.change_perf(k, new_val);
        }
    }

    pub fn n(&self) -> usize {
        self.alternatives.len()
    }

    pub fn q(&self) -> usize {
        self.alternatives[0].perfs().len()
    }
}
