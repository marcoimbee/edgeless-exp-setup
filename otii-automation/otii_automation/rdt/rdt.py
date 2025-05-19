import json

from .exception import RdtException
from .message import Message
from .util import crc_8, logger

MAX_CTR = 2 ** 8


class Rdt:
    def __init__(self, udt):
        self.udt = udt
        self.tx_ctr = 0
        self.rx_ctr = 0

    def udt_send(self, code: Message, payload: dict = None) -> None:
        if payload is None:
            msg = json.dumps({'code': code.value})
        else:
            msg = json.dumps({'code': code.value, 'payload': payload})

        # logger.info(f'Udt sent: {msg}')
        self.udt.send(msg)

    def udt_receive(self) -> [str, float]:
        msg, timestamp = self.udt.receive()
        # logger.info(f'Udt received: {msg}')

        return json.loads(msg), timestamp

    def send(self, code: Message, payload: dict = None, **kwargs) -> None:
        if payload is None:
            msg = json.dumps({'code': code.value})
        else:
            msg = json.dumps({'code': code.value, 'payload': payload})
        encoded_msg = msg.encode(encoding='utf-8')
        protected = self.tx_ctr.to_bytes(2, byteorder='big') + encoded_msg
        rdt_pkt = f'{msg}{crc_8(protected)}'
        ack = False

        reset_counter = 10
        while not ack:
            # reset_counter -= 1
            if reset_counter < 0:
                self._reset()
                raise RdtException('RDT send failed too many times')
            self.udt.send(rdt_pkt)
            ack = self._recv_ack() or kwargs.get('no_ack', False)

        # logger.info(f'Sent: {msg}')

        # Update tx_ctr
        self.tx_ctr = (self.tx_ctr + 1) % MAX_CTR

    def receive(self, timeout=None) -> [dict, float]:
        msg = ''
        reset_counter = 10
        while True:

            # reset_counter -= 1
            if reset_counter < 0:
                self._reset()
                raise RdtException('RDT receive failed too many times')

            rdt_pkt, timestamp = self.udt.receive(timeout=timeout)
            if len(rdt_pkt) < 3:
                self._send_ack(nack=True)
                continue

            crc: str = rdt_pkt[-2:]
            msg: str = rdt_pkt[:-2]
            if crc == crc_8(self.rx_ctr.to_bytes(length=2, byteorder='big') + msg.encode()):
                self._send_ack()
                break
            else:
                # logger.info(f'Invalid crc: {crc}')
                self._send_ack(nack=True)

        # logger.info(f"Received: {msg}")

        # Parse message
        json_msg = json.loads(msg)

        # Update rx_ctr
        self.rx_ctr = (self.rx_ctr + 1) % MAX_CTR

        return json_msg, timestamp

    def _send_ack(self, nack=False) -> None:
        if nack:
            ctr = ((self.rx_ctr - 1) % MAX_CTR).to_bytes(2, byteorder='big')
        else:
            ctr = self.rx_ctr.to_bytes(2, byteorder='big')

        ack = crc_8(ctr)
        self.udt.send(ack)

        # logger.info(f'{"Nack" if nack else "Ack"} sent: {ctr}')

    def _recv_ack(self) -> bool:
        ack, _ = self.udt.receive(timeout=8)
        if ack == '':
            return False
        if len(ack) != 2:
            # logger.info(f'Duplicated message: {ack}')
            # Send nack (ack for the previous message)
            self._send_ack(nack=True)
            return False
        if ack == crc_8(self.tx_ctr.to_bytes(2, byteorder='big')):
            # logger.info(f'Ack received: {ack}')
            return True
        else:
            # logger.info(f'Invalid ack: {ack}')
            return False

    def _reset(self):
        # self.tx_ctr = 0
        # self.rx_ctr = 0
        # self.udt.send(json.dumps({'code': Message.RST.value}))
        logger.info("RDT reset")
