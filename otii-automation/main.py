import traceback
import logging
from otii_automation import Mode, Environment


def main():
    try:
        mode = Environment.init()
        if mode == Mode.DEVICE:
            from otii_automation.device import device
            device()
        elif mode == Mode.CONTROLLER:
            from otii_automation.controller import controller
            controller()
    except Exception as e:
        logging.error(e)
        logging.error(traceback.format_exc())


if __name__ == "__main__":
    main()
