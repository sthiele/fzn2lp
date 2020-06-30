use anyhow::Result;
use flatzinc::*;
use log::{error, warn};
use std::io::Write;
use std::{
    fs::File,
    io::{self, prelude::*, BufReader, BufWriter},
    path::PathBuf,
};
use structopt::StructOpt;

/// Convert FlatZinc to AnsProlog facts
#[derive(StructOpt, Debug)]
#[structopt(name = "fzn2lp")]
struct Opt {
    /// Input file in flatzinc format
    #[structopt(name = "FILE", parse(from_os_str))]
    file: Option<PathBuf>,
}

fn main() {
    stderrlog::new()
        .module(module_path!())
        .verbosity(2)
        .init()
        .unwrap();
    if let Err(err) = run() {
        error!("{:?}", err);
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let opt = Opt::from_args();
    let mut level = 1;
    let mut constraint_counter = 1;

    if let Some(file) = opt.file {
        let file = File::open(file)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let mut out = std::io::stdout();
            write_fz_stmt(&mut out, &line?, &mut constraint_counter, &mut level)?;
        }
    } else {
        let mut buf = String::new();
        while 0 < io::stdin().read_line(&mut buf)? {
            let out = BufWriter::new(std::io::stdout());
            write_fz_stmt(out, &buf, &mut constraint_counter, &mut level)?;
            buf.clear();
        }
    }
    if level < 5 {
        return Err(FlatZincError::NoSolveItem.into());
    }
    Ok(())
}
use thiserror::Error;
#[derive(Error, Debug)]
pub enum FlatZincError {
    #[error("More than one solve item")]
    MultipleSolveItems,
    #[error("No solve item")]
    NoSolveItem,
    #[error("ParseError: {msg}")]
    ParseError { msg: String },
}
#[test]
fn test_variables() {
    let mut counter = 0;
    let mut level = 0;
    let mut res = Vec::new();
    write_fz_stmt(
        &mut res,
        "var int : a :: output_var = 1;",
        &mut counter,
        &mut level,
    )
    .unwrap();
    assert_eq!(
        std::str::from_utf8(&res).unwrap(),
        "variable_type(\"a\",int).\n\
         variable_value(\"a\",value,1).\n\
         output_var(\"a\").\n"
            .to_string()
    );
    let mut res = Vec::new();
    write_fz_stmt(&mut res, "var 1..3 : a;", &mut counter, &mut level).unwrap();
    assert_eq!(
        std::str::from_utf8(&res).unwrap(),
        "variable_type(\"a\",range,(value,1,value,3)).\n".to_string()
    );
    let mut res = Vec::new();
    write_fz_stmt(&mut res, "var {1,2,3} : a;", &mut counter, &mut level).unwrap();
    assert_eq!(
        std::str::from_utf8(&res).unwrap(),
        "variable_type(\"a\",set,(value,1)).\n\
         variable_type(\"a\",set,(value,2)).\n\
         variable_type(\"a\",set,(value,3)).\n"
            .to_string()
    );
    let mut res = Vec::new();
    write_fz_stmt(&mut res, "var float : b = 1.0;", &mut counter, &mut level).unwrap();
    assert_eq!(
        std::str::from_utf8(&res).unwrap(),
        "variable_type(\"b\",float).\n\
         variable_value(\"b\",value,\"1\").\n"
            .to_string()
    );
    let mut res = Vec::new();
    write_fz_stmt(&mut res, "var 0.5..1.5: b = 1.0;", &mut counter, &mut level).unwrap();
    assert_eq!(
        std::str::from_utf8(&res).unwrap(),
        "variable_type(\"b\",float,(bounds,value,\"0.5\",value,\"1.5\")).\n\
         variable_value(\"b\",value,\"1\").\n"
            .to_string()
    );
    let mut res = Vec::new();
    write_fz_stmt(&mut res, "var bool : c = true;", &mut counter, &mut level).unwrap();
    assert_eq!(
        std::str::from_utf8(&res).unwrap(),
        "variable_type(\"c\",bool).\n\
         variable_value(\"c\",value,true).\n"
            .to_string()
    );
    let mut res = Vec::new();
    write_fz_stmt(
        &mut res,
        "array [1..2] of var int : d = [42,23];",
        &mut counter,
        &mut level,
    )
    .unwrap();
    assert_eq!(
        std::str::from_utf8(&res).unwrap(),
        "variable_type(\"d\",array(2,int)).\n\
         variable_value(\"d\",array,(0,value,42)).\n\
         variable_value(\"d\",array,(1,value,23)).\n"
            .to_string()
    );
    let mut res = Vec::new();
    write_fz_stmt(
        &mut res,
        "array [1..2] of var float : e :: output_array([1..2, 1..2]) = [42.1,23.1];",
        &mut counter,
        &mut level,
    )
    .unwrap();
    assert_eq!(
        std::str::from_utf8(&res).unwrap(),
        "variable_type(\"e\",array(2,float)).\n\
         variable_value(\"e\",array,(0,value,\"42.1\")).\n\
         variable_value(\"e\",array,(1,value,\"23.1\")).\n\
         output_array(\"e\",0,(1,2)).\n\
         output_array(\"e\",1,(1,2)).\n"
            .to_string()
    );
    let mut res = Vec::new();
    write_fz_stmt(
        &mut res,
        "var set of 17..42: f = {17,23};",
        &mut counter,
        &mut level,
    )
    .unwrap();
    assert_eq!(
        std::str::from_utf8(&res).unwrap(),
        "variable_type(\"f\",set_of_int,(range,value,17,value,42)).\n\
         variable_value(\"f\",set,(value,17)).\n\
         variable_value(\"f\",set,(value,23)).\n"
            .to_string()
    );
    let mut res = Vec::new();
    write_fz_stmt(
        &mut res,
        "var set of {17,23,100}: f = {17,23};",
        &mut counter,
        &mut level,
    )
    .unwrap();
    assert_eq!(
        std::str::from_utf8(&res).unwrap(),
        "variable_type(\"f\",set_of_int,(set,value,17)).\n\
         variable_type(\"f\",set_of_int,(set,value,23)).\n\
         variable_type(\"f\",set_of_int,(set,value,100)).\n\
         variable_value(\"f\",set,(value,17)).\n\
         variable_value(\"f\",set,(value,23)).\n"
            .to_string()
    );
    // let mut res = Vec::new(); // TODO: Check if set of floats are allowed
    // write_fz_stmt(
    //     &mut res,
    //     "var set of float: g = {23.1,42.1};",
    //     &mut counter,
    //     &mut level,
    // )
    // .unwrap();
    // assert_eq!(
    //     std::str::from_utf8(&res).unwrap(),
    //     "variable_value(\"g\",set,(value,\"23.1\")).\nvariable_value(\"g\",set,(value,\"42.1\")).\n".to_string()
    // );
    let mut res = Vec::new();
    write_fz_stmt(
        &mut res,
        "array [1..3] of var set of 17..42: h = [{42,17},23..X,{}];", //TODO: check empty set
        &mut counter,
        &mut level,
    )
    .unwrap();
    assert_eq!(
        std::str::from_utf8(&res).unwrap(),
        "variable_type(\"h\",array(3,set_of_int,(range,value,17,value,42))).\n\
         variable_value(\"h\",array,(0,set,(value,42))).\n\
         variable_value(\"h\",array,(0,set,(value,17))).\n\
         variable_value(\"h\",array,(1,range,(value,23,var,\"X\"))).\n\
         variable_value(\"h\",array,(2,empty_set)).\n"
            .to_string()
    );
}
#[test]
fn test_parameters() {
    let mut counter = 0;
    let mut level = 0;
    let mut res = Vec::new();
    write_fz_stmt(&mut res, "int : a = 1;", &mut counter, &mut level).unwrap();
    assert_eq!(
        std::str::from_utf8(&res).unwrap(),
        "parameter_value(\"a\",value,1).\n".to_string()
    );
    let mut res = Vec::new();
    write_fz_stmt(&mut res, "float : b = 1.1;", &mut counter, &mut level).unwrap();
    assert_eq!(
        std::str::from_utf8(&res).unwrap(),
        "parameter_value(\"b\",value,\"1.1\").\n".to_string()
    );
    let mut res = Vec::new();
    write_fz_stmt(&mut res, "bool : c = true;", &mut counter, &mut level).unwrap();
    assert_eq!(
        std::str::from_utf8(&res).unwrap(),
        "parameter_value(\"c\",value,true).\n".to_string()
    );
    let mut res = Vec::new();
    write_fz_stmt(
        &mut res,
        "array [1..2] of int : d = [42,23];",
        &mut counter,
        &mut level,
    )
    .unwrap();
    assert_eq!(
        std::str::from_utf8(&res).unwrap(),
        "parameter_value(\"d\",array,(0,value,42)).\n\
         parameter_value(\"d\",array,(1,value,23)).\n"
            .to_string()
    );
    let mut res = Vec::new();
    write_fz_stmt(
        &mut res,
        "array [1..2] of float : e = [42.1,23.0];",
        &mut counter,
        &mut level,
    )
    .unwrap();
    assert_eq!(
        std::str::from_utf8(&res).unwrap(),
        "parameter_value(\"e\",array,(0,value,\"42.1\")).\n\
         parameter_value(\"e\",array,(1,value,\"23\")).\n"
            .to_string()
    );
    let mut res = Vec::new();
    write_fz_stmt(
        &mut res,
        "set of int: f = 23..42;",
        &mut counter,
        &mut level,
    )
    .unwrap();
    assert_eq!(
        std::str::from_utf8(&res).unwrap(),
        "parameter_value(\"f\",range,(value,23,value,42)).\n".to_string()
    );
    // let mut res = Vec::new(); // TODO: check if/where set of floats are allowed
    // write_fz_stmt(
    //     &mut res,
    //     "set of float : g = {42.1,23.0};",
    //     &mut counter,
    //     &mut level,
    // )
    // .unwrap();
    // assert_eq!(
    //     std::str::from_utf8(&res).unwrap(),
    //     "parameter_value(\"g\",set,(value,\"23\"))).\nparameter_value(\"g\",set,(value,\"42.1\"))).\n".to_string()
    // );
    let mut res = Vec::new();
    write_fz_stmt(
        &mut res,
        "array [1..3] of set of int : h = [{42,17},1..5,{}];",
        &mut counter,
        &mut level,
    )
    .unwrap();
    assert_eq!(
        std::str::from_utf8(&res).unwrap(),
        "parameter_value(\"h\",array,(0,set,(value,42))).\n\
         parameter_value(\"h\",array,(0,set,(value,17))).\n\
         parameter_value(\"h\",array,(1,range,(value,1,value,5))).\n\
         parameter_value(\"h\",array,(2,empty_set)).\n"
            .to_string()
    );
}
#[test]
fn test_constraint() {
    let mut counter = 0;
    let mut level = 0;
    let mut res = Vec::new();
    write_fz_stmt(
        &mut res,
        "constraint bla(42,42.1,true,a,[42,17,X],{X,34},37..48,[{42,17},17..34,{X,Y}]);",
        &mut counter,
        &mut level,
    )
    .unwrap();
    assert_eq!(
        std::str::from_utf8(&res).unwrap(),
        "constraint(c1,\"bla\").\n\
         constraint_value(c1,0,value,42).\n\
         constraint_value(c1,1,value,\"42.1\").\n\
         constraint_value(c1,2,value,true).\n\
         constraint_value(c1,3,var,\"a\").\n\
         constraint_value(c1,4,array,(0,value,42)).\n\
         constraint_value(c1,4,array,(1,value,17)).\n\
         constraint_value(c1,4,array,(2,var,\"X\")).\n\
         constraint_value(c1,5,set,(var,\"X\")).\n\
         constraint_value(c1,5,set,(value,34)).\n\
         constraint_value(c1,6,range,(value,37,value,48)).\n\
         constraint_value(c1,7,array,(0,set,(value,42))).\n\
         constraint_value(c1,7,array,(0,set,(value,17))).\n\
         constraint_value(c1,7,array,(1,range,(value,17,value,34))).\n\
         constraint_value(c1,7,array,(2,set,(var,\"X\"))).\n\
         constraint_value(c1,7,array,(2,set,(var,\"Y\"))).\n"
            .to_string()
    );
}
fn write_fz_stmt(
    mut out: impl Write,
    input: &str,
    constraint_counter: &mut usize,
    level: &mut i32,
) -> Result<()> {
    match fz_statement::<VerboseError<&str>>(&input) {
        Ok((_rest, stmt)) => {
            match stmt {
                FzStmt::Comment(s) => {
                    writeln!(out, "%{}", s)?;
                }
                FzStmt::Predicate(pred) => {
                    if *level > 1 {
                        warn!("Statements in wrong order.");
                    }
                    write_predicate(out, &pred)?;
                }
                FzStmt::Parameter(p) => {
                    if *level > 2 {
                        warn!("Statements in wrong order.");
                    } else {
                        *level = 2;
                    }
                    write_par_decl_item(out, &p)?;
                }
                FzStmt::Variable(d) => {
                    if *level > 3 {
                        warn!("Statements in wrong order.");
                    } else {
                        *level = 3;
                    }
                    write_var_decl_item(out, &d)?;
                }
                FzStmt::Constraint(c) => {
                    if *level > 4 {
                        warn!("Statements in wrong order.");
                    } else {
                        *level = 4;
                    }
                    *constraint_counter += 1;
                    write_constraint(out, &c, *constraint_counter)?;
                }
                FzStmt::SolveItem(i) => {
                    if *level > 4 {
                        return Err(FlatZincError::MultipleSolveItems.into());
                    }
                    *level = 5;
                    write_solve_item(out, &i)?;
                }
            }
            Ok(())
        }
        Err(Err::Error(e)) | Err(Err::Failure(e)) => {
            let bla = convert_error(&input, e);
            Err(FlatZincError::ParseError { msg: bla }.into())
        }
        Err(e) => Err(FlatZincError::ParseError {
            msg: format!("{}", e),
        }
        .into()),
    }
}

