use crate::{file_parser::Record, record_parser::TRecord, record_parser_gps::{ATRecord, BPRecord, GPSRecord, TipoDeReceptor}};
use chrono::NaiveDateTime;
use itertools::Itertools;
use std::{collections::{BTreeMap, HashMap}, convert::TryInto};
use rinex::{hardware::Antenna, marker::GeodeticMarker, observation::{event as rxevent, EpochFlag}, prelude::{Epoch, GroundPosition}};

// Funciones para grupos
pub fn gps_gs_gt_3(registros: Vec<Record>) -> Vec<Record> {
    let mut skip: usize = 0;
    registros.into_iter().circular_tuple_windows().filter_map(|t| {
        if skip > 0 {
            skip = skip - 1;
            None
        } else {
        match t {
            (Record::GPS(g),Record::GS(p),Record::GT(t)) => {skip = 2; Some(Record::GPS(g.aplicar_gs(p).expect("PANICO").aplicar_gt(t).expect("PANICO")))},
            (a,_,_) => Some(a)
       }}}).collect()
}

// Las funciones de combinación se cargan con este macro
// desde las implementaciones de From<(...,...)> de cada record.
macro_rules! genera_rama {
    ($g:ident, $p:ident, $output:path, $skip:ident) => {
        match ($g.clone(), $p).try_into() {
            Ok(v) => {
                $skip = 1;
                Some($output(v))
            }
            Err(_) => Some($g.into()),
        }
    };
}

pub fn combinar_registros(registros: Vec<Record>) -> Vec<Record> {
    let mut skip: usize = 0;
    registros
        .into_iter()
        .circular_tuple_windows()
        .filter_map(|t| {
            if skip > 0 {
                skip -= 1;
                None
            } else {
                match t {
                    (Record::GPS(g), Record::GS(p)) => genera_rama!(g, p, Record::GPS, skip),
                    (Record::GPS(g), Record::GT(p)) => genera_rama!(g, p, Record::GPS, skip),
                    (Record::BP(g), Record::GS(p)) => genera_rama!(g, p, Record::BP, skip),
                    (Record::AT(g), Record::LS(p)) => genera_rama!(g, p, Record::AT, skip),
                    (Record::EH(g), Record::AT(p)) => genera_rama!(g, p, Record::AT, skip),
                    (Record::T(g), Record::T(p)) => genera_rama!(g, p, Record::T, skip),
                    _ => Some(t.0),
                }
            }
        })
        .collect()
}

#[derive(Default)]
pub struct RelevamientoGNSS<'a> {
    records: Vec<&'a Record>,
    bases: BTreeMap<Epoch, (&'a BPRecord,&'a ATRecord)>,
    antenas_b: BTreeMap<Epoch, &'a ATRecord>,
    antenas_r: BTreeMap<Epoch, &'a ATRecord>,
    puntos: BTreeMap<Epoch, (&'a GPSRecord,&'a ATRecord)>,
}

impl <'a> RelevamientoGNSS<'a> {
    pub fn new(records: &'a Vec<Record>) -> RelevamientoGNSS {
        let rs: Vec<&'a Record> = records.iter().collect();

        RelevamientoGNSS {
            records: rs,
            bases: BTreeMap::default(),
            antenas_b: BTreeMap::default(),
            antenas_r: BTreeMap::default(),
            puntos: BTreeMap::default(),
            }
    }

    pub fn consolidar_antenas(&mut self) -> &Self
    {
        let mut antena_b = &ATRecord::default();
        let mut antena_r = &ATRecord::default();
        let mut reloj = &TRecord::default();

        for r in self.records.iter() {
            match r {
                Record::T(rr) => {
                    reloj = rr;
                    },
                Record::AT(rr) => {
                    match rr.modo {
                        Some(TipoDeReceptor::Base) => if rr != antena_b
                        {
                            self.antenas_b.insert(
                                reloj.clone().get_epoch(),
                                rr);
                            antena_b = rr;
                        },
                        Some(TipoDeReceptor::Rotador) => if rr != antena_r
                        {
                            self.antenas_r.insert(
                                reloj.clone().get_epoch(),
                                rr);
                            antena_r = rr;
                        },
                        _ => ()
                    }
                },
                _ => ()
            }
        };
        self
    }

