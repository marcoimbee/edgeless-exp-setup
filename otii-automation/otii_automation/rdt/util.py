import logging

from crc import Calculator, Configuration, Crc8

CONFIG: Configuration = Crc8.CCITT.value

logger = logging.getLogger('rdt')


def crc_8(data: bytes) -> str:
    calculator = Calculator(CONFIG)
    crc = calculator.checksum(data)

    return f'{crc:02x}'
