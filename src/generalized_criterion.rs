#[derive(Debug, PartialEq)]
pub enum GeneralizedCriterion {
    UShape { p: f64 },
    VShape { p: f64 },
    Linear { q: f64, p: f64 },
    Usual,
}

impl GeneralizedCriterion {
    pub fn normalisation(&self, d_ij: f64) -> f64 {
        match *self {
            GeneralizedCriterion::VShape { p } => normalize_v_shape(p, d_ij),
            GeneralizedCriterion::Linear { q, p } => normalize_linear(q, p, d_ij),
            GeneralizedCriterion::Usual => {
                if d_ij > 0.0 {
                    1.0
                } else {
                    0.0
                }
            }
            GeneralizedCriterion::UShape { p } => {
                if d_ij < p {
                    0.0
                } else {
                    1.0
                }
            }
        }
    }

    pub fn sym_normalisation(&self, d_ij: f64) -> f64 {
        d_ij.signum()
            * match *self {
                GeneralizedCriterion::VShape { p } => normalize_v_shape(p, d_ij.abs()),
                GeneralizedCriterion::Linear { q, p } => normalize_linear(q, p, d_ij.abs()),
                GeneralizedCriterion::Usual => {
                    if d_ij != 0.0 {
                        1.0
                    } else {
                        0.0
                    }
                }
                GeneralizedCriterion::UShape { p } => {
                    if d_ij.abs() < p {
                        0.0
                    } else {
                        1.0
                    }
                }
            }
    }
}

fn normalize_linear(q: f64, p: f64, d_ij: f64) -> f64 {
    if d_ij < q {
        0.0
    } else if d_ij < p {
        (d_ij - q) / (p - q)
    } else {
        1.0
    }
}

fn normalize_v_shape(p: f64, d_ij: f64) -> f64 {
    if d_ij < 0.0 {
        0.0
    } else if d_ij < p {
        d_ij / p
    } else {
        1.0
    }
}

pub fn from_params(ftype: &str, q: f64, p: f64) -> GeneralizedCriterion {
    match ftype {
        "Usual" => GeneralizedCriterion::Usual,
        "U-Shape" => GeneralizedCriterion::UShape { p },
        "V-Shape" => GeneralizedCriterion::VShape { p },
        "Linear" => GeneralizedCriterion::Linear { q, p },
        "Level" | "Gaussian" => unimplemented!(),
        _ => panic!("Wrong type"),
    }
}

#[cfg(test)]
mod test_generalized_normalisation {
    use super::normalize_linear;
    use super::normalize_v_shape;

    #[test]
    fn test_linear_q0() {
        let a = normalize_linear(0.0, 1.0, -1.0);
        let b = normalize_linear(0.0, 1.0, 0.5);
        let c = normalize_linear(0.0, 1.0, 1.1);

        assert_eq!(a, 0.0);
        assert_eq!(b, 0.5);
        assert_eq!(c, 1.0);
    }

    #[test]
    fn test_linear_q1() {
        let a = normalize_linear(1.0, 2.0, 0.0);
        let b = normalize_linear(1.0, 3.0, 1.5);
        let c = normalize_linear(1.0, 3.0, 2.0);

        assert_eq!(a, 0.0);
        assert_eq!(b, 0.25);
        assert_eq!(c, 0.5);
    }

    #[test]
    fn test_v_shape() {
        let a = normalize_v_shape(1.0, 0.0);
        let b = normalize_v_shape(1.0, 0.5);
        let c = normalize_v_shape(1.0, 1.2);

        assert_eq!(a, 0.0);
        assert_eq!(b, 0.5);
        assert_eq!(c, 1.0);
    }
}
