import itertools
import random
import time

from ...environment import Environment


class Experiment:
    def __init__(self):
        self.seed = time.time_ns()
        random.seed(self.seed)
        param_values = []
        param_names = []
        for param in Environment.config['params'].values():
            for key, value in param.items():
                param_values.append(value if isinstance(value, list) else [value])
                param_names.append(key)

        self.configs = [{k: v for k, v in zip(param_names, config)} for config in itertools.product(*param_values)]
        random.shuffle(self.configs)

    def __iter__(self):
        return iter(self.configs)

    def __len__(self):
        return len(self.configs)