    pub fn consolidar_bases(&mut self) -> &Self {
        let mut ultima_base = &BPRecord::default();
        let mut ultima_antena = &ATRecord::default();
        let mut reloj = &TRecord::default();

        for r in self.records.iter() {
            match r {
                Record::T(rr) => {
                    reloj = rr;
                },
                Record::BP(bp) => {
                    // Buscar la antena vigente en el BTreeMap antenas_b
                    if let Some((_, antena)) = self.antenas_b.range(..=reloj.get_epoch()).last() {
                        if bp != ultima_base || antena != &ultima_antena {
                            self.bases.insert(reloj.get_epoch(), (bp, antena));
                            ultima_base = bp;
                            ultima_antena = antena;
                        }
                    }
                },
                _ => (),
            }
        }
        self
    }

    pub fn consolidar_puntos(&mut self) -> &Self {

        let mut reloj = &TRecord::default();

        for r in self.records.iter() {
            match r {
                Record::T(rr) => {
                    reloj = rr;
                },
                Record::GPS(gps) => {
                    let start_time = gps.start_time.unwrap_or(reloj.get_epoch()); 
                    // Buscar la antena vigente en el BTreeMap antenas_r
                    let antena = match self.antenas_r.range(..=start_time).last() {
                        None => {eprintln!("Sin antena definida {}, saltea",start_time);
                        continue;},
                        Some((_,a)) => a
                    };

                    if start_time < reloj.get_epoch() {
                        eprintln!("Tiempo en reversa {}, saltea",start_time);
                        continue;
                    }

                    self.puntos.insert(start_time, (gps, antena));
                },
                _ => ()
            }
        };
        self
    }

    pub fn puntos_a_eventos(&mut self) -> rxevent::Record {
        self.puntos.iter().map(|(k,(p,a))| {
            let marker = GeodeticMarker::default();
            let pos = GroundPosition::from_geodetic((p.latitude,p.longitude,p.elevation));
            let ant = Antenna::default();

            let ev_info = rxevent::Event {
                    comments: vec![],
                    geodetic_marker: Some(
                        marker.with_name(p.occupy_point.as_str()
                            )),
                    ground_position: Some(pos),
                    rcvr_antenna: Some(
                        ant.with_model(&a.tipo)
                        .with_height(a.h_calculada.unwrap())
                        )
                };
            [((p.start_time.unwrap(),EpochFlag::NewSiteOccupation),(None,ev_info)),
             ((p.end_time.unwrap(),EpochFlag::AntennaBeingMoved),(None,rxevent::Event::default()))]
        }).flatten().collect()
    }
}

pub fn registros_a_eventos(registros_gps: Vec<Record>) -> rxevent::Record
{

        let mut rel = RelevamientoGNSS::new(&registros_gps);
        rel.consolidar_antenas();
        rel.consolidar_bases();
        rel.consolidar_puntos();
        
        let evt_record = rel.puntos_a_eventos();
        evt_record
}


