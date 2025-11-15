use crate::ast::{Comparison, Expression, RawFunctionCallStatement, Term, Unary};
use crate::ast::{ComparisonOperator, ExpressionOperator, LogicalOperator, TermOperator};
use crate::ast::{
    DataType, IfStatement, PrintStatement, ReturnStatement, Value, VariableAssignmentStatement,
    VariableDeclarationStatement, WhileStatement,
};
use crate::ast::{FunctionDeclarationStatement, FunctionSymbol, Statement, ValueType};
use std::fmt;

// Implement Display for DataType so we can call .to_string() on it
// TODO: move this shit it should not be here
impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataType::Number => write!(f, "int"),
            DataType::String => write!(f, "char*"),
            DataType::Void => write!(f, "void"),
            DataType::Boolean => write!(f, "bool"),
            DataType::Unknown => write!(f, "auto"),
            DataType::Invalid => write!(f, "/* invalid type */"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value_type {
            ValueType::InlineNumber => write!(f, "{}", &self.raw_text),
            ValueType::InlineString => write!(f, "\"{}\"", &self.raw_text),
            ValueType::Variable => write!(f, "{}", &self.raw_text),
            ValueType::FunctionCall => {
                // NOTE:
                // We need to be given the statement struct of this
                // value for us to properly include arguments.
                // This may jsut workout thoughl
                // For now leaving this naive impl, will come back later.
                write!(f, "{}", &self.raw_text)
            }
            _ => {
                write!(f, "NOT IMPLEMENTED YET")
            }
        }
    }
}

pub fn to_code_str(statement: &Statement) -> String {
    match statement {
        Statement::FunctionDeclaration(_) => {
            // This case is handled outside of the function, and this should never
            // be reached. returning empty string.
            String::new()
        }
        Statement::VariableDeclaration(var_decl_st) => to_code_str_var_decl(var_decl_st),
        Statement::VariableAssignment(var_assign_st) => to_code_str_var_assignment(var_assign_st),
        Statement::Return(return_statement) => to_code_str_return(return_statement),
        Statement::Print(print_statement) => to_code_str_print(print_statement),
        Statement::If(if_statement) => to_code_str_if(if_statement),
        Statement::While(while_statement) => to_code_str_while(while_statement),
        Statement::RawFunctionCall(stmt) => to_code_str_raw_function_call(stmt),
    }
}

fn to_code_str_var_decl(var_decl: &VariableDeclarationStatement) -> String {
    format!(
        "{} {} = {};\n",
        var_decl.data_type.to_string(),
        var_decl.symbol_name,
        //to_code_str_value(&var_decl.assigned_value)
        // TODO: null terminate the string here somehow
        to_code_str_logical(&var_decl.assigned_logical)
    )
}

fn to_code_str_var_assignment(var_assign: &VariableAssignmentStatement) -> String {
    format!(
        "{} = {};\n",
        var_assign.var_name,
        to_code_str_logical(&var_assign.assigned_logical)
    )
}

fn to_code_str_value(value: &Value) -> String {
    let mut code_str = String::new();
    match value.value_type {
        ValueType::FunctionCall => {
            // Use the dedicated function call converter
            to_code_str_function_call(value)
        }
        ValueType::InlineNumber | ValueType::Variable => {
            code_str.push_str(&value.raw_text);
            code_str
        }
        ValueType::InlineString => {
            code_str.push_str("\"");
            code_str.push_str(&value.raw_text);
            code_str.push_str("\"");
            code_str
        }
        ValueType::InlineBoolean => {
            code_str.push_str(&value.raw_text);
            code_str
        }
        _ => code_str,
    }
}

fn to_code_str_function_call(value: &Value) -> String {
    let mut code_str = String::new();

    code_str.push_str(&value.raw_text);
    code_str.push_str("(");

    for (idx, param) in value.params.iter().enumerate() {
        code_str.push_str(&to_code_str_logical(param));
        if idx < value.params.len() - 1 {
            code_str.push_str(", ");
        }
    }

    code_str.push_str(")");
    code_str
}

fn to_code_str_return(return_stmt: &ReturnStatement) -> String {
    format!(
        "return {};\n",
        to_code_str_logical(&return_stmt.return_value)
    )
}

