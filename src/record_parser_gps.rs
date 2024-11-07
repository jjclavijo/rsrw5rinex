// record_parser.rs

use std::convert::{TryFrom, TryInto};
// use crate::record_parser::CustomError;
use chrono::naive::{NaiveDate,NaiveDateTime};
//use chrono::Duration;
use anyhow::anyhow;
use rinex::prelude::{Duration, Epoch, TimeScale};
use serde::Serialize;


// const fn gps_start_date() -> NaiveDateTime 
// {
//     let d: Option<NaiveDate> = NaiveDate::from_ymd_opt(1980, 1, 6);
//     
//     let odt: Option<NaiveDateTime> = match d {
//         Some(dd) => dd.and_hms_opt(0, 0, 0),
//         None => panic!("panic!")
//     };
//     
//     let dt: NaiveDateTime = match odt {
//         Some(ddtt) => ddtt,
//         None => panic!("panic!")
//     };
//     
//     dt
// }
// 
// const GSD: NaiveDateTime = gps_start_date();

fn count_leap_seconds(date: NaiveDate) -> usize {
    // Lista de fechas con segundos intercalados (leap seconds)
    let leap_seconds_dates = vec![
        NaiveDate::from_ymd_opt(1981, 6, 30).unwrap(),
        NaiveDate::from_ymd_opt(1982, 6, 30).unwrap(),
        NaiveDate::from_ymd_opt(1983, 6, 30).unwrap(),
        NaiveDate::from_ymd_opt(1985, 6, 30).unwrap(),
        NaiveDate::from_ymd_opt(1987, 12, 31).unwrap(),
        NaiveDate::from_ymd_opt(1989, 12, 31).unwrap(),
        NaiveDate::from_ymd_opt(1990, 12, 31).unwrap(),
        NaiveDate::from_ymd_opt(1992, 6, 30).unwrap(),
        NaiveDate::from_ymd_opt(1993, 6, 30).unwrap(),
        NaiveDate::from_ymd_opt(1994, 6, 30).unwrap(),
        NaiveDate::from_ymd_opt(1995, 12, 31).unwrap(),
        NaiveDate::from_ymd_opt(1997, 6, 30).unwrap(),
        NaiveDate::from_ymd_opt(1998, 12, 31).unwrap(),
        NaiveDate::from_ymd_opt(2005, 12, 31).unwrap(),
        NaiveDate::from_ymd_opt(2008, 12, 31).unwrap(),
        NaiveDate::from_ymd_opt(2012, 6, 30).unwrap(),
        NaiveDate::from_ymd_opt(2015, 6, 30).unwrap(),
        NaiveDate::from_ymd_opt(2016, 12, 31).unwrap(),
    ];

    // Contar cuántas fechas de segundos intercalados son anteriores a la fecha dada
    leap_seconds_dates.iter().filter(|&&leap_date| leap_date < date).count()
}




#[derive(Debug, PartialEq, Clone, Serialize,Default)]
pub struct BPRecord {
    pub occupy_point: String,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: f64,
    pub antenna_ground: f64,
    pub phase_antenna: f64,
    pub elevation_alt: Option<f64>,
    pub note: String,
    pub north: Option<f64>,
    pub east: Option<f64>,
    pub start_time: Option<Epoch>,
    pub end_time: Option<Epoch>,
    
}




