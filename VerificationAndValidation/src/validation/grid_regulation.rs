use std::{fs::File, io::Write};

use polars::export::rayon::prelude::{IntoParallelIterator, ParallelIterator};
use polars::prelude::*;

pub fn capacity_per_15_minutes(folder: &String) {
    let filename = format!("./{folder}/tennet/csv/regulatingmargin.csv");
    let df = CsvReader::from_path(filename).unwrap().finish().unwrap();
    let mut selected = df.select(["Immediately", "less_than_15_min"]).unwrap();
    println!("Regulating capacity per 15 minutes - max inc per step -- values are in MW");
    println!("{}", selected.describe(None));
    println!("------------ \n");


    let title = "Regulating capacity per 15 minutes - max inc per step -- values are in MW";
    let file = "regulation_capacity_15min_MW";
    let mut text_file = File::create(&format!("./output/validation/grid/{file}.txt")).unwrap();
    text_file.write_all(title.as_bytes()).unwrap();
    text_file.write_all(selected.describe(None).to_string().as_bytes());

    let mut write_file = CsvWriter::new(File::create(&format!("./output/validation/grid/{file}.csv")).expect(&file));
    write_file.finish(&mut selected).unwrap();

    
}

pub fn regulation_room(folder: &String) {
    let filename = format!("./{folder}/tennet/csv/regulatingmargin.csv");
    let df = CsvReader::from_path(filename).unwrap().finish().unwrap();
    let selected = df
        .select([
            "Immediately",
            "less_than_15_min",
            "16_30_min",
            "31_120_min",
            "121_480_min",
            "more_than_480_min",
        ])
        .unwrap();

    let mut summed = selected
        .lazy()
        .select([fold_exprs(lit(0), |acc, x| Ok(acc + x), [col("*")]).alias("sum")])
        .collect()
        .unwrap();
    
    println!("Regulating capacity total - Energy storage -- values are in MW");
    println!("{}", summed.describe(None));
    println!("------------ \n");

    let title = "Regulating capacity total - Energy storage -- values are in MW";
    let file = "energy_storage_total";
    let mut text_file = File::create(&format!("./output/validation/grid/{file}.txt")).unwrap();
    text_file.write_all(title.as_bytes()).unwrap();
    text_file.write_all(summed.describe(None).to_string().as_bytes());

    let mut write_file = CsvWriter::new(File::create(&format!("./output/validation/grid/{file}.csv")).expect(&file));
    write_file.finish(&mut summed).unwrap();
    
}

pub fn power_grid_consumption(folder: &String) {
    let filename = format!("./{folder}/tennet/csv/volumesettledimbalance.csv");
    let df = CsvReader::from_path(filename).unwrap().finish().unwrap();
    let mut selected = df.select(["Absolute"]).unwrap();
    println!("Power grid consumption - bulk consumption with households");
    println!("{}", selected.describe(None));
    println!("------------ \n");

    let title = "Power grid consumption - bulk consumption with households";
    let file = "grid_consumption_bulk_consumption";
    let mut text_file = File::create(&format!("./output/validation/grid/{file}.txt")).unwrap();
    text_file.write_all(title.as_bytes()).unwrap();
    text_file.write_all(selected.describe(None).to_string().as_bytes());

    let mut write_file = CsvWriter::new(File::create(&format!("./output/validation/grid/{file}.csv")).expect(&file));
    write_file.finish(&mut selected).unwrap();
}

pub fn domestic_power_grid_consumption(folder: &String) {
    let filename = format!("./{folder}/tennet/csv/measurementdata.csv");
    let df = CsvReader::from_path(filename).unwrap().finish().unwrap();
    let mut selected = df.select(["Measured exchange"]).unwrap();
    println!(
        "Domestic power grid consumption - bulk consumption with households -- values are in MWh/PTE (15 minutes)"
    );
    println!("{}", selected.describe(None));
    println!("------------ \n");

    let title = "Domestic power grid consumption - bulk consumption with households -- values are in MWh/PTE (15 minutes)";
    let file = "domestic_grid_consumption_bulk_with_households";
    let mut text_file = File::create(&format!("./output/validation/grid/{file}.txt")).unwrap();
    text_file.write_all(title.as_bytes()).unwrap();
    text_file.write_all(selected.describe(None).to_string().as_bytes());

    let mut write_file = CsvWriter::new(File::create(&format!("./output/validation/grid/{file}.csv")).expect(&file));
    write_file.finish(&mut selected).unwrap();
}
