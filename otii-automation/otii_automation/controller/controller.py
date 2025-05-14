import json
import os
import time
import traceback
from time import sleep

from ..environment import Environment as Env
from ..rdt import Message
from .experiment import Experiment
from .otii import SimpleOtii
from .util import download_results, logger, build_config_message, build_trace_name, download_device_logs
from ..rdt.exception import RdtException

otii: SimpleOtii


def launch_config(params: dict, iteration: int) -> bool:
    trace = build_trace_name(params)

    logger.info('\n')
    logger.info(f'Start configuration: {trace}, iteration: {iteration}, params: {params}')

    # Start trace recording on Otii
    otii.start_recording()                  # First start recording, then right away start the experiment
    logger.info('Recording started')

    # Send configuration message to device via UART
    for i in range(0, 30):
        config_message = build_config_message(params, trace, i)
        otii.send(Message.START_CONFIG, config_message)
        logger.info(f'Configuration message sent: {config_message}')

        message, timestamp = otii.receive(timeout=120)
        # logger.info(f'Received this message from the RPI: {message}')



# def launch_config(params: dict, iteration: int) -> bool:
#     trace = build_trace_name(params)

#     logger.info('\n')
#     logger.info(f'Start configuration: {trace}, iteration: {iteration}, params: {params}')

#     # Start trace recording on Otii
#     otii.start_recording()                  # First start recording, then right away start the experiment
#     logger.info('Recording started')

#     # Send configuration message to device via UART
#     config_message = build_config_message(params, trace, iteration)
#     otii.send(Message.START_CONFIG, config_message)
#     logger.info(f'Configuration message sent: {config_message}')

#     while True:
#         results = {
#             'trace_name': trace,
#             'energy': None,
#             'current': None,
#             'power': None,
#             'voltage': None,
#             'device': None,
#             'messages': [],
#             'config': params,
#             'start_wf_execution': None,
#             'stop_wf_execution': None
#         }

#         recvd_start_wf_msg = False
#         recvd_stop_wf_msg = False

#         while True:
#             message, timestamp = otii.receive(timeout=120)
#             logger.info(f'Received this message from the RPI: {message}')

#             results['messages'].append({'timestamp': timestamp, 'message': message['code']})

#             if message['code'] == Message.START_WORKFLOW_EXECUTION.value:
#                 recvd_start_wf_msg = True
#                 results['start_wf_execution'] = timestamp
#                 logger.info('Workflow execution started (finished setting up EDGELESS data structures)')
#             elif message['code'] == Message.STOP_WORKFLOW_EXECUTION.value:
#                 recvd_stop_wf_msg = True
#                 results['stop_wf_execution'] = timestamp
#                 logger.info('Workflow execution stopped')
#             elif message['code'] == Message.STOP_CONFIG.value:
#                 if not recvd_start_wf_msg or not recvd_stop_wf_msg:
#                     logger.info('Received STOP_CONFIG prematurely. Restarting in 10s...')
#                     time.sleep(10)
#                     raise Exception(f'Error on device: {message['code']}')
#                 else:    
#                     break
#             else:
#                 raise Exception(f'Error on device: {message['code']}')
            
#         if len(results['messages']) == 3:
#             break

#     # Stop trace recording on Otii
#     otii.stop_recording(trace)
#     logger.info(f'Recording stopped')

#     # Retrieve energy results
#     results['energy'] = otii.get_energy(results['start_wf_execution'], results['stop_wf_execution'])
#     # Needed by legacy code
#     results['energy']['diff_t'] = results['stop_wf_execution'] - results['start_wf_execution']
#     results['energy']['diff_ej'] = results['energy'].pop('energy')

#     # Dump results
#     summary_path = os.path.join(Env.base_dir, 'summary.json')
#     summary = []
#     if os.path.exists(summary_path):
#         with open(summary_path, 'r') as fp:
#             summary = json.load(fp)

#     with open(summary_path, 'w') as fp:
#         summary.append(results)
#         json.dump(summary, fp, indent=2)

#     # Save Otii project
#     otii.save_project(os.path.join(Env.otii_dir, f'Iteration_{Env.iteration}'))

#     logger.info(f'Configuration completed: {trace}')

#     return True


def controller() -> None:
    global otii
    global observer
    try:
        # Initialize components
        otii = SimpleOtii()

        experiment = Experiment()
        meta = Env.config['meta']
        meta['seed'] = experiment.seed
        meta['config'] = Env.config['params']
        meta['config'].update(Env.config['params'])
        with open(os.path.join(Env.base_dir, 'meta.json'), 'w') as fp:
            json.dump(meta, fp, indent=1)
        logger.info('Initialization completed')
    except Exception as ex:
        logger.error(f'Initialization failed: {ex}')
        logger.error(traceback.format_exc())
        return

    logger.info(f'Running {len(experiment)} configurations')

    try:
        # Run all iterations
        for iteration in range(0, Env.config['meta']['repetition']):
            otii.create_project()

            # Run all configurations
            for config in experiment:
                completed = False
                while completed is not True:
                    try:
                        completed = launch_config(config, iteration)
                    except Exception as ex:
                        logger.error(f'Configuration failed: {ex}')
                        logger.error(traceback.format_exc())

                        if not isinstance(ex, RdtException):
                            sleep(10)
                            project_path = os.path.join(Env.otii_dir, f'Iteration_{iteration}') if (
                                    (Env.trace_counter % len(experiment)) != 1) else None
                            logger.info(f'Resetting {project_path}')
                            otii.reset(project_path)

                Env.trace_counter += 1

            logger.info(f'Iteration {iteration} completed. Increasing iteration counter.\n')
            Env.iteration += 1

        # End experiment
        for _ in range(3):
            otii.send(Message.END_EXPERIMENT, no_ack=True)

        time.sleep(10)              

        logger.info('Experiment completed')
    except Exception as ex:
        logger.error(f'Experiment failed: {ex}')
        logger.error(traceback.format_exc())
