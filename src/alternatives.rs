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

#[derive(Debug)]
pub struct AlternativeTable {
    alternatives: Box<[Alternative]>,
    criteria_names: Box<[Box<str>]>,
}

impl AlternativeTable {
    pub fn new(alternatives: &[Alternative]) -> Self {
        if alternatives.is_empty() {
            panic!("Empty table of alternatives");
        }
        if alternatives
            .iter()
            .any(|alt| alt.perfs().len() != alternatives[0].perfs().len())
        {
            panic!("Inconsistent number of evaluations for alternatives");
        }
        Self {
            alternatives: alternatives.into(),
            criteria_names: Box::new([]),
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
