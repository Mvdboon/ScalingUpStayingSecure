pub mod download;
pub mod grid_regulation;
pub mod household;

pub fn validate(download: &bool, folder: &String) {
    // if *download {
    //     download::get_tennet_files(folder);
    // }
    // grid_regulation::capacity_per_15_minutes(folder);
    // grid_regulation::regulation_room(folder);
    // grid_regulation::power_grid_consumption(folder);
    // grid_regulation::domestic_power_grid_consumption(folder);
    // household::year_power_usage(folder);
    // household::day_usage(folder);
    household::validation_consumption(folder);
    household::validation_production(folder);
}
