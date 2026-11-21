import socket
import sys
import threading
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "tools"))
import lan_probe  # noqa: E402


def test_classify_detects_tuya_prefix():
    r = lan_probe.classify(bytes.fromhex("000055aa") + b"x" * 10)
    assert r["tuya_prefix"] is True and r["length"] == 14


def test_classify_rejects_other_data():
    assert lan_probe.classify(b"hello")["tuya_prefix"] is False


def test_listen_receives_local_datagram():
    def send():
        time.sleep(0.5)
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.sendto(bytes.fromhex("000055aa") + b"abc", ("127.0.0.1", 6667))
        s.close()

    t = threading.Thread(target=send)
    t.start()
    seen = lan_probe.listen(2)
    t.join()
    assert any(x["port"] == 6667 and x["tuya_prefix"] for x in seen)


def test_scan_tcp_finds_open_port():
    srv = socket.socket()
    srv.bind(("127.0.0.1", 0))
    srv.listen(1)
    port = srv.getsockname()[1]
    old = lan_probe.TCP_PORTS
    lan_probe.TCP_PORTS = (port,)
    try:
        assert lan_probe.scan_tcp("127.0.0.1") == [port]
    finally:
        lan_probe.TCP_PORTS = old
        srv.close()