impl BPRecord {
    // Necesito que mueva la referencia para reusarlo
    pub fn aplicar_gs(self, gr: GSRecord) -> Result<Self,anyhow::Error> {

        if self.occupy_point != gr.occupy_point { 
            Err(anyhow!(format!("No coinciden los registros BP y GS {} != {}",self.occupy_point,gr.occupy_point)))
        }
        else if (self.elevation - gr.elevation).abs() > 0.001 {
            // Err(anyhow!(format!("No coinciden los registros GPS y GS {} != {}",self.elevation,gr.elevation)))
            Ok( Self 
                { occupy_point:self.occupy_point,
                  latitude: self.latitude, longitude: self.longitude,
                  elevation: self.elevation, antenna_ground: self.antenna_ground, 
                  phase_antenna: self.phase_antenna, elevation_alt: Some(gr.elevation),
                  note: self.note, north: Some(gr.north),
                  east: Some(gr.east), start_time: self.start_time,
                  end_time: self.end_time
            })
        }
        else {
            Ok( Self 
                { occupy_point:self.occupy_point,
                  latitude: self.latitude, longitude: self.longitude,
                  elevation: self.elevation, antenna_ground: self.antenna_ground, 
                  phase_antenna: self.phase_antenna, elevation_alt: Some(gr.elevation),
                  note: self.note, north: Some(gr.north),
                  east: Some(gr.east), start_time: self.start_time,
                  end_time: self.end_time
            })
        }
    }
}


#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct GPSRecord {
    pub occupy_point: String,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: f64,
    pub elevation_alt: Option<f64>,
    pub note: String,
    pub north: Option<f64>,
    pub east: Option<f64>,
    pub start_time: Option<Epoch>,
    pub end_time: Option<Epoch>,
}

impl GPSRecord {
    // Necesito que mueva la referencia para reusarlo
    pub fn aplicar_gs(self, gr: GSRecord) -> Result<GPSRecord,anyhow::Error> {

        if self.occupy_point != gr.occupy_point { 
            Err(anyhow!(format!("No coinciden los registros GPS y GS {} != {}",self.occupy_point,gr.occupy_point)))
        }
        else if (self.elevation - gr.elevation).abs() > 0.001 {
            // Err(anyhow!(format!("No coinciden los registros GPS y GS {} != {}",self.elevation,gr.elevation)))
            Ok( GPSRecord 
                { occupy_point:self.occupy_point,
                  latitude: self.latitude, longitude: self.longitude,
                  elevation: self.elevation, elevation_alt: Some(gr.elevation),
                  note: self.note, north: Some(gr.north),
                  east: Some(gr.east), start_time: self.start_time,
                  end_time: self.end_time
            })
        }
        else {
            Ok( GPSRecord 
                { occupy_point:self.occupy_point,
                  latitude: self.latitude, longitude: self.longitude,
                  elevation: self.elevation, elevation_alt: self.elevation_alt,
                  note: self.note, north: Some(gr.north),
                  east: Some(gr.east), start_time: self.start_time,
                  end_time: self.end_time
            })
        }

    }

    pub fn aplicar_gt(self, gt: GTRecord) -> Result<GPSRecord,anyhow::Error> {
        if self.occupy_point != gt.occupy_point { 
            Err(anyhow!("No coinciden los registros GPS y GT"))
        } else {
            Ok( GPSRecord { 
                occupy_point:self.occupy_point,
                latitude: self.latitude, longitude: self.longitude,
                elevation: self.elevation, elevation_alt: self.elevation_alt,
                note: self.note, north: self.north,
                east: self.east, start_time: Some(gt.start),
                end_time: Some(gt.end) 
            })
        }
    }
}


