crd: src/crd.rs src/crdgen.rs
	cargo run --bin crdgen > crd/schema/crd.yaml

apply-crd: crd
	kubectl apply -f crd/schema/crd.yaml

set-cluster:
	k3d cluster create test 

reset-cluster:
	k3d cluster delete test && k3d cluster create test 