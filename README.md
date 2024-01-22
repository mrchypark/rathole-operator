# rathole-operator

Kubernetes operator for [rathole](https://github.com/rapiz1/rathole).

![](./docs/arch.svg)

```sh
export RATHOLE_IMAGE=ghcr.io/mrchypark/rathole:0.5.0
```

서버crd
생성
업데이트
제거

클라이언트crd
생성
업데이트
제거

클라이언트는 serverref가 있으니, server 의존이 있음\


서버 생성
> 클라 있음
  > 반영해서 생성
> 클라 없음
  > 더미로 생성

클라 생성
> 서버 있음
  > 서버에 반영
  > 클라 생성
> 서버 없음
  > 에러 
  > 생성은 대기
  > status에 서버 없음으로 표기

서버 변경
> 클라 없음
  > 그냥 반영
> 클라 있음
  > 서비스 두고 반영

클라 변경
> 서버 없음
  > 생성과 같음
> 서버 있음
  > 서버에 반영
  > 클라에 반영

서버 삭제
> 클라 없음
  > 제거
> 클라 있음
  > 

클라 삭제
> 서버 없음
  > 삭제
> 서버 있음
  > 삭제
  > 서버에 반영