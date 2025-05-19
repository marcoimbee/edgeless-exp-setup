import subprocess


def start_workflow(cli_path, workflow_path):
    # Build shell command
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

def reset_ttc_log(ttc_logs_path, repetition, move):
    filename = f'ttc_log_{repetition}.log'

    # Build shell commands
    rm_cmd = f'rm /home/pi/Desktop/ttc_log.log'
    mv_cmd = f'mv /home/pi/Desktop/ttc_log.log {ttc_logs_path}/{filename}'
    touch_cmd = 'touch /home/pi/Desktop/ttc_log.log'
    chmod_cmd = 'sudo chmod 777 /home/pi/Desktop/ttc_log.log'

    if not move:
        subprocess.run(rm_cmd.split(), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        subprocess.run(touch_cmd.split(), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        subprocess.run(chmod_cmd.split(), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    else:
        subprocess.run(mv_cmd.split(), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
