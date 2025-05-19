import json
import logging
import os

from paramiko.client import SSHClient, AutoAddPolicy
from scp import SCPClient
from ..environment import Environment as Env

logger = logging.getLogger('controller')


def download_results(trace) -> dict:
    """ Download results from server """
    try:
        with SSHClient() as ssh:
            ssh.set_missing_host_key_policy(AutoAddPolicy())
            ssh.connect(
                hostname=Env.config['server']['host'],
                username=Env.config['server']['username'],
                key_filename=Env.config['server']['key_file']
            )

            with SCPClient(ssh.get_transport()) as scp:
                scp.get(Env.config['server']['path'] + f'{trace}.json', '.tmp.json')

            with open('.tmp.json') as fp:
                device_res = json.load(fp)

        os.remove('.tmp.json')
        return device_res
    except Exception as ex:
        logger.warning(f'Download results {trace} failed: {ex}')
        raise ex


def download_device_logs() -> None:
    try:
        with SSHClient() as ssh:
            ssh.set_missing_host_key_policy(AutoAddPolicy())
            ssh.connect(
                hostname=Env.config['server']['host'],
                username=Env.config['server']['username'],
                key_filename=Env.config['server']['key_file']
            )

            with SCPClient(ssh.get_transport()) as scp:
                scp.get(Env.config['server']['path'] + f'device.log', os.path.join(Env.log_dir, 'device.log'))

    except Exception as ex:
        logger.warning(f'Download logs failed: {ex}')
        raise ex


def build_config_message(params: dict, trace: str, iteration: int) -> dict:
    """ Build experiment configuration for the device """

    configuration = {
        'iteration': iteration,
        'experiment': Env.config['meta']['experiment'],
        'duration': Env.config['meta']['duration'],
        'edgeless_exec_dir': Env.config['rpi']['edgeless_exec_dir'],
        'workflow_path': Env.config['rpi']['workflow_path'],
        'aoi_logs_path': Env.config['rpi']['aoi_logs_path'],
        'results_dir': f'results/{Env.timestamp}/{trace}',
    }

    return configuration


def build_trace_name(params: dict) -> str:
    """ Build trace name from configuration """

    trace_name = f'exp_{Env.config['meta']['experiment']}_{Env.iteration:03d}'

    return trace_name
