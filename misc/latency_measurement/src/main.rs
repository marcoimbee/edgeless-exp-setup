use redis::Commands;
use serde_json;
use std::fs::File;
use std::io::Read;
use tokio;
use std::process::Command;
use std::str;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let config_path = "config.json";
    let mut file = File::open(config_path).expect("[ERROR] Failed to open config file.");
    let mut config_contents = String::new();
    file.read_to_string(&mut config_contents)
        .expect("[ERROR] Failed to read config file.");
    
    let config: serde_json::Value = serde_json::from_str(&config_contents)
        .expect("[ERROR] Failed to parse JSON config.");

    let redis_server_ip_address = config["redis_server_ip_address"]
        .as_str()
        .expect("[ERROR] Missing redis_server_ip_address in config.");
    let orchestrator_ip = config["orchestrator_ip"]
        .as_str()
        .expect("[ERROR] Missing rpi_node_ip in config.");
    let rpi_node_ip = config["rpi_node_ip"]
        .as_str()
        .expect("[ERROR] Missing vm_node_ip in config.");
    let vm_node_ip = config["vm_node_ip"]
        .as_str()
        .expect("[ERROR] Missing orchestrator_ip in config.");
    let vm_node_uuid = config["vm_node_uuid"]
        .as_str()
        .expect("[ERROR] Missing vm_node_uuid in config.");
    let rpi_node_uuid = config["rpi_node_uuid"]
        .as_str()
        .expect("[ERROR] Missing rpi_node_uuid in config.");

    let redis_client = redis::Client::open(format!("redis://{}/", redis_server_ip_address)).unwrap();
    let mut redis_conn = redis_client.get_connection().unwrap();

    loop {
        // Measuring latency FROM VM NODE TO ORCHESTRATOR
        let vm_orc_output = Command::new("ping")
            .arg("-c")          // Number of ICMP pkts
            .arg("1")           // Just one ICMP pkt
            .arg(orchestrator_ip)
            .output();

        // Measuring latency FROM VM NODE TO RPI NODE
        let vm_rpi_output = Command::new("ping")
            .arg("-c")          // Number of ICMP pkts
            .arg("1")           // Just one ICMP pkt
            .arg(rpi_node_ip)
            .output();


        match vm_orc_output {
            Ok(vm_orc_output) => {
                if vm_orc_output.status.success() {
                    let stdout = str::from_utf8(&vm_orc_output.stdout).unwrap_or("");
                    if let Some(latency) = parse_ping_output(stdout) {
                        // Pushing value to Redis 
                        // Key format: latency:<uuid_source_node>:orc
                        let redis_key = format!("latency:{}:orc", vm_node_uuid);
                        let _: () = redis_conn
                            .set(redis_key.clone(), latency)
                            .unwrap();

                        println!(
                            "[INFO] Latency measured: VM_node --- {:.2} ms ---> E-ORC.", 
                            latency,
                        );
                        println!("[INFO] Updated Redis key: {}", redis_key);
                    } else {
                        println!("[WARNING] Could not parse ping output.")
                    }
                } else {
                    eprintln!(
                        "[ERROR] Ping command failed with error: {}",
                        String::from_utf8_lossy(&vm_orc_output.stderr)
                    )
                }
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to execute ping command: {}", e);
            }
        }

        match vm_rpi_output {
            Ok(vm_rpi_output) => {
                if vm_rpi_output.status.success() {
                    let stdout = str::from_utf8(&vm_rpi_output.stdout).unwrap_or("");
                    if let Some(latency) = parse_ping_output(stdout) {
                        // Pushing value to Redis 
                        // Key format: latency:<uuid_source_node>:orc
                        let redis_key = format!("latency:{}:{}", vm_node_uuid, rpi_node_uuid);
                        let _: () = redis_conn
                            .set(redis_key.clone(), latency)
                            .unwrap();

                        println!(
                            "[INFO] Latency measured: VM_node --- {:.2} ms ---> RPI_node.", 
                            latency,
                        );
                        println!("[INFO] Updated Redis key: {}", redis_key);
                        println!("\n");
                    } else {
                        println!("[WARNING] Could not parse ping output.")
                    }
                } else {
                    eprintln!(
                        "[ERROR] Ping command failed with error: {}",
                        String::from_utf8_lossy(&vm_rpi_output.stderr)
                    )
                }
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to execute ping command: {}", e);
            }
        }

        // Wait a little before the next measurement
        std::thread::sleep(Duration::from_secs(5))
    }
}


fn parse_ping_output(ping_output: &str) -> Option<f64> {
    ping_output
        .lines()
        .find(|line| line.contains("time="))        // Looking for the "time=" field in the ping output
        .and_then(|line| {
            line.split_whitespace()
                .find(|field| field.starts_with("time="))
                .and_then(|time_field| {
                    time_field
                        .trim_start_matches("time=")
                        .parse::<f64>()
                        .ok()       // Converting to f64 if valid   
                })
        })
}
