use rs_promethee_core::{generalized_criterion::GeneralizedCriterion, PrometheeProblem};

#[test]
fn read_correct_excel() {
    let project_path = env!("CARGO_MANIFEST_DIR");
    let file_path = format!("{}/tests/files/test_correct.xlsx", project_path);

    let res = PrometheeProblem::from_excel(&file_path);
    match res {
        Ok(problem) => {
            println!("{:?}", problem);
            assert_eq!(*problem.perf(0, 0).unwrap(), 12000.0);
            assert_eq!(*problem.perf(0, 1).unwrap(), 80000.0);
            assert_eq!(*problem.perf(0, 2).unwrap(), 35000.0);

            assert_eq!(*problem.perf(1, 0).unwrap(), 110.0);
            assert_eq!(*problem.perf(1, 1).unwrap(), 290.0);
            assert_eq!(*problem.perf(1, 2).unwrap(), 190.0);

            assert_eq!(*problem.perf(2, 0).unwrap(), 0.4);
            assert_eq!(*problem.perf(2, 1).unwrap(), 0.4);
            assert_eq!(*problem.perf(2, 2).unwrap(), 0.8);

            assert_eq!(problem.n(), 3);
            assert_eq!(problem.q(), 3);

            let real_names = ["Prix", "Vitesse", "Robustesse"];

            problem
                .criteria_names()
                .iter()
                .enumerate()
                .for_each(|(k, name)| assert_eq!(name.as_ref(), real_names[k]));

            let pref_funs = [
                GeneralizedCriterion::VShape { p: 1000.0 },
                GeneralizedCriterion::Linear { q: 10.0, p: 30.0 },
                GeneralizedCriterion::Linear { q: 0.1, p: 0.3 },
            ];

            (0..problem.q()).for_each(|k| assert_eq!(*problem.pref_fun(k).unwrap(), pref_funs[k]));
        }
        Err(e) => assert!(false, "Should read excel file, error: {:?}", e),
    }
}
