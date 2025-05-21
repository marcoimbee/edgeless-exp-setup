from enum import Enum


class Message(Enum):
    START_CONFIG = 0
    STOP_CONFIG = 1
    START_REQ = 2
    STOP_REQ = 3
    PUSH_RESULTS = 4
    END_EXPERIMENT = 5
    CONFIG_OK = 6
    NETWORK_INFO_OK = 7
    START_WORKFLOW_EXECUTION = 8
    STOP_WORKFLOW_EXECUTION = 9
    ERROR = 10
    RST = 127
    TEST = 128
