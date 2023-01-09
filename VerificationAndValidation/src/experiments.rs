use std::path::Path;

mod data;
mod structs;
mod util;

pub fn analyse(experiment_folder: String, datafolder: String) {
    let df_filename = format!("{datafolder}/df.csv");
    let df_location = Path::new(&df_filename);
    let df = data::get_df(df_location, &experiment_folder, &datafolder).unwrap();
}