#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct GSRecord {
    pub occupy_point: String,
    pub north: f64,
    pub east: f64,
    pub elevation: f64,
    pub note: String,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct GTRecord {
    pub occupy_point: String,
    pub start: Epoch,
    pub end: Epoch,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct LSRecord {
    pub height_rod: f64,
}

#[derive(Debug,Clone,Serialize,Default,PartialEq)]
pub struct ATRecord {
    pub tipo: String,
    pub radio: f64,
    pub slant_h_mp: f64,
    pub l1h: f64,
    pub l2h: f64,
    pub h_ingresada: Option<f64>,
    pub h_tipo: Option<TipoDeAltura>,
    pub h_calculada: Option<f64>,
    pub modo: Option<TipoDeReceptor>
}

impl ATRecord {
    pub fn aplicar_eh(&self, eh: EHRecord) -> Result<Self,anyhow::Error>
    {
        Ok( Self {
                tipo: self.tipo.clone(),
                radio: self.radio,  
                slant_h_mp: self.slant_h_mp,
                l1h:self.l1h, l2h:self.l2h,
                h_ingresada: Some(eh.height_rod),
                h_tipo:Some(eh.tipo),
                h_calculada:self.h_calculada,
                modo: Some(eh.modo)
            }
        )
    }

    pub fn aplicar_ls(&self, ls: LSRecord) -> Result<Self,anyhow::Error>
    {
        Ok( Self {
                tipo: self.tipo.clone(),
                radio: self.radio,  
                slant_h_mp: self.slant_h_mp,
                l1h:self.l1h, l2h:self.l2h,
                h_ingresada: self.h_ingresada,
                h_tipo:self.h_tipo,
                h_calculada:Some(ls.height_rod),
                modo: self.modo
            }
        )
    }
}

#[derive(Debug,Copy,Clone,Serialize,PartialEq)]
pub enum TipoDeAltura {
    Vertical,
    Inclinada,
    AAltimetria
}

#[derive(Debug,Copy,Clone,Serialize,PartialEq)]
pub enum TipoDeReceptor {
    Base,
    Rotador
}

#[derive(Debug,Copy,Clone,Serialize)]
pub struct EHRecord {
    modo: TipoDeReceptor,
    height_rod: f64,
    tipo: TipoDeAltura
}

impl TryFrom<&str> for TipoDeAltura {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim().to_lowercase().as_str() {
            "altura vertical" => Ok(TipoDeAltura::Vertical),
            "altura inclinada" => Ok(TipoDeAltura::Inclinada),
            "altura inclinada a altimetria" => Ok(TipoDeAltura::AAltimetria),
            _ => Err(anyhow!(format!("'{}' no es un tipo de altura válido", value))),
        }
    }
}

impl TryFrom<String> for TipoDeAltura {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // Reutilizamos la implementación para &str
        TipoDeAltura::try_from(value.as_str())
    }
}

pub fn parse_gps_record(line: &str) -> Result<GPSRecord, anyhow::Error> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() >= 5 {
        let op = parts[1].trim_start_matches("PN").to_string();
        let la = parts[2].trim_start_matches("LA").parse::<f64>()?;
        let lo = parts[3].trim_start_matches("LN").parse::<f64>()?;
        let el = parts[4].trim_start_matches("EL").parse::<f64>()?;
        let mut note: String = "".to_string();
        if parts.len() == 6 {
            note = parts[5].to_string();
        };
        Ok(GPSRecord { occupy_point: op, latitude: la, longitude: lo, 
                       elevation: el, elevation_alt: None,
                       note, north: None, east: None,
                       start_time: None, end_time: None})
    } else {
        Err(anyhow!("Invalid GPS record format"))
    }
}

pub fn parse_bp_record(line: &str) -> Result<BPRecord, anyhow::Error> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() >= 9 {
        let op = parts[1].trim_start_matches("PN").to_string();
        let la = parts[2].trim_start_matches("LA").parse::<f64>()?;
        let lo = parts[3].trim_start_matches("LN").parse::<f64>()?;
        let el = parts[4].trim_start_matches("ET").parse::<f64>()?;
        let ag = parts[5].trim_start_matches("AG").parse::<f64>()?;
        let pa = parts[6].trim_start_matches("PA").parse::<f64>()?;
        let _at = parts[7].trim_start_matches("AT").to_string();
        let _sr = parts[8].trim_start_matches("SR").to_string();
        let mut note: String = "".to_string();
        if parts.len() == 10 {
            note = parts[9].to_string();
        };
        Ok(BPRecord { occupy_point: op, latitude: la, longitude: lo, 
                       elevation: el, antenna_ground: ag,
                       phase_antenna: pa, elevation_alt: None,
                       note, north: None, east: None,
                       start_time: None, end_time: None})
    } else {
        Err(anyhow!("Invalid BP record format"))
    }
}

