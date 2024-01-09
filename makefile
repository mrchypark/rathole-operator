crd: src/crd.rs src/crdgen.rs
	cargo run --bin crdgen > crd/schema/crd.yaml

apply-crd: crd
	kubectl apply -f crd/schema/crd.yaml

set-example:
	kubectl apply -f crd/example/simple/.

del-example:
	kubectl delete -f crd/example/simple/.

run: crd/schema/crd.yaml
	cargo run

set-cluster:
	k3d cluster create test 

reset-cluster:
	k3d cluster delete test && k3d cluster create test 