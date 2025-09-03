import toml
import sys
import json


def update_controller_toml(controller_file_path, controller_ip):
    try:
        config_file = toml.load(controller_file_path)

        config_file["controller_url"] = f"http://{controller_ip}:7001"
        config_file["domain_register_url"] = f"http://{controller_ip}:7002"
        
        with open(controller_file_path, "w") as node_config_file:
            toml.dump(config_file, node_config_file)

        print(f"[INFO] TOML config file updated ({controller_file_path})")

    except FileNotFoundError:
        print(f"[WARNING] File not found at {controller_file_path}")
    except Exception as e:
        print(f"[ERROR] An error occurred: {e}")


def update_orchestrator_toml(orchestrator_file_path, orchestrator_ip):
    try:
        config_file = toml.load(orchestrator_file_path)

        config_file["general"]["domain_register_url"] = f"http://{orchestrator_ip}:7002"
        config_file["general"]["orchestrator_url"] = f"http://{orchestrator_ip}:7003"
        config_file["general"]["node_register_url"] = f"http://{orchestrator_ip}:7004"
        
        with open(orchestrator_file_path, "w") as node_config_file:
            toml.dump(config_file, node_config_file)

        print(f"[INFO] TOML config file updated ({orchestrator_file_path})")

    except FileNotFoundError:
        print(f"[WARNING] File not found at {orchestrator_file_path}")
    except Exception as e:
        print(f"[ERROR] An error occurred: {e}")


def update_node_toml(node_file_path, node_ip, orchestrator_ip):
    try:
        config_file = toml.load(node_file_path)

        config_file["general"]["agent_url"] = f"http://{node_ip}:7005"
        config_file["general"]["invocation_url"] = f"http://{node_ip}:7006"
        config_file["general"]["node_register_url"] = f"http://{orchestrator_ip}:7004"
        config_file["telemetry"]["metrics_url"] = f"http://{orchestrator_ip}:7007"

        with open(node_file_path, "w") as node_config_file:
            toml.dump(config_file, node_config_file)

        print(f"[INFO] TOML config file updated ({node_file_path})")

    except FileNotFoundError:
        print(f"[WARNING] File not found at {node_file_path}")
    except Exception as e:
        print(f"[ERROR] An error occurred: {e}")


def update_cli_toml(cli_file_path, controller_ip):
    try:
        config_file = toml.load(cli_file_path)

        config_file["controller_url"] = f"http://{controller_ip}:7001"
    
        with open(cli_file_path, "w") as node_config_file:
            toml.dump(config_file, node_config_file)

        print(f"[INFO] TOML config file updated ({cli_file_path})")

    except FileNotFoundError:
        print(f"[WARNING] File not found at {cli_file_path}")
    except Exception as e:
        print(f"[ERROR] An error occurred: {e}")


def update_latency_measurement_config_file(config_file_path, orchestrator_ip, redis_server_ip, rpi_node_ip, vm_node_ip):
    try:
        with open(config_file_path, 'r') as f:
            data = json.load(f)

        data["orchestrator_ip"] = orchestrator_ip
        data["redis_server_ip_address"] = redis_server_ip
        data["vm_node_ip"] = vm_node_ip
        data["rpi_node_ip"] = rpi_node_ip

        with open(config_file_path, 'w') as f:
            json.dump(data, f, indent=4)

        print(f"[INFO] JSON config file updated ({config_file_path})")
    except FileNotFoundError:
        print(f"[WARNING] File not found at {cli_file_path}")
    except Exception as e:
        print(f"[ERROR] An error occurred: {e}")



if __name__ == "__main__":    
    print("Update files located on node:")
    print("1. RPI node")
    print("2. VM node")
    curr_node = input("Insert an option: ").strip()
    if curr_node != "1" and curr_node != "2":
        print("[ERROR] Invalid choice")
        sys.exit(0)

    print("Update EDGELESS files located in:")
    print("1. edgeless/target/debug")
    print("2. edgeless/targed/release")
    choice = input("Insert an option: ").strip()
    if choice != "1" and choice != "2":
        print("[ERROR] Invalid choice")
        sys.exit(0)

    edgeless_folder = ""
    if choice == "1":
        edgeless_folder = "debug"
    else:
        edgeless_folder = "release"

    rpi_node_ip = input("RPI node IP address: ").strip()
    vm_node_ip = input("VM node IP address: ").strip()
    orchestrator_controller_ip = input("Controller/Orchestrator IP address: ").strip()

    controller_file_path = f"edgeless/target/{edgeless_folder}/controller.toml"
    update_controller_toml(controller_file_path, orchestrator_controller_ip)

    orchestrator_file_path = f"edgeless/target/{edgeless_folder}/orchestrator.toml"
    update_orchestrator_toml(orchestrator_file_path, orchestrator_controller_ip)

    node_file_path = f"edgeless/target/{edgeless_folder}/node.toml"
    if curr_node == "1":        # RPI
        update_node_toml(node_file_path, rpi_node_ip, orchestrator_controller_ip)
    else:                       #VM
        update_node_toml(node_file_path, vm_node_ip, orchestrator_controller_ip)

    cli_file_path = f"edgeless/target/{edgeless_folder}/cli.toml"
    update_cli_toml(cli_file_path, orchestrator_controller_ip)

    latency_measurement_config_file = "latency_measurement/config.json"
    update_latency_measurement_config_file(latency_measurement_config_file, orchestrator_controller_ip, orchestrator_controller_ip, rpi_node_ip, vm_node_ip)
