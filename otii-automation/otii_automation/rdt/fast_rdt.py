import json

from .exception import RdtException
from .message import Message
from .util import crc_8, logger

MAX_CTR = 2 ** 8


class FastRdt:
    def __init__(self, udt):
        self.udt = udt

    def udt_send(self, code: Message, payload: dict = None) -> None:
        if payload is None:
            msg = json.dumps({'code': code.value})
        else:
            msg = json.dumps({'code': code.value, 'payload': payload})

        # logger.info(f'[UDT] Sending raw JSON: {msg}')
        self.udt.send(msg)

    def udt_receive(self) -> [str, float]:
        msg, timestamp = self.udt.receive()
        # logger.info(f'[UDT] Received raw JSON: {msg}')
        return json.loads(msg), timestamp

    def send(self, code: Message, payload: dict = None) -> None:
        if payload is None:
            msg = json.dumps({'code': code.value})
        else:
            msg = json.dumps({'code': code.value, 'payload': payload})

        encoded_msg = msg.encode(encoding='utf-8')
        crc_value = crc_8(encoded_msg)
        rdt_pkt = f'{msg}{crc_value}'

        # logger.info(f'[RDT] Sending encoded message: {encoded_msg}')
        # logger.info(f'[RDT] Computed CRC: {crc_value}')
        # logger.info(f'[RDT] Full packet to send: {rdt_pkt}')

        self.udt.send(rdt_pkt)

        # logger.info(f'[RDT] Sent rdt_pkt: {rdt_pkt}')

    def receive(self, timeout=None) -> [dict, float]:
        # logger.info('[RDT] RECEIVE CALLED - Waiting for data...')
        rdt_pkt, timestamp = self.udt.receive(timeout=timeout)
        
        # logger.info(f'[RDT] Raw received packet: {rdt_pkt}, len: {len(rdt_pkt)}')

        # Check integrity (message format)
        if len(rdt_pkt) < 3:
            # logger.info(f'[RDT] Received corrupted message or timeout expired {rdt_pkt}')
            raise RdtException(f'Received corrupted message or timeout expired {rdt_pkt}')

        # Check integrity (crc)
        crc_received = rdt_pkt[-2:]
        # crc: str = rdt_pkt[-2:]
        msg: str = rdt_pkt[:-2]
        crc_expected = crc_8(msg.encode(encoding='utf-8'))

        # logger.info(f'[RDT] Extracted message: {msg}')
        # logger.info(f'[RDT] Received CRC: {crc_received}, Expected CRC: {crc_expected}')


        # if crc != crc_8(msg.encode(encoding='utf-8')):
        if crc_received != crc_expected:
            # logger.error(f'[RDT] Invalid CRC - Packet: {rdt_pkt}')
            raise RdtException(f'Invalid crc {rdt_pkt}')

        # logger.info(f'[RDT] Successfully received and validated message: {msg}')

        # # Parse message
        json_msg = json.loads(msg)

        return json_msg, timestamp
