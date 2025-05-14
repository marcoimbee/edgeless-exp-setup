import subprocess
import paramiko
import json

VM_IP = '<your-VMpub-IP>'
VM_USERNAME = '<your-VMpub-username>'
VM_PASSWORD = '<your-VMpub-password>'

REMOTE_DIR = '<your-remote-VMpub-edgeless/target/debug-path>'
VM_LOG_FILE = 'vm_node_log.log'
REMOTE_LOG_FILE = f'{REMOTE_DIR}/{VM_LOG_FILE}'

LOCAL_VM_LOG_FILE = 'vm_node_log.log'
LOCAL_RPI_LOG_FILE = 'rpi_node_log.log'

ABS_PATH = '/home/pi/Desktop/otii-automation/otii_automation/device/edgeless'
EX_TIMES_LOG_FILE = f'{ABS_PATH}/exp_data.json'


# Remove the document that has "iteration" equal to exp_id
def rollback_ex_time_log(exp_id):
    with open(EX_TIMES_LOG_FILE, 'r') as file:
        data = json.load(file)

    if not isinstance(data, list):
        raise ValueError("Expected a list of documents")
    
    updated_data = [doc for doc in data if doc.get("iteration") != (exp_id)]

    with open(EX_TIMES_LOG_FILE, 'w') as file:
        json.dump(updated_data, file, indent=4)

    print(f"[EDGELESS INFO] Document for iteration {exp_id} has been removed from the JSON log file.")

def connect_to_vm():
    try:
        client = paramiko.SSHClient()
        client.set_missing_host_key_policy(paramiko.AutoAddPolicy())

        print(f"[EDGELESS INFO] Connecting to {VM_IP}...")
        client.connect(VM_IP, username=VM_USERNAME, password=VM_PASSWORD)
        print("[EDGELESS INFO] Connected.")

        return client
    except Exception as ex:
        print(f"[EDGELESS ERROR] connect_to_vm(): {ex}")
        raise Exception(ex) from ex

def get_vm_log_file():
    try:
        print("[EDGELESS INFO] Downloading VM log file...")
        ssh = paramiko.SSHClient()
        ssh.set_missing_host_key_policy(paramiko.AutoAddPolicy())
        ssh.connect(VM_IP, username=VM_USERNAME, password=VM_PASSWORD)

        vm_log_file_save_location = f"{ABS_PATH}/{LOCAL_VM_LOG_FILE}"

        sftp = ssh.open_sftp()
        sftp.get(REMOTE_LOG_FILE, vm_log_file_save_location)
        sftp.close()
        ssh.close()
        print(f"[EDGELESS INFO] VM log file saved to {vm_log_file_save_location}")
    except Exception as ex:
        print(f"[EDGELESS ERROR] get_vm_log_file(): {ex}")
        raise Exception(ex) from ex
    
def parse_log_files():
    # The log files will be already stored in the edgeless_log_parser folder for it to parse them
    try:
        # Build shell command
        cmd = 'python3 /home/pi/Desktop/edgeless_log_parser/log_parser.py'

        # Run command
        subprocess.run(cmd.split())
    except Exception as ex:
        print(f"[EDGELESS ERROR] parse_log_file(): {ex}")
        raise Exception(ex) from ex

def start_workflow(cli_path, workflow_path):
    # Build shell command
    # ./home/pi/Desktop/edgeless/target/release/edgeless_cli workflow start /home/pi/Desktop/accelerometer_classification_workflow/workflow.json
    cmd = f'{cli_path}/edgeless_cli workflow start {workflow_path}'

    # Run command
    result = subprocess.run(cmd.split(), stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=120, cwd=cli_path)
    stdout = result.stdout.decode().strip()
    stderr = result.stderr.decode().strip()

    return stdout, stderr

def stop_workflow(cli_path, uuid):
    # Build shell command
    cmd = f'{cli_path}/edgeless_cli workflow stop {uuid}'

    # Run command
    subprocess.run(cmd.split(), stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=120, cwd=cli_path)

def reset_aoi_log(aoi_logs_path, repetition, move):
    filename = f'aoi_log_{repetition}.log'

    # Build shell commands
    rm_cmd = f'rm /home/pi/Desktop/aoi_log.log'
    mv_cmd = f'mv /home/pi/Desktop/aoi_log.log {aoi_logs_path}/{filename}'
    touch_cmd = 'touch /home/pi/Desktop/aoi_log.log'
    chmod_cmd = 'sudo chmod 777 /home/pi/Desktop/aoi_log.log'

    if not move:
        subprocess.run(rm_cmd.split(), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        subprocess.run(touch_cmd.split(), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        subprocess.run(chmod_cmd.split(), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    else:
        subprocess.run(mv_cmd.split(), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
