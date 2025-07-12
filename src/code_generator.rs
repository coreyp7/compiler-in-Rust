use crate::comparison::{
    Comparison, ComparisonOperator, Expression, ExpressionOperator, Logical, LogicalOperator, Term,
};
use crate::statement::{
    AssignmentStatement, IfStatement, PrintStatement, Statement, VarInstantiationStatement,
    VarType, WhileStatement,
};

pub fn generate_code_str(ast: &[Statement]) -> String {
    let mut code_str = String::new();
    code_str.push_str("#include <stdio.h>\n");
    code_str.push_str("int main(){\n");

    for statement in ast {
        let statement_as_str = convert_statement_to_code(statement);
        code_str.push_str(&statement_as_str);
    }
    code_str.push_str("}");

    code_str
}

fn convert_statement_to_code(statement: &Statement) -> String {
    let statement_code_str: String = match statement {
        Statement::Print(statement_struct) => convert_print_statement_to_code(statement_struct),
        Statement::If(statement_struct) => convert_if_statement_to_code(statement_struct),
        Statement::While(statement_struct) => convert_while_statement_to_code(statement_struct),
        Statement::Assignment(statement_struct) => {
            convert_assignment_statement_to_code(statement_struct)
        }
        Statement::VarInstantiation(statement_struct) => {
            convert_instantiation_statement_to_code(statement_struct)
        }
        Statement::Newline => convert_newline_to_code(),
        Statement::TestStub => String::from("// TestStub\n"),
    };

    statement_code_str
}

fn convert_print_statement_to_code(statement_struct: &PrintStatement) -> String {
    let mut code = String::new();
    let content = &statement_struct.content;
    let is_content_identity_name = statement_struct.is_content_identity_name;
    code.push_str("printf(");
    if !is_content_identity_name {
        code.push_str("\"");
    }
    code.push_str(&content.clone());
    if !is_content_identity_name {
        code.push_str("\\n\"");
    }
    code.push_str(");");
    code
}

fn convert_if_statement_to_code(statement_struct: &IfStatement) -> String {
    let mut code = String::new();
    code.push_str("if(");
    code.push_str(&convert_logical_to_code(&statement_struct.logical));
    //code.push_str("){\n");
    code.push_str("){");

    // Convert nested statements in this if block
    for stmt in &statement_struct.statements {
        let stmt_code = convert_statement_to_code(stmt);
        code.push_str(&stmt_code);
    }

    code.push_str("}\n");
    code
}

fn convert_while_statement_to_code(statement_struct: &WhileStatement) -> String {
    let mut code = String::new();
    code.push_str("while (");
    code.push_str(&convert_logical_to_code(&statement_struct.logical));
    code.push_str(") {\n");

    // Convert nested statements in this while block
    for stmt in &statement_struct.statements {
        let stmt_code = convert_statement_to_code(stmt);
        code.push_str(&stmt_code);
    }

    code.push_str("}\n");
    code
}

fn convert_assignment_statement_to_code(statement_struct: &AssignmentStatement) -> String {
    let mut code = String::new();
    code.push_str(&statement_struct.identity);
    code.push_str(" = ");

    // Handle different value types
    match statement_struct.assigned_value_type {
        VarType::Str => {
            code.push_str("\"");
            code.push_str(&statement_struct.value);
            code.push_str("\"");
        }
        VarType::Num => {
            code.push_str(&statement_struct.value);
        }
        VarType::Unrecognized => {
            code.push_str(&statement_struct.value);
        }
    }

    code.push_str(";\n");
    code
}

fn convert_instantiation_statement_to_code(statement_struct: &VarInstantiationStatement) -> String {
    let mut code = String::new();

    // Add C type declaration
    match statement_struct.var_type {
        VarType::Str => code.push_str("char* "),
        VarType::Num => code.push_str("int "),
        VarType::Unrecognized => code.push_str("/* unknown type */ "),
    }

    code.push_str(&statement_struct.identity);
    code.push_str(" = ");

    // Handle different value types
    match statement_struct.assigned_value_type {
        VarType::Str => {
            code.push_str("\"");
            code.push_str(&statement_struct.value);
            code.push_str("\"");
        }
        VarType::Num => {
            code.push_str(&statement_struct.value);
        }
        VarType::Unrecognized => {
            code.push_str(&statement_struct.value);
        }
    }

    code.push_str(";\n");
    code
}

