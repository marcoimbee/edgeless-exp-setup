import logging

from paramiko.client import SSHClient, AutoAddPolicy

from ...environment import Environment as Env
from .scripts import *

logger = logging.getLogger('traffic-ctrl')


def exec_command(commands: str | list[str]):
    if not isinstance(commands, list):
        commands = [commands]

    with SSHClient() as ssh:
        ssh.set_missing_host_key_policy(AutoAddPolicy())
        ssh.connect(username=Env.config['server']['username'],
                    hostname=Env.config['server']['host'],
                    key_filename=Env.config['server']['key_file'])

        for cmd in commands:
            _stdin, stdout, stderr = ssh.exec_command(cmd)
            logger.debug(cmd)
            if stdout.channel.recv_exit_status() != 0:
                raise Exception(''.join(stderr.readlines()))

            out = ''.join(stdout.readlines()).strip()
            err = ''.join(stderr.readlines()).strip()

            if out != '':
                logger.debug(f'Stdout: {out}')
            if err != '':
                logger.debug(f'Stderr: {err}')


def init_bandwidth_and_delay():
    """ Init simulated network on server """

    # Init command

    exec_command(build_init())


####################################################### (i will not need to limit the bandwidth)
def set_bandwidth_and_delay(dl_bandwidth, ul_bandwidth, delay):
    """ Set bandwidth limit and additional delay on server """

    if dl_bandwidth == "100%":
        exec_command(build_set_1(delay))
    else:
        if ul_bandwidth is None:
            exec_command(build_set_2(delay, dl_bandwidth))
        else:
            exec_command(build_set_3(delay, dl_bandwidth, ul_bandwidth))

    if dl_bandwidth == '100%':
        logger.debug(f'Bandwidth: {dl_bandwidth}')
    else:
        logger.debug(f'Max download bandwidth: {dl_bandwidth}Mbps')
    if ul_bandwidth is not None:
        logger.debug(f'Max upload bandwidth: {ul_bandwidth}Mbps')
#########################################################


def restore_bandwidth_and_delay():
    """ Restore network """

    # Restore command
    exec_command(build_restore())
