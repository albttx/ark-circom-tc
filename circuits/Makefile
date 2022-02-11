
all: build

.PHONY: build
build:
	circom withdraw.circom --r1cs --wasm  --sym --json --inspect

generate-witness:
	snarkjs calculatewitness --wasm ./withdraw_js/withdraw.wasm --input input.json --witness witness.wtns
	# node withdraw_js/generate_witness.js ./withdraw_js/withdraw.wasm input.json witness.wtns

verify-proof:
	snarkjs groth16 verify verification_key.json public.json proof.json