fn convert_logical_to_code(logical: &Logical) -> String {
    let mut code = String::new();
    let mut operator_idx = 0;

    if !logical.operators.is_empty() && logical.operators[0] == LogicalOperator::Not {
        // If the first operator is a Not, we need to handle it specially
        code.push_str("!");
        operator_idx += 1; // Move past the Not operator we've already added to code str
    }
    let first_comparison = &logical.comparisons[0];
    code.push_str(&convert_comparison_to_code(first_comparison));

    if logical.comparisons.len() == 1 {
        return code; // No other comparisons
    }

    for i in 1..logical.comparisons.len() {
        let operator = &logical.operators[operator_idx];
        operator_idx += 1;

        // Add operator to code str
        match operator {
            LogicalOperator::And => code.push_str(" && "),
            LogicalOperator::Or => code.push_str(" || "),
            LogicalOperator::Not => {
                // This should never happen!!!!!!
                code.push_str("BANG ");
            }
            _ => {
                // Invalid operator, we should handle this case
                // For now, just return an empty string or log an error
                return String::from("// Invalid logical operator\n");
            }
        }

        let next_operator = logical.operators.get(operator_idx);
        // If there's a Not operator in front of the comparison, add it here and
        // increment the operator index again.
        match next_operator {
            Some(LogicalOperator::Not) => {
                // If the next operator is a Not, we need to handle it specially
                code.push_str("!");
                operator_idx += 1; // Move past the Not operator
            }
            _ => {
                // TODO: this may require further behavior implementation, but
                // this should be caught in the AST construction and analysis
                // phase before it ever gets here.
            }
        };

        // Add comparison to code str
        let comparison = &logical.comparisons[i];
        code.push_str(&convert_comparison_to_code(comparison));
    }

    return code;
}

fn convert_comparison_to_code(comparison: &Comparison) -> String {
    let mut code = String::new();

    // Add first expression
    if !comparison.expressions.is_empty() {
        code.push_str(&convert_expression_to_code(&comparison.expressions[0]));
    }

    // Add operators and remaining expressions
    for (i, operator) in comparison.operators.iter().enumerate() {
        // Add the comparison operator
        match operator {
            ComparisonOperator::equalequal => code.push_str(" == "),
            ComparisonOperator::notequal => code.push_str(" != "),
            ComparisonOperator::lessthan => code.push_str(" < "),
            ComparisonOperator::lessthanequalto => code.push_str(" <= "),
            ComparisonOperator::greaterthan => code.push_str(" > "),
            ComparisonOperator::greaterthanequalto => code.push_str(" >= "),
            _ => code.push_str(" /* unknown comparison */ "),
        }

        // Add the next expression (i+1 because first expression is already added)
        if i + 1 < comparison.expressions.len() {
            code.push_str(&convert_expression_to_code(&comparison.expressions[i + 1]));
        }
    }

    code
}

fn convert_expression_to_code(expression: &Expression) -> String {
    let mut code = String::new();

    // Add first term
    if !expression.terms.is_empty() {
        code.push_str(&convert_term_to_code(&expression.terms[0]));
    }

    // Add operators and remaining terms
    for (i, operator) in expression.operators.iter().enumerate() {
        match operator {
            ExpressionOperator::Plus => code.push_str(" + "),
            ExpressionOperator::Minus => code.push_str(" - "),
            _ => code.push_str(" /* unknown expression operator */ "),
        }

        // Add the next term (i+1 because first term is already added)
        if i + 1 < expression.terms.len() {
            code.push_str(&convert_term_to_code(&expression.terms[i + 1]));
        }
    }

    code
}

fn convert_term_to_code(term: &Term) -> String {
    let mut code = String::new();

    // Add first unary
    if !term.unarys.is_empty() {
        code.push_str(&convert_unary_to_code(&term.unarys[0]));
    }

    // Add operations and remaining unarys
    for (i, operation) in term.operations.iter().enumerate() {
        match operation {
            crate::comparison::TermOperator::Multiply => code.push_str(" * "),
            crate::comparison::TermOperator::Divide => code.push_str(" / "),
            _ => code.push_str(" /* unknown term operator */ "),
        }

        // Add the next unary (i+1 because first unary is already added)
        if i + 1 < term.unarys.len() {
            code.push_str(&convert_unary_to_code(&term.unarys[i + 1]));
        }
    }

    code
}

fn convert_unary_to_code(unary: &crate::comparison::Unary) -> String {
    let mut code = String::new();

    // Add unary operation if it exists
    if let Some(operation) = &unary.operation {
        match operation {
            ExpressionOperator::Plus => code.push_str("+"),
            ExpressionOperator::Minus => code.push_str("-"),
            _ => code.push_str("/* unknown unary operator */"),
        }
    }

    // Add the primary
    code.push_str(&convert_primary_to_code(&unary.primary));

    code
}

fn convert_primary_to_code(primary: &crate::comparison::Primary) -> String {
    match primary {
        crate::comparison::Primary::Number { value } => value.clone(),
        crate::comparison::Primary::Identity { name } => name.clone(),
        crate::comparison::Primary::Error { detail: _ } => String::from("/* error in primary */"),
    }
}

fn convert_newline_to_code() -> String {
    //String::from("\n")
    String::from("")
}
