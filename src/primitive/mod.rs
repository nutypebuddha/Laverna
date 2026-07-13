pub mod arithmetic;
pub mod nand;

/// Digital logic wrappers — backward-compatible access path for descent module.
pub mod digital {
    /// Pure function: NAND gate. Re-exports `nand::nand_gate`.
    pub fn nand(left_input: bool, right_input: bool) -> bool {
        super::nand::nand_gate(left_input, right_input)
    }
}
