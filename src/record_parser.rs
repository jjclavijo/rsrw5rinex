// record_parser.rs

//use std::error::Error;

// A GENERIC ERROR FOR PARSING FIELD

use anyhow::anyhow;
use rinex::prelude::{Duration, Epoch};
use std::convert::TryFrom;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::Serialize;
use crate::record_parser_gps as gps;

#[derive(Debug)]
pub struct CustomError {
    message: String,
    field: String,
}

impl CustomError {
    pub fn new(message: &str, field: &str) -> Self {
        CustomError {
            message: message.to_string(),
            field: field.to_string(),
        }
    }
}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}. Field: {}", self.message, self.field)
    }
}

impl std::error::Error for CustomError {}

#[derive(Debug, PartialEq,Default,Clone,Serialize)]
pub struct TRecord {
    date: Option<(i32,u8,u8)>,
    time: Option<(u8,u8,u8)>,
    dt: Option<Epoch>
}

impl TRecord {
    pub fn with_date(&self, t:(i32,u8,u8)) -> TRecord {
        TRecord { date: Some(t), time: self.time, dt: self.dt }
    }
    pub fn with_time(&self, t:(u8,u8,u8)) -> TRecord {
        TRecord { date: self.date, time: Some(t), dt: self.dt }
    }
    pub fn combine(self) -> TRecord {
        match self {
            TRecord { date: Some(d), time: Some(t), dt: _ } =>
                TRecord { date: self.date, time: self.time,
                          dt: Some(
                               Epoch::from_gregorian_tai(d.0,d.1,d.2,t.0,t.1,t.2,0)
                               ) 
                },
            _ => self.clone()
        }
    }

    pub fn merge(self, other: TRecord) -> Result<TRecord, anyhow::Error> {
        match self {
            TRecord { date: Some(d), time: None, dt: _ } =>
                match other {
                    TRecord { date: None, time: Some(t), dt: _ } =>
                        Ok(TRecord::default().with_time(t).with_date(d).combine()),
                    TRecord { date: Some(d1), time: Some(_t), dt: _ } =>
                        if d1 == d {
                            Ok(other)
                        } else {
                            Err(anyhow!("Fechas incompatibles"))
                        },
                    TRecord { date: Some(d1), time: None, dt: _ } =>
                        if d1 == d {
                            Ok(other)
                        } else {
                            Err(anyhow!("Fechas incompatibles"))
                        },
                    _ => Ok(self)
                }
            TRecord { date: None, time: Some(t), dt: _ } =>
                match other {
                    TRecord { date: Some(d), time: None, dt: _ } =>
                        Ok(TRecord::default().with_time(t).with_date(d).combine()),
                    TRecord { date: Some(_d), time: Some(t1), dt: _ } =>
                        if t1 == t {
                            Ok(other)
                        } else {
                            Err(anyhow!("Horas incompatibles"))
                        }
                    TRecord { date: None, time: Some(t1), dt: _ } =>
                        if t1 == t {
                            Ok(other)
                        } else {
                            Err(anyhow!("Horas incompatibles"))
                        }
                    _ => Ok(self)
                }
            TRecord { date: None, time: None, dt: _ } => {
                Err(anyhow!("Nada que combinar"))
            }
            TRecord { date: Some(_), time: Some(_), dt: _ } => {
                Err(anyhow!("Ya está combinado"))
            }
        }
    }

    pub fn get_epoch(&self) -> Epoch {
        match self.dt {
            None => Epoch::from_gpst_duration(Duration::from_seconds(0.)),
            Some(v) => v
        }
    }
}

crate::genera_try_from!(TRecord = TRecord, merge);

pub fn parse_dt_record(line: &str) -> Result<TRecord, anyhow::Error>
{
    let (cabeza, pies) = line.split_at(4);
    let tr = match cabeza {
        "--DT" => {
            let (mm,r) = pies.split_once('-').unwrap();
            let (dd,yyyy) = r.split_once('-').unwrap();
            Ok(TRecord::default().with_date((yyyy.parse()?, mm.parse()?, dd.parse()?)))
        },
        "--TM" => {
            let (hh,r) = pies.split_once(':').unwrap();
            let (mm,ss) = r.split_once(':').unwrap();
            Ok(TRecord::default().with_time((hh.parse()?, mm.parse()?, ss.parse()?)))
        },
        _ => {
            Err(anyhow!("Registro de tiempo invalido"))
        }
    }?;

    Ok(tr.combine())
}

