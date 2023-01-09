use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

use itertools::iproduct;
use serde::{Deserialize, Serialize};
use tinytemplate::TinyTemplate;
#[derive(Serialize, Deserialize)]
pub struct Context {
    // Model
    pub name:                           String,
    pub seed:                           u64,
    pub folder:                         String,
    // Attack
    pub patch_start:                    i64,
    pub infection_rate_per_step:        f32,
    pub infection_start:                i64,
    pub attack_behaviour:               String,
    pub patch_rate_per_step:            f32,
    pub percentage_vuln_devices:        f32,
    // Grid
    pub percentage_generation_of_usage: f32,
    pub pv_adoption:                    f32,
    pub max_gen_inc_tick:               i64,
    pub energy_storage:                 i64,
    pub bulk_consumption:               i64,
    pub power_consumption_bounds:       String,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            name:                           "No name".to_string(),
            seed:                           117,
            folder:                         "output".to_string(),
            percentage_vuln_devices:        0.5,
            infection_rate_per_step:        1.0,
            infection_start:                1,
            patch_start:                    500,
            patch_rate_per_step:            0.0,
            attack_behaviour:               "(24,300,1,0)".to_string(),
            percentage_generation_of_usage: 0.5,
            pv_adoption:                    0.5,
            max_gen_inc_tick:               42_000_000,
            energy_storage:                 850_000_000,
            bulk_consumption:               10_000_000_000,
            power_consumption_bounds:       "(1200, 200)".to_string(),
        }
    }
}

#[tokio::main]
pub async fn create_experiments(folder: &String) {
    println!("Experiments:");
    if Path::new(&folder).exists() {
        println!("Folder already exist, please change");
        return;
    };

    let seed = vec![117_u64, 2010_u64];
    let percentage_vuln_devices: Vec<f32> = (0..=100).step_by(20).map(|a| a as f32 / 100.0).collect();
    let pv_adoption: Vec<f32> = (0..=100).step_by(25).map(|a| a as f32 / 100.0).collect();
    let percentage_generation_of_usage: Vec<f32> = (0..=100).step_by(25).map(|a| a as f32 / 100.0).collect();
    let attack_behaviour = vec![
        "(24,250,1,0)".to_string(), // No power generated. Results in under generation.
        // "(48,250,1,0)".to_string(), // No power generated. Results in under generation.
        // "(72,250,1,0)".to_string(), // No power generated. Results in under generation.
        // "(96,300,1,0)".to_string(), // No power generated. Results in under generation.
        // "(192,288,1,3)".to_string(), // Report power generated but don't produce it. Results in under generation. */
    ];
    let max_gen_inc_tick: Vec<i64> = vec![42_000_000, 31_500_000, 52_500_000]; // Based on data-analysis of tennet files. Minus 25% and plus 25%.
    let energy_storage: Vec<i64> = vec![850_000_000, 637_500_000, 1_062_500_000]; // Based on data-analysis of tennet files. Minus 25% and plus 25%.
    let bulk_consumption: Vec<i64> = vec![10_000_000_000, 7_500_000_000, 12_500_000_000]; // Based on data-analysis of tennet files. Minus 25% and plus 25%.

    // let infection_rate_per_step: Vec<f32> = (10..=100).step_by(30).map(|a| a as f32/10.0).chain([100.0]).collect();// (1..=1).map(|a| a as f32));
    // let infection_rate_per_step: Vec<f32> = vec![100.0];
    // let patch_rate_per_step = (0..100).step_by(10).map(|a| a as f32/1000.0);
    // let power_consumption_bounds = (0..0);

    println!("seed: {:>2} - {:?}", seed.len(), seed);
    println!(
        "percentage_vuln_devices: {:>2} - {:?}",
        percentage_vuln_devices.len(),
        percentage_vuln_devices
    );
    println!(
        "attack_behaviour: {:>2} - {:?}",
        attack_behaviour.len(),
        attack_behaviour
    );
    println!("pv_adoption: {:>2} - {:?}", pv_adoption.len(), pv_adoption);
    println!(
        "percentage_generation_of_usage: {:>2} - {:?}",
        percentage_generation_of_usage.len(),
        percentage_generation_of_usage
    );
    println!(
        "max_gen_inc_tick: {:>2} - {:?}",
        max_gen_inc_tick.len(),
        max_gen_inc_tick
    );
    println!("energy_storage: {:>2} - {:?}", energy_storage.len(), energy_storage);
    println!(
        "bulk_consumption: {:>2} - {:?}",
        bulk_consumption.len(),
        bulk_consumption
    );

    let combined_iter = iproduct!(
        seed,
        percentage_vuln_devices,
        pv_adoption,
        attack_behaviour,
        percentage_generation_of_usage,
        max_gen_inc_tick,
        energy_storage,
        bulk_consumption
    );

    let mut count = 0;
    let mut handles = vec![];

    // #[allow(clippy::never_loop)]
    for (
        seed,
        percentage_vuln_devices,
        pv_adoption,
        attack_behaviour,
        percentage_generation_of_usage,
        max_gen_inc_tick,
        energy_storage,
        bulk_consumption,
    ) in combined_iter
    {
        let context = Context {
            percentage_generation_of_usage,
            name: format!("{count:04}"),
            seed,
            percentage_vuln_devices,
            attack_behaviour,
            pv_adoption,
            folder: folder.to_owned(),
            max_gen_inc_tick,
            energy_storage,
            bulk_consumption,
            ..Context::default()
        };
        handles.push(tokio::spawn(write_experiment(context, count, folder.clone())));
        count += 1;
    }
    println!("Count: {count}");
    for h in handles {
        h.await.unwrap();
    }
}

async fn write_experiment(context: Context, count: i64, folder: String) {
    let mut mp_tt = TinyTemplate::new();
    let mut ap_tt = TinyTemplate::new();
    let mut gp_tt = TinyTemplate::new();

    mp_tt
        .add_template("mp", include_str!("../input/ModelParameters.ini"))
        .expect("Could not load mp file");
    ap_tt
        .add_template("ap", include_str!("../input/AttackParameters.ini"))
        .expect("Could not load ap file");
    gp_tt
        .add_template("gp", include_str!("../input/GridParameters.ini"))
        .expect("Could not load gp file");

    let mp = mp_tt.render("mp", &context).unwrap();
    let ap = ap_tt.render("ap", &context).unwrap();
    let gp = gp_tt.render("gp", &context).unwrap();

    create_dir_all(format!("{folder}/{count:04}")).unwrap();

    let mut mpfile = File::create(format!("{folder}/{count:04}/ModelParameters.ini")).unwrap();
    let mut apfile = File::create(format!("{folder}/{count:04}/AttackParameters.ini")).unwrap();
    let mut gpfile = File::create(format!("{folder}/{count:04}/GridParameters.ini")).unwrap();
    let contextfile = File::create(format!("{folder}/{count:04}/context.json")).unwrap();

    mpfile.write_all(mp.as_bytes()).unwrap();
    apfile.write_all(ap.as_bytes()).unwrap();
    gpfile.write_all(gp.as_bytes()).unwrap();
    serde_json::to_writer_pretty(contextfile, &context).unwrap();
}
