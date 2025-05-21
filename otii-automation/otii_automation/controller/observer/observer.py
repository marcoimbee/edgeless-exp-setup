import logging
import json
import threading
import win_precise_time as wpt
import paho.mqtt.client as mqtt

from ...environment import Environment as Env


class Observer:
    logger = logging.getLogger('observer')

    def __init__(self):
        """ Non-blocking observer class to ensure that evaluated requests are successfully completed """

        self.host = Env.config['observer']['host']
        self.port = Env.config['observer']['port']
        self.ca_file = Env.config['observer']['ca_file']
        self.client_id = Env.config['observer']['client_id']
        self.topic = Env.config['observer']['topic']

        # Configure client
        self.client = mqtt.Client(client_id=self.client_id)
        # self.client.tls_set(ca_certs=self.ca_file)

        # Set callbacks
        self.client.on_connect = lambda client, _data, _flags, rc: self._on_connect(client, rc)
        self.client.on_message = lambda _client, _data, message: self._on_message(message)
        self.client.on_disconnect = lambda _client, _data, rc: self._on_disconnect(rc)

        self.observer_proc = None
        self.messages = []

    def __del__(self):
        if self.client is not None:
            self.stop_observing()

    def start_observing(self):
        """ Start client and subscribe to topic """

        # Connect to broker and subscribe to topic
        self.client.connect(self.host, self.port)

        # Launch worker thread
        self.observer_proc = threading.Thread(target=self.client.loop_forever)
        self.observer_proc.daemon = True
        self.observer_proc.start()

    def stop_observing(self):
        """ Stop client and disconnect """

        # Stop worker thread and disconnect
        self.client.disconnect()
        self.client = None

    def is_connected(self):
        return self.client.is_connected()

    def clean(self):
        self.messages = []

    def dump_observed(self, observer_file: str):
        """ Save observed messages on file """
        with open(observer_file, 'w') as fp:
            json.dump(self.messages, fp, indent=1)

        # Clean up dumped messages
        self.messages = []

    def _on_connect(self, client: mqtt.Client, rc):
        Observer.logger.info(f'Observer connected: {"success" if rc == 0 else rc}')
        if len(self.messages) > 0:
            Observer.logger.info(f'Initial observed messages:  {len(self.messages)}')

        client.subscribe(topic=self.topic, qos=0)

    def _on_message(self, message):
        rx_ts = wpt.time_ns()
        gen_ts = int(message.payload.decode()[:19])
        send_ts = None
        if message.payload.decode()[-19:].isdigit():
            send_ts = int(message.payload.decode()[-19:])

        # Observer.logger.debug(f'Message on "{message.topic}" of size {len(payload)} bytes')
        if send_ts is not None:
            self.messages.append({'rx_ts': rx_ts, 'gen_ts': gen_ts, 'send_ts': send_ts})
        else:
            self.messages.append({'rx_ts': rx_ts, 'gen_ts': gen_ts})

    def _on_disconnect(self, rc):
        Observer.logger.info(f'Observer disconnected: {"success" if rc == 0 else rc}')
        Observer.logger.info(f'Observed messages: {len(self.messages)}')
