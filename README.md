# rathole-operator

Kubernetes operator for rathole.

```d2
operator

serverCRD: {
    transport: transport setting {
        noise: {
            secretRef
        }
    }
}


clientCRD: {
    base
    serverRef
    services: {
        style.multiple: true
    }
}

server: {
    base
    transport: transport setting {
        noise
    }
    services: {
        style.multiple: true
    }
}

client
```


serverRef: req
heartbeatTimeout: 40
retryInterval: 1
configTo: 
    type: "Secret"
    name: 
services:
- name: req
  type: tcp
  token: 
    secretRef:
        name:
        key:
  localAddr: req
  nodelay: true
  retryInterval: 1


server에는 secret 마운트하는데, 그게 config.toml로 넣는다.
2분 정도 걸리지만 변경해줌.
그럼 server-config  가 secret으로 있고, 이건 옵셔널이 아니어야 함.
그리고 이 secret을 serverCRD와 clientCRD를 보고 수정함.
servername-config와 clientname-config 가 있음
clientname-config는 decode해서 사용하면 됨
servername-config는 server에서 마운트해서 사용 상태.


      
## set k8s env

use k3d

```sh
# install
wget -q -O - https://raw.githubusercontent.com/k3d-io/k3d/main/install.sh | bash
# create cluster
k3d cluster create mycluster
```