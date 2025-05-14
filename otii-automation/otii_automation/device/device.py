import traceback
import logging
import os
import time
from time import sleep

from .util import logger
from ..rdt import Rdt
from ..rdt.exception import RdtException
from ..rdt.message import Message
from ..rdt.udt.uart_serial import UdtUartSerial
from .edgeless.edgeless import *
from .edgeless.log_parser import edgeless_parse

rdt = Rdt(UdtUartSerial('/dev/ttyS0'))

PREV_ITERATION = None


def start_configuration(config):
    global PREV_ITERATION

    # Results folder
    os.makedirs(config['results_dir'], exist_ok=True)

    is_vm_involved = (config['experiment'] == 'first-func-on-vm-second-func-on-rpi' or config['experiment'] == 'first-func-on-rpi-second-func-on-vm' or config['experiment'] == 'both-funcs-on-vm')

    logger.info('\n')
    logger.info(f'Start configuration {config}. Is the VM node involved in this? {is_vm_involved}. Previous iteration ID: {PREV_ITERATION}')

    if config['experiment'] == 'both-funcs-on-rpi' or config['experiment'] == 'first-func-on-vm-second-func-on-rpi' or config['experiment'] == 'first-func-on-rpi-second-func-on-vm' or config['experiment'] == 'both-funcs-on-vm':
        # Reset AoI file
        reset_aoi_log(config['aoi_logs_path'], config['iteration'], False)

        # Start workflow on RPI
        workflow_uuid, errors = start_workflow(config['edgeless_exec_dir'], config['workflow_path'])

        if errors == '':
            logger.info(f"Started workflow with UUID: {workflow_uuid}.")
            logger.info('Waiting 30 seconds to setup everything...')
            
            sleep(30)
            try:
                rdt.send(Message.START_WORKFLOW_EXECUTION)
            except Exception as ex:
                logger.info(f"EXCEPTION IN RPI-to-PC UART: {ex}")
            logger.info('Notified controller...')

            # Wait for the workflow to run for target time (60s)
            logger.info(f"Started actual computations. Waiting {config['duration']} seconds...")
            sleep(config['duration'])

            # Stop workflow on RPI
            rdt.send(Message.STOP_WORKFLOW_EXECUTION)
            stop_workflow(config['edgeless_exec_dir'], workflow_uuid)
            logger.info(f"Stopped workflow with UUID {workflow_uuid} and notified controller")

            reset_aoi_log(config['aoi_logs_path'], config['iteration'], True)

            sleep(5)

            # Nodes' log files management
            # RPI log file already placed in dest folder
            if is_vm_involved:      # Only get VM's one if involved in the exp
                get_vm_log_file()
            
            edgeless_parse(config['iteration'])           # Parsing the files
        else:
            raise Exception('Error starting the workflow. Repeating the experiment.')
    else:
        logger.error(f"Undefined experiment type: {config['experiment']}")

    # Notify configuration completed
    time.sleep(5)
    rdt.send(Message.STOP_CONFIG)
    logger.info("Configuration completed")

    PREV_ITERATION = config['iteration']

def device():
    logging.getLogger('paramiko.transport').setLevel(logging.WARNING)

    while True:
        try:
            logger.info('\n')
            logger.info("Waiting for UART messages...")
            message, _ = rdt.receive()

            if message['code'] == Message.START_CONFIG.value:
                start_configuration(message['payload'])

            elif message['code'] == Message.END_EXPERIMENT.value:
                logger.info('Experiment concluded')
                break
            else:
                raise RdtException(f'Unknown command: {message}')
        except Exception as ex:
            logger.error(f'Exception on device: {ex}')
            logger.error(traceback.format_exc())

            # Send error message
            try:
                rdt.send(Message.ERROR)
            except Exception as ex:
                logger.warning(f'Error message not sent: {ex}')