pub fn to_code_str_func_decl_stmt(
    func_stmt: &FunctionDeclarationStatement,
    function_def: &FunctionSymbol,
) -> String {
    let mut code = String::new();

    code.push_str(&convert_function_header_to_code_str(function_def));

    code.push_str("{\n");

    for statement in &func_stmt.body {
        code.push_str("   ");

        // WARNING: this will break I think if a function is declared in a function.
        code.push_str(&to_code_str(statement));
    }

    code.push_str("}\n");
    code
}

pub fn convert_function_header_to_code_str(function_def: &FunctionSymbol) -> String {
    let mut code_str = String::new();
    code_str.push_str(&format!(
        "{} {}(",
        function_def.return_type.to_string(),
        function_def.identifier
    ));

    for (i, param) in function_def.parameters.iter().enumerate() {
        code_str.push_str(&format!("{} {}", param.data_type, param.name));
        if i < function_def.parameters.len() - 1 {
            code_str.push_str(", ");
        }
    }
    code_str.push_str(")");

    code_str
}

pub fn to_code_str_expr(expr: &Expression) -> String {
    let mut code_str = String::new();

    if !expr.terms.is_empty() {
        code_str.push_str(&to_code_str_term(&expr.terms[0]));

        for (idx, term) in expr.terms.iter().skip(1).enumerate() {
            if idx < expr.operators.len() {
                code_str.push_str(&format!(
                    " {} ",
                    expression_operator_to_str(&expr.operators[idx])
                ));
            }
            code_str.push_str(&to_code_str_term(term));
        }
    }

    code_str
}

fn to_code_str_term(term: &Term) -> String {
    let mut code_str = String::new();

    if !term.unarys.is_empty() {
        code_str.push_str(&to_code_str_unary(&term.unarys[0]));

        for (idx, unary) in term.unarys.iter().skip(1).enumerate() {
            if idx < term.operations.len() {
                code_str.push_str(&format!(
                    " {} ",
                    term_operator_to_str(&term.operations[idx])
                ));
            }
            code_str.push_str(&to_code_str_unary(unary));
        }
    }

    code_str
}

fn to_code_str_unary(unary: &Unary) -> String {
    let mut code_str = String::new();

    if let Some(ref operation) = unary.operation {
        code_str.push_str(&expression_operator_to_str(operation));
    }

    code_str.push_str(&to_code_str_value(&unary.primary));

    code_str
}

pub fn to_code_str_comparison(comparison: &Comparison) -> String {
    let mut code_str = String::new();
    // How will this work?
    // check if the common expression type of the comparison is a String.
    // If it is, then this comparison needs to be generated differently with strcmp.
    // Otherwise, generate normally.
    //
    // When generating string comparison, we need to loop through the list of expressions,
    // and make a strcmp for each "pair".
    //let comparison_expressions_are_strings = true; // TODO: set this with a function
    let comparison_expressions_are_strings =
        comparison.expressions[0].data_type == DataType::String;

    if comparison_expressions_are_strings && comparison.expressions.len() > 1 {
        return to_code_str_comparison_of_string_expressions(comparison);
    }

    code_str.push_str(&to_code_str_expr(&comparison.expressions[0]));

    for (idx, expr) in comparison.expressions.iter().skip(1).enumerate() {
        if idx < comparison.operators.len() {
            code_str.push_str(&format!(
                " {} ",
                comparison_operator_to_str(&comparison.operators[idx])
            ));
        }
        code_str.push_str(&to_code_str_expr(expr));
    }

    code_str
}

fn to_code_str_comparison_of_string_expressions(comparison: &Comparison) -> String {
    let mut code_str = String::new();
    for (idx, expression) in comparison.expressions.iter().enumerate() {
        if idx == comparison.expressions.len() - 1 {
            //stop here
            break;
        }

        // Get the comparison operator for this pair
        let op = if idx < comparison.operators.len() {
            &comparison.operators[idx]
        } else {
            &ComparisonOperator::equalequal // Default fallback
        };

        code_str.push_str("(strcmp(");
        code_str.push_str(&to_code_str_expr(expression));
        code_str.push_str(", ");
        code_str.push_str(&to_code_str_expr(&comparison.expressions[idx + 1]));
        code_str.push_str(") ");

        match op {
            ComparisonOperator::equalequal => code_str.push_str("== 0"),
            ComparisonOperator::notequal => code_str.push_str("!= 0"),
            _ => code_str.push_str("ERROR INCORRECT OPERATOR"), // Default to equality
        }

        code_str.push_str(")");

        // TODO: incorporate !=, rn its only ==
        if idx < comparison.expressions.len() - 2 {
            code_str.push_str(" && "); //purely for debugging readability
            //code_str.push_str(comparison_operator_to_str(&comparison.operators[idx]));
        }
    }
    code_str
}

