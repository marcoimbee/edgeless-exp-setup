use crossterm::{execute, terminal::ClearType};
use crossterm::terminal::Clear;
use std::io::{self, stdout, Write};
use serde_json::Value;
use std::fs;
use ctrlc;
use serde::Deserialize;
use std::process::Command;
use std::str;
use std::env;


#[derive(Deserialize, Debug)]
struct Config {
    nodes_uuids: NodesUUIDs,
    orchestration_params: OrchestrationParams,
    sample_workflow_path: String,
    edgeless_exec_directory: String,
}

#[derive(Deserialize, Debug)]
struct NodesUUIDs {
    rpi_node: String,
    vm_node: String,
}

#[derive(Deserialize, Debug)]
struct OrchestrationParams {
    proxy_type: String,
    redis_url: String,
    num_relocations: u64,
    latency_threshold: f64,
    relocation_wait_interval_ms: u64,
    monitoring_wait_interval_ms: u64
}


fn show_header(choice: &str) {
    let ascii_art_1 = r#"
           _           _                  _             
          | |         | |                (_)            
  _ __ ___| |__   __ _| | __ _ _ __   ___ _ _ __   __ _ 
 | '__/ _ \ '_ \ / _` | |/ _` | '_ \ / __| | '_ \ / _` |
 | | |  __/ |_) | (_| | | (_| | | | | (__| | | | | (_| |
 |_|  \___|_.__/ \__,_|_|\__,_|_| |_|\___|_|_| |_|\__, | o o o
                                                   __/ |
                                                  |___/ 
"#;

    let ascii_art_2 = r#"
                           _  _                _                        
                          (_)| |              (_)                       
  _ __ ___    ___   _ __   _ | |_  ___   _ __  _  _ __    __ _          
 | '_ ` _ \  / _ \ | '_ \ | || __|/ _ \ | '__|| || '_ \  / _` |         
 | | | | | || (_) || | | || || |_| (_) || |   | || | | || (_| | 
 |_| |_| |_| \___/ |_| |_||_| \__|\___/ |_|   |_||_| |_| \__, | o o o
                                                          __/ |         
                                                         |___/          
"#;

    let ascii_art_4 = r#"

    /$$$$$$$            /$$                                 /$$                     /$$                                 
    | $$__  $$          | $$                                | $$                    | $$                                 
    | $$  \ $$  /$$$$$$ | $$  /$$$$$$   /$$$$$$   /$$$$$$  /$$$$$$    /$$$$$$   /$$$$$$$                                 
    | $$  | $$ /$$__  $$| $$ /$$__  $$ /$$__  $$ |____  $$|_  $$_/   /$$__  $$ /$$__  $$                                 
    | $$  | $$| $$$$$$$$| $$| $$$$$$$$| $$  \ $$  /$$$$$$$  | $$    | $$$$$$$$| $$  | $$                                 
    | $$  | $$| $$_____/| $$| $$_____/| $$  | $$ /$$__  $$  | $$ /$$| $$_____/| $$  | $$                                 
    | $$$$$$$/|  $$$$$$$| $$|  $$$$$$$|  $$$$$$$|  $$$$$$$  |  $$$$/|  $$$$$$$|  $$$$$$$                                 
    |_______/  \_______/|__/ \_______/ \____  $$ \_______/   \___/   \_______/ \_______/                                 
                                       /$$  \ $$                                                                         
                                      |  $$$$$$/                                                                         
                                       \______/                                                                          
      /$$$$$$                     /$$                            /$$                          /$$                        
     /$$__  $$                   | $$                           | $$                         | $$                        
    | $$  \ $$  /$$$$$$  /$$$$$$$| $$$$$$$   /$$$$$$   /$$$$$$$/$$$$$$    /$$$$$$  /$$$$$$  /$$$$$$    /$$$$$$   /$$$$$$ 
    | $$  | $$ /$$__  $$/$$_____/| $$__  $$ /$$__  $$ /$$_____/_  $$_/   /$$__  $$|____  $$|_  $$_/   /$$__  $$ /$$__  $$
    | $$  | $$| $$  \__/ $$      | $$  \ $$| $$$$$$$$|  $$$$$$  | $$    | $$  \__/ /$$$$$$$  | $$    | $$  \ $$| $$  \__/
    | $$  | $$| $$     | $$      | $$  | $$| $$_____/ \____  $$ | $$ /$$| $$      /$$__  $$  | $$ /$$| $$  | $$| $$      
    |  $$$$$$/| $$     |  $$$$$$$| $$  | $$|  $$$$$$$ /$$$$$$$/ |  $$$$/| $$     |  $$$$$$$  |  $$$$/|  $$$$$$/| $$      
     \______/ |__/      \_______/|__/  |__/ \_______/|_______/   \___/  |__/      \_______/   \___/   \______/ |__/          
"#;

    if choice == "1" { println!("{}", ascii_art_1); }
    else if choice == "2" { println!("{}", ascii_art_2); }
    else { println!("{}", ascii_art_4); }
}


fn show_menu() {
    println!();
    println!("--- Options (Ctrl + D to close): ---");
    println!("1. Start the Network Aware Orchestrator (migrate)");
    println!("2. Monitor cluster with Network Aware Orchestrator (do not migrate)");
    println!("3. Move everything on the RPI node");
    // println!("4. Start sample workflow");
    // println!("5. Stop sample workflow");
}

fn show_nodes_names(config: &Config) {
    println!("RPI node: {}", config.nodes_uuids.rpi_node.to_string());
    println!("VM node: {}", config.nodes_uuids.vm_node.to_string());
    println!();
}

fn clear_console() {
    execute!(stdout(), Clear(ClearType::All)).unwrap();
}


fn main() -> anyhow::Result<()> {
    let json_data = fs::read_to_string("config.json")?;          // Reading and deserializing configuration file
    let config: Config = serde_json::from_str(&json_data)?;

    let mut workflow_uuid = String::new();

    let mut counter;

    let (tx, rx) = std::sync::mpsc::channel::<()>();
    ctrlc::set_handler(move || {
        tx.send(()).unwrap();
    })?;

    clear_console();

    loop {
        show_header("0");
        show_menu();

        println!("Choose an option: ");
        io::stdout().flush().expect("Failed to flush stdout.");

        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read line.");
        let choice = choice.trim();

        match choice {
            "1" => {
                counter = 0;

                let mut net_aware_orc = delegated_orc::network_aware_orchestrator::NetworkAwareOrchestrator::new(
                    &config.orchestration_params.redis_url,
                    config.orchestration_params.latency_threshold,
                    config.orchestration_params.num_relocations
                )?;

                loop {
                    show_header(choice);
                    show_nodes_names(&config);

                    let start_time = std::time::Instant::now();
                    counter += 1;
                    println!("[INFO] {}-th iteration.", &counter);
                    let (done, num_migrations) = net_aware_orc.rebalance();
                    if done {
                        if num_migrations != 0 {
                            println!(
                                "{} migrations performed, redistribution time: {} ms.",
                                num_migrations,
                                start_time.elapsed().as_millis()
                            );
                        } else {
                            println!("0 migrations performed.");
                        }
                    }

                    if rx.try_recv().is_ok() {
                        break;
                    }

                    std::thread::sleep(std::time::Duration::from_millis(config.orchestration_params.relocation_wait_interval_ms));
                    // clear_console();
                }
            },

            "2" => {
                counter = 0;

                let mut net_aware_orc = delegated_orc::network_aware_orchestrator::NetworkAwareOrchestrator::new(
                    &config.orchestration_params.redis_url,
                    config.orchestration_params.latency_threshold,
                    config.orchestration_params.num_relocations
                )?;

                loop {
                    show_header(choice);
                    show_nodes_names(&config);

                    counter += 1;
                    println!("[INFO] {}-th iteration.", &counter);
                    net_aware_orc.monitor_cluster();

                    if rx.try_recv().is_ok() {
                        break;
                    }

                    std::thread::sleep(std::time::Duration::from_millis(config.orchestration_params.monitoring_wait_interval_ms));
                    // clear_console();
                }
            },

            "3" => {
                let mut net_aware_orc = delegated_orc::network_aware_orchestrator::NetworkAwareOrchestrator::new(
                    &config.orchestration_params.redis_url,
                    config.orchestration_params.latency_threshold,
                    config.orchestration_params.num_relocations
                )?;

                println!();
                show_header(choice);
                show_nodes_names(&config);

                let json_data = fs::read_to_string("config.json").expect("Failed to read JSON config file.");
                let parsed: Value = serde_json::from_str(&json_data).expect("Failed to parse JSON config file.");
                let rpi_node_id = parsed
                    .get("nodes_uuids")
                    .and_then(|node| node.get("rpi_node"))
                    .and_then(|v| v.as_str())
                    .expect("'rpi_node' not found or not a string");

                let (done, moved_funcs) = net_aware_orc.move_to_rpi(rpi_node_id);
                if done {
                    println!("[INFO] Successfully moved {} functions to the RPI node.", moved_funcs);
                } else {
                    eprintln!("[ERROR] An error occurred.");
                }
            },

            // "4" => {
            //     println!();

            //     let output_start = Command::new("./edgeless_cli")
            //         .arg("workflow")
            //         .arg("start")
            //         .arg(&config.sample_workflow_path)
            //         .current_dir(&config.edgeless_exec_directory)
            //         .output()
            //         .expect("Failed to execute edgeless_cli command.");

            //     if output_start.status.success() {
            //         workflow_uuid = String::from_utf8_lossy(&output_start.stdout).trim().to_string();
            //         println!("[INFO] Successfully started the sample workflow. UUID: {}", workflow_uuid.clone());
            //     } else {
            //         println!("[ERROR] Error while starting the sample workflow.");
            //     }
            // },

            // "5" => {
            //     println!();

            //     if workflow_uuid.clone().is_empty() {
            //         println!("[INFO] No workflow is running.");
            //     } else {
            //         let output_stop = Command::new("./edgeless_cli")
            //             .arg("workflow")
            //             .arg("stop")
            //             .arg(workflow_uuid.clone())
            //             .current_dir(&config.edgeless_exec_directory)
            //             .output()
            //             .expect("Failed to execute edgeless_cli command.");

            //         if output_stop.status.success() {
            //             println!("[INFO] Successfully stopped the sample workflow.");
            //         } else {
            //             println!("[ERROR] Error while stopping the sample workflow.");
            //         }
            //     }
            // },

            &_ => return Ok(()),
        }
    }
}