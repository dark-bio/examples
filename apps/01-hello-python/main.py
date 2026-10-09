"""The smallest possible Ark app, in Python."""
import sys


def main():
    # With no data directory, print the manifest and stop.
    if len(sys.argv) < 2:
        print('manifest = 1\n\n[app]\nname = "hello-python"\nversion = "0.1.0"')
        return

    # The report pass receives a data directory, even when no data is requested
    print("Hello from an Ark app written in Python.")


if __name__ == "__main__":
    main()
