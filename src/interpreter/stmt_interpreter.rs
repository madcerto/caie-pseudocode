use crate::{stmt::Stmt, env::{Environment, Decl, Type}, token::Literal, expr::Expr};

impl super::Interpreter<()> for Stmt {
    fn interpret(&self, env: &mut Environment) {
        match self {
            Stmt::Block(stmts) => for stmt in stmts {
                stmt.interpret(env);
            },
            Stmt::ExprStmt(expr) => {expr.interpret(env);},
            Stmt::Declare(name, dtype) => {
                let dtype = match dtype.interpret(env) {
                    Literal::Type(inner) => inner,
                    _ => panic!("expected type")
                };
                env.declare(name.lexeme.clone(), Decl::new(true, dtype.clone()));
            },
            Stmt::Constant(name, val) => {
                let val = val.interpret(env);
                env.declare(name.lexeme.clone(), Decl::new(false, Type::from_literal(&val)));
                env.assign(name.lexeme.clone(), val);
            },
            Stmt::Assign(name, val) => {
                let val = val.interpret(env);
                let name = match name {
                    Expr::IdentExpr(tkn) => tkn.lexeme.clone(),
                    _ => panic!("expected identifier")
                };
                env.assign(name, val);
            },
            Stmt::ProcCall(name, args) => {
                let mut arg_list = Vec::new();
                for arg in args {
                    arg_list.push(arg.interpret(env));
                }
                env.call_proc(&name.lexeme, arg_list);
            },
            Stmt::Input(expr) => if let Expr::IdentExpr(name) = expr.clone() {
                let mut val = String::new();
                std::io::stdin().read_line(&mut val).unwrap();
                env.assign(name.lexeme.clone(), Literal::String(val.trim().to_string())); // Cut off newline
            } else { panic!("expected identifier") },
            Stmt::Output(exprs) => { for expr in exprs {
                let val = expr.interpret(env);
                match val {
                    Literal::String(val) => print!("{}", val),
                    Literal::Char(val) => print!("{}", val),
                    _ => print!("{}", val.to_string())
                }
            } println!("")}, // print newline at the end
            Stmt::Ret(val) => {
                let val = val.interpret(env);
                env.set_ret(val);
            },
            Stmt::Procedure(name, args, block) => {
                let mut arg_list = Vec::new();
                for arg in args {
                    arg_list.push((arg.0.lexeme.clone(), match arg.1.interpret(env) {
                        Literal::Type(val) => val,
                        _ => panic!("expected type")
                    }, arg.2))
                }
                env.def_proc(&name.lexeme, arg_list, *block.clone())
            },
            Stmt::Function(name, args, ret_type, block) => {
                let mut arg_list = Vec::new();
                for arg in args {
                    arg_list.push((arg.0.lexeme.clone(), match arg.1.interpret(env) {
                        Literal::Type(val) => val,
                        _ => panic!("expected type")
                    }, arg.2))
                }
                let ret_type = match ret_type.interpret(env) {
                    Literal::Type(inner) => inner,
                    _ => panic!("expected type")
                };
                env.def_func(&name.lexeme, arg_list, ret_type, *block.clone())
            },
            Stmt::ForTo(name, val1, val2, step_opt, block) => {
                let val1 = val1.interpret(env);
                let val2 = if let Literal::Int(val) = val2.interpret(env) { val }
                else { panic!("expected integer expression1") };
                let mut step = 1;
                if let Some(val) = step_opt {
                    step = match val.interpret(env) {
                        Literal::Int(val) => val,
                        _ => panic!("expected integer expression2")
                    }
                }

                let mut inner_env = Environment::new(Some(Box::new(env.clone())));
                inner_env.declare(name.lexeme.clone(), Decl::new(true, Type::Int));
                inner_env.assign(name.lexeme.clone(), val1);
                
                loop {
                    block.interpret(&mut inner_env);
                    let prev = if let Literal::Int(val) =
                        inner_env.get_stack(&name.lexeme) { val }
                    else { panic!("expected integer expression3") };
                    let next = prev + step;
                    if next > val2 { break }
                    inner_env.assign(name.lexeme.clone(), Literal::Int(next));
                }

                inner_env.update_parent(env);
            },
            Stmt::IfThen(cond, then_block, else_block) => {
                match cond.interpret(env) {
                    Literal::TRUE => then_block.interpret(env),
                    Literal::FALSE => match else_block {
                        Some(block) => block.interpret(env),
                        None => {}
                    },
                    _ => panic!("expected boolean expression")
                };
            },
            Stmt::Case(val, cases, otherwise) => {
                let val = val.interpret(env);
                for case in cases.into_iter() {
                    if val == case.0.interpret(env) { case.1.interpret(env); return }
                }
                if let Some(stmt) = otherwise {
                    stmt.interpret(env)
                }
            },
            Stmt::Repeat(cond, block) => loop {
                let inner_env = Environment::new(Some(Box::new(env.clone())));
                block.interpret(env);
                match cond.interpret(env) {
                    Literal::TRUE => {},
                    Literal::FALSE => break,
                    _ => panic!("Expected boolean expression")
                }
                inner_env.update_parent(env);
            },
            Stmt::WhileDo(cond, block) => loop {
                match cond.interpret(env) {
                    Literal::TRUE => {},
                    Literal::FALSE => break,
                    _ => panic!("Expected boolean expression")
                }
                block.interpret(env);
            },
        }
    }
}