pub fn to_code_str_logical(logical: &crate::ast::Logical) -> String {
    let mut code_str = String::new();

    if !logical.comparisons.is_empty() {
        code_str.push_str(&to_code_str_comparison(&logical.comparisons[0]));

        for (idx, comparison) in logical.comparisons.iter().skip(1).enumerate() {
            if idx < logical.operators.len() {
                code_str.push_str(&format!(
                    " {} ",
                    logical_operator_to_str(&logical.operators[idx])
                ));
            }
            code_str.push_str(&to_code_str_comparison(comparison));
        }
    }

    code_str
}

// Helper functions to convert operators to string representations

fn expression_operator_to_str(op: &ExpressionOperator) -> &'static str {
    match op {
        ExpressionOperator::Plus => "+",
        ExpressionOperator::Minus => "-",
        ExpressionOperator::invalidop => "/* invalid op */",
    }
}

fn term_operator_to_str(op: &TermOperator) -> &'static str {
    match op {
        TermOperator::Multiply => "*",
        TermOperator::Divide => "/",
        TermOperator::invalidop => "/* invalid op */",
    }
}

fn comparison_operator_to_str(op: &ComparisonOperator) -> &'static str {
    match op {
        ComparisonOperator::equalequal => "==",
        ComparisonOperator::notequal => "!=",
        ComparisonOperator::lessthan => "<",
        ComparisonOperator::lessthanequalto => "<=",
        ComparisonOperator::greaterthan => ">",
        ComparisonOperator::greaterthanequalto => ">=",
        ComparisonOperator::invalidop => "/* invalid op */",
    }
}

fn logical_operator_to_str(op: &LogicalOperator) -> &'static str {
    match op {
        LogicalOperator::And => "&&",
        LogicalOperator::Or => "||",
        LogicalOperator::Not => "!",
        LogicalOperator::invalidop => "/* invalid op */",
    }
}

fn to_code_str_print(print_stmt: &PrintStatement) -> String {
    let mut expr_str = to_code_str_logical(&print_stmt.logical);

    // TODO: The data type the expr evaluates to should be resolved by now, and
    // we can look at it and adjust the printf statement accordingly.
    // TODO: change this depending on ln flag later
    //let newline_or_empty = print_stmt.is_print_ln ? "\n" : "";
    let mut newline_or_empty = String::new();
    if print_stmt.is_print_ln {
        newline_or_empty.push_str("\n");
    }

    expr_str = match print_stmt.logical.data_type {
        DataType::Number => format!("printf(\"%d\",{});", expr_str),
        DataType::String => format!("printf({});", expr_str),
        // TODO: change this to print string of boolean
        DataType::Boolean => format!("printf(\"%s\", {} ? \"true\" : \"false\");", expr_str),
        _ => "not either of these".to_string(),
    };

    if print_stmt.is_print_ln {
        expr_str.push_str("printf(\"\\n\");\n");
    }

    expr_str.push_str("\n"); // purely for readability while debugging
    expr_str
}

fn to_code_str_if(if_stmt: &IfStatement) -> String {
    let mut code_str = String::new();

    let condition_str = to_code_str_logical(&if_stmt.condition);
    code_str.push_str(&format!("if ({}) {{\n", condition_str));

    for statement in &if_stmt.if_body {
        code_str.push_str("   ");
        code_str.push_str(&to_code_str(statement));
    }

    if let Some(else_body) = &if_stmt.else_body {
        code_str.push_str("} else {\n");
        for statement in else_body {
            //code_str.push_str("   ");
            code_str.push_str(&to_code_str(statement));
        }
    }

    code_str.push_str("}\n");
    code_str
}

fn to_code_str_while(while_stmt: &WhileStatement) -> String {
    let mut code_str = String::new();

    let condition_str = to_code_str_logical(&while_stmt.condition);
    code_str.push_str(&format!("while ({}) {{\n", condition_str));

    for statement in &while_stmt.body {
        code_str.push_str("   ");
        code_str.push_str(&to_code_str(statement));
    }

    code_str.push_str("}\n");
    code_str
}

fn to_code_str_raw_function_call(stmt: &RawFunctionCallStatement) -> String {
    let mut code_str = to_code_str_function_call(&stmt.value);
    code_str.push_str(";\n");
    code_str
}
