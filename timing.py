import subprocess
import time

FILE_NAME = "test.txt"
NUM_RUNS = 10
BINARY = r"target\release\cs581-project.exe"
USE_MICROSECONDS = False     # Set to True to display times in microseconds

def build_project():
    print("Building release binary…")
    subprocess.run(["cargo", "build", "--release"], check=True)

def benchmark():
    timings = []
    for i in range(1, NUM_RUNS + 1):
        start = time.perf_counter()
        subprocess.run([BINARY, "compress", FILE_NAME], check=True)
        end = time.perf_counter()
        elapsed = end - start 
        timings.append(elapsed)
        if USE_MICROSECONDS:
            elapsed_disp = elapsed * 1e6
            unit = "µs"
        else:
            elapsed_disp = elapsed
            unit = "s"
        print(f"Run {i}/{NUM_RUNS}: {elapsed_disp:.2f} {unit}")
    avg = sum(timings) / len(timings)
    if USE_MICROSECONDS:
        avg_disp = avg * 1e6
        unit = "µs"
    else:
        avg_disp = avg
        unit = "s"
    print(f"\nAverage over {NUM_RUNS} runs: {avg_disp:.2f} {unit}")

if __name__ == "__main__":
    build_project()
    benchmark()

