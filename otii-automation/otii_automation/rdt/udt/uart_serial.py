import time
import serial
from .util import logger


class UdtUartSerial:
    def __init__(self, port):
        self.ser = serial.Serial(
            port=port,
            baudrate=115200,
            parity=serial.PARITY_NONE,
            stopbits=serial.STOPBITS_ONE,
            bytesize=serial.EIGHTBITS,
            timeout=None
        )

        # Open connection is not already opened
        if not self.ser.isOpen():
            self.ser.open()

        # Reset buffers
        self.ser.reset_input_buffer()
        self.ser.reset_output_buffer()

        # Build time (used as base time for uart serial)
        self.reference = time.time()

    def __del__(self):
        self.ser.close()

    def send(self, message: str) -> None:

        # Send on uart channel (\r\n required)
        self.ser.write(f'{message}\n'.encode('UTF-8'))

    def receive(self, timeout: float = None) -> [str, float]:
        self.ser.timeout = timeout

        message = self.ser.readline().decode('UTF-8').strip()
        if message == '':
            logger.debug(f'Timeout expired')
        else:
            logger.debug(f'Received from uart channel: {message}')

        return message, time.time() - self.reference
