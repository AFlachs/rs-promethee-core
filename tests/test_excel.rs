use rs_promethee_core::PrometheeProblem;

#[test]
fn read_correct_excel() {
    let project_path = env!("CARGO_MANIFEST_DIR");
    let file_path = format!("{}/tests/files/test_correct.xlsx", project_path);

    let res = PrometheeProblem::from_excel(&file_path);
    match res {
        Ok(problem) => {
            println!("{:?}", problem);
            assert_eq!(*problem.get_eval(0, 0).unwrap(), 12000.0);
            assert_eq!(*problem.get_eval(0, 1).unwrap(), 80000.0);
            assert_eq!(*problem.get_eval(0, 2).unwrap(), 35000.0);

            assert_eq!(*problem.get_eval(1, 0).unwrap(), 110.0);
            assert_eq!(*problem.get_eval(1, 1).unwrap(), 290.0);
            assert_eq!(*problem.get_eval(1, 2).unwrap(), 190.0);

            assert_eq!(*problem.get_eval(2, 0).unwrap(), 0.4);
            assert_eq!(*problem.get_eval(2, 1).unwrap(), 0.4);
            assert_eq!(*problem.get_eval(2, 2).unwrap(), 0.8);

            todo!("Verifier les fonctions de preference");
        }
        Err(e) => assert!(false, "Should read excel file, error: {:?}", e),
    }
}