pub fn parse_gs_record(line: &str) -> Result<GSRecord, anyhow::Error> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() >= 5 {
        let op = parts[1].trim_start_matches("PN").to_string();
        let n = parts[2].trim_start_matches("N ").parse::<f64>()?;
        let e = parts[3].trim_start_matches("E ").parse::<f64>()?;
        let el = parts[4].trim_start_matches("EL").parse::<f64>()?;
        let mut note: String = "".to_string();
        if parts.len() == 6 {
            note = parts[5].to_string();
        };
        Ok(GSRecord { occupy_point: op, north: n, east: e, elevation: el, note })
    } else {
        Err(anyhow!("Invalid GS record format"))
    }
}

pub fn parse_gt_record(line: &str) -> Result<GTRecord, anyhow::Error> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() == 6 {
        let op = parts[1].trim_start_matches("PN").to_string();
        let sw = parts[2].trim_start_matches("SW").parse::<i64>()?;
        let st = parts[3].trim_start_matches("ST").parse::<i64>()?;
        let ew = parts[4].trim_start_matches("EW").parse::<i64>()?;
        let et = parts[5].trim_start_matches("ET").parse::<i64>()?;

        
        let sdelta = Duration::from_days((sw * 7) as f64) + Duration::from_milliseconds(st as f64);
        let edelta = Duration::from_days((ew * 7) as f64) + Duration::from_milliseconds(et as f64);

        let stime = Epoch::from_gpst_duration(sdelta);

        //let stime = GSD + sdelta;
        /* Tiempos en TAI, obviamos segundo intercalar.
        let leap = Duration::seconds(count_leap_seconds(stime.into()) as i64);
        stime = stime - leap;

        if Duration::seconds(count_leap_seconds(stime.into()) as i64) > leap {
            stime = stime - Duration::seconds(1);
        };
        */

        let etime = Epoch::from_gpst_duration(edelta);

        //let etime = GSD + edelta;
        /* Tiempos en TAI, obviamos segundo intercalar.
        let leap = Duration::seconds(count_leap_seconds(etime.into()) as i64);
        etime = etime - leap;

        if Duration::seconds(count_leap_seconds(etime.into()) as i64) > leap {
            stime = stime - Duration::seconds(1);
        };
        */


        Ok(GTRecord { occupy_point: op, start: stime, end: etime })
    } else {
        Err(anyhow!("Invalid GT record format"))
    }
}

pub fn parse_ls_record(line: &str) -> Result<LSRecord, anyhow::Error> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() == 2 {
        let hr = parts[1].trim_start_matches("HR").parse::<f64>()?;

        Ok(LSRecord { height_rod: hr })
    } else {
        Err(anyhow!("Invalid LS record format"))
    }
}

pub fn parse_antenna_type_record(line: &str) -> Result<ATRecord, anyhow::Error> {
    let main_parts: Vec<&str> = line.split(':').collect();
    let parts: Vec<&str>;
    if main_parts.len() == 2 {
        parts = main_parts[1].split(',').collect();
    } else {
        return Err(anyhow!("Linea mal estructurada, muchos (:)"));
    };

    if parts.len() == 6 {
        let am_ = parts[0].to_string();
        let am = am_.trim().trim_start_matches("[").trim_end_matches("]");
        let ra = parts[1].trim_start_matches("RA").trim_end_matches("m").parse::<f64>()?;
        let shmp = parts[2].trim_start_matches("SHMP").trim_end_matches("m").parse::<f64>()?;
        let l1 = parts[3].trim_start_matches("L1").trim_end_matches("m").parse::<f64>()?;
        let l2 = parts[4].trim_start_matches("L2").trim_end_matches("m").parse::<f64>()?;
        let _no = parts[5].trim_start_matches("--").to_string();

        Ok(ATRecord { 
            tipo: am.to_string(),
            radio: ra,
            slant_h_mp: shmp,
            l1h: l1, l2h: l2, 
            h_ingresada: None,
            h_tipo: None, h_calculada: None,
            modo: None
             })
    } else {
        Err(anyhow!("Invalid Antenna Type record format"))
    }
}

