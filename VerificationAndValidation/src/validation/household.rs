use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};

use plotly::{Bar, Histogram, ImageFormat, Plot, Scatter, Layout};
use polars::lazy::dsl::*;
use polars::lazy::prelude::*;
use polars::prelude::*;

pub fn year_power_usage(folder: &String) {
    let filename = format!("./{folder}/enexis/Klein verbruik/Enexis_kleinverbruiksgegevens_01012022.csv");
    let df = CsvReader::from_path(filename)
        .unwrap()
        .with_delimiter(b';')
        .finish()
        .unwrap();
    let mut selected = df.select(["SJA_GEMIDDELD"]).unwrap();
    let mask = df.column("PRODUCTSOORT").unwrap().equal("ELK").unwrap();
    selected = selected.filter(&mask).unwrap();

    let mut write_file = BufWriter::new(File::create(format!("./output/validation/household/usage_describe.txt")).unwrap());
    write_file.write_all(b"values in kWh\n").unwrap();
    write_file
        .write_all(selected.describe(None).to_string().as_bytes())
        .unwrap();
    println!("Household year consumption -- values are in kWh");
    println!("{:?}", selected.describe(None));
    println!("------------ \n");
    
    let mut write_file = CsvWriter::new(File::create(&format!("./output/validation/household/household_year_usage.csv")).expect(&"household_year_usage"));
    write_file.finish(&mut selected).unwrap();

    let select_column = df.column("SJA_GEMIDDELD").unwrap().filter(&mask).unwrap();
    let data_vec: Vec<f64> = select_column.f64().unwrap().into_no_null_iter().collect();
    plot_year_usage(folder, data_vec);

}


fn plot_year_usage(folder: &String, data: Vec<f64>) {
    let mut plot = Plot::new();
    let trace = Histogram::new(data);
    let mut layout = Layout::new().title("<b>Histogram - Household consumption per year</b>".into());
    plot.set_layout(layout);

    plot.add_trace(trace);
    plot.write_image(
        format!("./output/validation/household/usage_histogram.png"),
        ImageFormat::PNG,
        1200, 800,
        1.0,
    );
    plot.write_html(format!("./output/validation/household/usage_histogram.html"));
}

#[tokio::main]
pub async fn day_usage(folder: &String) {
    let filename =
        format!("./{folder}/MFFBAS/Profielen elektriciteit 2023/Standaardprofielen elektriciteit 2023 versie 1.00.csv");
    let df = CsvReader::from_path(filename)
        .unwrap()
        .with_delimiter(b';')
        .with_parse_dates(true)
        .finish()
        .unwrap();
    let mut handles = vec![];

    handles.push(tokio::spawn({
        let categories_names = ["AMI", "AZI", "AMI_A", "AMI_I", "AZI_A", "AZI_I"];
        let filename = format!("./output/validation/household/day_usage_all");
        parse_column_names(categories_names.to_vec(), filename, df.clone())
    }));
    handles.push(tokio::spawn({
        let categories_names = ["AMI", "AMI_A", "AMI_I"];
        let filename = format!("./output/validation/household/day_usage_AMI");
        parse_column_names(categories_names.to_vec(), filename, df.clone())
    }));
    handles.push(tokio::spawn({
        let categories_names = ["AZI", "AZI_A", "AZI_I"];
        let filename = format!("./output/validation/household/day_usage_AZI");
        parse_column_names(categories_names.to_vec(), filename, df.clone())
    }));
    handles.push(tokio::spawn({
        let categories_names = ["AMI_I", "AZI_I"];
        let filename = format!("./output/validation/household/day_usage_IN");
        parse_column_names(categories_names.to_vec(), filename, df.clone())
    }));
    handles.push(tokio::spawn({
        let categories_names = ["AMI_A", "AZI_A"];
        let filename = format!("./output/validation/household/day_usage_OUT");
        parse_column_names(categories_names.to_vec(), filename, df.clone())
    }));
    handles.push(tokio::spawn({
        let categories_names = ["AMI", "AZI"];
        let filename = format!("./output/validation/household/day_usage_AMI_AZI");
        parse_column_names(categories_names.to_vec(), filename, df.clone())
    }));
    handles.push(tokio::spawn({
        let categories_names = ["AZI", "AMI", "AMI_I", "AMI_A"];
        parse_column_names(
            categories_names.to_vec(),
            format!("./output/validation/household/day_usage_logical"),
            df,
        )
    }));

    for h in handles {
        h.await;
    }
}

