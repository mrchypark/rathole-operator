crd:
	cargo run --bin crdgen > crd/schema/crd.yaml

set-cluster:
	k3d cluster delete test && k3d cluster create test 