pub fn parse_entered_height_record(line: &str) -> Result<EHRecord, anyhow::Error> {
    let main_parts: Vec<&str> = line.split(':').collect();
    let parts: Vec<&str>;
    let mo: TipoDeReceptor;
    if main_parts.len() == 2 {
        mo = match main_parts[0].split(" ").collect::<Vec<_>>()[1] {
            "Base" => TipoDeReceptor::Base,
            "Rover" => TipoDeReceptor::Rotador,
            _ => {panic!("Modo de receptor desconocido")}
        };
        parts = main_parts[1].split(',').collect();
    } else {
        return Err(anyhow!("Linea mal estructurada, muchos (:)"));
    };

    if parts.len() == 2 {
        let hr_ = parts[0].to_string();
        let hr = hr_.strip_suffix(" m").unwrap()
                    .trim()
                    .parse::<f64>()?;
        let ti: TipoDeAltura = parts[1].try_into()?;

        Ok(EHRecord {
            modo: mo,
            height_rod: hr,tipo: ti
             })
    } else {
        Err(anyhow!("Invalid Antenna Type record format"))
    }
}

crate::genera_try_from!(BPRecord = GSRecord, aplicar_gs);

crate::genera_try_from!(GPSRecord = GSRecord, aplicar_gs);
crate::genera_try_from!(GPSRecord = GTRecord, aplicar_gt);

crate::genera_try_from!(ATRecord = LSRecord, aplicar_ls);
crate::genera_try_from!(EHRecord => ATRecord, aplicar_eh);

pub mod serialize_ndt {
    use chrono::NaiveDateTime;
    use serde::{self, Serializer};
    //use serde::{Deserialize, Deserializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(
        date: &NaiveDateTime,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    /* // No está en uso
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
        Ok(dt)
    }
    */
}

pub mod serialize_opt_ndt {
    use chrono::NaiveDateTime;
    use serde::{self, Serializer};
    //use serde::{Deserialize, Deserializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(
        date: &Option<NaiveDateTime>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match date {
            Some(dates) => format!("{}", dates.format(FORMAT)),
            None => "".into()
        };
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    /* // No está en uso
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
        Ok(dt)
    }
    */
}


pub mod serialize_opt_nd {
    use chrono::NaiveDate;
    use serde::{self, Serializer};
    //use serde::{Deserialize, Deserializer};

    const FORMAT: &'static str = "%Y-%m-%d";

    pub fn serialize<S>(
        date: &Option<NaiveDate>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match date {
            Some(dates) => format!("{}", dates.format(FORMAT)),
            None => "".into()
        };
        serializer.serialize_str(&s)
    }
}

pub mod serialize_opt_nt {
    use chrono::NaiveTime;
    use serde::{self, Serializer};
    //use serde::{Deserialize, Deserializer};

    const FORMAT: &'static str = "%H:%M:%S";

    pub fn serialize<S>(
        date: &Option<NaiveTime>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match date {
            Some(dates) => format!("{}", dates.format(FORMAT)),
            None => "".into()
        };
        serializer.serialize_str(&s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gps_record() {
        let line = "GPS,PN1,LA-35.02154763,LN-58.26577623,EL1.244110,--esq";
        let result = parse_gps_record(line);
        println!("{:?}", result);
        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.occupy_point, "1");
        assert_eq!(record.latitude, -35.02154763);
        assert_eq!(record.longitude, -58.26577623);
        assert_eq!(record.elevation, 1.244110);
    }

    #[test]
    fn test_parse_gs_record() {
        let line = "--GS,PN0,N 6123201.7319,E 504615.1011,EL1.6280,--Base";
        let result = parse_gs_record(line);
        println!("{:?}", result);
        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.occupy_point, "0");
        assert_eq!(record.north, 6123201.7319);
        assert_eq!(record.east, 504615.1011);
        assert_eq!(record.elevation, 1.6280);
    }

