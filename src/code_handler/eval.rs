#![allow(unexpected_cfgs)]

use crate::code_handler::map::{Map, MapNamespace, Vars};
use crate::preprocessor::config::MapEvalConfig;
use anyhow::{Context, Result};
use fasteval::{Compiler, Evaler, Parser, Slab};

pub(crate) struct EvalContext {
    pub(crate) parser: Parser,
    pub(crate) slab: Slab,
    pub(crate) config: MapEvalConfig,
}

impl EvalContext {
    pub(crate) fn new(config: &MapEvalConfig) -> Self {
        Self {
            parser: Parser::new(),
            slab: Slab::new(),
            config: config.clone(),
        }
    }

    pub(crate) fn eval(&mut self, expr: &str, map: &Map, vars: &Vars) -> Result<f64> {
        if self.config.reuse_slab {
            return self.eval_with_reused_slab(expr, map, vars);
        }

        eval_with_fresh_slab(expr, map, vars, &self.config)
    }

    fn eval_with_reused_slab(&mut self, expr: &str, map: &Map, vars: &Vars) -> Result<f64> {
        let mut namespace = MapNamespace::new(map, vars, &self.config.namespace_scope);

        if !self.config.enabled || !self.config.compile_expressions {
            let expr_ref = self
                .parser
                .parse(expr, &mut self.slab.ps)
                .with_context(|| format!("failed to parse expression `{}`", expr))?
                .from(&self.slab.ps);

            return expr_ref
                .eval(&self.slab, &mut namespace)
                .with_context(|| format!("failed to evaluate expression `{}`", expr));
        }

        let expr_ref = self
            .parser
            .parse(expr, &mut self.slab.ps)
            .with_context(|| format!("failed to parse expression `{}`", expr))?
            .from(&self.slab.ps);

        let compiled = expr_ref.compile(&self.slab.ps, &mut self.slab.cs);
        Ok(fasteval::eval_compiled!(
            compiled,
            &self.slab,
            &mut namespace
        ))
    }
}

fn eval_with_fresh_slab(expr: &str, map: &Map, vars: &Vars, config: &MapEvalConfig) -> Result<f64> {
    let parser = Parser::new();
    let mut slab = Slab::new();
    let mut namespace = MapNamespace::new(map, vars, &config.namespace_scope);

    if !config.enabled || !config.compile_expressions {
        let expr_ref = parser
            .parse(expr, &mut slab.ps)
            .with_context(|| format!("failed to parse expression `{}`", expr))?
            .from(&slab.ps);

        return expr_ref
            .eval(&slab, &mut namespace)
            .with_context(|| format!("failed to evaluate expression `{}`", expr));
    }

    let expr_ref = parser
        .parse(expr, &mut slab.ps)
        .with_context(|| format!("failed to parse expression `{}`", expr))?
        .from(&slab.ps);

    let compiled = expr_ref.compile(&slab.ps, &mut slab.cs);
    Ok(fasteval::eval_compiled!(compiled, &slab, &mut namespace))
}
