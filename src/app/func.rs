// nicer printing of functions and variables stored within our num_parser context object
// parser() turns num_parser expressions like `Binary(Func("sin", [Var("x")]), Plus, Literal(Int(1)))` into "sin(x)+1"
// funcs/vars_to_strings takes a list of similar expressions and their corresponding names and spits out the strings you see in the left-hand panels
// vars_to_strings will also find float approximations for each expression, when possible
// when not possible (e.g. variable x = y+1 defined in terms of unknown y) it returns the error
use num_parser::{Expression::*, *};
pub fn vars_to_strings(context: &Context) -> Vec<String> {
    // More complex than the functions one
    // Same principle but we also want to evaluate e.g. sin(10) as well as having the closed form
    // and if the closed form is the same as the evaluation (e.g. variable is an integer), then don't display it
    let name_bodies = context
        .variables
        .iter()
        .map(|(name, body)| (name, parser(&body)));
    let mut out = vec![];
    for (name, body) in name_bodies {
        match num_parser::eval_with_static_context(&*body, context) {
            Ok(val) => {
                if let Value::Int(_) = val {
                    // No need to give float approximation if answer is exact integer
                    // and "x = 2 = 2" looks silly
                    out.push(format!("\n{} = {}", name, body))
                } else {
                    out.push(format!("\n{} = {} = {}", name, body, val))
                }
            }
            Err(msg) => out.push(format!("\n{} = {} ({})", name, body, msg)),
        }
    }
    out
}
pub fn funcs_to_strings(context: &Context) -> Vec<String> {
    // we have a hashmap of functions. Roughly speaking f(x,y) = sin(xy) has key "f" and value ( [x,y], func(sin, func(x, times, y)))
    // iterate through the hashmap, parse the body, join the variables with commas. Newlines for seperation
    context
        .functions
        .iter()
        .map(|(name, (vars, body))| format!("\n{}({}) = {}", name, vars.join(","), parser(&body)))
        .collect::<Vec<String>>()
}
pub fn parser(body: &Expression) -> String {
    // I'm relying on num_parser to be well written to avoid recursion issues
    // No problems yet and I've done a bit of testing
    // But there's no depth limit etc.
    match body {
        Binary(expr1, token, expr2) => {
            format!("{}{}{}", parser(&*expr1), token, parser(&*expr2))
        }
        Unary(token, expr) => format!("{}{}", token, parser(&*expr)),
        Var(s) => s.clone(),
        Func(name, exprs) => {
            format!(
                "{}({})",
                name,
                exprs
                    .iter()
                    .map(|e| parser(&*e))
                    .collect::<Vec<String>>()
                    .join(",")
            )
        }
        Literal(val) => val.to_string(),
        Union(exprs) => exprs
            .iter()
            .map(|e| parser(&*e))
            .collect::<Vec<String>>()
            .join(","),
    }
}
