use anyhow::anyhow;
use serde::Serialize;
use std::fs;
use crate::post_parse_gps;
use crate::post_parse_gps::RelevamientoGNSS;
use crate::record_parser_gps as gps;
use crate::record_parser as rec;
use derive_more::From;

#[derive(Debug,Clone,Serialize,From)]
pub enum Record {
    GPS (gps::GPSRecord),
    GS (gps::GSRecord),
    GT (gps::GTRecord),
    BP (gps::BPRecord),
    AT (gps::ATRecord),
    EH (gps::EHRecord),
    LS (gps::LSRecord),
    T  (rec::TRecord)
}


impl Record {
    fn from_line( line: &str ) -> Result<Self, anyhow::Error>
    {

    if line.is_empty() {
        return Err(anyhow!("Linea Vacia"));
    }

    // Extraer el tipo de registro
    let record_type = line.split(',').flat_map(|s| s.split(':')).next().unwrap_or("").trim();

    match record_type {
        "GPS" => Ok(Record::GPS(gps::parse_gps_record(line)?)),
        "--GS" => Ok(Record::GS(gps::parse_gs_record(line)?)),
        "--GT" => Ok(Record::GT(gps::parse_gt_record(line)?)),
        "BP" => Ok(Record::BP(gps::parse_bp_record(line)?)),
        "--Entered Rover HR" => Ok(Record::EH(gps::parse_entered_height_record(line)?)),
        "--Entered Base HR" => Ok(Record::EH(gps::parse_entered_height_record(line)?)),
        "--Antenna Type" => Ok(Record::AT(gps::parse_antenna_type_record(line)?)),
        "LS" => Ok(Record::LS(gps::parse_ls_record(line)?)),
        &_ => if record_type.len() >= 4 
        { 
            match record_type.split_at(4) 
            { 
                ("--DT",_) | ("--TM",_) => Ok(Record::T(rec::parse_dt_record(line)?)), 
                _ => Err(anyhow!("Tipo de registro desconocido"))
            }
        } else { Err(anyhow!("Tipo de registro desconocido")) }
    }

    }
}

#[derive(Debug)]
pub struct ResultadoDeParseo{
    pub registros: Vec<Record>,
    pub errores: Vec<(String,anyhow::Error)>
}

pub fn lineas_a_registros(lineas: Vec<&str>) -> Result<ResultadoDeParseo,anyhow::Error> {
    let mut errores: Vec<(String,anyhow::Error)> = vec![];
    
    let registros: Vec<Record> = lineas.into_iter().filter_map( |linea|
        match Record::from_line(&linea) {
            Ok(v) => Some(v),
            Err(e) => {errores.extend(vec![(linea.to_string(),e)]);
                       None}
    }).collect();

    Ok(ResultadoDeParseo { registros, errores })
}

pub fn leer_archivo_y_parsear(archivo: &std::path::Path) -> ResultadoDeParseo
{
   let contenido = fs::read_to_string(archivo).expect("No se pudo abrir el archivo");
   let lineas = contenido.lines().collect();

   match lineas_a_registros(lineas) {
       Ok(v) => v,
       Err(_) => {panic!("No se pudo procesar el archivo")}
   }
}

pub fn de_archivo_a_registros(archivo: &std::path::Path) -> Vec<Record>
{
    let result = leer_archivo_y_parsear(archivo);
    let largo_total = result.registros.len();

    let mut largo_n = largo_total;
    let mut largo_p = 0;
    let mut registros_gps  = result.registros;
    
    while largo_n != largo_p
    {
        largo_p = largo_n;
        registros_gps = post_parse_gps::combinar_registros(registros_gps);
        largo_n = registros_gps.len();
    };
    
    registros_gps
}

// pub fn vec_a_rel<'a> (registros_gps: &'a Vec<Record>) -> &'a RelevamientoGNSS
// {
//     let mut rel = RelevamientoGNSS::new(registros_gps);
// 
//     rel.consolidar_antenas();
//     
//     rel.consolidar_bases()
// }

#[cfg(test)]
mod test {
    use crate::file_parser::leer_archivo_y_parsear;

    use super::lineas_a_registros;
    use super::Record;

    #[test]
    fn parse_y_es_gps()
    {
        let linea = "GPS,PN8,LA-35.02134388,LN-58.27006705,EL0.899195,--casa1";
        let r = Record::from_line(linea);

        match r {
            Ok(Record::GPS(_)) => assert!(true),
            _ => assert!(false,"{:?}",r),
        }
    }

    #[test]
    fn parse_y_es_gs()
    {
        let linea = "--GS,PN8,N 6123259.5201,E 504545.3544,EL0.8992,--casa1";
        let r = Record::from_line(linea);

        match r {
            Ok(Record::GS(_)) => assert!(true),
            _ => assert!(false,"{:?}",r),
        }
    }

