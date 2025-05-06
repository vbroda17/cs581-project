#!/usr/bin/env python3
import os
import argparse

def main():
    parser = argparse.ArgumentParser(
        description="List all files in a folder along with their sizes in bytes."
    )
    parser.add_argument(
        "folder",
        help="Path to the folder you want to scan"
    )
    args = parser.parse_args()
    folder = args.folder

    if not os.path.isdir(folder):
        print(f"Error: '{folder}' is not a directory.")
        return

    for entry in os.scandir(folder):
        if entry.is_file():
            size = entry.stat().st_size
            print(f"{entry.name}: {size} bytes")

if __name__ == "__main__":
    main()
