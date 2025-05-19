from enum import Enum


class Mode(Enum):
    CONTROLLER = 'controller'
    DEVICE = 'device'

    @classmethod
    def valueOf(cls, mode: str):
        try:
            return cls(mode) if mode is not None else None
        except ValueError:
            raise ValueError(f'{mode}')