    #[test]
    fn parse_y_es_gt()
    {
        let linea = "--GT,PN8,SW2205,ST244149000,EW2205,ET244150000";
        let r = Record::from_line(linea);

        match r {
            Ok(Record::GT(_)) => assert!(true),
            _ => assert!(false,"{:?}",r),
        }
    }

    #[test]
    fn test_parse_rw5_string_gps() {
        let content = "\
BP,PN0,LA-35.02153129,LN-58.26579176,ET1.6280,AG1.5230,PA1.6278,ATAPC,SRBASE,--
--GS,PN0,N 6123201.7319,E 504615.1011,EL1.6280,--Base
--Entered Rover HR: 0.5000 m, Altura vertical
--Antenna Type: [HX-CSX049A],RA0.0645m,SHMP0.0925m,L10.0260m,L20.0222m,--
LS,HR0.6185
GPS,PN1,LA-35.02154763,LN-58.26577623,EL1.244110,--esq
--GS,PN1,N 6123196.6946,E 504619.0351,EL1.2441,--esq
--GT,PN1,SW2205,ST242097000,EW2205,ET242107000
--DT04-12-2022
--TM16:15:06
--Valid Readings: 10 of 10
--Fixed Readings: 10 of 10
--Nor Min: 6123196.6893  Max: 6123196.7030
--Eas Min: 504619.0294  Max: 504619.0410
--Elv Min: 1.2421  Max: 1.2456
--Nor Avg: 6123196.6946  SD: 0.0040
--Eas Avg: 504619.0351  SD: 0.0038
--Elv Avg: 1.2441  SD: 0.0011
--NRMS Avg: 0.0031 SD: 0.0004 Min: 0.0026 Max: 0.0036
--ERMS Avg: 0.0031 SD: 0.0004 Min: 0.0026 Max: 0.0036
--HSDV Avg: 0.0044 SD: 0.0005 Min: 0.0037 Max: 0.0051
--VSDV Avg: 0.0060 SD: 0.0006 Min: 0.0051 Max: 0.0068
--HDOP Avg: 0.5300 Min: 0.5300 Max: 0.5300
--VDOP Avg: 0.7300 Min: 0.7300 Max: 0.7300
--PDOP Avg: 0.9400 Min: 0.9400 Max: 0.9400
--AGE Avg: 1.0000 Min: 1.0000 Max: 1.0000
--Number of Satellites Avg: 20 Min: 20 Max: 20
--Entered Rover HR: 0.5000 m, Altura vertical
--Antenna Type: [HX-CSX049A],RA0.0645m,SHMP0.0925m,L10.0260m,L20.0222m,--
LS,HR0.6185
GPS,PN2,LA-35.02156271,LN-58.26590066,EL1.210150,--esq
--GS,PN2,N 6123192.0617,E 504587.4940,EL1.2102,--esq
--GT,PN2,SW2205,ST242198000,EW2205,ET242209000
--DT04-12-2022
--TM16:16:48
--Valid Readings: 10 of 10
--Fixed Readings: 10 of 10
--Nor Min: 6123192.0586  Max: 6123192.0650
--Eas Min: 504587.4891  Max: 504587.5012
--Elv Min: 1.2078  Max: 1.2131
--Nor Avg: 6123192.0617  SD: 0.0022
--Eas Avg: 504587.4940  SD: 0.0036
--Elv Avg: 1.2102  SD: 0.0015
--NRMS Avg: 0.0026 SD: 0.0002 Min: 0.0019 Max: 0.0029
--ERMS Avg: 0.0026 SD: 0.0002 Min: 0.0019 Max: 0.0029
--HSDV Avg: 0.0036 SD: 0.0004 Min: 0.0027 Max: 0.0041
--VSDV Avg: 0.0052 SD: 0.0005 Min: 0.0040 Max: 0.0060
--HDOP Avg: 0.5300 Min: 0.5300 Max: 0.5300
--VDOP Avg: 0.7300 Min: 0.7300 Max: 0.7300
--PDOP Avg: 0.9300 Min: 0.9300 Max: 0.9300
--AGE Avg: 1.0000 Min: 1.0000 Max: 1.0000
--Number of Satellites Avg: 20 Min: 20 Max: 20
--Entered Rover HR: 0.5000 m, Altura vertical
--Antenna Type: [HX-CSX049A],RA0.0645m,SHMP0.0925m,L10.0260m,L20.0222m,--
LS,HR0.6185
GPS,PN3,LA-35.02146494,LN-58.27023241,EL1.280220,--esq
--GS,PN3,N 6123222.2352,E 504503.4234,EL1.2802,--esq
--GT,PN3,SW2205,ST242756000,EW2205,ET242766000";
        
        let result = lineas_a_registros(content.lines().collect());
        //println!("{:?}",result);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_archivo()
    {
        let result = leer_archivo_y_parsear(std::path::Path::new("tests/test.rw5"));
        //pprintln!("{:?}",result.registros);
        assert_eq!(result.registros.len(), 4596);
    }


}