    #[test]
    fn test_apply_gs_record() {
        let line = "GPS,PN1,LA-35.02154763,LN-58.26577623,EL1.244110,--esq";
        let gps_rec = parse_gps_record(line);
        let line = "--GS,PN1,N 6123201.7319,E 504615.1011,EL1.244110,--esc";
        let gs_rec = parse_gs_record(line);

        let record = gps_rec.unwrap().aplicar_gs(gs_rec.unwrap()).unwrap();

        assert_eq!(record.occupy_point, "1");
        assert_eq!(record.north, Some(6123201.7319));
        assert_eq!(record.east, Some(504615.1011));
        assert_eq!(record.elevation, 1.244110);
    }

    #[test]
    #[should_panic(expected = "No coinciden los registros")]
    fn test_apply_gs_record_panic() {
        let line = "GPS,PN1,LA-35.02154763,LN-58.26577623,EL1.244110,--esq";
        let gps_rec = parse_gps_record(line);
        let line = "--GS,PN0,N 6123201.7319,E 504615.1011,EL1.6280,--Base";
        let gs_rec = parse_gs_record(line);

        let record = gps_rec.unwrap().aplicar_gs(gs_rec.unwrap()).unwrap();

        assert_eq!(record.occupy_point, "1");
        assert_eq!(record.north, Some(6123201.7319));
        assert_eq!(record.east, Some(504615.1011));
        assert_eq!(record.elevation, 1.244110);
    }

    #[test]
    fn test_parse_gt_record() {
        let line = "--GT,PN1,SW2205,ST242097000,EW2205,ET242107000";
        let result = parse_gt_record(line);
        println!("{:?}", result);
        assert!(result.is_ok());
        let record = result.unwrap();
        //let e = NaiveDate::from_ymd_opt(2022,04,12).unwrap().and_hms_opt(19,15,06).unwrap(); ????
        let e = Epoch::from_gregorian_utc(2022,04,12,19,14,49,0);
        assert_eq!(record.end, e);
        //assert_eq!(record.end, "???");
    }

    #[test]
    #[should_panic(expected = "No coinciden los registros")]
    fn test_apply_gt_record_panic() {
        let line = "GPS,PN1,LA-35.02154763,LN-58.26577623,EL1.244110,--esq";
        let gps_rec = parse_gps_record(line);
        let line = "--GS,PN0,N 6123201.7319,E 504615.1011,EL1.6280,--Base";
        let gs_rec = parse_gs_record(line);
        let line = "--GT,PN1,SW2205,ST242097000,EW2205,ET242107000";
        let gt_rec = parse_gt_record(line);

        let record = gps_rec.unwrap().aplicar_gs(gs_rec.unwrap()).unwrap()
                            .aplicar_gt(gt_rec.unwrap()).expect("");

        assert_eq!(record.occupy_point, "1");
        assert_eq!(record.north, Some(6123201.7319));
        assert_eq!(record.east, Some(504615.1011));
        assert_eq!(record.elevation, 1.244110);
    }

    #[test]
    fn test_apply_gt_record() {
        let line = "GPS,PN1,LA-35.02154763,LN-58.26577623,EL1.244110,--esq";
        let gps_rec = parse_gps_record(line);
        let line = "--GS,PN1,N 6123201.7319,E 504615.1011,EL1.244110,--Base";
        let gs_rec = parse_gs_record(line);
        let line = "--GT,PN1,SW2205,ST242097000,EW2205,ET242107000";
        let gt_rec = parse_gt_record(line);

        let record = gps_rec.unwrap().aplicar_gs(gs_rec.unwrap()).expect("")
                            .aplicar_gt(gt_rec.unwrap()).unwrap();

        assert_eq!(record.occupy_point, "1");
        assert_eq!(record.north, Some(6123201.7319));
        assert_eq!(record.east, Some(504615.1011));
        assert_eq!(record.elevation, 1.244110);
    }

