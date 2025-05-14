import redis
import json
import os
import re

ABS_PATH = '/home/pi/Desktop/otii-automation/otii_automation/device/edgeless'
LOG_FILES = [f"{ABS_PATH}/rpi_node_log.log", f"{ABS_PATH}/vm_node_log.log"]       # The file that contains logs with UUIDs and function names
JSON_OUTPUT = f"{ABS_PATH}/exp_data.json"  # Output JSON file to store experiment iterations

REDIS_HOST = "<your-Redis-host-IP>"
REDIS_PORT = 6379
REDIS_DB = 0


def parse_log_file(log_file_path):
    function_pattern = r'Event: FunctionLogEntry\(Info, "(.*?)", .*?tags: {.*?"FUNCTION_ID": "(.*?)".*?}'

    with open(log_file_path, 'r') as log_file:
        log_data = log_file.read()

    function_info = re.findall(function_pattern, log_data)

    # Keep only one entry per UUID
    unique_function_info = {}
    for function_name, function_uuid in function_info:
        unique_function_info[function_uuid] = function_name

    # Sort entries by UUID
    results = dict(sorted(unique_function_info.items()))
    print(f"[LOG_PARSER] Results from {log_file_path}: {results}")
    return results

def append_experiment_iteration(uuid_func_map, redis_conn, output_path, iteration):
    if os.path.exists(output_path):
        with open(output_path, 'r') as f:
            try:
                existing = json.load(f)
            except Exception:
                existing = []
    else:
        existing = []

    data = []

    for uuid, func_name in uuid_func_map.items():
        samples_key = f"function:{uuid}:samples"

        try:
            samples = redis_conn.lrange(samples_key, 0, -1)
            samples = [s.decode("utf-8") for s in samples]
        except Exception as e:
            print(f"[LOG_PARSER] Error fetching Redis data for UUID {uuid}: {e}")
            raise Exception(e) from e 

        if len(samples) != 0:
            data.append({
                "uuid": uuid,
                "function_name": func_name,
                "samples": samples
            })

    new_entry = {
        "iteration": str(iteration),
        "data": data
    }

    existing.append(new_entry)

    with open(output_path, 'w') as f:
        json.dump(existing, f, indent=2)

    print(f"[LOG_PARSER] Appended iteration {iteration} to {output_path}")

def delete_function_keys(redis_conn):
    patterns = ["function:*:samples"]

    try:
        for pattern in patterns:
            matched = list(redis_conn.scan_iter(match=pattern))
            print(f"[LOG PARSER] Found {len(matched)} keys to delete")
            for key in matched:
                redis_conn.delete(key)
                print(f"[LOG_PARSER] Deleted key: {key.decode() if isinstance(key, bytes) else key}")
    except Exception as e:
        print(f"[LOG_PARSER] Failed to delete function keys: {e}")
        raise Exception(e) from e

def edgeless_parse(iteration):
    try:
        redis_conn = redis.Redis(host=REDIS_HOST, port=REDIS_PORT, db=REDIS_DB)
        redis_conn.ping()
        print(f"[LOG_PARSER] Connected to Redis at {REDIS_HOST}:{REDIS_PORT}")
    except Exception as e:
        print(f"[LOG_PARSER] Failed to connect to Redis: {e}")
        raise Exception('Failed to connect to Redis')

    combined_uuid_func_map = {}

    for log_file in LOG_FILES:
        print(f"[LOG_PARSER] Checking {log_file}...")
        if os.path.isfile(log_file):
            print(f"[LOG_PARSER] Parsing {log_file}...")
            uuid_func_map = parse_log_file(log_file)
            combined_uuid_func_map.update(uuid_func_map)
        else:
            print(f"[LOG_PARSER] {log_file} does not exist")

    if not uuid_func_map:
        print("[LOG_PARSER] No UUID-function pairs found in the log file.")
        raise Exception('No UUID-function pairs found in the log file')

    append_experiment_iteration(combined_uuid_func_map, redis_conn, JSON_OUTPUT, iteration)

    delete_function_keys(redis_conn)