async fn parse_column_names(categories_names: Vec<&str>, graph_name: String, mut df: DataFrame) {
    let to_drop_columns: Vec<String> = df
        .get_column_names_owned()
        .into_iter()
        .filter(|cn| cn.contains("E3") || cn.contains("E4"))
        .collect();
    for cn in to_drop_columns {
        let _ = df.drop_in_place(&cn).unwrap();
    }

    let tijd_series = df.column("Tijd").unwrap().to_owned();
    let tijd_series_vec: Vec<String> = tijd_series
        .utf8()
        .unwrap()
        .into_iter()
        .map(|s| s.unwrap().split_once(' ').unwrap().1.to_owned())
        .collect();
    let tijd_series = Series::new("Tijd", tijd_series_vec.clone());

    let categories: HashMap<String, Series> = categories_names
        .into_iter()
        .map(|name| {
            let columns: Vec<String> = df
                .get_column_names_owned()
                .into_iter()
                .filter(|cn| cn.contains(name))
                .collect();
            (
                name.to_owned(),
                df.clone()
                    .lazy()
                    .select([fold_exprs(lit(0), |acc, x| Ok(acc + x), &[cols(columns)]).alias(name)])
                    .collect()
                    .unwrap()
                    .column(name)
                    .unwrap()
                    .to_owned(),
            )
        })
        .collect();

    let tijd: Vec<String> = tijd_series_vec;
    let mut categories_vec: Vec<Series> = categories.values().cloned().collect();
    categories_vec.push(tijd_series);

    let categories_float: HashMap<String, Vec<f64>> = categories_vec
        .clone()
        .into_iter()
        .filter_map(|s| {
            if s.name() != "Tijd" {
                let floats: Vec<f64> = s.f64().unwrap().into_no_null_iter().collect();
                let total: f64 = floats.iter().sum();
                let res = floats.into_iter().map(|v| v / total).collect();
                Some((s.name().to_string(), res))
            } else {
                None
            }
        })
        .collect();

    let mut df = DataFrame::new(categories_vec).unwrap();
    df = df
        .lazy()
        .groupby_stable([col("Tijd")])
        .agg([col("*").sum()])
        .collect()
        .unwrap();
    let mut write_file = CsvWriter::new(File::create(&format!("{graph_name}.csv")).expect(&graph_name));
    write_file.finish(&mut df).unwrap();

    plot_bar(&graph_name, tijd, categories_float);
}

fn plot_bar(graph_name: &str, x: Vec<String>, traces: HashMap<String, Vec<f64>>) {
    let mut plot = Plot::new();
    plot.use_local_plotly();

    for (name, data) in traces {
        let trace = Bar::new(x.clone(), data).show_legend(true).name(name);
        plot.add_trace(trace);
    }
    let mut layout = Layout::new().title(graph_name.into());
    plot.set_layout(layout);
    plot.write_image(format!("{graph_name}.png"), ImageFormat::PNG, 1200, 800, 1.0);
    plot.write_html(format!("{graph_name}.html"));
}

fn plot_line(graph_name: &str, x: Vec<String>, traces: HashMap<String, Vec<f64>>) {
    let mut plot = Plot::new();
    plot.use_local_plotly();

    for (name, data) in traces {
        let trace = Scatter::new(x.clone(), data).show_legend(true).name(name);
        plot.add_trace(trace);
    }
    let mut layout = Layout::new().title(graph_name.into());
    plot.set_layout(layout);
    plot.write_image(format!("{graph_name}.png"), ImageFormat::PNG, 1200, 800, 1.0);
    plot.write_html(format!("{graph_name}.html"));
}

pub fn validation_consumption(folder: &String) {
    let filename = format!("./model_output/No_PV_validation_consumption.csv");
    let mut df = CsvReader::from_path(filename)
        .unwrap()
        .with_delimiter(b';')
        .with_parse_dates(true)
        .finish()
        .unwrap();

    df = df.transpose().unwrap();
    let binding = df.mean().transpose().unwrap();
    let mean = binding.column("column_0").unwrap();

    let binding = df.std(2).transpose().unwrap();
    let std = binding.column("column_0").unwrap();

    let plus = mean + std;
    let min = mean - std;

    let mut azi_df = CsvReader::from_path(format!("./output/validation/household/day_usage_AZI.csv"))
        .unwrap()
        .finish()
        .unwrap();

    let divider = azi_df.sum().column("AZI").unwrap().f64().unwrap().sum().unwrap();

    let azi: Vec<f64> = azi_df
        .column("AZI")
        .unwrap()
        .f64()
        .unwrap()
        .into_no_null_iter()
        .map(|v| v / divider)
        .collect();

    let data: HashMap<String, Vec<f64>> = HashMap::from([
        ("min".to_owned(), min.f64().unwrap().into_no_null_iter().collect()),
        ("mean".to_owned(), mean.f64().unwrap().into_no_null_iter().collect()),
        ("plus".to_owned(), plus.f64().unwrap().into_no_null_iter().collect()),
        ("azi".to_owned(), azi.clone()),
    ]);
    

    plot_line(
        &format!("./model_output/No_PV_validation_consumption"),
        get_step_to_time().into_iter().map(|v| v.to_owned()).collect(),
        data,
    );

    let mut selected = df!(
        "Time of day" => get_step_to_time(),
        "Minimum" => min.f64().unwrap().into_no_null_iter().collect::<Vec<f64>>(),
        "Mean" => mean.f64().unwrap().into_no_null_iter().collect::<Vec<f64>>(),
        "Maximum" => plus.f64().unwrap().into_no_null_iter().collect::<Vec<f64>>(),
        "Reference" => azi.clone(),
    ).unwrap();
    println!("{:?}", selected.describe(None));
    let mut write_file = CsvWriter::new(File::create(&format!("./output/verification/consumption.csv")).expect(&"household_year_usage"));
    write_file.finish(&mut selected).unwrap();
}

