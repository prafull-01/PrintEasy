use std::collections::HashSet;

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum PageType {
    #[default]
    A3,
    A4,
    A5,
}

impl TryFrom<String> for PageType {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "A3" => Ok(PageType::A3),
            "A4" => Ok(PageType::A4),
            "A5" => Ok(PageType::A5),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum PrintType {
    Colored,
    #[default]
    BlackAndWhite,
}

impl TryFrom<String> for PrintType {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "Color Print" => Ok(PrintType::Colored),
            "Black and White Print" => Ok(PrintType::BlackAndWhite),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct CreateShopArgs {
    pub page_capabilities: HashSet<PageType>,
    pub print_capabilities: HashSet<PrintType>,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, Clone)]
pub struct NewPrintArgs {
    pub name: String,
    pub system_id: String,
    pub phone_number: String,
    pub email_id: String,
    pub file: Vec<u8>,
    pub page_type: PageType,
    pub print_type: PrintType,
}
