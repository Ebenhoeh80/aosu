#!/usr/bin/env python3
"""Passive LAN probe for the Aosu base station (H1L) -- run on the H1L's network.

What it does (read-only, no authentication, no decryption, no payload sent):
  1. Listens for UDP broadcasts on 6666/6667 (Tuya LAN discovery) and logs
     source IP, length and whether the frame has the Tuya 0x000055AA prefix.
  2. Optionally (--host IP) tries plain TCP connects to a short list of ports
     and reports which are open. It only connects and closes immediately.

It never talks to any cloud and never stores payload bytes beyond a length
and a 4-byte prefix. Run only on networks you own.

  python tools/lan_probe.py --listen 120
  python tools/lan_probe.py --host 192.168.1.50
"""
from __future__ import annotations

import argparse
import json
import select
import socket
import time

DISCOVERY_PORTS = (6666, 6667)
TCP_PORTS = (80, 443, 554, 6668, 8000, 8080, 8554, 8883, 8899)
TUYA_PREFIX = bytes.fromhex("000055aa")


def classify(data: bytes) -> dict:
    """Describe a datagram without decoding its payload."""
    return {
        "length": len(data),
        "tuya_prefix": data[:4] == TUYA_PREFIX,
        "prefix_hex": data[:4].hex(),
    }


def listen(seconds: float) -> list[dict]:
    socks = []
    for port in DISCOVERY_PORTS:
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        s.bind(("", port))
        socks.append(s)
    seen: list[dict] = []
    end = time.monotonic() + seconds
    try:
        while time.monotonic() < end:
            ready, _, _ = select.select(socks, [], [], 0.5)
            for s in ready:
                data, (ip, _) = s.recvfrom(4096)
                seen.append(
                    {"ip": ip, "port": s.getsockname()[1], **classify(data)}
                )
    finally:
        for s in socks:
            s.close()
    return seen


def scan_tcp(host: str, timeout: float = 1.0) -> list[int]:
    open_ports = []
    for port in TCP_PORTS:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.settimeout(timeout)
            if s.connect_ex((host, port)) == 0:
                open_ports.append(port)
    return open_ports


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--listen", type=float, default=0, help="seconds to listen")
    ap.add_argument("--host", help="IP to check for open TCP ports")
    args = ap.parse_args()
    if not args.listen and not args.host:
        ap.error("use --listen SECONDS and/or --host IP")
    report: dict = {}
    if args.listen:
        report["discovery"] = listen(args.listen)
    if args.host:
        report["open_tcp_ports"] = scan_tcp(args.host)
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
