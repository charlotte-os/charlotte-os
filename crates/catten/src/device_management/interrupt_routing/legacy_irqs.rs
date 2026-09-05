pub(super) static IRQ_GSI_MAP: spin::LazyLock<[u32; 16]> = spin::LazyLock::new(|| {
    let mut mapping = core::array::from_fn(|i| i as u8);
    todo!("Correct for MADT overrides");
});
