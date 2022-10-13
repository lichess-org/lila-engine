"""External engine provider example for lichess.org"""

import argparse
import logging
import requests
import sys
import os

def register_engine(args, token):
    pass

def main(args):
    return

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--name", default="Example", help="Engine name to register")
    parser.add_argument("--engine", help="Shell command to launch UCI engine", required=True)
    parser.add_argument("--endpoint", default="https://engine.lichess.ovh", help="Defaults to https://engine.lichess.ovh")
    parser.add_argument("--token", default=os.environ.get("LICHESS_API_TOKEN"), help="API token with engine:read and engine:write scopes")

    try:
        import argcomplete
    except ImportError:
        pass
    else:
        argcomplete.autocomplete(parser)

    args = parser.parse_args()

    if not args.token:
        print("Need LICHESS_API_TOKEN environment variable from one of:")
        for lichess in ["https://lichess.org", "https://lichess.dev", "http://l.org", "http://localhost:9663"]:
            print(f"* {lichess}/account/oauth/token/create?scopes[]=engine:read&scopes[]=engine:write")
        sys.exit(128)

    main(args, token)
