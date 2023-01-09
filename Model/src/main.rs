fn main() {
    // smartgrid_iot_security::print_feature();
    scaling_up_staying_secure::run_from_config("ModelParameters.ini").expect("Running the model failed");
}
