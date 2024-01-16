crd: src/crd.rs src/crdgen.rs
	cargo run --bin crdgen > deploy/crd/crd.yaml

apply-crd: deploy/crd/crd.yaml
	kubectl apply -f deploy/crd/crd.yaml

set-example:
	kubectl apply -f deploy/crd/example/simple/ns.yaml && kubectl apply -f deploy/crd/example/simple/.

del-example:
	kubectl delete -f deploy/crd/example/simple/client.yaml -f deploy/crd/example/simple/server.yaml

run: apply-crd deploy/crd/crd.yaml
	cargo run

cluster:
	k3d cluster delete test && k3d cluster create test 

.PHONY: deploy
deploy:
	kubectl kustomize ./deploy | kubectl apply -f -

.PHONY: destroy
destroy:
	kubectl kustomize ./deploy | kubectl delete -f -