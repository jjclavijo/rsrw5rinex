mod record_parser;

use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};

pub fn parse_rw5_file(input: &str) -> Result<(), Box<dyn Error>> {
    if input.contains('\n') {
        // Treat input as content
        for line in input.lines() {
            if !record_parser::parse_record_line(line).is_ok() {
                return record_parser::parse_record_line(line)
            };
        }
    } else {
        // Treat input as a file path
        let file = File::open(input)?;
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            record_parser::parse_record_line(&line)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rw5_file_with_content() {
        let content = "\
            JB,NMMY RW5 JOB,DT07-22-2004,TM13:13:51
            MO,AD0,UN0,SF1.00000000,EC1,EO0.0,AU0
            --SP,PN111,N 16556174.237,E 942130.662,EL 16.404
            --SP,PN108,N 16556174.237,E 942130.662,EL 17.945
            OC,OP111,N 16556174.237,E 942130.662,EL 16.404
            BK,OP111,BP108,BS0.00000,BC0.00000
            LS,HI5.684,HR5.500
            SS,OP111,FP108,AR0.00000,ZE0.00017,SD3.3566,--FENCE1";
        
        let result = parse_rw5_file(content);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    //#[test]
    //fn test_parse_rw5_file_with_path() {
    //    let filename = "example.rw5"; // Replace with the actual file path
    //    let result = parse_rw5_file(filename);
    //    assert!(result.is_ok());
    //}
}

