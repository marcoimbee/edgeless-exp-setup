import redis
import time
import os
import sys
import subprocess

SLEEP_TIME = 5
OUTPUT_FILE = "edgeless/target/debug/node_output.log"

# Connect to Redis
r = redis.Redis(host='localhost', port=6379, db=0)

def get_function_keys():
    """Retrieve all 'function:<id>:samples' keys and their corresponding average keys."""
    sample_keys = r.keys("function:*:samples")
    keys = []
    for sample_key in sample_keys:
        sample_key = sample_key.decode("utf-8")
        base = sample_key.rsplit(":", 1)[0]  # remove ':samples'
        function_id = base.split(":", 1)[1]  # extract <whatever>
        average_key = f"{base}:average"
        keys.append((function_id, sample_key, average_key))
    return keys

def monitor():
    try:
        while True:
            os.system("clear")
            keys = get_function_keys()
            tot_keys = len(keys)

            print(f"Retrieved keys for {tot_keys} functions:\n")
            for function_id, sample_key, average_key in keys:
                samples = r.lrange(sample_key, 0, -1)
                samples = [s.decode("utf-8") for s in samples]
                sample_count = len(samples)
                average_raw = r.get(average_key)
                average = average_raw.decode("utf-8") if average_raw else "None"

                print(f"{function_id}:")
                if sample_count > 6:
                    first_ten_samples = samples[:6]
                    print(f"    samples ({sample_count}): {first_ten_samples}, [...]")
                else:
                    print(f"    samples ({sample_count}): {samples}")
                print(f"    Exponential Moving Average (EMA): {average} ms")

                classical_avg = sum(int(x.split(',')[0]) for x in samples) / sample_count
                print(f"    Classical average: {classical_avg} ms")
                print()

            time.sleep(SLEEP_TIME)

    except KeyboardInterrupt:
        print("\nInterrupted by user.")
        response = input("FLUSHALL the Redis keys? (y/n): ").strip().lower()
        if response in ("yes", "y"):
            try:
                r.flushall()
                print("Redis FLUSHALL executed successfully.")
            except Exception as e:
                print(f"Failed to flush Redis: {e}")
        else:
            print("Redis FLUSHALL aborted.")
        sys.exit(0)

# def extract_functionlogentry(line):
#     if "FunctionLogEntry" in line:
#         start_index = line.find("FunctionLogEntry") + len("FunctionLogEntry")
#         remaining_line = line[start_index:].strip()
#         return remaining_line.split(1)      # Gets the function name
#     return None

# def read_new_lines(file_path, last_position):
#     with open(file_path, 'r') as file:
#         file.seek(last_position)
#         new_lines = file.readlines()
#         return new_lines, file.tell()
    
# def monitor_log_file(file_path, interval):
#     last_position = 0
#     while True:
#         new_lines, last_position, = read_new_lines(file_path, last_position)

#         for line in new_lines:
#             result = extract_functionlogentry(line)
#             if result:


if __name__ == "__main__":
    monitor()
