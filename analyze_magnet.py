import sys
import os
import re
from urllib.parse import urlparse, parse_qs

def analyze_magnet_link(file_paths_or_strings):
    # Regular expression pattern for a magnet link
    magnet_pattern = r'magnet:\?xt=urn:[a-zA-Z0-9]+:[a-zA-Z0-9]{32,40}'

    magnets = []
    for thing in file_paths_or_strings:
        if os.path.isfile(thing):
            with open(thing, 'r') as file:
                for line in file:
                    line = line.strip()
                    if re.match(magnet_pattern, line):
                        magnets.append(line)
                    else:
                        print(f'Invalid magnet link: {line}')
        else:
            thing = thing.strip()
            if re.match(magnet_pattern, thing):
                magnets.append(thing)
            else:
                print(f'Invalid magnet link: {thing}')

    for magnet in magnets:
        # Parse the magnet link
        parsed_link = urlparse(magnet)
        # Get the parameters
        params = parse_qs(parsed_link.query)
        # Print the parameters
        print(f'Parameters for magnet link: {line}')
        for key, values in params.items():
            if key == 'tr':
                # Sort the "tr" parameters by their values
                values = sorted(values)
            for value in values:
                print(f'  {key}: {value}')

if 2 > len(sys.argv):
    print("Missing arguments")
    exit(1)

analyze_magnet_link(sys.argv[1:])