#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use rinex::{observation::HeaderFields, record, writer::BufferedWriter};

    use crate::{file_parser::{leer_archivo_y_parsear, Record}, post_parse_gps::{combinar_registros, gps_gs_gt_3}};

    use super::RelevamientoGNSS;

    #[test]
    fn test_gps_gs_gt_3()
    {
        let result = leer_archivo_y_parsear(std::path::Path::new("tests/test.rw5"));
        let largo_total = result.registros.len();
        let n_gps = 555;

        let registros_gps = gps_gs_gt_3(result.registros);
        // println!("{:?}",registros_gps);
        // Probar que se consumieron 2 líneas por cada GPS.
        assert_eq!(registros_gps.len(), largo_total - (2 * n_gps));
    }

    #[test]
    fn test_gps_gs_gt()
    {
        let result = leer_archivo_y_parsear(std::path::Path::new("tests/test.rw5"));
        let largo_total = result.registros.len();
        let n_gps = 555;
        let n_bp = 26;

        let mut largo_n = largo_total;
        let mut largo_p = 0;
        let mut registros_gps  = result.registros;

        assert_eq!(registros_gps.clone().iter().filter_map(|r| match r {
            Record::GPS(_) => Some(1),
            _ => None
        }).collect::<Vec<_>>().len(), n_gps);

        assert_eq!(registros_gps.clone().iter().filter_map(|r| match r {
            Record::GS(_) => Some(1),
            _ => None
        }).collect::<Vec<_>>().len(), n_gps + n_bp);

        assert_eq!(registros_gps.clone().iter().filter_map(|r| match r {
            Record::GT(_) => Some(1),
            _ => None
        }).collect::<Vec<_>>().len(), n_gps);

        assert_eq!(registros_gps.clone().iter().filter_map(|r| match r {
            Record::T(_) => Some(1),
            _ => None
        }).collect::<Vec<_>>().len(), (n_gps + n_bp) * 2);

        assert_eq!(registros_gps.clone().iter().filter_map(|r| match r {
            Record::AT(_) => Some(1),
            _ => None
        }).collect::<Vec<_>>().len(), (n_gps + n_bp) );

        assert_eq!(registros_gps.clone().iter().filter_map(|r| match r {
            Record::EH(_) => Some(1),
            _ => None
        }).collect::<Vec<_>>().len(), (n_gps + n_bp) );
        
        assert_eq!(registros_gps.clone().iter().filter_map(|r| match r {
            Record::LS(_) => Some(1),
            _ => None
        }).collect::<Vec<_>>().len(), n_gps );

        assert_eq!(registros_gps.clone().iter().filter_map(|r| match r {
            Record::BP(_) => Some(1),
            _ => None
        }).collect::<Vec<_>>().len(), n_bp );

        while largo_n != largo_p
        {
            largo_p = largo_n;
            registros_gps = combinar_registros(registros_gps);
            largo_n = registros_gps.len();
        };

        //println!("{:?}",registros_gps);
        // Probar que se consumieron 2 líneas por cada GPS y una por cada BP.
        // Probar que se consumieron 2 líneas por cada antena GPS y una por cada antena BP.
        assert_eq!(registros_gps.clone().iter().filter_map(|r| match r {
            Record::GPS(_) => Some(1),
            _ => None
        }).collect::<Vec<_>>().len(), n_gps);

        assert_eq!(registros_gps.clone().iter().filter_map(|r| match r {
            Record::GS(_) => Some(1),
            _ => None
        }).collect::<Vec<_>>().len(), 0);

        assert_eq!(registros_gps.clone().iter().filter_map(|r| match r {
            Record::GT(_) => Some(1),
            _ => None
        }).collect::<Vec<_>>().len(), 0);

        assert_eq!(registros_gps.clone().iter().filter_map(|r| match r {
            Record::T(_) => Some(1),
            _ => None
        }).collect::<Vec<_>>().len(), (n_gps + n_bp));

        assert_eq!(registros_gps.clone().iter().filter_map(|r| match r {
            Record::AT(_) => Some(1),
            _ => None
        }).collect::<Vec<_>>().len(), (n_gps + n_bp) );

        assert_eq!(registros_gps.clone().iter().filter_map(|r| match r {
            Record::EH(_) => Some(1),
            _ => None
        }).collect::<Vec<_>>().len(), 0 );

        assert_eq!(registros_gps.clone().iter().filter_map(|r| match r {
            Record::LS(_) => Some(1),
            _ => None
        }).collect::<Vec<_>>().len(), 0 );

        // una antena por cada punto y cada base
        assert_eq!(registros_gps.len(), 3 * (n_bp + n_gps));


    }
    

    #[test]
    fn test_gps_to_json()
    {
        let result = leer_archivo_y_parsear(std::path::Path::new("tests/test.rw5"));
        let largo_total = result.registros.len();

        let mut largo_n = largo_total;
        let mut largo_p = 0;
        let mut registros_gps  = result.registros;
        
        while largo_n != largo_p
        {
            largo_p = largo_n;
            registros_gps = combinar_registros(registros_gps);
            largo_n = registros_gps.len();
        };

        let json = serde_json::to_string_pretty(&registros_gps);

        assert!(json.is_ok());

        //println!("{:?}",serde_json::to_string_pretty(&registros_gps));
        assert!(json.expect("").len() > 0)
    }

    #[test]
    fn test_default_relevamiento()
    {
        let _r = RelevamientoGNSS::default();
    }

    #[test]
    fn test_consolidar_antenas()
    {
        let result = leer_archivo_y_parsear(std::path::Path::new("tests/test.rw5"));
        let largo_total = result.registros.len();

        let mut largo_n = largo_total;
        let mut largo_p = 0;
        let mut registros_gps  = result.registros;
        
        while largo_n != largo_p
        {
            largo_p = largo_n;
            registros_gps = combinar_registros(registros_gps);
            largo_n = registros_gps.len();
        };
        
        let mut rel = RelevamientoGNSS::new(&registros_gps);

        rel.consolidar_antenas();
        
        assert!(rel.antenas_b.len() > 0);
        assert!(rel.antenas_r.len() > 0);
    }

    #[test]
    fn test_consolidar_bases()
    {
        let result = leer_archivo_y_parsear(std::path::Path::new("tests/test.rw5"));
        let largo_total = result.registros.len();

        let mut largo_n = largo_total;
        let mut largo_p = 0;
        let mut registros_gps  = result.registros;
        
        while largo_n != largo_p
        {
            largo_p = largo_n;
            registros_gps = combinar_registros(registros_gps);
            largo_n = registros_gps.len();
        };
        
        let mut rel = RelevamientoGNSS::new(&registros_gps);

        rel.consolidar_antenas();
        
        rel.consolidar_bases();
        println!("{:?}",rel.bases);
    }

    #[test]
    fn test_consolidar_puntos()
    {
        let result = leer_archivo_y_parsear(std::path::Path::new("tests/test.rw5"));
        let largo_total = result.registros.len();

        let mut largo_n = largo_total;
        let mut largo_p = 0;
        let mut registros_gps  = result.registros;
        
        while largo_n != largo_p
        {
            largo_p = largo_n;
            registros_gps = combinar_registros(registros_gps);
            largo_n = registros_gps.len();
        };
        
        let mut rel = RelevamientoGNSS::new(&registros_gps);

        rel.consolidar_antenas();
        
        rel.consolidar_bases();

        rel.consolidar_puntos();
        println!("{:?}",rel.puntos);
    }


    use tempfile::NamedTempFile;
    use std::io::Write;
    use std::io::prelude::*;
    use std::fs::File;

    #[test]
    fn test_a_eventos()
    {
        let result = leer_archivo_y_parsear(std::path::Path::new("tests/test.rw5"));
        let largo_total = result.registros.len();

        let mut largo_n = largo_total;
        let mut largo_p = 0;
        let mut registros_gps  = result.registros;
        
        while largo_n != largo_p
        {
            largo_p = largo_n;
            registros_gps = combinar_registros(registros_gps);
            largo_n = registros_gps.len();
        };
        
        let mut rel = RelevamientoGNSS::new(&registros_gps);

        rel.consolidar_antenas();
        
        rel.consolidar_bases();

        rel.consolidar_puntos();
        
        let evt_record = rel.puntos_a_eventos();

        let re = record::Record::ObsEvtRecord( BTreeMap::default(), evt_record);


        let file = NamedTempFile::new().expect("panic!");

        {
            let mut wter = BufferedWriter::new(&file.path().to_string_lossy()).expect("");

            let mut head = rinex::header::Header::default();

            head = head.with_observation_fields(HeaderFields::default());

            re.to_file(&head , &mut wter).expect("");
        }

        let mut file = File::open(file.path()).expect("Unable to open the file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Unable to read the file");

        // println!("{}", contents);
        let vline: Vec<&str> = contents.lines().collect();
        assert!(vline.len() > 1)

    }
}
