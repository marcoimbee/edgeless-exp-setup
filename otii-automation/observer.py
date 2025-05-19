from analysis.aoi import fast_aoi
from otii_automation.controller.observer import Observer
from otii_automation.environment import Environment as Env


def observer():
    Env.init(experiment=False)
    obs = Observer()
    obs.start_observing()
    input("Press enter to stop observing ...")
    obs.stop_observing()
    obs.dump_observed('results/observer.json')


if __name__ == '__main__':
    observer()
