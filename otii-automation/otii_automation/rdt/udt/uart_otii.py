import time

from .util import logger
from otii_tcp_client.arc import Arc
from otii_tcp_client.recording import Recording


class UdtUartOtii:

    def __init__(self, arc: Arc, recording: Recording):
        # Arc device
        self.arc = arc

        # Otii current recording
        self.recording = recording

        # Init recv message counter
        self.c_msg = 0

    def send(self, msg: str) -> None:
        logger.debug(f'Sending on uart channel...')
        self.arc.write_tx(f'{msg}\n')
        logger.debug(f'Sent on uart channel: {msg}')

    def receive(self, timeout=None) -> [str, float]:
        logger.debug(f'Receiving from uart channel...')

        start_recv = time.time()
        while (self.recording.get_channel_data_count(self.arc.id, 'rx') - self.c_msg) == 0:
            if timeout is not None and time.time() - start_recv > timeout:
                logger.debug(f'Timeout expired')
                return '', 0

        rx_data = self.recording.get_channel_data(
            self.arc.id,
            channel='rx',
            index=self.c_msg,
            count=1
        )

        logger.debug(f'Received from uart channel: {rx_data["values"][0]}')

        # Extract data
        message = rx_data['values'][0]

        # Update received messages counter
        self.c_msg += 1

        return message['value'], message['timestamp']
