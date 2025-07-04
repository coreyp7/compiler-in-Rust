use crate::ast::Statement;

pub fn generate_code_str(ast: &Vec<Statement>) -> String {

    let mut code_str: String = String::new();

    for statement in ast {
        let statement_as_str = convert_statement_to_code(&statement);
        code_str.push_str(&statement_as_str);
    }
    code_str
}

fn convert_statement_to_code(statement: &Statement) -> String {
    //let mut code: String = String::new();

    // Okay this will be our switch statement again.
    let statement_code_str: String = match statement {
        Statement::Print{
            content, 
            is_content_identity_name, 
            line_number
        } => {
            let mut code = String::new(); 
            code.push_str("print(");
            
            if(!is_content_identity_name){
                code.push_str("\"");
            }

            code.push_str(&content.clone());

            if(!is_content_identity_name){
                code.push_str("\"");
            }

            code.push_str(");");
            code
        },
        _ => {
            String::from("")
        }
        /*
        Statement::If => {

        },
        Statement::While => {

        },
        Statement::Assignment => {

        },
        Statement::Instantiation => {

        },
        Statement::Newline => {

        },
        */
    };

    statement_code_str
}

/*
fn convert_print_to_code(print: Statement) -> String {
    let code = String::new(); 
    code.push_str("print(");
    
    if(!print.is_content_identity_name){
        code.push_str("\"");
    }

    code.push_str(print.content.clone());

    if(!print.is_content_identity_name){
        code.push_str("\"");
    }

    code.push_str(");");

    code
}
*/