#[derive(Debug, PartialEq)]
pub struct BacksightRecord {
    pub op: String,
    pub bp: String,
    pub bs: f64,
    pub bc: f64,
}

#[derive(Debug, PartialEq)]
pub struct JobRecord {
    pub nm: String,
    pub dt: String,
    pub tm: String,
}

#[derive(Debug, PartialEq)]
pub struct LineOfSightRecord {
    pub hi: f64,
    pub hr: Option<f64>,
}

#[derive(Debug, PartialEq)]
pub struct ModeSetupRecord {
    pub ad: u32,
    pub un: u32,
    pub sf: f64,
    pub ec: u32,
    pub eo: f64,
    pub au: u32,
}

#[derive(Debug, PartialEq)]
pub struct OccupyRecord {
    pub op: String,
    pub n: f64,
    pub e: f64,
    pub el: f64,
    pub note: String,
}

#[derive(Debug, PartialEq)]
pub struct OffCenterShotRecord {
    pub ar: f64,
    pub ze: f64,
    pub sd: f64,
}

#[derive(Debug, PartialEq)]
pub struct StorePointRecord {
    pub pn: String,
    pub n: f64,
    pub e: f64,
    pub el: f64,
    pub note: String,
}

#[derive(Debug, PartialEq)]
pub struct LabelRecord {
    pub label: String,
}

#[derive(Debug, PartialEq)]
pub enum AngleOption {
    Azimuth(f64),
    Bearing(f64),
    AngleRight(f64),
    AngleLeft(f64),
    DeflectionRight(f64),
    DeflectionLeft(f64),
}

#[derive(Debug, PartialEq)]
pub enum ZenithOption {
    Zenith(f64),
    VerticalAngle(f64),
    ChangeElevation(f64),
}

#[derive(Debug, PartialEq)]
pub enum DistanceOption {
    SlopeDistance(f64),
    HorizontalDistance(f64),
}

#[derive(Debug, PartialEq)]
pub struct TraverseRecord {
    pub occupy_point: String,
    pub foresight_point: String,
    pub angle_option: AngleOption,
    pub zenith_option: ZenithOption,
    pub distance_option: DistanceOption,
    pub note: String,
}


pub fn parse_label_record(line: &str) -> Result<(), Box<dyn std::error::Error>> {
    if line.len() < 2 || !line.starts_with("--") {
        return Err("Invalid label record format".into());
    }

    // Extract the label text from the line (excluding "--")
    let label_text = line.trim_start_matches("--").trim();
    let label = label_text.to_string();

    // Create a LabelRecord and store the label
    let label_record = LabelRecord { label };

    println!("{:?}", label_record); // You can print or use the label_record here as needed

    Ok(())
}


// Add other record structs as needed

pub fn parse_backsight_record(line: &str) -> Result<BacksightRecord, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() >= 5 {
        let op = parts[1].trim_start_matches("OP").to_string();
        let bp = parts[2].trim_start_matches("BP").to_string();
        let bs = parts[3].trim_start_matches("BS").parse::<f64>()?;
        let bc = parts[4].trim_start_matches("BC").parse::<f64>()?;
        Ok(BacksightRecord { op, bp, bs, bc })
    } else {
        Err("Invalid Backsight record format".into())
    }
}

pub fn parse_job_record(line: &str) -> Result<JobRecord, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() >= 4 {
        let nm = parts[1].trim_start_matches("NM").to_string();
        let dt = parts[2].trim_start_matches("DT").to_string();
        let tm = parts[3].trim_start_matches("TM").to_string();
        Ok(JobRecord { nm, dt, tm })
    } else {
        Err("Invalid Job record format".into())
    }
}

