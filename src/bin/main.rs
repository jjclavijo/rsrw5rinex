use std::path::PathBuf;

use rinex::{observation::{event, EpochFlag}, record, Rinex};
use rw5_file_parser::{file_parser::de_archivo_a_registros, post_parse_gps::registros_a_eventos};
use clap::Parser;

#[derive(Parser,Debug)]
struct Arguments {
    rwpath: PathBuf,
    rxpath: PathBuf,
    outpath: PathBuf,
}

fn main() {
        
    let args = Arguments::parse();

    let registros_gps = de_archivo_a_registros(&args.rwpath);

    let all_evt_record = registros_a_eventos(registros_gps);

    
    let rx = Rinex::from_file(&args.rxpath.to_string_lossy()).unwrap();

    let obs_record = rx.record.as_obs().unwrap();

    let e1 = obs_record.keys().next().unwrap().clone();
    let e2 = obs_record.keys().rev().next().unwrap().clone();

    let mut evt_record: event::Record = 
        all_evt_record.range(e1..e2)
        .map(|(k, v)| (*k, v.clone())).collect();

    // flag de que el archivo es cinemático
    evt_record.insert((e1.0,EpochFlag::AntennaBeingMoved),(None,event::Event::default()));


    let re = record::Record::ObsEvtRecord(obs_record.clone(), evt_record);

    rx.with_record(re).to_file(&args.outpath.to_string_lossy())
        .expect("Falló la escritura del Rinex");
}
