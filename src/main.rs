use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;


fn main() -> std::io::Result<()> {

    let f = File::open("log.txt")?;
    let reader = BufReader::new(f);
    
    for line_result in reader.lines() {
        if let Ok(line_str) = line_result {
            tokenize_line(&line_str);         
        }
    }

    println!("EOF");
    Ok(())
}

// TODO: return result indicating success
fn tokenize_line(mut line: &String) {
    println!("{line}");
}



