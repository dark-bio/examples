"""The smallest possible Ark app, in Python."""
import sys


def main():
    # With no data directory, print the manifest and stop.
    if len(sys.argv) < 2:
        print('[package]\nname = "hello-python"\nversion = "0.1.0"\ndatasets = []')
        return

    # The run pass receives a data directory, even when no data is requested.
    print("Hello from an Ark app written in Python.")


if __name__ == "__main__":
    main()
