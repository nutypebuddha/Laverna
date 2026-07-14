pub mod convert;
pub mod determinism;
pub mod formulas;
pub mod math;
pub mod natural;
pub mod parser;
pub mod pipeline;
pub mod rational;
pub mod sanity;
pub mod solver;
pub mod thinking;
pub mod verify;

use std::collections::HashMap;

pub type TantoEnv = HashMap<String, f64>;

pub fn create_env() -> TantoEnv {
    let mut env = TantoEnv::new();
    env.insert("pi".to_string(), std::f64::consts::PI);
    env.insert("e".to_string(), std::f64::consts::E);
    env.insert("c".to_string(), 299792458.0);
    env.insert("g".to_string(), 9.80665);
    env.insert("G".to_string(), 6.674e-11);
    env.insert("h".to_string(), 6.626e-34);
    env.insert("kB".to_string(), 1.381e-23);
    env.insert("R_air".to_string(), 287.0586949114925);
    env.insert("R_universal".to_string(), 8.314462618);
    env.insert("c_squared".to_string(), 89875517873681764.0);
    env.insert("avogadro".to_string(), 6.022e23);
    env
}

pub fn evaluate_expr(expr: &str, env: &TantoEnv) -> Option<f64> {
    let processed = natural::preprocess_line(expr.as_bytes());
    let expr_str = std::str::from_utf8(&processed).ok()?;
    let expr_bytes = expr_str.as_bytes();
    parser::eval_math(expr_bytes, env).or_else(|| parser::eval_op_format(expr_bytes, env))
}

pub fn evaluate_nl(expr: &str, env: &TantoEnv) -> Option<f64> {
    evaluate_expr(expr, env)
}

pub fn compute_formula(name: &str, args: &str) -> Option<formulas::FormulaResult> {
    let line = format!("{} {}", name, args);
    formulas::compute_formula(&line)
}

pub fn solve_problem(input: &str) -> Option<solver::SolverResult> {
    solver::solve(input)
}

pub fn apply_thinking(framework: &str, problem: &str) -> Option<thinking::ThinkResult> {
    thinking::think(&format!("{} {}", framework, problem))
}

pub fn evaluate_pipeline(line: &str, env: &TantoEnv) -> Option<f64> {
    pipeline::evaluate_pipeline(line, env)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_env() {
        let env = create_env();
        assert!(env.contains_key("pi"));
        assert!(env.contains_key("e"));
        assert!((env["pi"] - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_evaluate_expr_basic() {
        let env = create_env();
        assert_eq!(evaluate_expr("2+3", &env), Some(5.0));
        assert_eq!(evaluate_expr("10-4", &env), Some(6.0));
        assert_eq!(evaluate_expr("3*4", &env), Some(12.0));
        assert_eq!(evaluate_expr("12/4", &env), Some(3.0));
    }

    #[test]
    fn test_evaluate_expr_constants() {
        let env = create_env();
        assert!((evaluate_expr("pi", &env).unwrap() - std::f64::consts::PI).abs() < 1e-10);
        assert!((evaluate_expr("e", &env).unwrap() - std::f64::consts::E).abs() < 1e-10);
    }

    #[test]
    fn test_evaluate_nl() {
        let env = create_env();
        assert_eq!(evaluate_nl("2+3", &env), Some(5.0));
    }
}
