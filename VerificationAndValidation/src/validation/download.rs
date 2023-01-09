use std::fs::{create_dir_all, File};
use std::io::Write;

#[tokio::main]
pub async fn get_tennet_files(folder: &String) {
    println!("Downloading files");
    create_dir_all(format!("{folder}/tennet")).expect("Could not create Tennet output dir");
    create_dir_all(format!("{folder}/tennet/xls")).expect("Could not create Tennet output dir");
    create_dir_all(format!("{folder}/tennet/csv")).expect("Could not create Tennet output dir");

    let categories = vec![
        "availablecapacity",
        "BalansdeltaPrices",
        "balancedeltaIGCC",
        "balancedelta2017",
        "bidpriceladder",
        "deployed",
        "volumerrecapacity",
        "volumesettledimbalance",
        "Intraday",
        "laddersize15",
        "imbalance",
        "settlementprices",
        "installedcapacity",
        "measurementdata",
        "regulatingmargin",
        "thirtydaysahead",
        "laddersizetotal",
    ];
    let data_types = vec!["csv", "xls"];
    let date_from = "01-01-2019";
    let date_to = "01-01-2020";

    let mut handles = vec![];
    for dtype in &data_types {
        println!("Getting {dtype} files");
        for cat in &categories {
            let link = format!("http://www.tennet.org/english/operational_management/export_data.aspx?exporttype={cat}&format={dtype}&datefrom={date_from}&dateto={date_to}&submit=1");
            let filename = format!("{folder}/tennet/{dtype}/{cat}.{dtype}");
            handles.push((filename.clone(), tokio::spawn(download_file(link, filename))));
        }
    }
    for h in handles {
        match h.1.await {
            Ok(v) => println!("  Downloaded {v}"),
            Err(e) => println!("  Error with {} due to: {}", h.0, e),
        }
    }
}

async fn download_file(link: String, filename: String) -> String {
    let response = reqwest::get(link).await.unwrap();
    let mut file = File::create(&filename).unwrap();
    file.write_all(&response.bytes().await.unwrap()).unwrap();
    filename
}