    #[test]
    fn test_parse_ls_record() {
        let line = "LS,HR1.4735";
        let result = parse_ls_record(line);
        println!("{:?}", result);
        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.height_rod, 1.4735);
    }

    #[test]
    fn test_parse_entered_height_record() {
        let line = "--Entered Rover HR: 1.3550 m, Altura vertical";
        let result = parse_entered_height_record(line);
        println!("{:?}", result);
        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.height_rod, 1.3550);
        assert_eq!(record.tipo, TipoDeAltura::Vertical);
    }

    #[test]
    fn test_parse_antenna_type_record() {
        let line = "--Antenna Type: [HX-CSX049A],RA0.0645m,SHMP0.0925m,L10.0260m,L20.0222m,--";
        let result = parse_antenna_type_record(line);
        println!("{:?}", result);
        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.tipo, "HX-CSX049A");
        assert_eq!(record.radio, 0.0645);
        assert_eq!(record.slant_h_mp, 0.0925);
        assert_eq!(record.l1h, 0.0260);
        assert_eq!(record.l2h, 0.0222);
    }

    #[test]
    fn test_antenna_y_ls_record() {
        let line = "--Antenna Type: [HX-CSX049A],RA0.0645m,SHMP0.0925m,L10.0260m,L20.0222m,--";
        let result = parse_antenna_type_record(line);
        let arecord = result.unwrap();

        let line = "LS,HR1.4735";
        let result = parse_ls_record(line);
        let lsrecord = result.unwrap();

        let r:Result<ATRecord,_> = (arecord,lsrecord).try_into();

        assert!(r.is_ok());

    }

    #[test]
    fn test_rover_antenna_y_ls_record() {
        let line = "--Entered Rover HR: 1.3550 m, Altura vertical";
        let result = parse_entered_height_record(line);
        let errecord = result.unwrap();

        let line = "--Antenna Type: [HX-CSX049A],RA0.0645m,SHMP0.0925m,L10.0260m,L20.0222m,--";
        let result = parse_antenna_type_record(line);
        let arecord = result.unwrap();

        let line = "LS,HR1.4735";
        let result = parse_ls_record(line);
        let lsrecord = result.unwrap();

        let r:Result<ATRecord,_> = (errecord,arecord).try_into();
        let r:Result<ATRecord,_> = (r.unwrap(),lsrecord).try_into();

        println!("{:?}",r);
        assert!(r.is_ok());

    }


    #[test]
    fn test_leap_secs() {
        // Ejemplo de uso de la función
        let input_date = NaiveDate::from_ymd_opt(1990, 12, 31).unwrap().and_hms_opt(23,59,59).unwrap();
        let leap_seconds_count = count_leap_seconds(input_date.into());
        assert_eq!(leap_seconds_count, 6);
    }

    #[test]
    fn test_parse_bp_record() {
        let line = "BP,PN0,LA-35.02255202,LN-58.26477676,ET22.0720,AG1.6890,PA1.7942,ATAPC,SRBASE,--";
        let result = parse_bp_record(line);
        println!("{:?}", result);
        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.occupy_point, "0");
        assert_eq!(record.latitude, -35.02255202);
        assert_eq!(record.longitude, -58.26477676);
        assert_eq!(record.elevation, 22.0720);
    }

    #[test]
    fn test_apply_gs_record_to_bp() {
        let line = "BP,PN0,LA-35.02255202,LN-58.26477676,ET22.0720,AG1.6890,PA1.7942,ATAPC,SRBASE,--";
        let bp_rec = parse_bp_record(line);
        let line = "--GS,PN0,N 6122887.0388,E 504872.1981,EL22.0720,--Base";
        let gs_rec = parse_gs_record(line);

        let record = bp_rec.unwrap().aplicar_gs(gs_rec.unwrap()).unwrap();

        assert_eq!(record.occupy_point, "0");
        assert_eq!(record.north, Some(6122887.0388));
        assert_eq!(record.east, Some(504872.1981));
        assert_eq!(record.elevation, 22.0720);
    }
}
