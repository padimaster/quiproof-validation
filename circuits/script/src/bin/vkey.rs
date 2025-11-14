use sp1_sdk::{include_elf, HashableKey, Prover, ProverClient};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const AGE_PROOF_ELF: &[u8] = include_elf!("age-proof-program");

fn main() {
    let prover = ProverClient::builder().cpu().build();
    let (_, vk) = prover.setup(AGE_PROOF_ELF);
    println!("{}", vk.bytes32());
}