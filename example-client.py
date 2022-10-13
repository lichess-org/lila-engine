"""External engine provider example for lichess.org"""

import argparse
import logging
import requests
import sys
import os
import secrets
import chess


def ok(res):
    try:
        res.raise_for_status()
    except requests.HTTPError:
        logging.exception("Response: %s", res.text)
    return res


def register_engine(args, http):
    res = ok(http.get(f"{args.lichess}/api/external-engine"))

    secret = secrets.token_urlsafe(32)

    registration = {
        "name": args.name,
        "maxThreads": 1,
        "maxHash": 16,
        "shallowDepth": 25,
        "deepDepth": 25,
        "providerSecret": secret,
    }

    for engine in res.json():
        if engine["name"] == args.name:
            logging.info("Updating engine %s", engine["id"])
            ok(http.put(f"{args.lichess}/api/external-engine/{engine['id']}", json=registration))
            break
    else:
        logging.info("Registering new engine")
        ok(http.post(f"{args.lichess}/api/external-engine", json=registration))

    return secret


def main(args):
    http = requests.Session()
    http.headers["Authorization"] = f"Bearer {args.token}"

    secret = register_engine(args, http)

    while True:
        logging.debug("Serving ...")
        res = ok(http.post(f"{args.broker}/api/external-engine/work", json={"providerSecret": secret}))
        if res.status_code != 200:
            continue

        job = res.json()
        ok(http.post(f"{args.broker}/api/external-engine/work/{job['id']}", data=analyse(job)))


def analyse(job):
    yield b"info pv e2e4 depth 20 score cp 40"


if __name__ == "__main__":
    logging.basicConfig(level=logging.DEBUG)

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--name", default="Example", help="Engine name to register")
    parser.add_argument("--engine", help="Shell command to launch UCI engine", required=True)
    parser.add_argument("--lichess", default="https://lichess.org", help="Defaults to https://lichess.org")
    parser.add_argument("--broker", default="https://engine.lichess.ovh", help="Defaults to https://engine.lichess.ovh")
    parser.add_argument("--token", default=os.environ.get("LICHESS_API_TOKEN"), help="API token with engine:read and engine:write scopes")

    try:
        import argcomplete
    except ImportError:
        pass
    else:
        argcomplete.autocomplete(parser)

    args = parser.parse_args()

    if not args.token:
        print(f"Need LICHESS_API_TOKEN environment variable from {args.lichess}/account/oauth/token/create?scopes[]=engine:read&scopes[]=engine:write")
        sys.exit(128)

    main(args)
