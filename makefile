crd: src/crd.rs src/crdgen.rs
	cargo run --bin crdgen > crd/schema/crd.yaml

apply-crd: crd/schema/crd.yaml
	kubectl apply -f crd/schema/crd.yaml

set-example:
	kubectl apply -f crd/example/simple/ns.yaml && kubectl apply -f crd/example/simple/.

del-example:
	kubectl delete -f crd/example/simple/client.yaml -f crd/example/simple/server.yaml

run: apply-crd crd/schema/crd.yaml
	cargo run

cluster:
	k3d cluster delete test && k3d cluster create test 