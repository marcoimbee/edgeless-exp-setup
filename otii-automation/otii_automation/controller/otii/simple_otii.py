import logging
import os
import time
import traceback

from otii_tcp_client.otii import Otii
from otii_tcp_client.otii_connection import OtiiConnection
from otii_tcp_client.arc import Arc
from otii_tcp_client.recording import Recording
from otii_tcp_client.project import Project

from ...rdt import Rdt
from ...rdt.message import Message
from ...rdt.udt.uart_otii import UdtUartOtii
from ...environment import Environment as Env

logger = logging.getLogger('otii')


class SimpleOtii:
    def __init__(self):
        # Connect to Otii Server
        connection = OtiiConnection(Env.config['otii']['hostname'], Env.config['otii']['port'])
        connection.connect_to_server(try_for_seconds=3)

        self.otii = Otii(connection)
        self.otii.login(Env.config['otii']['license_user'], Env.config['otii']['license_psw'])
        self.project: Project = None
        self.arc: Arc = None
        self.rdt: Rdt = Rdt(None)

    def create_project(self) -> None:
        """ Create a new project """
        self.project: Project = self.otii.create_project()
        self._init_device()

    def save_project(self, path: str) -> None:
        self.project.save_as(os.path.join(os.getcwd(), path), force=True)

    def start_recording(self) -> None:
        self.project.start_recording()
        self.rdt.udt = UdtUartOtii(self.arc, self.project.get_last_recording())

    def stop_recording(self, trace_name: str) -> None:
        self.project.stop_recording()
        recording: Recording = self.project.get_last_recording()
        recording.rename(trace_name)

    def get_energy(self, start: float, stop: float) -> dict:
        """ Retrieve energy required by current configuration """
        recording: Recording = self.project.get_last_recording()

        return recording.get_channel_statistics(self.arc.id, 'mc', start, stop)

    def send(self, code: Message, payload: dict = None, **kwargs) -> None:
        """ Send message on uart channel """

        if kwargs.get('udt', False) is True:
            self.rdt.udt_send(code, payload)
        else:
            self.rdt.send(code, payload)

    def receive(self, timeout=None) -> [dict, float]:
        """ Receive message from uart channel """
        return self.rdt.receive(timeout)

    def reset(self, project_path: str) -> None:
        """ Reset Otii device """
        try:
            self.project.close()
            self.otii.connection.close_connection()
        except Exception as ex:
            logger.warning(f'Failed to release resources during reset: {ex}')
            logger.error(traceback.format_exc())

        new_connection = OtiiConnection(Env.config['otii']['hostname'], Env.config['otii']['port'])
        new_connection.connect_to_server(try_for_seconds=10)
        self.otii = Otii(new_connection)
        time.sleep(3)
        self.otii.login(Env.config['otii']['license_user'], Env.config['otii']['license_psw'])

        if project_path is not None:
            self.project: Project = self.otii.open_project(os.path.join(os.getcwd(), project_path))
        else:
            self.project: Project = self.otii.create_project()

        self._init_device()

    def _init_device(self):

        # Get Otii devices
        devices: list[Arc] = self.otii.get_devices()
        for device in devices:
            logger.info(f'Device found: {device.name}')
        if len(devices) == 0:
            raise Exception("No devices found")
        else:
            self.arc: Arc = devices[0]

        # Set range
        self.arc.set_range("high")
        time.sleep(0.5)

        # Main voltage
        self.arc.set_main_voltage(5)
        time.sleep(0.5)

        # Expansion voltage
        self.arc.set_exp_voltage(3.3)
        time.sleep(0.5)

        # Max current (set cut-off)
        self.arc.set_max_current(4.5)
        time.sleep(0.5)

        # Enable 'mc' channel
        self.arc.enable_channel('mc', True)
        time.sleep(0.5)

        # Enable 'mv' channel
        self.arc.enable_channel('mv', True)
        time.sleep(0.5)

        # Set main power to all devices
        self.otii.set_all_main(True)
        time.sleep(0.5)

        # Enable expansion port
        self.arc.enable_exp_port(True)
        time.sleep(0.5)

        # Enable uart channel
        self.arc.enable_uart(True)
        time.sleep(0.5)

        # Set baudrate
        self.arc.set_uart_baudrate(Env.config['otii']['baudrate'])
        time.sleep(0.5)

        # Enable rx channel
        self.arc.enable_channel("rx", True)
        time.sleep(0.5)

        # Enable tx channel
        self.arc.set_tx(True)
        time.sleep(0.5)

    def __del__(self):
        try:
            if hasattr(self, 'otii'):
                self.otii.connection.close_connection()
                logging.info("Disconnected from Otii server")
        except Exception as ex:
            logger.warning(f'Failed to release resources: {ex}')
