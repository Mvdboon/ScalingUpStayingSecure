mod anova;

use std::fs::File;
use std::io::Write;

use anyhow::Error;
use linregress::{FormulaRegressionBuilder, RegressionDataBuilder, RegressionModel};
use ndarray::{Array, Array1, Array2, Axis};
use polars::export::chrono::format;
use polars::lazy::dsl::*;
use polars::lazy::prelude::*;
use polars::prelude::*;
use serde::Serialize;

// use crate::experiments::structs::*;
// // use crate::experiments::util::*;

// pub fn parameters() {
//     let filename = "./experiments/parameters_all.csv".to_string();
//     let target_column = "has_err";
//     let feature_columns = vec![
//         "percentage_vuln_devices".to_owned(),
//         "pv_adoption".to_owned(),
//         "percentage_generation_of_usage".to_owned(),
//     ];

//     let df = LazyCsvReader::new(filename).has_header(true).finish().unwrap();
//     // let df = df.filter(col("time_to_err").is_not_null());

//     // Full analysis
//     information("Full_data", &df, target_column, &feature_columns);

//     // Time of day start attack
//     let exp_df = df.clone().filter(col("attack_behaviour").str().contains("Morning"));
//     information("Attack Start - Morning", &exp_df, target_column, &feature_columns);

//     let exp_df = df.clone().filter(col("attack_behaviour").str().contains("MidDay"));
//     information("Attack Start - Midday", &exp_df, target_column, &feature_columns);

//     let exp_df = df.filter(col("attack_behaviour").str().contains("AfterNoon"));
//     information("Attack Start - Afternoon", &exp_df, target_column, &feature_columns);
// }

// fn information(name: &str, df: &LazyFrame, target_column: &'static str, feature_columns: &Vec<String>) {
//     let exp_df = df.clone().collect().unwrap();
//     let has_err_full = AnalysisResult::new(
//         &format!(" {name} Target: has_err - Full Data"),
//         &exp_df,
//         &create_fitted_model(target_column, &exp_df, feature_columns),
//     );
//     println!("{}", serde_json::to_string_pretty(&has_err_full).unwrap());
//     let exp_df = df.clone().filter(col("has_err").gt_eq(1)).collect().unwrap();
//     sub_information_data_discribe(&format!("{name} - with_err"), feature_columns, exp_df);
//     let exp_df = df.clone().filter(col("has_err").lt(1)).collect().unwrap();
//     sub_information_data_discribe(&format!("{name} - without_err"), feature_columns, exp_df);
// }

// fn sub_information_data_discribe(name: &str, feature_columns: &[String], exp_df: DataFrame) {
//     if exp_df.shape().0 == 0 {
//         println!("Empty dataframe for {name}");
//     } else {
//         let only_without_err = DataDiscription::create(name, feature_columns, &exp_df);
//         println!("{}", serde_json::to_string_pretty(&only_without_err).unwrap());
//     }
// }