fn write_predicate(mut buf: impl Write, predicate: &PredicateItem) -> Result<()> {
    writeln!(buf, "predicate({}).", identifier(&predicate.id))?;
    for (pos, p) in predicate.parameters.iter().enumerate() {
        match p {
            (PredParType::Basic(par_type), id) => {
                for element in basic_pred_par_type(&par_type) {
                    writeln!(
                        buf,
                        "predicate_parameter({},{},{},{}).",
                        identifier(&predicate.id),
                        pos,
                        identifier(id),
                        element
                    )?;
                }
            }
            (PredParType::Array { ix, par_type }, id) => {
                for element in basic_pred_par_type(&par_type) {
                    writeln!(
                        buf,
                        "predicate_parameter({},{},{},{}).",
                        identifier(&predicate.id),
                        pos,
                        identifier(id),
                        array_type(&pred_index(&ix), &element)
                    )?;
                }
            }
        }
    }
    Ok(())
}
fn write_par_decl_item(mut buf: impl Write, item: &ParDeclItem) -> Result<()> {
    match item {
        ParDeclItem::Bool { id, bool } => {
            // writeln!(buf, "variable_type({},bool).", identifier(id))?;
            writeln!(
                buf,
                "parameter_value({},value,{}).",
                identifier(id),
                bool_literal(*bool)
            )?;
        }
        ParDeclItem::Int { id, int } => {
            // writeln!(buf, "variable_type({},int).", identifier(id))?;
            writeln!(
                buf,
                "parameter_value({},value,{}).",
                identifier(id),
                int_literal(int)
            )?;
        }
        ParDeclItem::Float { id, float } => {
            // writeln!(buf, "variable_type({},float).", identifier(id))?;
            writeln!(
                buf,
                "parameter_value({},value,{}).",
                identifier(id),
                float_literal(*float)
            )?;
        }
        ParDeclItem::SetOfInt {
            id,
            set_literal: sl,
        } => {
            // writeln!(buf, "variable_type({},set_of_int).", identifier(id))?;
            let set = dec_set_literal(sl);
            for element in set {
                writeln!(buf, "parameter_value({},{}).", identifier(id), element)?;
            }
        }
        ParDeclItem::ArrayOfBool { ix, id, v } => {
            // writeln!(
            //     buf,
            //     "variable_type({},{}).",
            //     identifier(id),
            //     array_type(&index(ix), "bool")
            // )?;
            for (pos, e) in v.iter().enumerate() {
                writeln!(
                    buf,
                    "parameter_value({},array,({},value,{})).",
                    identifier(id),
                    pos,
                    bool_literal(*e)
                )?;
            }
        }
        ParDeclItem::ArrayOfInt { ix, id, v } => {
            // writeln!(
            //     buf,
            //     // "variable_type({},{}).",
            //     identifier(id),
            //     array_type(&index(ix), "int")
            // )?;
            for (pos, e) in v.iter().enumerate() {
                writeln!(
                    buf,
                    "parameter_value({},array,({},value,{})).",
                    identifier(id),
                    pos,
                    int_literal(e)
                )?;
            }
        }
        ParDeclItem::ArrayOfFloat { ix, id, v } => {
            // writeln!(
            //     buf,
            //     // "variable_type({},{}).",
            //     identifier(id),
            //     array_type(&index(ix), "float"),
            // )?;
            for (pos, e) in v.iter().enumerate() {
                writeln!(
                    buf,
                    "parameter_value({},array,({},value,{})).",
                    identifier(id),
                    pos,
                    float_literal(*e)
                )?;
            }
        }
        ParDeclItem::ArrayOfSet { ix, id, v } => {
            // writeln!(
            //     buf,
            //     "variable_type({},{}).",
            //     identifier(id),
            //     array_type(&index(ix), "set")
            // )?;
            for (pos, e) in v.iter().enumerate() {
                let set = dec_set_literal(e);
                for element in set {
                    writeln!(
                        buf,
                        "parameter_value({},array,({},{})).",
                        identifier(id),
                        pos,
                        element
                    )?;
                }
            }
        }
    }
    Ok(())
}
fn write_var_decl_item(mut buf: impl Write, item: &VarDeclItem) -> Result<()> {
    match item {
        VarDeclItem::Bool { id, expr, annos } => {
            writeln!(buf, "variable_type({},bool).", identifier(id))?;
            if let Some(expr) = expr {
                writeln!(
                    buf,
                    "variable_value({},{}).",
                    identifier(id),
                    bool_expr(expr)
                )?;
            }
            write_output_var(buf, id, annos)?;
        }
        VarDeclItem::Int { id, expr, annos } => {
            writeln!(buf, "variable_type({},int).", identifier(id))?;
            if let Some(expr) = expr {
                writeln!(
                    buf,
                    "variable_value({},{}).",
                    identifier(id),
                    int_expr(expr)
                )?;
            }
            write_output_var(buf, id, annos)?;
        }
        VarDeclItem::IntInRange {
            id,
            lb,
            ub,
            expr,
            annos,
        } => {
            writeln!(
                buf,
                "variable_type({},{}).",
                identifier(id),
                int_in_range(lb, ub)
            )?;
            if let Some(expr) = expr {
                writeln!(
                    buf,
                    "variable_value({},{}).",
                    identifier(id),
                    int_expr(expr)
                )?;
            }
            write_output_var(buf, id, annos)?;
        }
        VarDeclItem::IntInSet {
            id,
            set,
            expr,
            annos,
        } => {
            for element in set {
                writeln!(
                    buf,
                    "variable_type({},set,(value,{})).",
                    identifier(id),
                    element,
                )?;
            }
            if let Some(expr) = expr {
                writeln!(
                    buf,
                    "variable_value({},{}).",
                    identifier(id),
                    int_expr(expr)
                )?;
            }
            write_output_var(buf, id, annos)?;
        }
        VarDeclItem::Float { id, expr, annos } => {
            writeln!(buf, "variable_type({},float).", identifier(id))?;
            if let Some(expr) = expr {
                writeln!(
                    buf,
                    "variable_value({},{}).",
                    identifier(id),
                    float_expr(expr)
                )?;
            }
            write_output_var(buf, id, annos)?;
        }
        VarDeclItem::BoundedFloat {
            id,
            lb,
            ub,
            expr,
            annos,
        } => {
            writeln!(
                buf,
                "variable_type({},{}).",
                identifier(id),
                bounded_float(*lb, *ub)
            )?;
            if let Some(expr) = expr {
                writeln!(
                    buf,
                    "variable_value({},{}).",
                    identifier(id),
                    float_expr(expr)
                )?;
            }
            write_output_var(buf, id, annos)?;
        }
        VarDeclItem::SetOfInt { id, annos, expr } => {
            writeln!(buf, "variable_type({},set_of_int).", identifier(id))?;
            if let Some(expr) = expr {
                let set = dec_set_expr(expr);
                for element in set {
                    writeln!(buf, "variable_value({},{}).", identifier(id), element)?;
                }
            }
            write_output_var(buf, id, annos)?;
        }
        VarDeclItem::SubSetOfIntRange {
            id,
            lb,
            ub,
            expr,
            annos,
        } => {
            writeln!(
                buf,
                "variable_type({},{}).",
                identifier(id),
                subset_of_int_range(lb, ub),
            )?;
            if let Some(expr) = expr {
                let set = dec_set_expr(expr);
                for element in set {
                    writeln!(buf, "variable_value({},{}).", identifier(id), element)?;
                }
            }
            write_output_var(buf, id, annos)?;
        }
        VarDeclItem::SubSetOfIntSet {
            id,
            set,
            expr,
            annos,
        } => {
            for element in set {
                writeln!(
                    buf,
                    "variable_type({},set_of_int,(set,value,{})).",
                    identifier(id),
                    element,
                )?;
            }
            if let Some(expr) = expr {
                let set = dec_set_expr(expr);
                for element in set {
                    writeln!(buf, "variable_value({},{}).", identifier(id), element)?;
                }
            }
            write_output_var(buf, id, annos)?;
        }
        VarDeclItem::ArrayOfBool {
            id,
            ix,
            array_expr,
            annos,
        } => {
            writeln!(
                buf,
                "variable_type({},{}).",
                identifier(id),
                array_type(&index(ix), "bool")
            )?;
            match array_expr {
                Some(ArrayOfBoolExpr::Array(v)) => {
                    for (pos, e) in v.iter().enumerate() {
                        writeln!(
                            buf,
                            "variable_value({},array,({},{})).",
                            identifier(id),
                            pos,
                            bool_expr(e)
                        )?;
                    }
                }
                Some(ArrayOfBoolExpr::VarParIdentifier(id2)) => {
                    writeln!(
                        buf,
                        "variable_value({},value,{}).",
                        identifier(id),
                        identifier(id2)
                    )?;
                }
                None => {}
            }
            write_output_array(buf, id, annos)?;
        }
        VarDeclItem::ArrayOfInt {
            id,
            ix,
            array_expr,
            annos,
        } => {
            writeln!(
                buf,
                "variable_type({},{}).",
                identifier(id),
                array_type(&index(ix), "int"),
            )?;
            match array_expr {
                Some(ArrayOfIntExpr::Array(v)) => {
                    for (pos, e) in v.iter().enumerate() {
                        writeln!(
                            buf,
                            "variable_value({},array,({},{})).",
                            identifier(id),
                            pos,
                            int_expr(e)
                        )?;
                    }
                }
                Some(ArrayOfIntExpr::VarParIdentifier(id2)) => {
                    writeln!(
                        buf,
                        "variable_value({},value,{}).",
                        identifier(id),
                        identifier(id2)
                    )?;
                }
                None => {}
            }
            write_output_array(buf, id, annos)?;
        }
        VarDeclItem::ArrayOfIntInRange {
            id,
            ix,
            lb,
            ub,
            array_expr,
            annos,
        } => {
            writeln!(
                buf,
                "variable_type({},{}).",
                identifier(id),
                array_type(&index(ix), &int_in_range(lb, ub)),
            )?;
            match array_expr {
                Some(ArrayOfIntExpr::Array(v)) => {
                    for (pos, e) in v.iter().enumerate() {
                        writeln!(
                            buf,
                            "variable_value({},array,({},{})).",
                            identifier(id),
                            pos,
                            int_expr(e)
                        )?;
                    }
                }
                Some(ArrayOfIntExpr::VarParIdentifier(id2)) => {
                    writeln!(
                        buf,
                        "variable_value({},value,{}).",
                        identifier(id),
                        identifier(id2)
                    )?;
                }
                None => {}
            }
            write_output_array(buf, id, annos)?;
        }
        VarDeclItem::ArrayOfIntInSet {
            id,
            ix,
            set,
            array_expr,
            annos,
        } => {
            for element in set {
                writeln!(
                    buf,
                    "variable_type({},{}).",
                    identifier(id),
                    array_type(&index(ix), &format!("set,(value,{})", element))
                )?;
            }
            match array_expr {
                Some(ArrayOfIntExpr::Array(v)) => {
                    for (pos, e) in v.iter().enumerate() {
                        writeln!(
                            buf,
                            "variable_value({},array,({},{})).",
                            identifier(id),
                            pos,
                            int_expr(e)
                        )?;
                    }
                }
                Some(ArrayOfIntExpr::VarParIdentifier(id2)) => {
                    writeln!(
                        buf,
                        "variable_value({},value,{}).",
                        identifier(id),
                        identifier(id2)
                    )?;
                }
                None => {}
            }
            write_output_array(buf, id, annos)?;
        }
        VarDeclItem::ArrayOfFloat {
            id,
            ix,
            annos,
            array_expr,
        } => {
            writeln!(
                buf,
                "variable_type({},{}).",
                identifier(id),
                array_type(&index(ix), "float"),
            )?;
            match array_expr {
                Some(ArrayOfFloatExpr::Array(v)) => {
                    for (pos, e) in v.iter().enumerate() {
                        writeln!(
                            buf,
                            "variable_value({},array,({},{})).",
                            identifier(id),
                            pos,
                            float_expr(e)
                        )?;
                    }
                }
                Some(ArrayOfFloatExpr::VarParIdentifier(id2)) => {
                    writeln!(
                        buf,
                        "variable_value({},value,{}).",
                        identifier(id),
                        identifier(id2)
                    )?;
                }
                None => {}
            }
            write_output_array(buf, id, annos)?;
        }
        VarDeclItem::ArrayOfBoundedFloat {
            id,
            ix,
            lb,
            ub,
            array_expr,
            annos,
        } => {
            writeln!(
                buf,
                "variable_type({},{}).",
                identifier(id),
                array_type(&index(ix), &bounded_float(*lb, *ub)),
            )?;
            match array_expr {
                Some(ArrayOfFloatExpr::Array(v)) => {
                    for (pos, e) in v.iter().enumerate() {
                        writeln!(
                            buf,
                            "variable_value({},array,({},{})).",
                            identifier(id),
                            pos,
                            float_expr(e)
                        )?;
                    }
                }
                Some(ArrayOfFloatExpr::VarParIdentifier(id2)) => {
                    writeln!(
                        buf,
                        "variable_value({},value,{}).",
                        identifier(id),
                        identifier(id2)
                    )?;
                }
                None => {}
            }
            write_output_array(buf, id, annos)?;
        }
        VarDeclItem::ArrayOfSet {
            id,
            ix,
            array_expr,
            annos,
        } => {
            writeln!(
                buf,
                "variable_type({},{}).",
                identifier(id),
                array_type(&index(ix), "set"),
            )?;
            match array_expr {
                Some(ArrayOfSetExpr::Array(v)) => {
                    for (pos, e) in v.iter().enumerate() {
                        let set = dec_set_expr(e);
                        for element in set {
                            writeln!(
                                buf,
                                "variable_value({},array,({},{})).",
                                identifier(id),
                                pos,
                                element
                            )?;
                        }
                    }
                }
                Some(ArrayOfSetExpr::VarParIdentifier(id2)) => {
                    writeln!(
                        buf,
                        "variable_value({},value,{}).",
                        identifier(id),
                        identifier(id2)
                    )?;
                }
                None => {}
            }
            write_output_array(buf, id, annos)?;
        }
        VarDeclItem::ArrayOfSubSetOfIntRange {
            id,
            ix,
            lb,
            ub,
            array_expr,
            annos,
        } => {
            writeln!(
                buf,
                "variable_type({},{}).",
                identifier(id),
                array_type(&index(ix), &subset_of_int_range(lb, ub))
            )?;
            match array_expr {
                Some(ArrayOfSetExpr::Array(v)) => {
                    for (pos, e) in v.iter().enumerate() {
                        let set = dec_set_expr(e);
                        for element in set {
                            writeln!(
                                buf,
                                "variable_value({},array,({},{})).",
                                identifier(id),
                                pos,
                                element
                            )?;
                        }
                    }
                }
                Some(ArrayOfSetExpr::VarParIdentifier(id2)) => {
                    writeln!(
                        buf,
                        "variable_value({},value,{}).",
                        identifier(id),
                        identifier(id2)
                    )?;
                }
                None => {}
            }
            write_output_array(buf, id, annos)?;
        }
        VarDeclItem::ArrayOfSubSetOfIntSet {
            id,
            ix,
            set,
            array_expr,
            annos,
        } => {
            for element in set {
                writeln!(
                    buf,
                    "variable_type({},{}).",
                    identifier(id),
                    array_type(&index(ix), &format!("set_of_int,(set,value,{})", element)),
                )?;
            }
            match array_expr {
                Some(ArrayOfSetExpr::Array(v)) => {
                    for (pos, se) in v.iter().enumerate() {
                        for e in dec_set_expr(se) {
                            writeln!(
                                buf,
                                "variable_value({},array,({},{})).",
                                identifier(id),
                                pos,
                                e
                            )?;
                        }
                    }
                }
                Some(ArrayOfSetExpr::VarParIdentifier(id2)) => {
                    writeln!(
                        buf,
                        "variable_value({},value,{}).",
                        identifier(id),
                        identifier(id2)
                    )?;
                }
                None => {}
            }
            write_output_array(buf, id, annos)?;
        }
    }
    Ok(())
}
fn basic_var_type(t: &BasicVarType) -> Vec<String> {
    match t {
        BasicVarType::BasicType(BasicType::Bool) => vec!["bool".to_string()],
        BasicVarType::BasicType(BasicType::Int) => vec!["int".to_string()],
        BasicVarType::IntInRange(lb, ub) => vec![int_in_range(lb, ub)],
        BasicVarType::IntInSet(set) => int_in_set(set),
        BasicVarType::BasicType(BasicType::Float) => vec!["float".to_string()],
        BasicVarType::BoundedFloat(lb, ub) => vec![bounded_float(*lb, *ub)],
        BasicVarType::SubSetOfIntRange(lb, ub) => vec![subset_of_int_range(lb, ub)],
        BasicVarType::SubSetOfIntSet(set) => subset_of_int_set(set),
    }
}
fn int_in_range(lb: &i128, ub: &i128) -> String {
    format!("range,(value,{},value,{})", lb, ub)
}
fn int_in_set(set: &[i128]) -> Vec<String> {
    let mut ret = vec![];
    for integer in set {
        ret.push(format!("set,(value,{})", integer))
    }
    ret
}
fn float_in_set(set: &[f64]) -> Vec<String> {
    let mut ret = vec![];
    for float in set {
        ret.push(format!("float_in_set({})", float))
    }
    ret
}
fn bounded_float(lb: f64, ub: f64) -> String {
    format!(
        "float,(bounds,value,{},value,{})",
        float_literal(lb),
        float_literal(ub)
    )
}
fn subset_of_int_range(lb: &i128, ub: &i128) -> String {
    format!(
        "set_of_int,(range,value,{},value,{})",
        int_literal(lb),
        int_literal(ub)
    )
}
fn subset_of_int_set(set: &[i128]) -> Vec<String> {
    let mut ret = vec![];
    for i in set {
        ret.push(format!("set_of_int,(set,value,{})", int_literal(i)))
    }
    ret
}
fn write_constraint(mut buf: impl Write, c: &ConstraintItem, i: usize) -> Result<()> {
    writeln!(buf, "constraint(c{},{}).", i, identifier(&c.id))?;
    for (cpos, ce) in c.exprs.iter().enumerate() {
        match ce {
            Expr::VarParIdentifier(id) => {
                // writeln!(buf, "constraint_type_at(c{},{},var_par).", i, cpos)?;
                writeln!(
                    buf,
                    "constraint_value(c{},{},var,{}).",
                    i,
                    cpos,
                    identifier(id)
                )?;
            }
            Expr::Bool(e) => {
                // writeln!(buf, "constraint_type_at(c{},{},bool).", i, cpos)?;
                writeln!(
                    buf,
                    "constraint_value(c{},{},value,{}).",
                    i,
                    cpos,
                    bool_literal(*e)
                )?;
            }
            Expr::Int(e) => {
                // writeln!(buf, "constraint_type_at(c{},{},int).", i, cpos)?;
                writeln!(
                    buf,
                    "constraint_value(c{},{},value,{}).",
                    i,
                    cpos,
                    int_literal(e)
                )?;
            }
            Expr::Float(e) => {
                // writeln!(buf, "constraint_type_at(c{},{},float).", i, cpos)?;
                writeln!(
                    buf,
                    "constraint_value(c{},{},value,{}).",
                    i,
                    cpos,
                    float_literal(*e)
                )?;
            }
            Expr::Set(e) => {
                // writeln!(buf, "constraint_type_at(c{},{},set).", i, cpos)?;
                let set = dec_set_literal_expr(e);
                for element in set {
                    writeln!(buf, "constraint_value(c{},{},{}).", i, cpos, element)?;
                }
            }
            Expr::ArrayOfBool(v) => {
                // writeln!(buf, "constraint_type_at(c{},{},array).", i, cpos)?;
                for (apos, ae) in v.iter().enumerate() {
                    writeln!(
                        buf,
                        "constraint_value(c{},{},array,({},{})).",
                        i,
                        cpos,
                        apos,
                        bool_expr(&ae)
                    )?;
                }
            }
            Expr::ArrayOfInt(v) => {
                // writeln!(buf, "constraint_type_at(c{},{},array).", i, cpos)?;
                for (apos, ae) in v.iter().enumerate() {
                    writeln!(
                        buf,
                        "constraint_value(c{},{},array,({},{})).",
                        i,
                        cpos,
                        apos,
                        int_expr(&ae)
                    )?;
                }
            }
            Expr::ArrayOfFloat(v) => {
                // writeln!(buf, "constraint_type_at(c{},{},array).", i, cpos,)?;
                for (apos, ae) in v.iter().enumerate() {
                    writeln!(
                        buf,
                        "constraint_value(c{},{},array,({},{})).",
                        i,
                        cpos,
                        apos,
                        float_expr(&ae)
                    )?;
                }
            }
            Expr::ArrayOfSet(v) => {
                // writeln!(buf, "constraint_type_at(c{},{},array_of_set).", i, cpos)?;
                for (apos, ae) in v.iter().enumerate() {
                    let set = dec_set_expr(ae);
                    for element in set {
                        writeln!(
                            buf,
                            "constraint_value(c{},{},array,({},{})).",
                            i, cpos, apos, element
                        )?;
                    }
                }
            }
        }
    }
    Ok(())
}
fn write_solve_item(mut buf: impl Write, i: &SolveItem) -> Result<()> {
    match &i.goal {
        Goal::Satisfy => {
            writeln!(buf, "solve(satisfy).")?;
        }
        Goal::OptimizeBool(ot, e) => {
            writeln!(buf, "solve({},{}).", opt_type(ot), bool_expr(&e))?;
        }
        Goal::OptimizeInt(ot, e) => {
            writeln!(buf, "solve({},{}).", opt_type(ot), int_expr(&e))?;
        }
        Goal::OptimizeFloat(ot, e) => {
            writeln!(buf, "solve({},{}).", opt_type(ot), float_expr(&e))?;
        }
        Goal::OptimizeSet(ot, e) => {
            let set = dec_set_expr(e);
            for element in set {
                writeln!(buf, "solve({},{}).", opt_type(ot), element)?;
            }
        }
    }
    Ok(())
}
fn basic_par_type(t: &BasicParType) -> String {
    match t {
        BasicParType::BasicType(BasicType::Bool) => "bool".to_string(),
        BasicParType::BasicType(BasicType::Float) => "float".to_string(),
        BasicParType::BasicType(BasicType::Int) => "int".to_string(),
        BasicParType::SetOfInt => "set_of_int".to_string(),
    }
}
fn basic_pred_par_type(t: &BasicPredParType) -> Vec<String> {
    match t {
        BasicPredParType::BasicParType(t) => vec![basic_par_type(t)],
        BasicPredParType::BasicVarType(t) => basic_var_type(t),
        BasicPredParType::VarSetOfInt => vec!["set_of_int".to_string()],
        BasicPredParType::BoundedFloat(lb, ub) => vec![bounded_float(*lb, *ub)],
        BasicPredParType::IntInRange(lb, ub) => vec![int_in_range(lb, ub)],
        BasicPredParType::IntInSet(set) => int_in_set(set),
        BasicPredParType::FloatInSet(set) => float_in_set(set),
        BasicPredParType::SubSetOfIntRange(lb, ub) => vec![subset_of_int_range(lb, ub)],
        BasicPredParType::SubSetOfIntSet(set) => subset_of_int_set(set),
    }
}
fn array_type(idx: &str, element_type: &str) -> String {
    format!("array({},{})", idx, element_type)
}
fn opt_type(opt_type: &OptimizationType) -> String {
    match opt_type {
        OptimizationType::Minimize => "minimize".to_string(),
        OptimizationType::Maximize => "maximize".to_string(),
    }
}
fn index(IndexSet(i): &IndexSet) -> String {
    i.to_string()
}
fn identifier(id: &str) -> String {
    format!("\"{}\"", id)
}
fn pred_index(is: &PredIndexSet) -> String {
    match is {
        PredIndexSet::IndexSet(i) => i.to_string(),
        PredIndexSet::Int => "int".to_string(),
    }
}
fn bool_expr(e: &BoolExpr) -> String {
    match e {
        BoolExpr::Bool(b) => format!("value,{}", bool_literal(*b)),
        BoolExpr::VarParIdentifier(id) => format!("var,{}", identifier(id)),
    }
}
fn bool_literal(b: bool) -> String {
    if b {
        "true".to_string()
    } else {
        "false".to_string()
    }
}
fn int_expr(e: &IntExpr) -> String {
    match e {
        IntExpr::Int(i) => format!("value,{}", int_literal(i)),
        IntExpr::VarParIdentifier(id) => format!("var,{}", identifier(id)),
    }
}
fn int_literal(i: &i128) -> String {
    i.to_string()
}
fn float_expr(e: &FloatExpr) -> String {
    match e {
        FloatExpr::Float(f) => format!("value,{}", float_literal(*f)),
        FloatExpr::VarParIdentifier(id) => format!("var,{}", identifier(id)),
    }
}
fn float_literal(f: f64) -> String {
    format!("\"{}\"", f)
}
fn dec_set_expr(e: &SetExpr) -> Vec<String> {
    match e {
        SetExpr::Set(sl) => dec_set_literal_expr(sl),
        SetExpr::VarParIdentifier(id) => vec![format!("var,{}", identifier(id))],
    }
}
fn dec_set_literal_expr(l: &SetLiteralExpr) -> Vec<String> {
    let mut ret = Vec::new();
    match l {
        SetLiteralExpr::BoundedFloat(f1, f2) => ret.push(format!(
            "float_bound,({},{})",
            float_expr(f1),
            float_expr(f2)
        )),
        SetLiteralExpr::IntInRange(i1, i2) => {
            ret.push(format!("range,({},{})", int_expr(i1), int_expr(i2)))
        }
        SetLiteralExpr::SetFloats(v) => {
            if v.is_empty() {
                ret.push("empty_set".to_string());
            } else {
                for f in v {
                    ret.push(format!("set,({})", float_expr(f)));
                }
            }
        }
        SetLiteralExpr::SetInts(v) => {
            if v.is_empty() {
                ret.push("empty_set".to_string());
            } else {
                for i in v {
                    ret.push(format!("set,({})", int_expr(i)));
                }
            }
        }
    }
    ret
}
fn dec_set_literal(l: &SetLiteral) -> Vec<String> {
    let mut ret = Vec::new();
    match l {
        SetLiteral::BoundedFloat(f1, f2) => ret.push(format!(
            "bounds,(value,{},value,{})",
            float_literal(*f1),
            float_literal(*f2)
        )),
        SetLiteral::IntRange(i1, i2) => ret.push(format!("range,(value,{},value,{})", i1, i2)),
        SetLiteral::SetFloats(v) => {
            if v.is_empty() {
                ret.push("empty_set".to_string());
            } else {
                for f in v {
                    ret.push(format!("set,(value,{})", float_literal(*f)));
                }
            }
        }
        SetLiteral::SetInts(v) => {
            if v.is_empty() {
                ret.push("empty_set".to_string());
            } else {
                for f in v {
                    ret.push(format!("set,(value,{})", f));
                }
            }
        }
    }
    ret
}
fn write_output_var(mut buf: impl Write, id: &str, annos: &[Annotation]) -> Result<()> {
    for a in annos {
        if a.id == "output_var" {
            writeln!(buf, "output_var({}).", identifier(id))?;
            break;
        }
    }
    Ok(())
}
fn write_output_array(mut buf: impl Write, id: &str, annos: &[Annotation]) -> Result<()> {
    for a in annos {
        if a.id == "output_array" {
            match a.expressions.get(0) {
                Some(AnnExpr::Expr(Expr::ArrayOfSet(v))) => {
                    for (pos, e) in v.iter().enumerate() {
                        match e {
                            SetExpr::Set(SetLiteralExpr::IntInRange(
                                IntExpr::Int(lb),
                                IntExpr::Int(ub),
                            )) => {
                                writeln!(
                                    buf,
                                    "output_array({},{},({},{})).",
                                    identifier(id),
                                    pos,
                                    int_literal(lb),
                                    int_literal(ub)
                                )?;
                            }
                            x => panic!("unexpected set expr: {:?}", x),
                        }
                    }
                }
                _ => panic!("expected an array of index sets!"),
            }
            break;
        }
    }
    Ok(())
}
