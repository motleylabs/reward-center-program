use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AttributeParams {
    pub memo: String,
}

#[derive(Accounts, Clone)]
#[instruction(attribute_params: AttributeParams)]
pub struct Attribute {}

pub fn handler(
    _ctx: Context<Attribute>,
    AttributeParams { memo: _ }: AttributeParams,
) -> Result<()> {
    Ok(())
}