pub fn parse_line_of_sight_record(line: &str) -> Result<LineOfSightRecord, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() >= 2 {
        let hi = parts[1].trim_start_matches("HI").parse::<f64>()?;
        let hr = parts.get(2).map(|&s| s.trim_start_matches("HR").parse::<f64>()).transpose()?;
        Ok(LineOfSightRecord { hi, hr })
    } else {
        Err("Invalid Line of Sight record format".into())
    }
}

pub fn parse_mode_setup_record(line: &str) -> Result<ModeSetupRecord, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() >= 7 {
        let ad = parts[1].trim_start_matches("AD").parse::<u32>()?;
        let un = parts[2].trim_start_matches("UN").parse::<u32>()?;
        let sf = parts[3].trim_start_matches("SF").parse::<f64>()?;
        let ec = parts[4].trim_start_matches("EC").parse::<u32>()?;
        let eo = parts[5].trim_start_matches("EO").parse::<f64>()?;
        let au = parts[6].trim_start_matches("AU").parse::<u32>()?;
        Ok(ModeSetupRecord { ad, un, sf, ec, eo, au })
    } else {
        Err("Invalid Mode Setup record format".into())
    }
}

pub fn parse_occupy_record(line: &str) -> Result<OccupyRecord, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() >= 5 {
        let op = parts[1].trim_start_matches("OP").to_string();
        let n = parts[2].trim_start_matches("N ").parse::<f64>()?;
        let e = parts[3].trim_start_matches("E ").parse::<f64>()?;
        let el = parts[4].trim_start_matches("EL").parse::<f64>()?;
        let mut note: String = "".to_string();
        if parts.len() == 6 {
            note = parts[5].to_string();
        };
        Ok(OccupyRecord { op, n, e, el, note })
    } else {
        Err("Invalid Occupy record format".into())
    }
}

pub fn parse_off_center_shot_record(line: &str) -> Result<OffCenterShotRecord, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() >= 4 {
        let ar = parts[1].trim_start_matches("AR").parse::<f64>()?;
        let ze = parts[2].trim_start_matches("ZE").parse::<f64>()?;
        let sd = parts[3].trim_start_matches("SD").parse::<f64>()?;
        Ok(OffCenterShotRecord { ar, ze, sd })
    } else {
        Err("Invalid Off Center Shot record format".into())
    }
}

pub fn parse_store_point_record(line: &str) -> Result<StorePointRecord, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() >= 5 {
        let pn = parts[1].trim_start_matches("PN").to_string();
        let n = parts[2].trim_start_matches("N ").parse::<f64>()?;
        let e = parts[3].trim_start_matches("E ").parse::<f64>()?;
        let el = parts[4].trim_start_matches("EL").parse::<f64>()?;
        let mut note: String = "".to_string();
        if parts.len() == 6 {
            note = parts[5].to_string();
        };
        Ok(StorePointRecord { pn, n, e, el, note })
    } else {
        Err("Invalid Store Point record format".into())
    }
}

fn parse_traverse_record(line: &str) -> Result<TraverseRecord, Box<dyn std::error::Error>> {
    let fields: Vec<&str> = line.split(',').collect();
    if fields.len() > 7 {
        return Err("Invalid traverse record format".into());
    }

    let occupy_point = fields[1].trim_start_matches("OP").to_string();
    let foresight_point = fields[2].trim_start_matches("FP").to_string();

    let angle_option = match &fields[3][..2] {
        "AZ" => AngleOption::Azimuth(fields[3].trim_start_matches("AZ").parse::<f64>()?),
        "BR" => AngleOption::Bearing(fields[3].trim_start_matches("BR").parse::<f64>()?),
        "AR" => AngleOption::AngleRight(fields[3].trim_start_matches("AR").parse::<f64>()?),
        "AL" => AngleOption::AngleLeft(fields[3].trim_start_matches("AL").parse::<f64>()?),
        "DR" => AngleOption::DeflectionRight(fields[3].trim_start_matches("DR").parse::<f64>()?),
        "DL" => AngleOption::DeflectionLeft(fields[3].trim_start_matches("DL").parse::<f64>()?),
        _ => return Err(Box::new(CustomError::new("Invalid angle option", fields[3]))),
    };

    let zenith_option = match &fields[4][..2] {
        "ZE" => ZenithOption::Zenith(fields[4].trim_start_matches("ZE").parse::<f64>()?),
        "VA" => ZenithOption::VerticalAngle(fields[4].trim_start_matches("VA").parse::<f64>()?),
        "CE" => ZenithOption::ChangeElevation(fields[4].trim_start_matches("CE").parse::<f64>()?),
        _ => return Err("Invalid zenith option".into()),
    };

    let distance_option = match &fields[5][..2] {
        "SD" => DistanceOption::SlopeDistance(fields[5].trim_start_matches("SD").parse::<f64>()?),
        "HD" => DistanceOption::HorizontalDistance(fields[5].trim_start_matches("HD").parse::<f64>()?),
        _ => return Err("Invalid distance option".into()),
    };

    let note = fields[6].trim_start_matches("--").to_string();

    Ok(TraverseRecord {
        occupy_point,
        foresight_point,
        angle_option,
        zenith_option,
        distance_option,
        note,
    })
}




