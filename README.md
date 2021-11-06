# docker-compose-host

## Usage

```sh
$ docker-compose ps
                  Name                                Command               State    Ports
--------------------------------------------------------------------------------------------
docker-compose-host_api_1                  uvicorn --reload --host 0. ...   Up      8000/tcp
docker-compose-host_api_without_expose_1   uvicorn --reload --host 0. ...   Up
```

```sh
$ docker-compose-host
                  Name                        Ip      Protocol  Port            Url
----------------------------------------------------------------------------------------------
docker-compose-host_api_1                 172.27.0.2  tcp       8000  http://172.27.0.2:8000
docker-compose-host_api_without_expose_1  172.27.0.3
```
