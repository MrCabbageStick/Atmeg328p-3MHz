pub trait LabeledReadout<const N_BYTES: usize>{
    type Data;

    fn get_label(&self) -> u8;
    fn get_data(&self) -> Self::Data;
    fn get_bytes(&self) -> [u8; N_BYTES];
}

