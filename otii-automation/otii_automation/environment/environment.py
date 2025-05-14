import os
import logging
import datetime
from argparse import ArgumentParser
from sys import stdout

import tomli

from .mode import Mode


class Environment(object):
    log_time_format = '%Y-%m-%d %H:%M:%S'
    log_format = '[%(asctime)s][%(name)-15s][%(levelname)-7s] - %(message)s'
    log_level = logging.DEBUG
    config = None
    trace_counter = 1
    iteration = 0
    timestamp = str(datetime.datetime.today().strftime('%Y-%m-%d_%H-%M-%S'))
    base_dir = 'results'
    otii_dir: str
    log_dir: str
    log_file: str

    @classmethod
    def init(cls, experiment=True) -> any:
        if not hasattr(cls, 'instance'):
            cls.instance = super(Environment, cls).__new__(cls)

            # Argument parser
            parser = ArgumentParser(description='Launch experiment')
            subparsers = parser.add_subparsers(help='Experiment side sub-commands', dest='mode', required=True)

            # Controller arguments parser
            parser_controller = subparsers.add_parser('controller', help='Launch experiment on controller side')
            parser_controller.add_argument('-c', '--config', type=str, nargs=1, default=['config.toml'], metavar='',
                                           help='experiment configuration file.toml')

            # Device arguments parser
            subparsers.add_parser('device', help='Launch experiment on device side')

            # Retrieve configuration
            args = parser.parse_args()
            if Mode.valueOf(args.mode) == Mode.CONTROLLER:
                with open(args.config[0], 'rb') as fin:
                    cls.config: dict = tomli.load(fin)
                if not experiment:
                    return None

                cls.base_dir = os.path.join(cls.base_dir, cls.config['meta']['experiment'], cls.timestamp)
                cls.otii_dir = os.path.join(cls.base_dir, 'otii')
                cls.log_dir = os.path.join(cls.base_dir, 'logs')

                os.makedirs("results", exist_ok=True)
                os.makedirs("results/aoi", exist_ok=True)
                os.makedirs("results/energy", exist_ok=True)
                os.makedirs(cls.base_dir, exist_ok=True)
                os.makedirs(cls.otii_dir, exist_ok=True)
                os.makedirs(os.path.join(cls.base_dir, 'logs'), exist_ok=True)
                cls.log_file = os.path.join(cls.log_dir, 'controller.log')
            else:
                os.makedirs('logs', exist_ok=True)
                cls.log_file = f'logs/device_{cls.timestamp}.log'

            # Logging configuration
            logging.basicConfig(
                level=cls.log_level,
                format=cls.log_format,
                datefmt=cls.log_time_format,
                filename=cls.log_file
            )
            logging.getLogger('paramiko.transport').setLevel(logging.WARNING)

            if experiment:
                stdout_handler = logging.StreamHandler(stdout)
                stdout_handler.setLevel(logging.INFO)
                stdout_handler.setFormatter(logging.Formatter(cls.log_format, datefmt='%Y-%m-%d %H:%M:%S'))

                logging.getLogger().addHandler(stdout_handler)

            return Mode.valueOf(args.mode)

    def __str__(self):
        return f"Environment: {self.__class__.__dict__}"