pub fn validation_production(folder: &String) {
    let filename = format!("./model_output/PV_validation_generation.csv");
    let mut df = CsvReader::from_path(filename)
        .unwrap()
        .with_delimiter(b';')
        .with_parse_dates(true)
        .finish()
        .unwrap();

    df = df.transpose().unwrap();
    let binding = df.mean().transpose().unwrap();
    let mean = binding.column("column_0").unwrap();

    let binding = df.std(2).transpose().unwrap();
    let std = binding.column("column_0").unwrap();

    let plus = mean + std;
    let min = mean - std;

    let mut ami_df = CsvReader::from_path(format!("./output/validation/household/day_usage_AMI.csv"))
        .unwrap()
        .finish()
        .unwrap();

    let divider = ami_df.sum().column("AMI_I").unwrap().f64().unwrap().sum().unwrap();

    let ami: Vec<f64> = ami_df
        .column("AMI_I")
        .unwrap()
        .f64()
        .unwrap()
        .into_no_null_iter()
        .map(|v| v / divider)
        .collect();

    let data: HashMap<String, Vec<f64>> = HashMap::from([
        ("min".to_owned(), min.f64().unwrap().into_no_null_iter().collect()),
        ("mean".to_owned(), mean.f64().unwrap().into_no_null_iter().collect()),
        ("plus".to_owned(), plus.f64().unwrap().into_no_null_iter().collect()),
        ("ami".to_owned(), ami.clone()),
    ]);

    plot_line(
        &format!("./model_output/PV_validation_generation"),
        get_step_to_time().into_iter().map(|v| v.to_owned()).collect(),
        data,
    );

    let mut selected = df!(
        "Time of day" => get_step_to_time(),
        "Minimum" => min.f64().unwrap().into_no_null_iter().collect::<Vec<f64>>(),
        "Mean" => mean.f64().unwrap().into_no_null_iter().collect::<Vec<f64>>(),
        "Maximum" => plus.f64().unwrap().into_no_null_iter().collect::<Vec<f64>>(),
        "Reference" => ami.clone(),
    ).unwrap();
    println!("{:?}", selected.describe(None));
    let mut write_file = CsvWriter::new(File::create(&format!("./output/verification/generation.csv")).expect(&"household_year_usage"));
    write_file.finish(&mut selected).unwrap();
}

fn get_step_to_time() -> Vec<&'static str> {
    vec![
        "00:15", "00:30", "00:45", "01:00", "01:15", "01:30", "01:45", "02:00", "02:15", "02:30", "02:45", "03:00",
        "03:15", "03:30", "03:45", "04:00", "04:15", "04:30", "04:45", "05:00", "05:15", "05:30", "05:45", "06:00",
        "06:15", "06:30", "06:45", "07:00", "07:15", "07:30", "07:45", "08:00", "08:15", "08:30", "08:45", "09:00",
        "09:15", "09:30", "09:45", "10:00", "10:15", "10:30", "10:45", "11:00", "11:15", "11:30", "11:45", "12:00",
        "12:15", "12:30", "12:45", "13:00", "13:15", "13:30", "13:45", "14:00", "14:15", "14:30", "14:45", "15:00",
        "15:15", "15:30", "15:45", "16:00", "16:15", "16:30", "16:45", "17:00", "17:15", "17:30", "17:45", "18:00",
        "18:15", "18:30", "18:45", "19:00", "19:15", "19:30", "19:45", "20:00", "20:15", "20:30", "20:45", "21:00",
        "21:15", "21:30", "21:45", "22:00", "22:15", "22:30", "22:45", "23:00", "23:15", "23:30", "23:45", "00:00",
    ]
}
