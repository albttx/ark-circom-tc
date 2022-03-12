#![feature(core_intrinsics)]

use num_bigint::{BigInt, RandomBits};
// use num_traits::cast::ToPrimitive;
use serde::Deserialize;
use serde_json;
// use num_traits::Zero;

use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::rand::thread_rng;

use ark_bn254::{Bn254, Fr};
use ark_circom::{CircomBuilder, CircomConfig};
use ark_groth16::{
    create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof, Proof,
    VerifyingKey,
};

use rand::Rng;

pub fn rbigint(nbytes: u64) -> BigInt {
    let mut rng = rand::thread_rng();
    return rng.sample(RandomBits::new(nbytes));
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DepositInput {
    root: String,
    nullifier_hash: String,
    relayer: String,
    recipient: String,
    fee: String,
    refund: String,
    nullifier: String,
    secret: String,
    path_elements: Vec<String>,
    path_indices: Vec<i32>,
}

#[derive(Debug, Deserialize)]
struct Withdraw {
    proof: String,
    vk: String,
    inputs: String,
}

fn main() {
    let input_json_data = r#"
    {
        "root": "16580815572075448356340562071457318374788383705496843314621489741537959124258",
        "nullifierHash": "10700765031549737019695892226146175986360939787941694441715836142154146527645",
        "relayer": "0x8EBb0380a0C88a743867A14409AED16eb3eC93eA",
        "recipient": "768046622761304935951257164293598741076624715619",
        "fee": "50000000000000000",
        "refund": "100000000000000000",
        "nullifier": "337750441743537117259945809957681472613953802882236680664715428204316132880",
        "secret": "173631503638659843485100444520947221493771326223250355257366689899361589280",
        "pathElements": [
            "21663839004416932945382355908790599225266501822907911457504978515578255421292",
            "16923532097304556005972200564242292693309333953544141029519619077135960040221",
            "7833458610320835472520144237082236871909694928684820466656733259024982655488",
            "14506027710748750947258687001455876266559341618222612722926156490737302846427",
            "4766583705360062980279572762279781527342845808161105063909171241304075622345",
            "16640205414190175414380077665118269450294358858897019640557533278896634808665",
            "13024477302430254842915163302704885770955784224100349847438808884122720088412",
            "11345696205391376769769683860277269518617256738724086786512014734609753488820",
            "17235543131546745471991808272245772046758360534180976603221801364506032471936",
            "155962837046691114236524362966874066300454611955781275944230309195800494087",
            "14030416097908897320437553787826300082392928432242046897689557706485311282736",
            "12626316503845421241020584259526236205728737442715389902276517188414400172517",
            "6729873933803351171051407921027021443029157982378522227479748669930764447503",
            "12963910739953248305308691828220784129233893953613908022664851984069510335421",
            "8697310796973811813791996651816817650608143394255750603240183429036696711432",
            "9001816533475173848300051969191408053495003693097546138634479732228054209462",
            "13882856022500117449912597249521445907860641470008251408376408693167665584212",
            "6167697920744083294431071781953545901493956884412099107903554924846764168938",
            "16572499860108808790864031418434474032816278079272694833180094335573354127261",
            "11544818037702067293688063426012553693851444915243122674915303779243865603077"
        ],
        "pathIndices": [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]
    }
    "#;
    let input_deposit: DepositInput =
        serde_json::from_str(input_json_data).expect("JSON was not well-formatted");

    println!("json: {:?}\n\n", input_deposit);

    let withdraw = generate_proof(input_deposit);

    let vk_bytes = hex::decode(&withdraw.vk).unwrap();
    let vk = VerifyingKey::<Bn254>::deserialize(&mut vk_bytes.as_slice()).unwrap();
    let pvk = prepare_verifying_key(&vk);

    let proof_bytes = hex::decode(&withdraw.proof).unwrap();
    let proof = Proof::<Bn254>::deserialize(&mut proof_bytes.as_slice()).unwrap();

    let input_bytes_arr = hex::decode(&withdraw.inputs).unwrap();

    let intputs = Vec::<Fr>::deserialize(&mut input_bytes_arr.as_slice()).unwrap();

    // Verify
    let verified = verify_proof(&pvk, &proof, &intputs).unwrap();
    assert!(verified);
    println!("verified: {} !", verified);
}

fn generate_proof(input_deposit: DepositInput) -> Withdraw {
    let cfg = CircomConfig::<Bn254>::new(
        "./circuits/withdraw_js/withdraw.wasm",
        "./circuits/withdraw.r1cs",
    )
    .unwrap();

    // Insert our public inputs as key value pairs
    let mut builder = CircomBuilder::new(cfg);

    builder.push_input(
        "root",
        BigInt::parse_bytes(input_deposit.root.as_bytes(), 10).unwrap(),
    );
    builder.push_input(
        "nullifierHash",
        BigInt::parse_bytes(input_deposit.nullifier_hash.as_bytes(), 10).unwrap(),
    );

    builder.push_input(
        "recipient",
        BigInt::parse_bytes(input_deposit.recipient.as_bytes(), 10).unwrap(),
    );

    builder.push_input(
        "relayer",
        BigInt::parse_bytes(
            input_deposit.relayer.strip_prefix("0x").unwrap().as_bytes(),
            16,
        )
        .unwrap(),
    );

    builder.push_input(
        "fee",
        BigInt::parse_bytes(input_deposit.fee.as_bytes(), 10).unwrap(),
    );
    builder.push_input(
        "refund",
        BigInt::parse_bytes(input_deposit.refund.as_bytes(), 10).unwrap(),
    );
    builder.push_input(
        "nullifier",
        BigInt::parse_bytes(input_deposit.nullifier.as_bytes(), 10).unwrap(),
    );

    builder.push_input(
        "secret",
        BigInt::parse_bytes(input_deposit.secret.as_bytes(), 10).unwrap(),
    );

    for v in input_deposit.path_elements.iter() {
        builder.push_input(
            "pathElements",
            BigInt::parse_bytes(v.as_bytes(), 10).unwrap(),
        );
    }

    for v in input_deposit.path_indices.iter() {
        builder.push_input("pathIndices", BigInt::from(*v));
    }

    // Create an empty instance for setting it up
    let circom = builder.setup();

    let mut rng = thread_rng();
    let params = generate_random_parameters::<Bn254, _, _>(circom, &mut rng).unwrap();

    let circom = builder.build().unwrap();

    let inputs = circom.get_public_inputs().unwrap();
    let mut inputs_bytes: Vec<u8> = Vec::new();
    inputs
        .serialize(&mut inputs_bytes)
        .expect("serialization of inputs into array should be infallible");

    let proof = create_random_proof(circom, &params, &mut rng).unwrap();
    let mut proof_bytes = [0u8; 128];
    proof
        .serialize(&mut proof_bytes[..])
        .expect("serialization of proof into array should be infallible");

    // let pvk = prepare_verifying_key(&params.vk);
    let mut vk_bytes: Vec<u8> = Vec::new();
    params
        .vk
        .serialize(&mut vk_bytes)
        .expect("should serialize pvk");

    Withdraw {
        proof: hex::encode(proof_bytes),
        vk: hex::encode(vk_bytes),
        inputs: hex::encode(inputs_bytes),
    }
}
