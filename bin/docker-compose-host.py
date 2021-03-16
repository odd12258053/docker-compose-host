#!/usr/bin/env python
import json
import os
from shutil import get_terminal_size
from typing import Any, Dict, Iterator, List, Optional

from pydantic import BaseModel, Field, root_validator
from typer import Option, Typer


def get_tty_width() -> int:
    return get_terminal_size(fallback=(999, 0))[0]


class Network(BaseModel):
    ip_address: str = Field(..., alias='IPAddress')


class NetworkSettings(BaseModel):
    ports: Dict[str, Any] = Field(..., alias='Ports')
    networks: Dict[str, Network] = Field(..., alias='Networks')


class Inspect(BaseModel):
    name: str = Field(..., alias='Name')
    network_settings: NetworkSettings = Field(..., alias='NetworkSettings')


def container_ids(file: Optional[str] = None) -> Iterator[str]:
    cmd = ' '.join(('docker-compose', f'-f {file}' if file else '', 'ps -q'))
    yield from map(str.strip, os.popen(cmd))


def container_inspect(*container_id: str) -> Iterator[Inspect]:
    cmd = f'docker container inspect ' + ' '.join(container_id)
    yield from map(Inspect.parse_obj, json.load(os.popen(cmd)))


app = Typer()


class Host(BaseModel):
    name: str
    protocol: str
    ip: str
    port: str

    @property
    def url(self) -> str:
        if self.port and self.ip:
            return f'http://{self.ip}:{self.port}'
        return ''


class Hosts(BaseModel):
    __root__: List[Host]

    @root_validator()
    def sort(cls, values: Dict[str, List[Host]]) -> Dict[str, List[Host]]:
        values['__root__'] = sorted(values['__root__'], key=lambda host: host.ip)
        return values

    @property
    def max_len_name(self) -> int:
        return max(max((len(h.name) for h in self.__root__)), 4)

    @property
    def max_len_protocol(self) -> int:
        return max(max((len(h.protocol) for h in self.__root__)), 8)

    @property
    def max_len_ip(self) -> int:
        return max(max((len(h.ip) for h in self.__root__)), 2)

    @property
    def max_len_port(self) -> int:
        return max(max((len(h.port) for h in self.__root__)), 4)

    @property
    def max_len_url(self) -> int:
        return max(max((len(h.url) for h in self.__root__)), 3)

    def print(self) -> None:
        print(
            'Name'.center(self.max_len_name),
            'Protocol'.center(self.max_len_protocol),
            'Ip'.center(self.max_len_ip),
            'Port'.center(self.max_len_port),
            'Url'.center(self.max_len_url),
            sep='  ',
        )
        print(
            '-'
            * (
                self.max_len_name
                + self.max_len_protocol
                + self.max_len_ip
                + self.max_len_port
                + self.max_len_url
                + 10
            )
        )

        for host in self.__root__:
            print(
                host.name.ljust(self.max_len_name),
                host.protocol.ljust(self.max_len_protocol),
                host.ip.ljust(self.max_len_ip),
                host.port.ljust(self.max_len_port),
                host.url.ljust(self.max_len_url),
                sep='  ',
            )


@app.command()
def main(
    file: Optional[str] = Option(
        None, '--file', '-f', help='Specify an alternate compose file'
    )
):
    def _gene() -> Iterator[Host]:
        for inspect in container_inspect(*container_ids(file=file)):
            name = inspect.name
            network_settings = inspect.network_settings
            try:
                port, protocol = next(iter(network_settings.ports)).split('/')
            except StopIteration:
                port = ''
                protocol = ''
            try:
                ip = next(iter(network_settings.networks.values())).ip_address
            except StopIteration:
                ip = ''
            yield Host(name=name[1:], protocol=protocol, ip=ip, port=port)

    hosts = Hosts(__root__=list(_gene()))
    hosts.print()


app()