pub fn parse_record_line(line: &str) -> Result<(), Box<dyn std::error::Error>> {
    if line.is_empty() {
        return Ok(());
    }

    // Extract the record type identifier
    let record_type = line.split(',').next().unwrap_or("").trim();

    match record_type {
        "JB" => {
            // Call the parse_job_record function here, passing the entire line
            parse_job_record(line)?;
        }
        "MO" => {
            // Call the parse_mode_setup_record function here, passing the entire line
            parse_mode_setup_record(line)?;
        }
        "--SP" => {
            // Call the parse_station_point_record function here, passing the entire line
            parse_store_point_record(line)?;
        }
        "OC" => {
            // Call the parse_occupy_record function here, passing the entire line
            parse_occupy_record(line)?;
        }
        "OF" => {
            // Call the parse_off_center_shot_record function here, passing the entire line
            parse_off_center_shot_record(line)?;
        }
        "BK" => {
            // Call the parse_backsight_record function here, passing the entire line
            parse_backsight_record(line)?;
        }
        "LS" => {
            // Call the parse_line_of_sight_record function here, passing the entire line
            parse_line_of_sight_record(line)?;
        }
        "SS" | "TR" | "BD" | "BR" | "FD" | "FR" => {
            // Call the parse_traverse_record function here, passing the entire line
            parse_traverse_record(line)?;
        }
        "--" => {
            // Handle label records
            parse_label_record(line)?;
        }
        // Add more cases for other record types as needed
        _ => {
            // Handle unknown record type or raise an error
            return Err("Unknown record type".into());
        }
    }

    Ok(())
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_backsight_record() {
        let line = "BK,OP1,BP2,BS315.0000,BC0.0044";
        let result = parse_backsight_record(line);
        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.op, "1");
        assert_eq!(record.bp, "2");
        assert_eq!(record.bs, 315.0);
        assert_eq!(record.bc, 0.0044);
    }

    #[test]
    fn test_parse_job_record() {
        let line = "JB,NMSAMPLE,DT06-27-2003,TM14:21:53";
        let result = parse_job_record(line);
        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.nm, "SAMPLE");
        assert_eq!(record.dt, "06-27-2003");
        assert_eq!(record.tm, "14:21:53");
    }

    #[test]
    fn test_parse_line_of_sight_record() {
        let line1 = "LS,HI5.000000,HR6.000000";
        let line2 = "LS,HI5.000000";
        let result1 = parse_line_of_sight_record(line1);
        let result2 = parse_line_of_sight_record(line2);
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        let record1 = result1.unwrap();
        let record2 = result2.unwrap();
        assert_eq!(record1.hi, 5.0);
        assert_eq!(record1.hr, Some(6.0));
        assert_eq!(record2.hi, 5.0);
        assert_eq!(record2.hr, None);
    }

    #[test]
    fn test_parse_mode_setup_record() {
        let line = "MO,AD0,UN0,SF1.00000000,EC1,EO0.0,AU0";
        let result = parse_mode_setup_record(line);
        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.ad, 0);
        assert_eq!(record.un, 0);
        assert_eq!(record.sf, 1.0);
        assert_eq!(record.ec, 1);
        assert_eq!(record.eo, 0.0);
        assert_eq!(record.au, 0);
    }

    #[test]
    fn test_parse_occupy_record() {
        let line = "OC,OP1,N 5000.00000,E 5000.00000,EL100.000,--CP";
        let result = parse_occupy_record(line);
        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.op, "1");
        assert_eq!(record.n, 5000.0);
        assert_eq!(record.e, 5000.0);
        assert_eq!(record.el, 100.0);
        assert_eq!(record.note, "--CP");
    }

    #[test]
    fn test_parse_off_center_shot_record() {
        let line = "OF,AR90.3333,ZE90.0000,SD25.550000";
        let result = parse_off_center_shot_record(line);
        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.ar, 90.3333);
        assert_eq!(record.ze, 90.0);
        assert_eq!(record.sd, 25.55);
    }

    #[test]
    fn test_parse_store_point_record() {
        let line = "SP,PN100,N 5002.0000,E 5000.0000,EL100.0000,--PP";
        let result = parse_store_point_record(line);
        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.pn, "100");
        assert_eq!(record.n, 5002.0);
        assert_eq!(record.e, 5000.0);
        assert_eq!(record.el, 100.0);
        assert_eq!(record.note, "--PP");
    }

    #[test]
    fn test_parse_traverse_record_azimuth() {
        let line = "TR,OP1,FP4,AZ90.3333,ZE90.3333,SD25.550000,--CP";
        let expected = TraverseRecord {
            occupy_point: "1".to_string(),
            foresight_point: "4".to_string(),
            angle_option: AngleOption::Azimuth(90.3333),
            zenith_option: ZenithOption::Zenith(90.3333),
            distance_option: DistanceOption::SlopeDistance(25.550000),
            note: "CP".to_string(),
        };
        assert_eq!(expected, parse_traverse_record(line).unwrap());
    }

    #[test]
    fn test_parse_traverse_record_bearing() {
        let line = "BD,OP1,FP2,BR123.4500,ZE86.0133,SD10.313750,--CP";
        let expected = TraverseRecord {
            occupy_point: "1".to_string(),
            foresight_point: "2".to_string(),
            angle_option: AngleOption::Bearing(123.4500),
            zenith_option: ZenithOption::Zenith(86.0133),
            distance_option: DistanceOption::SlopeDistance(10.313750),
            note: "CP".to_string(),
        };
        assert_eq!(expected, parse_traverse_record(line).unwrap());
    }

    #[test]
    fn test_parse_traverse_record_angle_right() {
        let line = "TR,OP1,FP4,AR45.6789,ZE90.3333,SD25.550000,--CP";
        let expected = TraverseRecord {
            occupy_point: "1".to_string(),
            foresight_point: "4".to_string(),
            angle_option: AngleOption::AngleRight(45.6789),
            zenith_option: ZenithOption::Zenith(90.3333),
            distance_option: DistanceOption::SlopeDistance(25.550000),
            note: "CP".to_string(),
        };
        assert_eq!(expected, parse_traverse_record(line).unwrap());
    }

    #[test]
    fn test_parse_traverse_record_angle_left() {
        let line = "SS,OP1,FP2,AL12.3456,ZE86.0133,SD10.313750,--CP";
        let expected = TraverseRecord {
            occupy_point: "1".to_string(),
            foresight_point: "2".to_string(),
            angle_option: AngleOption::AngleLeft(12.3456),
            zenith_option: ZenithOption::Zenith(86.0133),
            distance_option: DistanceOption::SlopeDistance(10.313750),
            note: "CP".to_string(),
        };
        assert_eq!(expected, parse_traverse_record(line).unwrap());
    }

    #[test]
    fn test_parse_traverse_record_deflection_right() {
        let line = "FR,OP1,FP3,DR34.5678,ZE89.4305,SD7.393000,--CP";
        let expected = TraverseRecord {
            occupy_point: "1".to_string(),
            foresight_point: "3".to_string(),
            angle_option: AngleOption::DeflectionRight(34.5678),
            zenith_option: ZenithOption::Zenith(89.4305),
            distance_option: DistanceOption::SlopeDistance(7.393000),
            note: "CP".to_string(),
        };
        assert_eq!(expected, parse_traverse_record(line).unwrap());
    }

}

