pub fn priceid_mapping(price_id: Option<String>) -> Option<Tiers> {
    price_id.as_ref()?;

    let price_id_unwrapped = price_id.unwrap();

    let price_id = price_id_unwrapped.as_str();

    match price_id {
        "price_1Ori8qEbfEExjZVcPTUzocfV" => Some(Tiers::Free),

        "price_1Orhy2EbfEExjZVcAuLQCTeP" => Some(Tiers::BusinessYearly),
        "price_1OrhxMEbfEExjZVcwSbRQhPS" => Some(Tiers::BusinessMonthly),

        "price_1OrhwgEbfEExjZVcQOgZQZ2B" => Some(Tiers::PlusYearly),
        "price_1Orhs5EbfEExjZVcLPTo1voo" => Some(Tiers::PlusMonthly),

        "price_1OrhvfEbfEExjZVc128jvppy" => Some(Tiers::StandardYearly),
        "price_1Orhr6EbfEExjZVcIFD0JrvY" => Some(Tiers::StandardMonthly),

        "price_1OrhtzEbfEExjZVcaK8HD69C" => Some(Tiers::BaseYearly),
        "price_1OrhocEbfEExjZVcHIiMYvwk" => Some(Tiers::BaseMonthly),

        _ => None,
    }
}

#[derive(Debug, Clone)]
pub enum Tiers {
    Free = 0,
    BusinessYearly = 8,
    BusinessMonthly = 7,
    PlusYearly = 6,
    PlusMonthly = 5,
    StandardYearly = 4,
    StandardMonthly = 3,
    BaseYearly = 2,
    BaseMonthly = 1,
}
