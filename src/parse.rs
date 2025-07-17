use calamine::{open_workbook, DataType, HeaderRow, Reader, Xlsx};
use crate::alternatives::{Alternative, AlternativeTable, OptimizationDirection};
use crate::{PrometheeProblem, generalized_criterion};
use std::{error::Error, str::FromStr};


pub fn from_excel(file_path: &str) -> Result<PrometheeProblem, Box<dyn Error>> {
    let mut workbook: Xlsx<_> = open_workbook(file_path)?;

    let range = workbook
        .with_header_row(HeaderRow::FirstNonEmptyRow)
        .worksheet_range("Promethee")?;

    let ncrits = range.width() - 1;
    let mut weights: Vec<f64> = Vec::with_capacity(ncrits);
    let mut pref_funs = Vec::with_capacity(ncrits);
    let mut fun_types: Vec<&str> = Vec::with_capacity(ncrits);
    let mut qs = Vec::with_capacity(ncrits);
    let mut ps = Vec::with_capacity(ncrits);

    let criteria_names: Vec<String> = range
        .headers()
        .map(|headers| headers.into_iter().skip(1).collect())
        .expect("To have headers");

    let mut alternatives: Vec<Alternative> = Vec::new();
    let mut criteria_directions: Vec<OptimizationDirection> = Vec::new();

    for (i, row) in range.rows().enumerate() {
        if i == 0 {
            continue;
        } else if i == 1 {
            let (_, directions) = row.split_at(1);
            criteria_directions = directions
                .into_iter()
                .map(|data_dir| {
                    OptimizationDirection::from_str(
                        data_dir.get_string().expect("to be string"),
                    )
                    .expect("to be valid direction")
                })
                .collect();
        } else if i == 2 {
            let (_, ws) = row.split_at(1);
            weights = ws
                .into_iter()
                .map(|data_w| data_w.get_float().expect("to be f64"))
                .collect();
        } else if i == 3 {
            let (_, ftypes) = row.split_at(1);
            fun_types = ftypes
                .into_iter()
                .map(|data_ft| data_ft.get_string().expect("to be string"))
                .collect();
        } else if i == 4 {
            let (_, q_data) = row.split_at(1);
            qs = q_data
                .into_iter()
                .map(|q| q.get_float().expect("to be f64"))
                .collect();
        } else if i == 5 {
            let (_, p_data) = row.split_at(1);
            ps = p_data
                .into_iter()
                .map(|p| p.get_float().expect("to be f64"))
                .collect();
        } else {
            if row.len() != ncrits + 1 {
                return Err("Invalid number of columns".into());
            }
            let name = row[0].get_string().expect("To be string");
            let performances = row
                .into_iter()
                .skip(1)
                .map(|data| data.get_float().expect("to be f64"))
                .collect();
            println!("{:?}", performances);
            alternatives.push(Alternative::new(name.to_string(), performances));
        }
    }

    let alt_table = AlternativeTable::new(alternatives.into_boxed_slice())
        .with_criteria_names(criteria_names)
        .with_criteria_directions(criteria_directions);

    for k in 0..ncrits {
        pref_funs.push(generalized_criterion::from_params(
            fun_types[k],
            qs[k],
            ps[k],
        ))
    }

    Ok(PrometheeProblem::new(alt_table, pref_funs, weights))
}